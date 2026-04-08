fn main() {
    println!("result: {}", build_dns_hint())
}

fn build_dns_hint() -> String {
    // C2 IPv4 address: 4 bytes - if 0.0.0.0, then localhost (which is during dev/demo)
    let port: u16 = 8888;
    let port_bytes = port.to_be_bytes();
    let bytes: [u8; 7] = [0, 0, 0, 0, port_bytes[0], port_bytes[1], 0x01];

    base85::encode(&bytes)
}
