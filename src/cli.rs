use clap::{Parser, Subcommand};

#[derive(Parser, Debug)]
#[command(name = "healthmonitor")]
#[command(author, version, about, long_about = None, arg_required_else_help = true)]
pub struct Cli {
    #[command(subcommand)]
    command: Option<Commands>,
}

#[derive(Subcommand, Debug)]
enum Commands {
    /// Commands related to the web server.
    Server {
        #[command(subcommand)]
        command: Option<ServerCommands>,
    },
}

#[derive(Subcommand, Debug)]
#[command(arg_required_else_help = true)]
enum ServerCommands {
    /// Starts the web server.
    Start,
    /// Stops the web server.
    Stop,
    /// Returns the current status of the web server.
    Status,
}
