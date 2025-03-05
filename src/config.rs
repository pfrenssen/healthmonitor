use crate::status::DeploymentPhase;
use std::sync::LazyLock;
use std::{env, fmt};

pub struct Config {
    pub server: ServerConfig,
    pub checks: ChecksConfig,
}

impl Config {
    pub fn new() -> Self {
        let scheme = env::var("HEALTHMONITOR_SERVER_SCHEME").unwrap_or_else(|_| "http".to_string());
        let address =
            env::var("HEALTHMONITOR_SERVER_ADDRESS").unwrap_or_else(|_| "127.0.0.1".to_string());
        let port = env::var("HEALTHMONITOR_SERVER_PORT")
            .ok()
            .and_then(|p| p.parse().ok())
            .unwrap_or(8080);
        let phase = DeploymentPhase::try_from(
            env::var("HEALTHMONITOR_SERVER_PHASE")
                .unwrap_or_else(|_| "online".to_string())
                .as_str(),
        )
        .unwrap_or(DeploymentPhase::Online);

        let file_check_interval = env::var("HEALTHMONITOR_FILECHECK_INTERVAL")
            .ok()
            .and_then(|p| p.parse().ok())
            .unwrap_or(30);
        let file_check_files = env::var("HEALTHMONITOR_FILECHECK_FILES")
            .unwrap_or_else(|_| "".to_string())
            .split(',')
            .filter(|s| !s.trim().is_empty())
            .map(|s| s.trim().to_string())
            .collect();

        let url_check_interval = env::var("HEALTHMONITOR_URLCHECK_INTERVAL")
            .ok()
            .and_then(|p| p.parse().ok())
            .unwrap_or(30);
        let url_check_urls = env::var("HEALTHMONITOR_URLCHECK_URLS")
            .unwrap_or_else(|_| "".to_string())
            .split(',')
            .filter(|s| !s.trim().is_empty())
            .map(|s| s.trim().to_string())
            .collect();
        let url_check_timeout = env::var("HEALTHMONITOR_URLCHECK_TIMEOUT")
            .ok()
            .and_then(|p| p.parse().ok())
            .unwrap_or(10);

        Config {
            server: ServerConfig {
                scheme,
                address,
                port,
                phase,
            },
            checks: ChecksConfig {
                file_check: FileCheckConfig {
                    interval: file_check_interval,
                    files: file_check_files,
                },
                url_check: UrlCheckConfig {
                    interval: url_check_interval,
                    urls: url_check_urls,
                    timeout: url_check_timeout,
                },
            },
        }
    }
}

impl fmt::Debug for Config {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("Config")
            .field("server", &self.server)
            .field("checks", &self.checks)
            .finish()
    }
}

pub struct ServerConfig {
    pub scheme: String,
    pub address: String,
    pub port: u16,
    pub phase: DeploymentPhase,
}

impl fmt::Debug for ServerConfig {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("ServerConfig")
            .field("scheme", &self.scheme)
            .field("address", &self.address)
            .field("port", &self.port)
            .finish()
    }
}

impl fmt::Display for ServerConfig {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}://{}:{}", self.scheme, self.address, self.port)
    }
}

#[derive(Debug)]
pub struct ChecksConfig {
    pub file_check: FileCheckConfig,
    pub url_check: UrlCheckConfig,
}

#[derive(Debug)]
pub struct FileCheckConfig {
    pub interval: usize,
    pub files: Vec<String>,
}

#[derive(Debug)]
pub struct UrlCheckConfig {
    pub interval: usize,
    pub urls: Vec<String>,
    pub timeout: usize,
}

pub static CONFIG: LazyLock<Config> = LazyLock::new(Config::new);
