use axum::extract::State;
use axum::response::Json;
use domain::AppContext;
use serde::Serialize;
use std::sync::Arc;

use crate::api::ApiError;

#[derive(Serialize)]
pub struct NodeListItem {
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

pub async fn list_nodes(
    State(app_ctx): State<Arc<AppContext>>,
) -> Result<Json<Vec<NodeListItem>>, ApiError> {
    let nodes = app_ctx
        .node_repo
        .get_all()
        .await
        .map_err(|err| ApiError::internal(format!("failed to load nodes list: {err}")))?;

    let out: Vec<NodeListItem> = nodes
        .into_iter()
        .map(|r| NodeListItem {
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
