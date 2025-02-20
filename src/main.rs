mod cli;

use clap::Parser;
use dotenv::dotenv;

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

    println!("args: {:?}", args);

}
