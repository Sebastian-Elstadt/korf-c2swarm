use asymsec::AsymSecHandler;
use sha2::{Digest, Sha256};
use std::time::{SystemTime, UNIX_EPOCH};

/**
 * Scrapes identification info from the host device and uses the hash of it as the nodus ID.
 */

pub struct Identity {
    pub nodus_id: String,
    pub cpu_arch: String,
    pub hostname: Option<String>,
    pub username: Option<String>,
    pub account_name: Option<String>,
    pub device_name: Option<String>,
    pub mac_addr: Option<String>,
    pub asym_sec: Box<dyn AsymSecHandler>,
}

pub fn init() -> Identity {
    println!("+--- scraping id ---+");
    let cpu_arch = whoami::cpu_arch().to_string();
    let hostname = whoami::hostname().ok();
    let username = whoami::username().ok();
    let account_name = whoami::account().ok();
    let device_name = whoami::devicename().ok();
    let mac_addr = mac_address::get_mac_address()
        .ok()
        .flatten()
        .map(|mac| mac.to_string());

    let mut hash = Sha256::new();
    for detail in [
        &Some(cpu_arch.clone()),
        &hostname,
        &username,
        &account_name,
        &device_name,
        &mac_addr,
    ] {
        if let Some(str) = detail {
            hash.update(str.as_bytes());
        } else {
            hash.update(b"?");
        }

        hash.update(b"\0");
    }

    let time_since_epoch = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .expect("Failure determing time since epoch");
    hash.update(time_since_epoch.as_millis().to_le_bytes());
    hash.update(b"\0korf-nodus");
    let nodus_id = base85::encode(&hash.finalize());

    println!();
    Identity {
        nodus_id,
        cpu_arch,
        hostname,
        username,
        account_name,
        device_name,
        mac_addr,
        asym_sec: asymsec::get_implementation(),
    }
}
