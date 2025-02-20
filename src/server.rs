use std::sync::{Arc, Mutex};
use axum::{
    routing::get,
    Router,
};
use crate::config::CONFIG;
use log::{debug, error, info, warn};
use tokio::net::TcpListener;
use tokio::task::JoinHandle;
use tower_http::trace::TraceLayer;

pub struct Server {
    is_running: bool,
    handle: Option<JoinHandle<()>>,
    status: Arc<Mutex<bool>>,
}

impl Server {
    pub fn new(status: Arc<Mutex<bool>>) -> Self {
        Server {
            is_running: false,
            handle: None,
            status,
        }
    }

    pub fn is_running(&self) -> bool {
        self.is_running
    }

    pub async fn start(&mut self) -> Result<(), ()> {
        if self.is_running {
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
        self.is_running = true;
        self.handle = Some(tokio::spawn(async move {
            axum::serve(listener, app)
                .await
                .unwrap();
        }));
        info!("Server started.");
        Ok(())
    }

    pub async fn stop(&mut self) {
        if !self.is_running {
            return;
        }

        // Implement graceful shutdown logic here
        // For simplicity, we're not implementing it fully
        self.is_running = false;

        if let Some(handle) = self.handle.take() {
            handle.abort();
        }
        println!("Server stopped.");
    }
}

/// Returns the current health status of the monitored application.
async fn status(status: Arc<Mutex<bool>>) -> String {
    let status = status.lock().unwrap();
    if *status {
        "Healthy".to_string()
    } else {
        "Unhealthy".to_string()
    }
}

/// Returns the application name and version.
async fn info() -> String {
    let name = env!("CARGO_PKG_NAME");
    let version = env!("CARGO_PKG_VERSION");
    format!("{{\"name\": \"{}\", \"version\": \"{}\"}}", name, version)
}