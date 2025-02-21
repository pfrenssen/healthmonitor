mod cli;
mod config;
mod server;
mod status;

use std::process::exit;
use clap::Parser;
use config::CONFIG;
use dotenv::dotenv;
use log::{debug, info};
use server::Server;
use std::sync::{Arc, Mutex};
use tokio::signal::unix::{signal, SignalKind};
use crate::status::HealthStatus;

#[tokio::main]
async fn main() {
    dotenv().ok();
    env_logger::init();

    debug!("Config: {:?}", *CONFIG);

    // Parse the CLI arguments.
    let args = cli::Cli::parse();
    debug!("Parsed args: {:?}", args);

    let status = Arc::new(Mutex::new(HealthStatus::new()));
    let server = Arc::new(Mutex::new(Server::new(status.clone())));

    let mut server_started = false;
    match args.command {
        Some(cli::Commands::Server { command }) => {
            match command {
                Some(cli::ServerCommands::Start) => {
                    let mut srv = server.lock().unwrap();
                    match srv.start().await {
                        Ok(_) => server_started = true,
                        Err(_) => exit(1),
                    }
                }
                Some(cli::ServerCommands::Status) => {
                    let srv = server.lock().unwrap();
                    if srv.is_running().await {
                        println!("running");
                    } else {
                        println!("not running");
                        exit(1);
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
    let mut sigint = signal(SignalKind::interrupt()).unwrap();
    debug!("Waiting for SIGINT signal");
    loop {
        tokio::select! {
            _ = sigint.recv() => {
                info!("Received SIGINT, shutting down.");
                let mut srv = server.lock().unwrap();
                srv.stop().await;
                break;
            }
            _ = toggle_status_periodically(status.clone()) => {}
        }
    }
}

async fn toggle_status_periodically(status: Arc<Mutex<HealthStatus>>) {
    loop {
        {
            let mut status = status.lock().unwrap();
            status.healthy = !status.healthy;
            info!("Toggled status to {}", status.to_string());
        }
        tokio::time::sleep(tokio::time::Duration::from_secs(5)).await;
    }
}
