use crate::config::CONFIG;
use crate::status::HealthStatus;

use axum::{routing::get, Router};
use log::{debug, error, info, warn};
use reqwest::Client;
use std::sync::Arc;
use tokio::net::TcpListener;
use tokio::sync::Mutex;
use tokio::task::JoinHandle;
use tower_http::trace::TraceLayer;

pub struct Server {
    handle: Option<JoinHandle<()>>,
    status: Arc<Mutex<HealthStatus>>,
}

impl Server {
    pub fn new(status: Arc<Mutex<HealthStatus>>) -> Self {
        Server {
            handle: None,
            status,
        }
    }

    pub async fn start(&mut self) -> Result<(), ()> {
        if self.is_running().await {
            warn!("Server is already running.");
            return Err(());
        }

        let status_clone = self.status.clone();

        let app = Router::new()
            .route("/status", get(move || status(status_clone.clone())))
            .route("/info", get(info))
            .layer(TraceLayer::new_for_http());

        let addr = format!("{}:{}", CONFIG.server.address, CONFIG.server.port);
        debug!("Connecting to {}", addr);
        let listener = match TcpListener::bind(&addr).await {
            Ok(listener) => listener,
            Err(e) => {
                error!("Failed to bind to {}: {}", addr, e);
                return Err(());
            }
        };
        self.handle = Some(tokio::spawn(async move {
            axum::serve(listener, app)
                .await
                .unwrap();
        }));
        info!("Server started.");
        Ok(())
    }

    pub async fn stop(&mut self) {
        if !self.is_running().await {
            return;
        }

        if let Some(handle) = self.handle.take() {
            handle.abort();
        }
        info!("Server stopped.");
    }

    pub async fn is_running(&self) -> bool {
        if self.handle.is_some() {
            return true;
        }

        let addr = format!("http://{}:{}/info", CONFIG.server.address, CONFIG.server.port);
        let client = Client::new();

        if let Ok(response) = client.get(&addr).send().await {
            if response.status().is_success() {
                if let Ok(body) = response.text().await {
                    let name = env!("CARGO_PKG_NAME");
                    let version = env!("CARGO_PKG_VERSION");
                    let expected_body = format!("{{\"name\": \"{}\", \"version\": \"{}\"}}", name, version);
                    return body == expected_body;
                }
            }
        }

        false
    }
}

/// Returns the current health status of the monitored application.
async fn status(status: Arc<Mutex<HealthStatus>>) -> String {
    use serde_json::to_string;

    let status = status.lock().await;
    to_string(&*status).unwrap()
}

/// Returns the application name and version.
async fn info() -> String {
    let name = env!("CARGO_PKG_NAME");
    let version = env!("CARGO_PKG_VERSION");
    format!("{{\"name\": \"{}\", \"version\": \"{}\"}}", name, version)
}
