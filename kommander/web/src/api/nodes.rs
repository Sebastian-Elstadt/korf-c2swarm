use axum::extract::{Path, State};
use axum::response::Json;
use domain::AppContext;
use serde::Serialize;
use uuid::Uuid;
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
    host_local_time: Option<String>,
}

pub async fn list_nodes(
    State(app_ctx): State<Arc<AppContext>>,
) -> Result<Json<Vec<NodeListItem>>, ApiError> {
    let mut nodes = app_ctx
        .node_repo
        .get_all()
        .await
        .map_err(|err| ApiError::internal(format!("failed to load nodes list: {err}")))?;

    nodes.sort_by_key(|n| n.last_seen_at);
    nodes.reverse();

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
            host_local_time: r.host_local_time.map(|ts| ts.to_rfc3339()),
        })
        .collect();

    Ok(Json(out))
}


#[derive(Serialize)]
pub struct NodeLogEntryItem {
    id: String,
    created_at: String,
    event_type: u8,
    text_content: Option<String>,
    ipv4_addr: Option<String>,
    network_port: Option<u16>,
    network_protocol: Option<u8>,
}

pub async fn get_node_logs(
    State(app_ctx): State<Arc<AppContext>>,
    Path(node_id): Path<Uuid>
) -> Result<Json<Vec<NodeLogEntryItem>>, ApiError> {
    let mut logs = app_ctx
        .node_log_repo
        .get_by_node_id(node_id)
        .await
        .map_err(|err| ApiError::internal(format!("failed to load node logs: {err}")))?;

    logs.sort_by_key(|n| n.created_at);
    logs.reverse();

    let out: Vec<NodeLogEntryItem> = logs
        .into_iter()
        .map(|r| NodeLogEntryItem {
            id: r.id.to_string(),
            created_at: r.created_at.to_rfc3339(),
            event_type: r.event_type as u8,
            text_content: r.text_content,
            ipv4_addr: r.ipv4_addr.map(|v| v.to_string()),
            network_port: r.network_port.map(|v| v as u16),
            network_protocol: r.network_protocol.map(|v| v as u8)
        })
        .collect();

    Ok(Json(out))
}
