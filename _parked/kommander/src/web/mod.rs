use axum::Router;
use sqlx::PgPool;
use std::net::SocketAddr;
use tower_http::services::{ServeDir, ServeFile};

mod api;

pub async fn serve(pool: PgPool, bind: SocketAddr) -> Result<(), std::io::Error> {
    let listener = tokio::net::TcpListener::bind(bind).await?;

    let gui_server = ServeDir::new("/var/www/")
        .not_found_service(ServeFile::new("/var/www/index.html"))
        .append_index_html_on_directories(true);

    let app = Router::new()
        .merge(api::router())
        .fallback_service(gui_server)
        .with_state(pool);

    axum::serve(listener, app).await
}
