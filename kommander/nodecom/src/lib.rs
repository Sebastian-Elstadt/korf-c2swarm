use domain::{
    AppContext,
    node::{NodeCommandType, NodeLogNetworkProtocol},
};
use std::{net::SocketAddr, sync::Arc, time::Duration};
use tokio::{
    net::UdpSocket,
    sync::mpsc,
    time::{Instant, sleep, timeout},
};

mod udp;

pub struct NodeComOptions {
    pub app_ctx: Arc<AppContext>,
    pub udp_bind: SocketAddr,
}

pub async fn run(options: NodeComOptions) -> Result<(), std::io::Error> {
    let sock = Arc::new(UdpSocket::bind(options.udp_bind).await?);
    let (ack_tx, ack_rx) = mpsc::channel::<(SocketAddr, Vec<u8>)>(64);

    let dispatcher_thread = tokio::spawn(run_cmd_dispatcher(
        options.app_ctx.clone(),
        sock.clone(),
        ack_rx,
    ));

    udp::run(sock, options.app_ctx.clone(), ack_tx).await?;

    let _ = tokio::join!(dispatcher_thread);

    Ok(())
}

// the command dispatcher only handles commands which we can execute by contacting the node directly, e.g. via an open udp port.
// it is reasonable to expect that most of the time nodes wont have an open tunnel.
// if the nodes are communicating via http requests, then a separate mechanism would be needed to always fetch a list of queued commands on any http request
// to supply that command list to the node in the response.
// however, as this entire project is for demonstration purposes, and I have so far only built UDP comms, this will do.
//
// the udp socket is shared with the listener so outgoing commands use the same source port nodus registered to.
// this matters because NAT/conntrack entries (including docker's masquerade) only route return traffic that
// matches an already-active flow; a fresh ephemeral socket would be dropped at the host boundary.
async fn run_cmd_dispatcher(
    app_ctx: Arc<AppContext>,
    udp_sock: Arc<UdpSocket>,
    mut ack_rx: mpsc::Receiver<(SocketAddr, Vec<u8>)>,
) {
    loop {
        let queue = app_ctx.node_cmd_repo.get_queued().await;
        if let Err(err) = queue {
            eprintln!("nodecom: failed to read command queue: {err}");
            sleep(Duration::from_secs(2)).await;
            continue;
        }

        let queue = queue.unwrap();
        if queue.len() == 0 {
            sleep(Duration::from_secs(2)).await;
            continue;
        }

        for mut cmd in queue.into_iter() {
            let last_net_log = app_ctx
                .node_log_repo
                .get_last_network_log_by_node_id(cmd.node_id, Some(NodeLogNetworkProtocol::Udp))
                .await;
            if let Err(err) = last_net_log {
                eprintln!("nodecom: failed to query last network log for command: {err}");
                continue;
            }

            let last_net_log = last_net_log.unwrap();
            if last_net_log.is_none() {
                println!(
                    "nodecom: no last network log to execute command. (node id: {})",
                    cmd.node_id
                );
                continue;
            }

            let last_net_log = last_net_log.unwrap();
            if last_net_log.ipv4_addr.is_none() || last_net_log.network_port.is_none() {
                println!(
                    "nodecom: no last network log contains invalid address. (node id: {})",
                    cmd.node_id
                );
                continue;
            }

            let target_addr: SocketAddr = SocketAddr::new(
                std::net::IpAddr::V4(last_net_log.ipv4_addr.unwrap()),
                last_net_log.network_port.unwrap(),
            );

            cmd.status = domain::node::NodeCommandStatus::Executing;
            let _ = app_ctx.node_cmd_repo.update(&cmd).await;

            let mut skip_send = false;
            let mut cmd_buf: Vec<u8> = vec![77, 33, cmd.command_type.clone() as u8];
            match cmd.command_type {
                NodeCommandType::ShellScript => {
                    if let Some(text_content) = cmd.text_content.as_deref() {
                        if !text_content.is_empty() {
                            let text_content_bytes = text_content.as_bytes();
                            let len = text_content_bytes.len() as u32;
                            cmd_buf.extend_from_slice(&len.to_be_bytes());
                            cmd_buf.extend_from_slice(text_content_bytes);
                        } else {
                            skip_send = true;
                        }
                    } else {
                        skip_send = true;
                    }
                }
                _ => {}
            };

            let mut node_responded = false;
            if !skip_send {
                // drain any stale ACKs left over from prior iterations
                while ack_rx.try_recv().is_ok() {}

                if let Err(err) = udp_sock.send_to(&cmd_buf, target_addr).await {
                    eprintln!("nodecom: failed to send udp command: {err}");
                    cmd.status = domain::node::NodeCommandStatus::Queued;
                    let _ = app_ctx.node_cmd_repo.update(&cmd).await;
                    continue;
                }

                let wait_start = Instant::now();
                let deadline = Duration::from_secs(10);
                while wait_start.elapsed() < deadline {
                    let remaining = deadline.saturating_sub(wait_start.elapsed());
                    match timeout(remaining, ack_rx.recv()).await {
                        Ok(Some((addr, data))) => {
                            if addr == target_addr && data == b"ACK" {
                                node_responded = true;
                                break;
                            }
                        }
                        Ok(None) | Err(_) => break,
                    }
                }

                if !node_responded {
                    cmd.status = domain::node::NodeCommandStatus::Queued;
                    let _ = app_ctx.node_cmd_repo.update(&cmd).await;
                }
            }

            if skip_send || node_responded {
                cmd.status = domain::node::NodeCommandStatus::Completed;
                let _ = app_ctx.node_cmd_repo.update(&cmd).await;
            }
        }

        sleep(Duration::from_secs(2)).await;
    }
}
