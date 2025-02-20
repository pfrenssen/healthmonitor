use axum::{
    routing::get,
    Router,
};
use log::info;
use tokio::net::TcpListener;
use tokio::task::JoinHandle;
use tower_http::trace::TraceLayer;

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

    pub async fn start(&mut self) -> bool {
        if self.is_running {
            info!("Server is already running");
            return false;
        }

        let app = Router::new()
            .route("/status", get(status_handler))
            .layer(TraceLayer::new_for_http());

        let listener = TcpListener::bind("127.0.0.1:8080").await.unwrap();
        info!("Listening on {}", listener.local_addr().unwrap());
        self.is_running = true;
        self.handle = Some(tokio::spawn(async move {
            axum::serve(listener, app)
                .await
                .unwrap();
        }));
        true
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
