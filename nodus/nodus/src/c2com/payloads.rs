use chrono::Local;

use crate::identity::Identity;

const MAGIC_1: u8 = 77;
const MAGIC_2: u8 = 33;

const HEARTBEAT_ACTION_CODE: u8 = 0;
const REGISTRATION_ACTION_CODE: u8 = 1;

pub fn registration(identity: &Identity) -> Result<Vec<u8>, String> {
    let mut buf = new_payload(REGISTRATION_ACTION_CODE);

    // 1. nodus id
    buf.extend_from_slice(
        &base85::decode(&identity.nodus_id)
            .map_err(|err| format!("Nodus id decode failure. {err}"))?,
    );

    // 2. mac addr
    if let Some(m) = &identity.mac_addr {
        let mac_bytes: Vec<u8> = m
            .split(':')
            .filter_map(|hex| u8::from_str_radix(hex, 16).ok())
            .collect();
        if mac_bytes.len() == 6 {
            buf.extend_from_slice(&mac_bytes);
        } else {
            buf.extend_from_slice(&[0, 0, 0, 0, 0, 0]);
        }
    } else {
        buf.extend_from_slice(&[0, 0, 0, 0, 0, 0]);
    }

    // 3. public key
    buf.extend_from_slice(&[1]); // algo type
    append_bytes_with_len(&mut buf, &identity.asym_sec.get_public_key());

    // 4. cpu arch
    append_str_with_len(&mut buf, &identity.cpu_arch);

    // 5. hostname
    append_optional_str_with_len(&mut buf, &identity.hostname);

    // 6. username
    append_optional_str_with_len(&mut buf, &identity.username);

    // 7. device name
    append_optional_str_with_len(&mut buf, &identity.device_name);

    // 8. account name
    append_optional_str_with_len(&mut buf, &identity.account_name);

    Ok(buf)
}

pub fn heartbeat(identity: &Identity) -> Result<Vec<u8>, String> {
    let mut buf = new_payload(HEARTBEAT_ACTION_CODE);

    // 1. nodus id - 32 bytes
    buf.extend_from_slice(
        &base85::decode(&identity.nodus_id)
            .map_err(|err| format!("Nodus id decode failure. {err}"))?,
    );

    // 2. local device time - unix ms, i64, big endian - 8 bytes
    let local_ts_ms: i64 = Local::now().timestamp_millis();
    buf.extend_from_slice(&local_ts_ms.to_be_bytes());

    // 3. signature of payload - 2 bytes (len) + sig bytes
    let sig_bytes = identity.asym_sec.sign(&buf);
    append_bytes_with_len(&mut buf, &sig_bytes);

    Ok(buf)
}

fn new_payload(action: u8) -> Vec<u8> {
    vec![MAGIC_1, MAGIC_2, action]
}

fn append_bytes_with_len(buf: &mut Vec<u8>, bytes: &[u8]) {
    let len = bytes.len() as u16;
    buf.extend_from_slice(&len.to_be_bytes());
    buf.extend_from_slice(bytes);
}

fn append_str_with_len(buf: &mut Vec<u8>, s: &str) {
    let bytes = s.as_bytes();
    append_bytes_with_len(buf, bytes);
}

fn append_optional_str_with_len(buf: &mut Vec<u8>, opt: &Option<String>) {
    match opt {
        Some(s) => append_str_with_len(buf, s),
        None => buf.extend_from_slice(&0u16.to_be_bytes()),
    };
}
