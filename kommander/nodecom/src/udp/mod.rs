use chrono::Utc;
use domain::{
    AppContext,
    node::{self, NodeLogEntry, NodeLogNetworkProtocol},
};
use std::{net::SocketAddr, sync::Arc};
use tokio::net::UdpSocket;
use uuid::Uuid;

mod protocol;

const ASYM_SEC_ALGO_ED25519: i16 = 1;
const HEARTBEAT_SIGNED_LEN: usize = 3 + 32 + 8;

pub async fn run(bind_addr: SocketAddr, app_ctx: Arc<AppContext>) -> Result<(), std::io::Error> {
    let sock = UdpSocket::bind(bind_addr).await?;
    let mut buf = [0u8; 65535];

    loop {
        let (len, addr) = sock.recv_from(&mut buf).await?;
        let data = &buf[..len];

        if data.len() < 3 || data[0] != protocol::MAGIC_0 || data[1] != protocol::MAGIC_1 {
            continue;
        }

        match data[2] {
            protocol::MSG_HEARTBEAT => {
                handle_heartbeat(app_ctx.clone(), data, &addr).await;
            }
            protocol::MSG_REGISTER => {
                handle_register(app_ctx.clone(), data, &sock, &addr).await;
            }
            _ => {
                if sock.send_to(b"HUH", addr).await.is_err() {
                    eprintln!("udp send HUH failed for unknown message");
                }
            }
        }
    }
}

async fn handle_heartbeat(app_ctx: Arc<AppContext>, data: &[u8], addr: &SocketAddr) {
    println!("nodecom: received heartbeat.");

    match protocol::parse_heartbeat(data) {
        Ok(heartbeat) => match app_ctx.node_repo.get_by_nodus_id(heartbeat.nodus_id).await {
            Ok(query_result) => {
                if let Some(mut node) = query_result {
                    if node.asym_sec_algo != ASYM_SEC_ALGO_ED25519 {
                        eprintln!(
                            "nodecom: heartbeat rejected (unsupported asym_sec_algo={})",
                            node.asym_sec_algo
                        );
                        return;
                    }

                    if node.asym_sec_pubkey.len() != korf_ed25519::PUBLIC_KEY_LENGTH {
                        eprintln!("nodecom: heartbeat rejected (bad pubkey length)");
                        return;
                    }

                    if heartbeat.sig_bytes.len() != korf_ed25519::SIGNATURE_LENGTH {
                        eprintln!("nodecom: heartbeat rejected (bad signature length)");
                        return;
                    }

                    let pk: &[u8; korf_ed25519::PUBLIC_KEY_LENGTH] =
                        match node.asym_sec_pubkey.as_slice().try_into() {
                            Ok(v) => v,
                            Err(_) => {
                                eprintln!("nodecom: heartbeat rejected (pubkey slice)");
                                return;
                            }
                        };

                    let sig: &[u8; korf_ed25519::SIGNATURE_LENGTH] =
                        match heartbeat.sig_bytes.as_slice().try_into() {
                            Ok(v) => v,
                            Err(_) => {
                                eprintln!("nodecom: heartbeat rejected (signature slice)");
                                return;
                            }
                        };

                    if data.len() < HEARTBEAT_SIGNED_LEN + 2 + korf_ed25519::SIGNATURE_LENGTH {
                        eprintln!("nodecom: heartbeat rejected (packet too short for verify)");
                        return;
                    }

                    if !korf_ed25519::verify_signature(pk, &data[0..HEARTBEAT_SIGNED_LEN], sig) {
                        eprintln!("nodecom: heartbeat rejected (bad Ed25519 signature)");
                        return;
                    }

                    {
                        let mut log_entry =
                            NodeLogEntry::new(node.id, node::NodeLogEventType::Heartbeat);
                        log_entry.network_port = Some(addr.port());
                        log_entry.network_protocol = Some(NodeLogNetworkProtocol::Udp);
                        if let SocketAddr::V4(addr_v4) = addr {
                            log_entry.ipv4_addr = Some(*addr_v4.ip());
                        }

                        match app_ctx.node_log_repo.add(&mut log_entry).await {
                            Ok(()) => {}
                            Err(err) => {
                                eprintln!("nodecom: heartbeat log entry error: {err}");
                            }
                        }
                    }

                    node.last_seen_at = Utc::now();
                    node.host_local_time =
                        chrono::DateTime::from_timestamp_millis(heartbeat.node_local_time_ms);
                    match app_ctx.node_repo.update(&node).await {
                        Ok(()) => {
                            println!("nodecom: node heartbeat update completed.");
                            return;
                        }
                        Err(err) => {
                            eprintln!("nodecom: heartbeat node update error: {err}");
                        }
                    }
                }

                eprintln!("nodecom: heartbeat node query found nothing.");
            }
            Err(err) => {
                eprintln!("nodecom: heartbeat node query error: {err}");
            }
        },
        Err(err) => {
            eprintln!("nodecom: heartbeat payload parse failed: {err}");
        }
    }
}

async fn handle_register(
    app_ctx: Arc<AppContext>,
    data: &[u8],
    sock: &UdpSocket,
    addr: &SocketAddr,
) {
    println!("nodecom: received registration payload.");

    match protocol::parse_registration(data) {
        Ok(payload) => {
            let mut node = node::Node {
                id: Uuid::nil(),
                nodus_id: payload.nodus_id.to_vec(),
                mac_addr: payload.mac_addr,
                asym_sec_algo: payload.asym_sec_algo,
                asym_sec_pubkey: payload.asym_sec_pubkey,
                cpu_arch: payload.cpu_arch,
                hostname: payload.hostname,
                username: payload.username,
                device_name: payload.device_name,
                account_name: payload.account_name,
                first_seen_at: Utc::now(),
                last_seen_at: Utc::now(),
                host_local_time: None, // could update registration to also pass this.
            };

            if let Err(err) = app_ctx.node_repo.add(&mut node).await {
                eprintln!("nodecom: registration payload parse failed: {err}");
                return;
            }

            {
                let mut log_entry = NodeLogEntry::new(node.id, node::NodeLogEventType::Registration);
                log_entry.network_port = Some(addr.port());
                log_entry.network_protocol = Some(NodeLogNetworkProtocol::Udp);
                if let SocketAddr::V4(addr_v4) = addr {
                    log_entry.ipv4_addr = Some(*addr_v4.ip());
                }

                match app_ctx.node_log_repo.add(&mut log_entry).await {
                    Ok(()) => {}
                    Err(err) => {
                        eprintln!("nodecom: registration log entry error: {err}");
                    }
                }
            }

            println!("nodecom: new node registered.");
            if let Err(err) = sock.send_to(b"ACK", addr).await {
                eprintln!("nodecome: new node failed response: {err}");
            }
        }
        Err(err) => {
            eprintln!("nodecom: registration payload parse failed: {err}");
        }
    };
}
