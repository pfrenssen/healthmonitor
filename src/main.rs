mod cli;
mod server;

use clap::Parser;
use dotenv::dotenv;
use server::Server;
use std::sync::{Arc, Mutex};
use tokio::sync::Notify;

#[tokio::main]
async fn main() {
    dotenv().ok();

    // Print the environment variables
    for (key, value) in std::env::vars() {
        // Check if the key starts with "HEALTHMONITOR"
        if key.starts_with("HEALTHMONITOR") {
            println!("{}: {}", key, value);
        }
    }

    // Parse the CLI arguments.
    let args = cli::Cli::parse();

    // Shared state to manage server status
    let server = Arc::new(Mutex::new(Server::new()));
    let notify = Arc::new(Notify::new());

    match args.command {
        Some(cli::Commands::Server { command }) => {
            match command {
                Some(cli::ServerCommands::Start) => {
                    let mut srv = server.lock().unwrap();
                    if srv.is_running() {
                        println!("Server is already running.");
                    } else {
                        srv.start(notify.clone()).await;
                        println!("Server started.");
                    }
                }
                Some(cli::ServerCommands::Stop) => {
                    let mut srv = server.lock().unwrap();
                    if srv.is_running() {
                        srv.stop().await;
                        println!("Server stopped.");
                    } else {
                        println!("Server is not running.");
                    }
                }
                Some(cli::ServerCommands::Status) => {
                    let srv = server.lock().unwrap();
                    if srv.is_running() {
                        println!("running");
                    } else {
                        println!("not running");
                    }
                }
                None => {
                    println!("No server command provided.");
                }
            }
        }
        None => {
            println!("No command provided.");
        }
    }
}
