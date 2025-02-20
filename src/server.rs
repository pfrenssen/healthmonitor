use axum::{
    routing::get,
    Router,
};
use std::sync::Arc;
use tokio::net::TcpListener;
use tokio::sync::Notify;
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

        let listener = TcpListener::bind("127.0.0.1:8080").await.unwrap();
        println!("Listening on {}", listener.local_addr().unwrap());
        self.is_running = true;
        self.handle = Some(tokio::spawn(async move {
            axum::serve(listener, app)
                .await
                .unwrap();
        }));
        println!("Server started");
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

async fn status_handler() -> &'static str {
    "200 OK"
}
