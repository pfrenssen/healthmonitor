use serde::Serialize;

#[derive(Serialize)]
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
        status.push_str(if self.healthy { "Healthy" } else { "Unhealthy" });
        if !self.messages.is_empty() {
            status.push_str(": ");
            status.push_str(&self.messages.join(", "));
        }
        status
    }
}

#[derive(Serialize)]
pub enum DeploymentState {
    Deploying,
    Running,
}
