use axum::Router;
use axum::extract::State;
use axum::http::StatusCode;
use axum::response::{IntoResponse, Json};
use axum::routing::get;
use serde::Serialize;
use sqlx::PgPool;

use crate::data;

pub fn router() -> Router<PgPool> {
    Router::new()
        .route("/api/health", get(health))
        .route("/api/nodes", get(list_nodes))
}

#[derive(Serialize)]
struct HealthResponse {
    status: &'static str,
    database: &'static str,
}

async fn health(State(pool): State<PgPool>) -> impl IntoResponse {
    match data::ping_db(&pool).await {
        Ok(()) => (
            StatusCode::OK,
            Json(HealthResponse {
                status: "ok",
                database: "ok",
            }),
        )
            .into_response(),
        Err(_) => (
            StatusCode::SERVICE_UNAVAILABLE,
            Json(HealthResponse {
                status: "degraded",
                database: "error",
            }),
        )
            .into_response(),
    }
}

#[derive(Serialize)]
struct NodeJson {
    id: String,
    nodus_id_hex: String,
    mac_addr: String,
    asym_sec_algo: i16,
    asym_sec_pubkey_hex: String,
    cpu_arch: String,
    hostname: Option<String>,
    username: Option<String>,
    device_name: Option<String>,
    account_name: Option<String>,
    first_seen_at: String,
    last_seen_at: String,
}

async fn list_nodes(State(pool): State<PgPool>) -> Result<Json<Vec<NodeJson>>, StatusCode> {
    let rows = data::list_nodes(&pool)
        .await
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

    let out: Vec<NodeJson> = rows
        .into_iter()
        .map(|r| NodeJson {
            id: r.id.to_string(),
            nodus_id_hex: hex::encode(&r.nodus_id),
            mac_addr: r.mac_addr,
            asym_sec_algo: r.asym_sec_algo,
            asym_sec_pubkey_hex: hex::encode(&r.asym_sec_pubkey),
            cpu_arch: r.cpu_arch,
            hostname: r.hostname,
            username: r.username,
            device_name: r.device_name,
            account_name: r.account_name,
            first_seen_at: r.first_seen_at.to_rfc3339(),
            last_seen_at: r.last_seen_at.to_rfc3339(),
        })
        .collect();
    
    Ok(Json(out))
}
