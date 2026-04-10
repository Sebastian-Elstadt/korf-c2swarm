use std::net::Ipv4Addr;

pub enum ReachCommMode {
    HTTP,
    UDP,
}

pub struct ReachInfo {
    pub ipv4: Ipv4Addr,
    pub port: u16,
    pub comm_mode: ReachCommMode,
}