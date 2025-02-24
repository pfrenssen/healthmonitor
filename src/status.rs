use axum::http::StatusCode;
use serde::{Deserialize, Serialize};
use std::fmt;

#[derive(Deserialize, Serialize)]
pub struct Status {
    pub health: HealthState,
    pub messages: Vec<String>,
    pub deployment_state: DeploymentState,
}

impl Status {
    pub fn new() -> Self {
        Status {
            health: HealthState::Healthy,
            messages: Vec::new(),
            deployment_state: DeploymentState::Deploying,
        }
    }

    pub fn add_message(&mut self, message: String) {
        self.messages.push(message);
    }
}

impl fmt::Display for Status {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let mut status = String::new();
        status.push_str(if self.health == HealthState::Healthy {
            "healthy"
        } else {
            "unhealthy"
        });
        if !self.messages.is_empty() {
            status.push_str(": ");
            status.push_str(&self.messages.join(", "));
        }
        write!(f, "{}", status)
    }
}

#[derive(Debug, PartialEq, Deserialize, Serialize)]
pub enum DeploymentState {
    Deploying,
    Running,
}

#[derive(Clone, Copy, Debug, PartialEq, Deserialize, Serialize)]
pub enum HealthState {
    Healthy,
    Unhealthy,
}

impl From<crate::cli::HealthState> for HealthState {
    fn from(arg: crate::cli::HealthState) -> Self {
        match arg {
            crate::cli::HealthState::Healthy => HealthState::Healthy,
            crate::cli::HealthState::Unhealthy => HealthState::Unhealthy,
        }
    }
}

impl From<HealthState> for StatusCode {
    fn from(health: HealthState) -> Self {
        match health {
            HealthState::Healthy => StatusCode::OK,
            HealthState::Unhealthy => StatusCode::SERVICE_UNAVAILABLE,
        }
    }
}

impl TryFrom<&str> for HealthState {
    type Error = &'static str;
    fn try_from(value: &str) -> Result<Self, Self::Error> {
        match value.to_lowercase().as_str() {
            "healthy" => Ok(HealthState::Healthy),
            "unhealthy" => Ok(HealthState::Unhealthy),
            _ => Err("Invalid health state"),
        }
    }
}

impl fmt::Display for HealthState {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let status = match self {
            HealthState::Healthy => "healthy",
            HealthState::Unhealthy => "unhealthy",
        };
        write!(f, "{}", status)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_health_status_new() {
        let status = Status::new();
        assert_eq!(status.health, HealthState::Healthy);
        assert!(status.messages.is_empty());
        assert_eq!(status.deployment_state, DeploymentState::Deploying);
    }

    #[test]
    fn test_add_message() {
        let mut status = Status::new();
        status.add_message("Test message".to_string());
        assert_eq!(status.messages.len(), 1);
        assert_eq!(status.messages[0], "Test message");
    }

    #[test]
    fn test_to_string() {
        let mut status = Status::new();
        assert_eq!(status.to_string(), "healthy");

        status.add_message("All systems go".to_string());
        assert_eq!(status.to_string(), "healthy: All systems go");

        status.health = HealthState::Unhealthy;
        status.add_message("Houston, we have a problem".to_string());
        assert_eq!(
            status.to_string(),
            "unhealthy: All systems go, Houston, we have a problem"
        );
    }
}
