mod cli;
mod client;
mod config;
mod server;
mod status;

use crate::status::{HealthState, Status};

use clap::Parser;
use config::CONFIG;
use dotenv::dotenv;
use log::{debug, info};
use server::Server;
use std::process::exit;
use std::sync::Arc;
use tokio::signal::unix::{signal, SignalKind};
use tokio::sync::Mutex;

#[tokio::main]
async fn main() {
    dotenv().ok();
    env_logger::init();

    debug!("Config: {:?}", *CONFIG);

    // Parse the CLI arguments.
    let args = cli::Cli::parse();
    debug!("Parsed args: {:?}", args);

    // Initialize the server and status.
    // Todo: Rethink how to use the status. Currently we have a local status and a status in the
    //   server. Probably we should only rely on the server status and fully communicate over the
    //   client.
    let status = Arc::new(Mutex::new(Status::new()));
    let server = Arc::new(Mutex::new(Server::new(status.clone())));

    // Check if the server is already running in another process. If it is, update the status.
    // Todo: integrate the status of our running server in the HealthStatus struct?
    debug!("Checking if our server is already running.");
    let mut server_running = false;
    if client::is_running().await {
        if let Ok(s) = client::get_status().await {
            let mut status = status.lock().await;
            *status = s;
            server_running = true;
        }
    }
    debug!(
        "Health monitor server is {}. Application is {}.",
        if server_running { "online" } else { "offline" },
        status.lock().await.to_string()
    );

    // Todo: We can keep track of the server status in an enum: Offline, Online, Started.
    let mut server_started = false;
    match args.command {
        Some(cli::Commands::Server { command }) => match command {
            Some(cli::ServerCommands::Start) => {
                let mut srv = server.lock().await;
                match srv.start().await {
                    Ok(_) => server_started = true,
                    Err(_) => exit(1),
                }
            }
            Some(cli::ServerCommands::Status) => {
                if server_running {
                    println!("running");
                } else {
                    println!("not running");
                    exit(1);
                }
            }
            None => {}
        },
        Some(cli::Commands::Status { command }) => match command {
            Some(cli::StatusCommands::Get) => match client::get_status().await {
                Ok(status) => {
                    println!("{}", status);
                    if status.health == HealthState::Unhealthy {
                        exit(1);
                    }
                }
                Err(e) => {
                    eprintln!("Failed to get status: {}", e);
                    exit(1);
                }
            },
            Some(cli::StatusCommands::Set {
                health_state,
                message,
            }) => match client::set_health_state(health_state.into(), message).await {
                Ok(_) => {}
                Err(e) => {
                    eprintln!("Failed to set status: {}", e);
                    exit(1);
                }
            },
            None => {}
        },
        None => {}
    }

    if !server_started {
        debug!("Exiting.");
        return;
    }

    // The server has been started, keep it running alongside the monitoring threads until a Ctrl+C
    // signal is received.
    let mut sigint = signal(SignalKind::interrupt()).unwrap();
    debug!("Waiting for SIGINT signal");
    loop {
        tokio::select! {
            _ = sigint.recv() => {
                info!("Received SIGINT, shutting down.");
                let mut srv = server.lock().await;
                srv.stop().await;
                break;
            }
            _ = toggle_status_periodically(status.clone()) => {}
        }
    }
}

async fn toggle_status_periodically(status: Arc<Mutex<Status>>) {
    loop {
        {
            tokio::time::sleep(tokio::time::Duration::from_secs(5)).await;
            let mut status = status.lock().await;
            status.health = if status.health == HealthState::Healthy {
                HealthState::Unhealthy
            } else {
                HealthState::Healthy
            };
            info!("Toggled status to {}", status.to_string());
        }
    }
}
