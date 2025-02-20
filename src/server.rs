use axum::{
    routing::get,
    Router,
};
use std::{net::SocketAddr, sync::Arc};
use tokio::sync::Mutex;
use tokio::task::JoinHandle;

pub struct Server {
    is_running: bool,
    handle: Option<JoinHandle<()>>,
}

impl Server {
    pub fn new() -> Self {
        Server {
            is_running: false,
            handle: None,
        }
    }

    pub fn is_running(&self) -> bool {
        self.is_running
    }

    pub async fn start(&mut self, notify: Arc<Notify>) {
        if self.is_running {
            return;
        }

        let app = Router::new().route("/status", get(status_handler));

        let addr = SocketAddr::from(([127, 0, 0, 1], 8080));
        self.is_running = true;

        let handle = tokio::spawn(async move {
            axum::Server::bind(&addr)
                .serve(app.into_make_service())
                .with_graceful_shutdown(shutdown_signal(notify))
                .await
                .unwrap();
        });

        self.handle = Some(handle);
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
    }
}

async fn status_handler() -> &'static str {
    "200 OK"
}

async fn shutdown_signal(notify: Arc<Notify>) {
    notify.notified().await;
}
