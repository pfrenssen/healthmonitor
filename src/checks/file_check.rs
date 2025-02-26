use crate::checks::HealthCheck;
use async_trait::async_trait;
use log::debug;
use std::fs;

pub struct FileCheck {
    name: &'static str,
    interval: usize,
    is_quick_check: bool,
    files: Vec<String>,
}

impl FileCheck {
    pub fn new() -> Self {
        Self {
            name: "FileCheck",
            interval: std::env::var("HEALTHMONITOR_FILECHECK_INTERVAL")
                .unwrap_or("30".to_string())
                .parse()
                .unwrap_or(30),
            is_quick_check: true,
            files: std::env::var("HEALTHMONITOR_FILECHECK_FILES")
                .unwrap_or_else(|_| "".to_string())
                .split(',')
                .filter(|s| !s.trim().is_empty())
                .map(|s| s.trim().to_string())
                .collect(),
        }
    }
}

#[async_trait]
impl HealthCheck for FileCheck {
    fn name(&self) -> &str {
        self.name
    }

    fn interval(&self) -> usize {
        self.interval
    }

    fn is_quick_check(&self) -> bool {
        self.is_quick_check
    }

    async fn run(&self) -> Result<(), String> {
        debug!("Running checks");

        for file in &self.files {
            let metadata =
                fs::metadata(file).map_err(|e| format!("Failed to access {}: {}", file, e))?;
            if metadata.len() == 0 {
                return Err(format!("File {} is empty", file));
            }
        }
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use serial_test::serial;
    use std::env;
    use std::io::Write;
    use tempfile::NamedTempFile;

    #[test]
    fn test_name() {
        let check = FileCheck::new();
        assert_eq!(check.name(), "FileCheck");
    }

    #[test]
    fn test_is_quick_check() {
        let check = FileCheck::new();
        assert!(check.is_quick_check());
    }

    #[test]
    #[serial] // Tests that manipulate environment variables should not run concurrently.
    fn test_interval_from_env() {
        env::set_var("HEALTHMONITOR_FILECHECK_INTERVAL", "45");
        let check = FileCheck::new();
        assert_eq!(check.interval(), 45);
        env::remove_var("HEALTHMONITOR_FILECHECK_INTERVAL");
    }

    #[test]
    #[serial] // Tests that manipulate environment variables should not run concurrently.
    fn test_interval_default() {
        env::remove_var("HEALTHMONITOR_FILECHECK_INTERVAL");
        let check = FileCheck::new();
        assert_eq!(check.interval(), 30);
    }

    #[tokio::test]
    #[serial] // Tests that manipulate environment variables should not run concurrently.
    async fn test_run_env_var_not_set() {
        env::remove_var("HEALTHMONITOR_FILECHECK_FILES");
        let check = FileCheck::new();
        assert!(check.run().await.is_ok());
    }

    #[tokio::test]
    #[serial] // Tests that manipulate environment variables should not run concurrently.
    async fn test_run_single_existing_non_empty_file() {
        let mut file = NamedTempFile::new().unwrap();
        writeln!(file, "Test").unwrap();
        env::set_var(
            "HEALTHMONITOR_FILECHECK_FILES",
            file.path().to_str().unwrap(),
        );
        let check = FileCheck::new();
        assert!(check.run().await.is_ok());
    }

    #[tokio::test]
    #[serial] // Tests that manipulate environment variables should not run concurrently.
    async fn test_run_single_existing_empty_file() {
        let file = NamedTempFile::new().unwrap();
        env::set_var(
            "HEALTHMONITOR_FILECHECK_FILES",
            file.path().to_str().unwrap(),
        );
        let check = FileCheck::new();
        assert!(check.run().await.is_err());
    }

    #[tokio::test]
    #[serial] // Tests that manipulate environment variables should not run concurrently.
    async fn test_run_single_non_existing_file() {
        env::set_var("HEALTHMONITOR_FILECHECK_FILES", "non_existing_file.txt");
        let check = FileCheck::new();
        assert!(check.run().await.is_err());
    }

    #[tokio::test]
    #[serial] // Tests that manipulate environment variables should not run concurrently.
    async fn test_run_two_existing_non_empty_files() {
        let mut file1 = NamedTempFile::new().unwrap();
        writeln!(file1, "Test").unwrap();
        let mut file2 = NamedTempFile::new().unwrap();
        writeln!(file2, "Test").unwrap();
        env::set_var(
            "HEALTHMONITOR_FILECHECK_FILES",
            format!(
                "{},{}",
                file1.path().to_str().unwrap(),
                file2.path().to_str().unwrap()
            ),
        );
        let check = FileCheck::new();
        assert!(check.run().await.is_ok());
    }

    #[tokio::test]
    #[serial] // Tests that manipulate environment variables should not run concurrently.
    async fn test_run_two_files_first_empty() {
        let file1 = NamedTempFile::new().unwrap();
        let mut file2 = NamedTempFile::new().unwrap();
        writeln!(file2, "Test").unwrap();
        env::set_var(
            "HEALTHMONITOR_FILECHECK_FILES",
            format!(
                "{},{}",
                file1.path().to_str().unwrap(),
                file2.path().to_str().unwrap()
            ),
        );
        let check = FileCheck::new();
        assert!(check.run().await.is_err());
    }

    #[tokio::test]
    #[serial] // Tests that manipulate environment variables should not run concurrently.
    async fn test_run_two_files_second_non_existing() {
        let mut file1 = NamedTempFile::new().unwrap();
        writeln!(file1, "Test").unwrap();
        env::set_var(
            "HEALTHMONITOR_FILECHECK_FILES",
            format!("{},non_existing_file.txt", file1.path().to_str().unwrap()),
        );
        let check = FileCheck::new();
        assert!(check.run().await.is_err());
    }
}
