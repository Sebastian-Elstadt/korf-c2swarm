use thiserror::Error;

// -- Reach

#[derive(Error, Debug)]
pub enum DnsReachError {
    #[error(transparent)]
    Resolve(#[from] hickory_resolver::ResolveError),

    #[error("Missing DNS TXT record")]
    MissingRecord,

    #[error(transparent)]
    BadUtf8Encoding(#[from] std::str::Utf8Error),

    #[error("Cannot decode DNS TXT record as Base85")]
    BadBase85Encoding,

    #[error("DNS TXT payload len {0} too short, expected 7 bytes")]
    PayloadTooShort(usize),

    #[error("Unknown communications mode {0} from DNS payload")]
    UnknownCommMode(u8),
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

// -- Registration

#[derive(Error, Debug)]
pub enum RegistrationError {
    #[error(transparent)]
    Reach(#[from] ReachError),

    #[error("Connection has failed: {0:?}")]
    Connection(std::io::Error),

    #[error("Communication has failed: {0:?}")]
    Communication(std::io::Error),

    #[error("Server did not respond to registration")]
    ServerSilence,
    
    #[error("Server did not respond in time. {0}")]
    Timeout(String),

    #[error("{0}")]
    Unknown(String),
}
