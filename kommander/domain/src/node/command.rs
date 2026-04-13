use core::panic;

use sqlx::{
    Row,
    postgres::PgRow,
    prelude::FromRow,
    types::chrono::{DateTime, Utc},
};
use uuid::Uuid;

#[derive(Debug, Clone)]
pub enum NodeCommandStatus {
    Queued = 0,
    Executing = 1,
    Completed = 2,
    Cancelled = 4,
}

impl From<i16> for NodeCommandStatus {
    fn from(value: i16) -> Self {
        match value {
            0 => NodeCommandStatus::Queued,
            1 => NodeCommandStatus::Executing,
            2 => NodeCommandStatus::Completed,
            3 => NodeCommandStatus::Cancelled,
            _ => panic!("unknown node command status: {value}"),
        }
    }
}

#[derive(Debug, Clone)]
pub enum NodeCommandType {
    Shutdown = 0,
    ShellScript = 1,
}

impl From<i16> for NodeCommandType {
    fn from(value: i16) -> Self {
        match value {
            0 => NodeCommandType::Shutdown,
            1 => NodeCommandType::ShellScript,
            _ => panic!("unknown node command type: {value}"),
        }
    }
}

#[derive(Debug, Clone, FromRow)]
pub struct NodeCommandEntry {
    pub id: Uuid,
    pub node_id: Uuid,
    pub created_at: DateTime<Utc>,

    pub status: NodeCommandStatus,
    pub command_type: NodeCommandType,
    pub last_attempted_at: Option<DateTime<Utc>>,
    pub completed_at: Option<DateTime<Utc>>,
    pub text_content: Option<String>,
}

impl NodeCommandEntry {
    pub fn new(node_id: Uuid, command_type: NodeCommandType) -> Self {
        Self {
            id: Uuid::nil(),
            node_id,
            created_at: Utc::now(),
            status: NodeCommandStatus::Queued,
            command_type,
            last_attempted_at: None,
            completed_at: None,
            text_content: None,
        }
    }

    pub fn from_pg_row(row: PgRow) -> Result<Self, sqlx::Error> {
        let command_type: i16 = row.try_get("command_type")?;
        let status: i16 = row.try_get("status")?;

        Ok(Self {
            id: row.try_get("id")?,
            node_id: row.try_get("node_id")?,
            created_at: row.try_get("created_at")?,
            status: NodeCommandStatus::from(status),
            command_type: NodeCommandType::from(command_type),
            last_attempted_at: row.try_get("last_attempted_at")?,
            completed_at: row.try_get("completed_at")?,
            text_content: row.try_get("text_content")?,
        })
    }
}
