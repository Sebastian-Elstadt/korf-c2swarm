mod anti_analysis;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    anti_analysis::check_environment();

    Ok(())
}
