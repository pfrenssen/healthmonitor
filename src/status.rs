use crate::config::CONFIG;
use axum::http::StatusCode;
use serde::{Deserialize, Serialize};
use std::fmt;

#[derive(Deserialize, Serialize)]
pub struct Status {
    pub state: HealthState,
    pub messages: Vec<String>,
    pub phase: DeploymentPhase,
}

impl Status {
    pub fn new() -> Self {
        Status {
            state: HealthState::Healthy,
            messages: Vec::new(),
            phase: CONFIG.server.phase.clone(),
        }
    }

    pub fn add_message(&mut self, message: String) {
        self.messages.push(message);
    }
}

impl fmt::Display for Status {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let mut status = String::new();
        status.push_str(if self.state == HealthState::Healthy {
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

#[derive(Clone, Debug, PartialEq, Deserialize, Serialize)]
pub enum DeploymentPhase {
    #[serde(rename = "deploying")]
    Deploying,
    #[serde(rename = "online")]
    Online,
}

impl From<crate::cli::DeploymentPhase> for DeploymentPhase {
    fn from(arg: crate::cli::DeploymentPhase) -> Self {
        match arg {
            crate::cli::DeploymentPhase::Deploying => DeploymentPhase::Deploying,
            crate::cli::DeploymentPhase::Online => DeploymentPhase::Online,
        }
    }
}

impl TryFrom<&str> for DeploymentPhase {
    type Error = &'static str;
    fn try_from(value: &str) -> Result<Self, Self::Error> {
        match value.to_lowercase().as_str() {
            "deploying" => Ok(DeploymentPhase::Deploying),
            "online" => Ok(DeploymentPhase::Online),
            _ => Err("Invalid deployment phase"),
        }
    }
}

impl fmt::Display for DeploymentPhase {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let phase = match self {
            DeploymentPhase::Deploying => "deploying",
            DeploymentPhase::Online => "online",
        };
        write!(f, "{}", phase)
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Deserialize, Serialize)]
pub enum HealthState {
    #[serde(rename = "healthy")]
    Healthy,
    #[serde(rename = "unhealthy")]
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
        assert_eq!(status.state, HealthState::Healthy);
        assert!(status.messages.is_empty());
        assert_eq!(status.phase, DeploymentPhase::Online);
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

        status.state = HealthState::Unhealthy;
        status.add_message("Houston, we have a problem".to_string());
        assert_eq!(
            status.to_string(),
            "unhealthy: All systems go, Houston, we have a problem"
        );
    }
}
