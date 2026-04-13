use std::net::Ipv4Addr;

use sqlx::{
    Row,
    postgres::PgRow,
    prelude::FromRow,
    types::chrono::{DateTime, Utc},
};
use uuid::Uuid;

#[derive(Debug, Clone)]
pub enum NodeLogEventType {
    Unknown = 0,
    Heartbeat = 1,
    Registration = 2,
}

impl From<i16> for NodeLogEventType {
    fn from(value: i16) -> Self {
        match value {
            1 => NodeLogEventType::Heartbeat,
            2 => NodeLogEventType::Registration,
            _ => NodeLogEventType::Unknown,
        }
    }
}

#[derive(Debug, Clone)]
pub enum NodeLogNetworkProtocol {
    Unknown = 0,
    Udp = 1,
}

impl From<i16> for NodeLogNetworkProtocol {
    fn from(value: i16) -> Self {
        match value {
            1 => NodeLogNetworkProtocol::Udp,
            _ => NodeLogNetworkProtocol::Unknown,
        }
    }
}

#[derive(Debug, Clone, FromRow)]
pub struct NodeLogEntry {
    pub id: Uuid,
    pub node_id: Uuid,
    pub created_at: DateTime<Utc>,

    pub event_type: NodeLogEventType,
    pub text_content: Option<String>,
    pub ipv4_addr: Option<Ipv4Addr>,
    pub network_port: Option<u16>,
    pub network_protocol: Option<NodeLogNetworkProtocol>,
}

impl NodeLogEntry {
    pub fn new(node_id: Uuid, event_type: NodeLogEventType) -> Self {
        Self {
            id: Uuid::nil(),
            node_id,
            created_at: Utc::now(),
            event_type,
            text_content: None,
            ipv4_addr: None,
            network_port: None,
            network_protocol: None,
        }
    }

    pub fn from_pg_row(row: PgRow) -> Result<Self, sqlx::Error> {
        let event_type: i16 = row.try_get("event_type")?;
        let ipv4_addr: Option<String> = row.try_get("ipv4_addr")?;
        let network_port: Option<i32> = row.try_get("network_port")?;
        let network_protocol: Option<i16> = row.try_get("network_protocol")?;

        Ok(Self {
            id: row.try_get("id")?,
            node_id: row.try_get("node_id")?,
            created_at: row.try_get("created_at")?,
            event_type: NodeLogEventType::from(event_type),
            text_content: row.try_get("text_content")?,
            ipv4_addr: ipv4_addr.map(|v| v.parse().ok()).flatten(),
            network_port: network_port.map(|v| v as u16),
            network_protocol: network_protocol.map(|v| NodeLogNetworkProtocol::from(v)),
        })
    }
}
