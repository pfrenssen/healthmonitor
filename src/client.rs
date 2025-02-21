/// This module contains an HTTP client that queries our own server.
use crate::config::CONFIG;
use crate::status::HealthStatus;

use log::debug;
use reqwest::Client;
use thiserror::Error;

#[derive(Error, Debug)]
pub enum ClientError {
    #[error("Request error: {0}")]
    Request(#[from] reqwest::Error),
    #[error("JSON deserialization error: {0}")]
    Json(#[from] serde_json::Error),
}

/// Check if the server is running on the configured address and port.
pub async fn is_running() -> bool {
    if let Ok(body) = get("info").await {
        let name = env!("CARGO_PKG_NAME");
        let version = env!("CARGO_PKG_VERSION");
        let expected_body = format!("{{\"name\": \"{}\", \"version\": \"{}\"}}", name, version);
        return body == expected_body;
    }

    false
}

/// Retrieve the health status from the server.
pub async fn get_status() -> Result<HealthStatus, ClientError> {
    let json = get("status").await?;
    let status = serde_json::from_str(&json)?;
    Ok(status)
}

/// Send a GET request to the server.
async fn get(uri: &str) -> Result<String, ClientError> {
    debug!("GET {}", uri);
    let addr = format!(
        "http://{}:{}/{}",
        CONFIG.server.address, CONFIG.server.port, uri
    );
    let client = Client::new();
    let response = client.get(&addr).send().await?;
    let body = response.text().await?;
    Ok(body)
}
