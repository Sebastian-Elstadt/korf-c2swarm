mod db;
mod node;
mod web;

use std::net::SocketAddr;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
    let database_url = std::env::var("DATABASE_URL").map_err(|_| "DATABASE_URL must be set")?;
    let db_pool = db::connect(&database_url).await?;
    db::run_migrations(&db_pool).await?;

    let udp_bind: SocketAddr = std::env::var("UDP_BIND")
        .unwrap_or_else(|_| "0.0.0.0:8888".into())
        .parse()
        .map_err(|e: std::net::AddrParseError| format!("UDP_BIND: {e}"))?;

    let http_bind: SocketAddr = std::env::var("HTTP_BIND")
        .unwrap_or_else(|_| "0.0.0.0:8080".into())
        .parse()
        .map_err(|e: std::net::AddrParseError| format!("HTTP_BIND: {e}"))?;

    let udp_db_pool = db_pool.clone();
    let udp_task = tokio::spawn(async move {
        if let Err(e) = node::run(udp_db_pool, udp_bind).await {
            eprintln!("udp server error: {e}");
        }
    });

    let web_db_pool = db_pool.clone();
    let web_task = tokio::spawn(async move {
        if let Err(e) = web::serve(web_db_pool, http_bind).await {
            eprintln!("http server error: {e}");
        }
    });

    let _ = tokio::join!(udp_task, web_task);
    Ok(())
}
