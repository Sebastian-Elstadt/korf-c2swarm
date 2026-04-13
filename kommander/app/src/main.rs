use std::{net::SocketAddr, sync::Arc};

use domain::AppContext;
use tokio::task::JoinHandle;

#[tokio::main]
async fn main() {
    dotenvy::dotenv().ok();

    println!("kommander starting...");
    let app_ctx = Arc::new(init_ctx().await.unwrap());

    let web_task = init_web(app_ctx.clone());
    let nodecom_task = init_nodecom(app_ctx.clone());
    let _ = tokio::join!(web_task, nodecom_task);

    println!("kommander exiting...");
}

async fn init_ctx() -> Result<AppContext, String> {
    println!("+--- initializing context ---+");
    let database_url = std::env::var("DATABASE_URL").map_err(|_| "DATABASE_URL must be set")?;
    let db_pool = data::create_database_pool(&database_url).await;
    println!(" - database connection pool created.");
    data::run_migrations(&db_pool)
        .await
        .map_err(|err| format!("Database migration failed. {err}"))?;
    println!(" - ran database migrations.");

    println!();
    Ok(AppContext {
        health_port: Box::new(data::ports::PgHealthPort::new(db_pool.clone())),
        node_repo: Box::new(data::repositories::PgNodeRepository::new(db_pool.clone())),
        node_log_repo: Box::new(data::repositories::PgNodeLogRepository::new(db_pool.clone())),
        node_cmd_repo: Box::new(data::repositories::PgNodeCommandRepository::new(db_pool.clone()))
    })
}

fn init_web(app_ctx: Arc<AppContext>) -> JoinHandle<()> {
    println!("+--- initializing web thread ---+");

    let http_bind: SocketAddr = std::env::var("HTTP_BIND")
        .unwrap_or_else(|_| "0.0.0.0:8080".into())
        .parse()
        .map_err(|e: std::net::AddrParseError| format!("HTTP_BIND: {e}"))
        .unwrap();
    println!(" - binding to {http_bind}");

    println!();
    tokio::spawn(async move {
        if let Err(e) = web::serve(http_bind, app_ctx).await {
            eprintln!("http server error: {e}");
        }
    })
}

fn init_nodecom(app_ctx: Arc<AppContext>) -> JoinHandle<()> {
    println!("+--- initializing nodecom thread ---+");

    let udp_bind: SocketAddr = std::env::var("UDP_BIND")
        .unwrap_or_else(|_| "0.0.0.0:8888".into())
        .parse()
        .map_err(|e: std::net::AddrParseError| format!("UDP_BIND: {e}"))
        .unwrap();
    println!(" - binding UDP to {udp_bind}");

    println!();
    tokio::spawn(async move {
        if let Err(e) = nodecom::run(nodecom::NodeComOptions {
            app_ctx,
            udp_bind
        }).await {
            eprintln!("nodecom error: {e}");
        }
    })
}
