mod file_check;
pub mod plugin_manager;

use async_trait::async_trait;

#[async_trait]
pub trait HealthCheck {
    fn name(&self) -> &str;
    fn interval(&self) -> usize;
    fn is_quick_check(&self) -> bool;
    async fn run(&self) -> Result<(), String>;
}
