use serde::{Deserialize, Serialize};

#[derive(Deserialize, Serialize)]
pub struct HealthStatus {
    pub healthy: bool,
    pub messages: Vec<String>,
    pub deployment_state: DeploymentState,
}

impl HealthStatus {
    pub fn new() -> Self {
        HealthStatus {
            healthy: true,
            messages: Vec::new(),
            deployment_state: DeploymentState::Deploying,
        }
    }

    pub fn add_message(&mut self, message: String) {
        self.messages.push(message);
    }

    pub fn set_unhealthy(&mut self, message: String) {
        self.healthy = false;
        self.add_message(message);
    }

    /// Returns the health status as a human-readable string.
    pub fn to_string(&self) -> String {
        let mut status = String::new();
        status.push_str(if self.healthy { "healthy" } else { "unhealthy" });
        if !self.messages.is_empty() {
            status.push_str(": ");
            status.push_str(&self.messages.join(", "));
        }
        status
    }
}

#[derive(Debug, PartialEq, Deserialize, Serialize)]
pub enum DeploymentState {
    Deploying,
    Running,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_health_status_new() {
        let status = HealthStatus::new();
        assert!(status.healthy);
        assert!(status.messages.is_empty());
        assert_eq!(status.deployment_state, DeploymentState::Deploying);
    }

    #[test]
    fn test_add_message() {
        let mut status = HealthStatus::new();
        status.add_message("Test message".to_string());
        assert_eq!(status.messages.len(), 1);
        assert_eq!(status.messages[0], "Test message");
    }

    #[test]
    fn test_set_unhealthy() {
        let mut status = HealthStatus::new();
        status.set_unhealthy("Error occurred".to_string());
        assert!(!status.healthy);
        assert_eq!(status.messages.len(), 1);
        assert_eq!(status.messages[0], "Error occurred");
    }

    #[test]
    fn test_to_string() {
        let mut status = HealthStatus::new();
        assert_eq!(status.to_string(), "healthy");

        status.add_message("All systems go".to_string());
        assert_eq!(status.to_string(), "healthy: All systems go");

        status.set_unhealthy("Houston, we have a problem".to_string());
        assert_eq!(status.to_string(), "unhealthy: All systems go, Houston, we have a problem");
    }
}