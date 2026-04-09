use std::{net::SocketAddr, sync::Arc};
use axum::Router;
use domain::AppContext;
use tower_http::services::{ServeDir, ServeFile};

mod api;

pub async fn serve(bind_addr: SocketAddr, app_ctx: Arc<AppContext>) -> Result<(), std::io::Error> {
    let listener = tokio::net::TcpListener::bind(bind_addr).await?;

    let gui_server = ServeDir::new("/var/www/")
        .not_found_service(ServeFile::new("/var/www/index.html"))
        .append_index_html_on_directories(true);

    let router = Router::new()
        .merge(api::router())
        .fallback_service(gui_server)
        .with_state(app_ctx);

    axum::serve(listener, router).await
}
