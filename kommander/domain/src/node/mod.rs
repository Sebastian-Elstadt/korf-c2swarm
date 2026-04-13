use sqlx::{
    prelude::FromRow,
    types::chrono::{DateTime, Utc},
};
use uuid::Uuid;

mod log;
pub use log::*;

mod command;
pub use command::*;

#[derive(Debug, Clone, FromRow)]
pub struct Node {
    pub id: Uuid,
    pub nodus_id: Vec<u8>,
    pub mac_addr: String,
    pub asym_sec_algo: i16,
    pub asym_sec_pubkey: Vec<u8>,
    pub cpu_arch: String,
    pub hostname: Option<String>,
    pub username: Option<String>,
    pub device_name: Option<String>,
    pub account_name: Option<String>,
    pub first_seen_at: DateTime<Utc>,
    pub last_seen_at: DateTime<Utc>,
    pub host_local_time: Option<DateTime<Utc>>,
}
