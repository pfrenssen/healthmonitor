use std::sync::LazyLock;
use std::{env, fmt};

pub struct Config {
    pub server: ServerConfig,
}

impl Config {
    pub fn new() -> Self {
        let address =
            env::var("HEALTHMONITOR_SERVER_ADDRESS").unwrap_or_else(|_| "127.0.0.1".to_string());
        let port = env::var("HEALTHMONITOR_SERVER_PORT")
            .ok()
            .and_then(|p| p.parse().ok())
            .unwrap_or(8080);

        Config {
            server: ServerConfig { address, port },
        }
    }
}

impl fmt::Debug for Config {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("Config")
            .field("server", &self.server)
            .finish()
    }
}

pub struct ServerConfig {
    pub address: String,
    pub port: u16,
}

impl fmt::Debug for ServerConfig {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("ServerConfig")
            .field("address", &self.address)
            .field("port", &self.port)
            .finish()
    }
}

pub static CONFIG: LazyLock<Config> = LazyLock::new(Config::new);
