use crate::client;
use crate::config::CONFIG;
use crate::status::{HealthState, Status};

use axum::http::StatusCode;
use axum::response::{IntoResponse, Response};
use axum::routing::patch;
use axum::{routing::get, Json, Router};
use log::{debug, error, info, warn};
use serde_json::Value;
use std::sync::Arc;
use tokio::net::TcpListener;
use tokio::sync::Mutex;
use tokio::task::JoinHandle;
use tower_http::trace::TraceLayer;

pub struct Server {
    handle: Option<JoinHandle<()>>,
    status: Arc<Mutex<Status>>,
}

impl Server {
    pub fn new(status: Arc<Mutex<Status>>) -> Self {
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

        let status_get = self.status.clone();
        let status_patch = self.status.clone();

        let app = Router::new()
            .route("/status", get(move || status(status_get.clone())))
            .route(
                "/status",
                patch(move |Json(payload)| patch_status(status_patch, payload)),
            )
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
            axum::serve(listener, app).await.unwrap();
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

    async fn is_running(&self) -> bool {
        if self.handle.is_some() {
            return true;
        }

        client::is_running().await
    }
}

/// Returns the current health status of the monitored application.
async fn status(status: Arc<Mutex<Status>>) -> String {
    use serde_json::to_string;

    let status = status.lock().await;
    to_string(&*status).unwrap()
}

/// Patches the status of the monitored application.
async fn patch_status(status: Arc<Mutex<Status>>, payload: Value) -> Response {
    debug!("Receive PATCH request for status: {:?}", payload);
    // Check if the payload is valid for a Status struct, otherwise return a 400 error.
    // Since this is a PATCH request, the payload might be a partial struct.
    // Todo: Check if the payload contains any keys that are not in the Status struct.
    // Todo: Implement 'message' and 'deployment_state' keys.

    let mut status = status.lock().await;

    // If the 'health' key is present, check if the value is a valid before updating the status.
    if let Some(health) = payload.get("health") {
        if let Some(health_str) = health.as_str() {
            match HealthState::try_from(health_str) {
                Ok(health_state) => {
                    debug!("Setting health state to: {}", health_state);
                    status.health = health_state
                }
                Err(err) => {
                    debug!("Invalid health state: {}", err);
                    return (StatusCode::BAD_REQUEST, err).into_response();
                }
            }
        } else {
            debug!("Invalid health state: {:?}", health);
            return (StatusCode::BAD_REQUEST, "Invalid health state.").into_response();
        }
    }

    (StatusCode::OK, "Status updated.").into_response()
}

/// Returns the application name and version.
async fn info() -> String {
    let name = env!("CARGO_PKG_NAME");
    let version = env!("CARGO_PKG_VERSION");
    format!("{{\"name\": \"{}\", \"version\": \"{}\"}}", name, version)
}
