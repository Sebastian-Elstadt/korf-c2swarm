use chrono::{DateTime, Utc};
use sqlx::FromRow;
use uuid::Uuid;

#[derive(Debug, Clone)]
pub struct RegistrationInput {
    pub nodus_id: [u8; 32],
    pub mac_addr: String,
    pub asym_sec_algo: i16,
    pub asym_sec_pubkey: Vec<u8>,
    pub cpu_arch: String,
    pub hostname: Option<String>,
    pub username: Option<String>,
    pub device_name: Option<String>,
    pub account_name: Option<String>,
}

#[derive(Debug, Clone, FromRow)]
pub struct NodeRecord {
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
}
