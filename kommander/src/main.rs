fn main() {
    println!("result: {}", build_dns_hint())
}

fn build_dns_hint() -> String {
    // C2 IPv4 address: 4 bytes - if 0.0.0.0, then localhost (which is during dev/demo)
    let bytes: [u8; 4] = [0, 0, 0, 0];

    base85::encode(&bytes)
}
