use clap::{Parser, Subcommand};
use std::str::FromStr;

#[derive(Parser, Debug)]
#[command(name = "healthmonitor")]
#[command(author, version, about, long_about = None, arg_required_else_help = true)]
pub struct Cli {
    #[command(subcommand)]
    pub command: Option<Commands>,
}

#[derive(Subcommand, Debug)]
pub enum Commands {
    /// Commands related to the web server.
    Server {
        #[command(subcommand)]
        command: Option<ServerCommands>,
    },
    /// Commands for getting and setting the health state of the monitored application.
    State {
        #[command(subcommand)]
        command: Option<StateCommands>,
    },
    /// Commands for getting and setting the deployment phase.
    Phase {
        #[command(subcommand)]
        command: Option<PhaseCommands>,
    },
    /// Performs a quick health check of the server environment.
    Check,
}

#[derive(Subcommand, Debug)]
#[command(arg_required_else_help = true)]
pub enum ServerCommands {
    /// Starts the web server.
    Start,
    /// Returns the current status of the web server.
    Status,
}

#[derive(Subcommand, Debug)]
#[command(arg_required_else_help = true)]
pub enum StateCommands {
    /// Returns the current health state of the monitored application.
    Get,
    /// Sets the health state of the monitored application.
    Set {
        /// The new health state of the monitored application.
        health_state: HealthState,
        #[arg(long)]
        message: Option<String>,
    },
}

#[derive(Clone, Debug)]
pub enum HealthState {
    Healthy,
    Unhealthy,
}

impl FromStr for HealthState {
    type Err = String;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s.to_lowercase().as_str() {
            "healthy" => Ok(HealthState::Healthy),
            "unhealthy" => Ok(HealthState::Unhealthy),
            _ => Err(format!("Invalid state: {}", s)),
        }
    }
}

#[derive(Subcommand, Debug)]
#[command(arg_required_else_help = true)]
pub enum PhaseCommands {
    /// Returns the current deployment phase of the monitored application.
    Get,
    /// Sets the deployment phase of the monitored application.
    Set {
        /// The new deployment phase of the monitored application.
        phase: DeploymentPhase,
    },
}

#[derive(Clone, Debug)]
pub enum DeploymentPhase {
    Deploying,
    Online,
}

impl FromStr for DeploymentPhase {
    type Err = String;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s.to_lowercase().as_str() {
            "deploying" => Ok(DeploymentPhase::Deploying),
            "online" => Ok(DeploymentPhase::Online),
            _ => Err(format!("Invalid phase: {}", s)),
        }
    }
}
