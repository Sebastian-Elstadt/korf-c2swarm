use domain::AppContext;
use std::{net::SocketAddr, sync::Arc};

mod udp;

pub struct NodeComOptions {
    pub app_ctx: Arc<AppContext>,
    pub udp_bind: SocketAddr,
}

pub async fn run(options: NodeComOptions) -> Result<(), std::io::Error> {
    udp::run(options.udp_bind, options.app_ctx.clone()).await?;

    Ok(())
}
