use std::net::SocketAddr;

use tokio::net::UdpSocket;

use crate::db::{self, PgPool};
use crate::node::protocol;

pub async fn run(pool: PgPool, bind: SocketAddr) -> Result<(), std::io::Error> {
    let sock = UdpSocket::bind(bind).await?;
    let mut buf = [0u8; 65535];

    loop {
        let (len, addr) = sock.recv_from(&mut buf).await?;
        let data = &buf[..len];

        if data.len() < 3 || data[0] != protocol::MAGIC_0 || data[1] != protocol::MAGIC_1 {
            continue;
        }

        if data[2] == protocol::MSG_REGISTER {
            match protocol::parse_registration(data) {
                Ok(reg) => match db::upsert_registration(&pool, &reg).await {
                    Ok(()) => {
                        if let Err(e) = sock.send_to(b"ACK", addr).await {
                            eprintln!("udp send ACK failed: {e}");
                        }
                    }
                    Err(e) => eprintln!("database error (registration): {e}"),
                },
                Err(e) => eprintln!("registration parse error: {e}"),
            }
        } else if sock.send_to(b"ACK", addr).await.is_err() {
            eprintln!("udp send ACK failed for non-register message");
        }
    }
}
