use clap::{Parser, Subcommand};

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
}

#[derive(Subcommand, Debug)]
#[command(arg_required_else_help = true)]
pub enum ServerCommands {
    /// Starts the web server.
    Start,
    /// Returns the current status of the web server.
    Status,
}
