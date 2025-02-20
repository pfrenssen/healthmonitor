mod cli;
mod server;
mod config;

use clap::Parser;
use config::CONFIG;
use dotenv::dotenv;
use log::{debug, info};
use server::Server;
use std::sync::{Arc, Mutex};
use tokio::signal;

#[tokio::main]
async fn main() {
    dotenv().ok();
    env_logger::init();

    debug!("Config: {:?}", *CONFIG);

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
