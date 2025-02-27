use crate::checks::file_check::FileCheck;
use crate::checks::HealthCheck;
use crate::status::{DeploymentPhase, Status};
use log::{debug, error};
use std::sync::Arc;
use tokio::sync::Mutex;

pub struct PluginManager {
    plugins: Vec<Arc<dyn HealthCheck + Send + Sync>>,
}

impl PluginManager {
    pub fn new() -> Self {
        Self {
            plugins: create_plugins(),
        }
    }

    /// Starts health monitoring by running checks in a loop.
    pub async fn monitor(&self, status: Arc<Mutex<Status>>) {
        debug!("Running checks");
        for plugin in &self.plugins {
            let plugin = Arc::clone(plugin);
            debug!("Initializing loop for {}", plugin.name());
            let status = status.clone();
            let interval = plugin.interval();
            let plugin_name = plugin.name().to_string();
            tokio::spawn(async move {
                loop {
                    // If the application is in the deployment phase, wait until we are online.
                    let current_status = status.lock().await;
                    if current_status.phase == DeploymentPhase::Deploying {
                        debug!(
                            "Not executing {} because the application is being deployed",
                            plugin_name
                        );
                        drop(current_status);
                        tokio::time::sleep(tokio::time::Duration::from_secs(1)).await;
                        continue;
                    }

                    // Only run the check if the status is healthy.
                    if current_status.state == crate::status::HealthState::Unhealthy {
                        debug!(
                            "Not executing {} because the status is unhealthy",
                            plugin_name
                        );
                        break;
                    }
                    drop(current_status);

                    if let Err(e) = plugin.run().await {
                        status.lock().await.state = crate::status::HealthState::Unhealthy;
                        let message: String = format!("{}: {}", plugin_name, e).to_string();
                        status.lock().await.add_message(message);
                        error!("{} failed: {}", plugin_name, e);
                        break;
                    }

                    debug!("{} succeeded", plugin_name);
                    tokio::time::sleep(tokio::time::Duration::from_secs(interval as u64)).await;
                }
            });
        }
    }

    /// Runs all the quick checks once.
    pub async fn quick_check(&self) -> Result<(), String> {
        debug!("Running quick checks");
        for plugin in &self.plugins {
            if plugin.is_quick_check() {
                plugin.run().await?;
            }
        }
        Ok(())
    }
}

fn create_plugins() -> Vec<Arc<dyn HealthCheck + Send + Sync>> {
    vec![Arc::new(FileCheck::new())]
}
