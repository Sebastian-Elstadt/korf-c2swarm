use domain::{AppContext, node::NodeLogNetworkProtocol};
use std::{net::SocketAddr, sync::Arc, time::Duration};
use tokio::time::sleep;

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

        for cmd in queue.into_iter() {
            let last_net_log = app_ctx.node_log_repo.get_last_network_log_by_node_id(cmd.node_id, Some(NodeLogNetworkProtocol::Udp)).await;
            if let Err(err) = last_net_log {
                eprintln!("nodecom: failed to query last network log for command: {err}");
                continue;
            }

            let last_net_log = last_net_log.unwrap();
            if last_net_log == None {
                println!("nodecom: no last network log to execute command. (node id: {})", cmd.node_id);
                continue;
            }

            let last_net_log = last_net_log.unwrap();
            // todo: send to ipv4+port
        }

        sleep(Duration::from_secs(2)).await;
    }
}
