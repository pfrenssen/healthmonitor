mod checks;
mod cli;
mod client;
mod config;
mod server;
mod status;

use crate::checks::plugin_manager::PluginManager;
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

    // Initialize a new status.
    let status = Arc::new(Mutex::new(Status::new()));

    // Check if the server is already running in another process. If it is, update the status.
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

    match args.command {
        Some(cli::Commands::Server { command }) => match command {
            Some(cli::ServerCommands::Start) => {
                if (start_server().await).is_err() {
                    exit(1);
                }
                return;
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
        Some(cli::Commands::State { command }) => match command {
            Some(cli::StateCommands::Get) => match client::get_status().await {
                Ok(status) => {
                    println!("{}", status);
                    if status.state == HealthState::Unhealthy {
                        exit(1);
                    }
                }
                Err(e) => {
                    eprintln!("Failed to get state: {}", e);
                    exit(1);
                }
            },
            Some(cli::StateCommands::Set {
                health_state,
                message,
            }) => match client::set_health_state(health_state.into(), message).await {
                Ok(_) => {}
                Err(e) => {
                    eprintln!("Failed to set state: {}", e);
                    exit(1);
                }
            },
            None => {}
        },
        Some(cli::Commands::Phase { command }) => match command {
            Some(cli::PhaseCommands::Get) => match client::get_status().await {
                Ok(status) => {
                    println!("{}", status.phase);
                }
                Err(e) => {
                    eprintln!("Failed to get phase: {}", e);
                    exit(1);
                }
            },
            Some(cli::PhaseCommands::Set { phase }) => {
                match client::set_deployment_phase(phase.into()).await {
                    Ok(_) => {}
                    Err(e) => {
                        eprintln!("Failed to set phase: {}", e);
                        exit(1);
                    }
                }
            }
            None => {}
        },
        Some(cli::Commands::Check) => {
            let plugin_manager = PluginManager::new();
            let result = plugin_manager.quick_check().await;
            match result {
                Ok(_) => {
                    println!("ok");
                }
                Err(e) => {
                    println!("error: {}", e);
                    exit(1);
                }
            }
        }
        None => {}
    }

    debug!("Exiting.");
}

async fn start_server() -> Result<(), ()> {
    let status = Arc::new(Mutex::new(Status::new()));
    let server = Arc::new(Mutex::new(Server::new(status.clone())));

    // Start the server.
    server.lock().await.start().await?;

    // Start the plugins that will monitor the health of the system.
    let plugin_manager = PluginManager::new();
    let status_clone = status.clone();
    debug!("Starting health monitoring.");
    tokio::spawn(async move {
        plugin_manager.monitor(status_clone).await;
    });

    // Keep the server running alongside the monitoring threads until a Ctrl+C signal is received.
    let mut sigint = signal(SignalKind::interrupt()).unwrap();
    debug!("Waiting for SIGINT signal.");
    tokio::select! {
        _ = sigint.recv() => {
            info!("Received SIGINT, shutting down.");
            let mut srv = server.lock().await;
            srv.stop().await;
            Ok(())
        }
    }
}
