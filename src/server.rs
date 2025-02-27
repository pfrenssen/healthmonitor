use crate::client;
use crate::config::CONFIG;
use crate::status::{HealthState, Status};

use axum::http::StatusCode;
use axum::response::{IntoResponse, Response};
use axum::{
    routing::{get, patch},
    Json, Router,
};
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

        let status = self.status.clone();

        let app = create_router(status.clone());

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

/// Create a new router with the given status. Having this in a separate function allows us to use
/// the router also in tests.
fn create_router(app_status: Arc<Mutex<Status>>) -> Router {
    let status_get = app_status.clone();
    let status_patch = app_status.clone();

    Router::new()
        .route("/status", get(move || status(status_get)))
        .route(
            "/status",
            patch(move |Json(payload)| patch_status(status_patch, payload)),
        )
        .route("/info", get(info))
        .layer(TraceLayer::new_for_http())
}

/// Returns the current health status of the monitored application.
async fn status(status: Arc<Mutex<Status>>) -> Response {
    use serde_json::to_string;

    let status = status.lock().await;
    let response = to_string(&*status).unwrap_or_else(|_| "".to_string());
    let status_code: StatusCode = status.state.into();
    (status_code, response).into_response()
}

/// Patches the status of the monitored application.
async fn patch_status(status: Arc<Mutex<Status>>, payload: Value) -> Response {
    debug!("Receive PATCH request for status: {:?}", payload);
    // Check if the payload is valid for a Status struct, otherwise return a 400 error.
    // Since this is a PATCH request, the payload might be a partial struct.
    // Todo: Check if the payload contains any keys that are not in the Status struct.
    // Todo: Implement setting the deployment_state.

    let mut status = status.lock().await;

    // If the 'health' key is present, check if the value is a valid before updating the status.
    if let Some(health) = payload.get("health") {
        if let Some(health_str) = health.as_str() {
            match HealthState::try_from(health_str) {
                Ok(health_state) => {
                    debug!("Setting health state to: {}", health_state);
                    status.state = health_state
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

    // If the 'message' key is present, append the message.
    if let Some(message) = payload.get("message") {
        if let Some(message_str) = message.as_str() {
            debug!("Appending message: {}", message_str);
            status.add_message(message_str.to_string());
        } else {
            debug!("Invalid message: {:?}", message);
            return (StatusCode::BAD_REQUEST, "Invalid message.").into_response();
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

#[cfg(test)]
mod tests {
    use super::*;
    use axum::body::Body;
    use axum::http::{Request, Response, StatusCode};
    use axum::Router;
    use http_body_util::BodyExt;
    use serde_json::json;
    use std::sync::Arc;
    use tokio::sync::Mutex;
    use tower::ServiceExt;

    async fn setup() -> Router {
        let status = Arc::new(Mutex::new(Status::new()));
        create_router(status.clone())
    }

    #[tokio::test]
    async fn test_status() {
        let app = setup().await;

        let response = app
            .oneshot(
                Request::builder()
                    .uri("/status")
                    .body(Body::empty())
                    .unwrap(),
            )
            .await
            .unwrap();

        assert_eq!(response.status(), StatusCode::OK);
        let expected = r#"{"state":"healthy","messages":[],"phase":"deploying"}"#;
        let body = response.into_body().collect().await.unwrap().to_bytes();
        assert_eq!(&body[..], expected.as_bytes());
    }

    #[tokio::test]
    async fn test_status_unhealthy() {
        let status = Arc::new(Mutex::new(Status::new()));
        let app = create_router(status.clone());

        status.lock().await.state = HealthState::Unhealthy;

        let response = app
            .oneshot(
                Request::builder()
                    .uri("/status")
                    .body(Body::empty())
                    .unwrap(),
            )
            .await
            .unwrap();

        assert_eq!(response.status(), StatusCode::SERVICE_UNAVAILABLE);
    }

    #[tokio::test]
    async fn test_patch_status() {
        let status = Arc::new(Mutex::new(Status::new()));
        let app = create_router(status.clone());
        assert_eq!(status.lock().await.state, HealthState::Healthy);

        let payload = json!({ "health": "unhealthy" });
        let response = get_patch_response(&app, payload).await;
        assert_eq!(response.status(), StatusCode::OK);
        assert_eq!(status.lock().await.state, HealthState::Unhealthy);

        let payload = json!({ "health": "healthy" });
        let response = get_patch_response(&app, payload).await;
        assert_eq!(response.status(), StatusCode::OK);
        assert_eq!(status.lock().await.state, HealthState::Healthy);

        let payload = json!({ "health": "invalid" });
        let response = get_patch_response(&app, payload).await;
        assert_eq!(response.status(), StatusCode::BAD_REQUEST);
        assert_eq!(status.lock().await.state, HealthState::Healthy);

        let payload = json!({ "message": "Test message" });
        let response = get_patch_response(&app, payload).await;
        assert_eq!(response.status(), StatusCode::OK);
        assert_eq!(status.lock().await.messages.len(), 1);
        assert!(status
            .lock()
            .await
            .messages
            .contains(&"Test message".to_string()));
    }

    async fn get_patch_response(app: &Router, payload: Value) -> Response<Body> {
        app.clone()
            .oneshot(
                Request::builder()
                    .method("PATCH")
                    .uri("/status")
                    .header("content-type", "application/json")
                    .body(Body::from(payload.to_string()))
                    .unwrap(),
            )
            .await
            .unwrap()
    }

    #[tokio::test]
    async fn test_info() {
        let app = setup().await;

        let response = app
            .oneshot(Request::builder().uri("/info").body(Body::empty()).unwrap())
            .await
            .unwrap();

        assert_eq!(response.status(), StatusCode::OK);

        let body = response.into_body().collect().await.unwrap().to_bytes();
        let expected = format!(
            "{{\"name\": \"healthmonitor\", \"version\": \"{}\"}}",
            env!("CARGO_PKG_VERSION")
        );
        assert_eq!(&body[..], expected.as_bytes());
    }
}
