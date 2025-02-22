use std::fmt;
use serde::{Deserialize, Serialize};

#[derive(Deserialize, Serialize)]
// Todo: We should make the health field private and provide accessors, so we can propagate the
//   health status changes to the server.
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

    pub fn set_unhealthy(&mut self, message: String) {
        self.health = HealthState::Unhealthy;
        self.add_message(message);
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

#[derive(Debug, PartialEq, Deserialize, Serialize)]
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
    fn test_set_unhealthy() {
        let mut status = Status::new();
        status.set_unhealthy("Error occurred".to_string());
        assert_eq!(status.health, HealthState::Unhealthy);
        assert_eq!(status.messages.len(), 1);
        assert_eq!(status.messages[0], "Error occurred");
    }

    #[test]
    fn test_to_string() {
        let mut status = Status::new();
        assert_eq!(status.to_string(), "healthy");

        status.add_message("All systems go".to_string());
        assert_eq!(status.to_string(), "healthy: All systems go");

        status.set_unhealthy("Houston, we have a problem".to_string());
        assert_eq!(
            status.to_string(),
            "unhealthy: All systems go, Houston, we have a problem"
        );
    }
}
