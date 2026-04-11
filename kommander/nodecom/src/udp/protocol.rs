use thiserror::Error;

pub const MAGIC_0: u8 = 77;
pub const MAGIC_1: u8 = 33;
pub const MSG_HEARTBEAT: u8 = 0;
pub const MSG_REGISTER: u8 = 1;

#[derive(Debug, Error)]
pub enum ProtocolError {
    #[error("packet too short")]
    TooShort,
    #[error("invalid magic bytes")]
    BadMagic,
    #[error("unexpected message type: {0}")]
    UnexpectedMessageType(u8),
    #[error("invalid UTF-8 in string field")]
    InvalidUtf8,
}

fn validate_payload(data: &[u8], msg_type: u8, expected_len: usize) -> Result<(), ProtocolError> {
    if data.len() < 3 {
        return Err(ProtocolError::TooShort);
    }
    if data[0] != MAGIC_0 || data[1] != MAGIC_1 {
        return Err(ProtocolError::BadMagic);
    }
    if data[2] != msg_type {
        return Err(ProtocolError::UnexpectedMessageType(data[2]));
    }

    if data.len() < 3usize + expected_len {
        return Err(ProtocolError::TooShort);
    }

    Ok(())
}

#[derive(Debug, Clone)]
pub struct RegistrationPayload {
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

pub fn parse_registration(data: &[u8]) -> Result<RegistrationPayload, ProtocolError> {
    validate_payload(data, MSG_REGISTER, 32 + 6 + 1 + 2)?;

    let mut i = 3usize;

    let mut nodus_id = [0u8; 32];
    nodus_id.copy_from_slice(&data[i..i + 32]);
    i += 32;

    let mac_addr = format_mac(&data[i..i + 6]);
    i += 6;

    let asym_sec_algo = i16::from(data[i]);
    i += 1;

    let pk_len = u16::from_be_bytes([data[i], data[i + 1]]) as usize;
    i += 2;
    if data.len() < i + pk_len {
        return Err(ProtocolError::TooShort);
    }
    let asym_sec_pubkey = data[i..i + pk_len].to_vec();
    i += pk_len;

    let (cpu_arch, c) = read_string_segment(data, i)?;
    i += c;
    let (hostname, c) = read_string_segment(data, i)?;
    i += c;
    let (username, c) = read_string_segment(data, i)?;
    i += c;
    let (device_name, c) = read_string_segment(data, i)?;
    i += c;
    let (account_name, _) = read_string_segment(data, i)?;

    Ok(RegistrationPayload {
        nodus_id,
        mac_addr,
        asym_sec_algo,
        asym_sec_pubkey,
        cpu_arch,
        hostname: opt_string(hostname),
        username: opt_string(username),
        device_name: opt_string(device_name),
        account_name: opt_string(account_name),
    })
}

#[derive(Debug, Clone)]
pub struct HeartbeatPayload {
    pub nodus_id: [u8; 32],
    #[allow(dead_code)]
    pub node_local_time_ms: i64,
    pub sig_bytes: Vec<u8>,
}

pub fn parse_heartbeat(data: &[u8]) -> Result<HeartbeatPayload, ProtocolError> {
    validate_payload(data, MSG_HEARTBEAT, 32 + 8 + 2)?;

    let mut i = 3usize;

    let mut nodus_id = [0u8; 32];
    nodus_id.copy_from_slice(&data[i..i + 32]);
    i += 32;

    let mut ts_bytes = [0u8; 8];
    ts_bytes.copy_from_slice(&data[i..i + 8]);
    let node_local_time_ms = i64::from_be_bytes(ts_bytes);
    i += 8;

    let sig_len = u16::from_be_bytes([data[i], data[i + 1]]) as usize;
    i += 2;
    if data.len() < i + sig_len {
        return Err(ProtocolError::TooShort);
    }
    let sig_bytes = data[i..i + sig_len].to_vec();

    Ok(HeartbeatPayload {
        nodus_id,
        node_local_time_ms,
        sig_bytes,
    })
}

fn format_mac(bytes: &[u8]) -> String {
    bytes
        .iter()
        .map(|b| format!("{b:02x}"))
        .collect::<Vec<_>>()
        .join(":")
}

fn opt_string(s: String) -> Option<String> {
    if s.is_empty() { None } else { Some(s) }
}

fn read_string_segment(data: &[u8], index: usize) -> Result<(String, usize), ProtocolError> {
    if data.len() < index + 2 {
        return Err(ProtocolError::TooShort);
    }
    let len = u16::from_be_bytes([data[index], data[index + 1]]) as usize;
    if len < 1 {
        return Ok(("".into(), 2));
    }
    if data.len() < index + 2 + len {
        return Err(ProtocolError::TooShort);
    }
    let bytes = &data[index + 2..index + 2 + len];
    let s = std::str::from_utf8(bytes)
        .map_err(|_| ProtocolError::InvalidUtf8)?
        .to_owned();
    Ok((s, len + 2))
}
