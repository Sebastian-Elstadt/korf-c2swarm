mod anti_analysis;
mod identity;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Ensure proper environment, otherwise die
    anti_analysis::check_environment();

    let identity = identity::new();
    println!("identity: {:?}", identity);

    Ok(())
}
