use domain::{
    AppContext,
    node::{NodeCommandType, NodeLogNetworkProtocol},
};
use std::{
    net::{SocketAddr, SocketAddrV4},
    sync::Arc,
    time::Duration,
};
use tokio::{
    net::{UdpSocket, unix::SocketAddr},
    time::{sleep, timeout},
};

mod udp;

pub struct NodeComOptions {
    pub app_ctx: Arc<AppContext>,
    pub udp_bind: SocketAddr,
}

pub async fn run(options: NodeComOptions) -> Result<(), std::io::Error> {
    let dispatcher_thread = tokio::spawn(run_cmd_dispatcher(options.app_ctx.clone()));

    udp::run(options.udp_bind, options.app_ctx.clone()).await?;

    let _ = tokio::join!(dispatcher_thread);

    Ok(())
}

// the command dispatcher only handles commands which we can execute by contacting the node directly, e.g. via an open udp port.
// it is reasonable to expect that most of the time nodes wont have an open tunnel.
// if the nodes are communicating via http requests, then a separate mechanism would be needed to always fetch a list of queued commands on any http request
// to supply that command list to the node in the response.
// however, as this entire project is for demonstration purposes, and I have so far only built UDP comms, this will do.
// building other communication methods is "trivial" - all this just proves I can do it if I need to.
async fn run_cmd_dispatcher(app_ctx: Arc<AppContext>) {
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

        let udp_sock = UdpSocket::bind("0.0.0.0:0").await;
        if let Err(err) = udp_sock {
            eprintln!("nodecom: failed to bind local udp port: {err}");
            sleep(Duration::from_secs(2)).await;
            continue;
        }

        let udp_sock = udp_sock.unwrap();

        for cmd in queue.into_iter() {
            let last_net_log = app_ctx
                .node_log_repo
                .get_last_network_log_by_node_id(cmd.node_id, Some(NodeLogNetworkProtocol::Udp))
                .await;
            if let Err(err) = last_net_log {
                eprintln!("nodecom: failed to query last network log for command: {err}");
                continue;
            }

            let last_net_log = last_net_log.unwrap();
            if last_net_log == None {
                println!(
                    "nodecom: no last network log to execute command. (node id: {})",
                    cmd.node_id
                );
                continue;
            }

            let last_net_log = last_net_log.unwrap();
            if last_net_log.ipv4_addr == None || last_net_log.network_port == None {
                println!(
                    "nodecom: no last network log contains invalid address. (node id: {})",
                    cmd.node_id
                );
                continue;
            }

            let target_addr: SocketAddr = SocketAddrV4(
                last_net_log.ipv4_addr.unwrap(),
                last_net_log.network_port.unwrap(),
            );

            cmd.status = domain::node::NodeCommandStatus::Executing;
            app_ctx.node_cmd_repo.update(&cmd).await;

            let skip_send = false;
            let mut cmd_buf: Vec<u8> = vec![77, 33, cmd.command_type as u8];
            match cmd.command_type {
                NodeCommandType::ShellScript => {
                    if let Some(text_content) = cmd.text_content
                        && text_content.len() > 0
                    {
                        let text_content_bytes = text_content.as_bytes();
                        let len = text_content_bytes.len() as u32;
                        cmd_buf.extend_from_slice(len.to_be_bytes());
                        cmd_buf.extend_from_slice(text_content_bytes);
                    } else {
                        skip_send = true;
                    }
                }
                _ => {}
            };

            let node_responded = false;
            if !skip_send {
                udp_sock.send_to(&cmd_buf, target_addr);

                let mut response_buf = [u8; 32];
                let response_result = timeout(
                    Duration::from_secs(10),
                    udp_sock.recv_from(&mut response_buf),
                )
                .await;
                // this isnt great, would be better to filter through any incoming packets until the end of the 10 seconds
                // but, I need to get this POC done as soon as possible.
                if let Ok(Ok((len, addr))) = response_result
                    && addr == target_addr
                    && response_buf[..len] == b"ACK"
                {
                    node_responded = true;
                } else {
                    cmd.status = domain::node::NodeCommandStatus::Queued;
                    app_ctx.node_cmd_repo.update(&cmd).await;
                }
            }

            if skip_send || node_responded {
                cmd.status = domain::node::NodeCommandStatus::Completed;
                app_ctx.node_cmd_repo.update(&cmd).await;
            }
        }

        sleep(Duration::from_secs(2)).await;
    }
}
