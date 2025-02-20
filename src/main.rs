mod cli;
mod server;

use clap::Parser;
use dotenv::dotenv;
use server::Server;
use std::sync::{Arc, Mutex};
use tokio::signal;
use log::{debug, info};

#[tokio::main]
async fn main() {
    dotenv().ok();
    env_logger::init();

    // Print the environment variables
    for (key, value) in std::env::vars() {
        // Check if the key starts with "HEALTHMONITOR"
        if key.starts_with("HEALTHMONITOR") {
            println!("{}: {}", key, value);
        }
    }

    // Parse the CLI arguments.
    let args = cli::Cli::parse();
    debug!("Parsed args: {:?}", args);

    // Shared state to manage server status
    let server = Arc::new(Mutex::new(Server::new()));

    let mut server_started = false;
    match args.command {
        Some(cli::Commands::Server { command }) => {
            match command {
                Some(cli::ServerCommands::Start) => {
                    let mut srv = server.lock().unwrap();
                    server_started = srv.start().await;
                }
                Some(cli::ServerCommands::Stop) => {
                    let mut srv = server.lock().unwrap();
                    srv.stop().await;
                }
                Some(cli::ServerCommands::Status) => {
                    let srv = server.lock().unwrap();
                    if srv.is_running() {
                        println!("running");
                    } else {
                        println!("not running");
                    }
                }
                None => {}
            }
        }
        None => {}
    }

    if !server_started {
        debug!("Exiting.");
        return;
    }

    // The server has been started, keep it running until a Ctrl+C signal is received.
    loop {
        tokio::select! {
            _ = signal::ctrl_c() => {
                info!("Received Ctrl+C, shutting down.");
                let mut srv = server.lock().unwrap();
                if srv.is_running() {
                    srv.stop().await;
                }
                break;
            }
        }
    }
}
