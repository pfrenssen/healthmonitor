use crate::config::CONFIG;
use crate::status::{HealthState, Status};
/// This module contains an HTTP client that queries our own server.
use std::fmt::Debug;

use log::debug;
use reqwest::Client;
use thiserror::Error;

#[derive(Error, Debug)]
pub enum ClientError {
    #[error("Request error: {0}")]
    Request(#[from] reqwest::Error),
    #[error("JSON deserialization error: {0}")]
    Json(#[from] serde_json::Error),
    #[error("Server error: {0} - {1}")]
    ServerError(reqwest::StatusCode, String),
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
pub async fn get_status() -> Result<Status, ClientError> {
    let json = get("status").await?;
    let status = serde_json::from_str(&json)?;
    Ok(status)
}

/// Sets the health status on the server.
pub async fn set_health_state(
    state: HealthState,
    message: Option<String>,
) -> Result<(), ClientError> {
    debug!(
        "Setting health state: {} with message: {:?}",
        state, message
    );
    let mut json = serde_json::json!({"health": state});
    if let Some(message) = message {
        json["message"] = serde_json::json!(message);
    }
    patch("status", &json.to_string()).await?;
    Ok(())
}

/// Send a GET request to the server.
async fn get(uri: &str) -> Result<String, ClientError> {
    debug!("GET {}", uri);
    let client = Client::new();
    let response = client.get(get_url(uri)).send().await?;
    let body = response.text().await?;
    Ok(body)
}

/// Send a PATCH request to the server.
async fn patch(uri: &str, body: &str) -> Result<(), ClientError> {
    debug!("PATCH {} with body: {}", uri, body);
    let client = Client::new();
    let response = client
        .patch(get_url(uri))
        .header("Content-Type", "application/json")
        .body(body.to_string())
        .send()
        .await?;
    // If we get a 200 OK, return Ok(()), otherwise return an error.
    if response.status().is_success() {
        Ok(())
    } else {
        let status = response.status();
        let body = response.text().await?;
        Err(ClientError::ServerError(status, body))
    }
}

/// Generate the endpoint URL from the given URI.
fn get_url(uri: &str) -> String {
    format!("{}/{}", &CONFIG.server.to_string(), uri)
}
