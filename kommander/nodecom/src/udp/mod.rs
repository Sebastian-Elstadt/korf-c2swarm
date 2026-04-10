use chrono::Utc;
use domain::{AppContext, node};
use std::{net::SocketAddr, sync::Arc};
use tokio::net::UdpSocket;
use uuid::Uuid;

mod protocol;

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
                handle_heartbeat(app_ctx.clone(), data).await;
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

async fn handle_heartbeat(
    app_ctx: Arc<AppContext>,
    data: &[u8]
) {
    println!("nodecom: received heartbeat.");

    match protocol::parse_heartbeat(data) {
        Ok(heartbeat) => match app_ctx.node_repo.get_by_nodus_id(heartbeat.nodus_id).await {
            Ok(query_result) => {
                if let Some(mut node) = query_result {
                    // todo: check if node.asym_sec_algo = 1, then use ed25519 to verify the signature with node.asym_sec_pubkey

                    node.last_seen_at = Utc::now();
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
            };

            if let Err(err) = app_ctx.node_repo.add(&mut node).await {
                eprintln!("nodecom: registration payload parse failed: {err}");
                return;
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
