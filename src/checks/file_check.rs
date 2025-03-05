use crate::checks::HealthCheck;
use crate::config::Config;
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
    pub fn new(config: &Config) -> Self {
        Self {
            name: "FileCheck",
            interval: config.checks.file_check.interval,
            is_quick_check: true,
            files: config.checks.file_check.files.clone(),
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
    use std::io::Write;
    use tempfile::NamedTempFile;

    #[test]
    fn test_name() {
        let check = FileCheck::new(&Config::new());
        assert_eq!(check.name(), "FileCheck");
    }

    #[test]
    fn test_is_quick_check() {
        let check = FileCheck::new(&Config::new());
        assert!(check.is_quick_check());
    }

    #[test]
    fn test_custom_interval() {
        let mut config = Config::new();
        config.checks.file_check.interval = 45;
        let check = FileCheck::new(&config);
        assert_eq!(check.interval(), 45);
    }

    #[tokio::test]
    async fn test_run_without_files() {
        let mut config = Config::new();
        config.checks.file_check.files = vec![];
        let check = FileCheck::new(&config);
        assert!(check.run().await.is_ok());
    }

    #[tokio::test]
    async fn test_run_single_existing_non_empty_file() {
        let mut file = NamedTempFile::new().unwrap();
        writeln!(file, "Test").unwrap();
        let mut config = Config::new();
        config.checks.file_check.files = vec![file.path().to_str().unwrap().to_string()];
        let check = FileCheck::new(&config);
        assert!(check.run().await.is_ok());
    }

    #[tokio::test]
    async fn test_run_single_existing_empty_file() {
        let file = NamedTempFile::new().unwrap();
        let mut config = Config::new();
        config.checks.file_check.files = vec![file.path().to_str().unwrap().to_string()];
        let check = FileCheck::new(&config);
        assert!(check.run().await.is_err());
    }

    #[tokio::test]
    async fn test_run_single_non_existing_file() {
        let mut config = Config::new();
        config.checks.file_check.files = vec!["non_existing_file.txt".to_string()];
        let check = FileCheck::new(&config);
        assert!(check.run().await.is_err());
    }

    #[tokio::test]
    async fn test_run_two_existing_non_empty_files() {
        let mut file1 = NamedTempFile::new().unwrap();
        writeln!(file1, "Test").unwrap();
        let mut file2 = NamedTempFile::new().unwrap();
        writeln!(file2, "Test").unwrap();
        let mut config = Config::new();
        config.checks.file_check.files = vec![
            file1.path().to_str().unwrap().to_string(),
            file2.path().to_str().unwrap().to_string(),
        ];
        let check = FileCheck::new(&Config::new());
        assert!(check.run().await.is_ok());
    }

    #[tokio::test]
    async fn test_run_two_files_first_empty() {
        let file1 = NamedTempFile::new().unwrap();
        let mut file2 = NamedTempFile::new().unwrap();
        writeln!(file2, "Test").unwrap();
        let mut config = Config::new();
        config.checks.file_check.files = vec![
            file1.path().to_str().unwrap().to_string(),
            file2.path().to_str().unwrap().to_string(),
        ];
        let check = FileCheck::new(&config);
        assert!(check.run().await.is_err());
    }

    #[tokio::test]
    async fn test_run_two_files_second_non_existing() {
        let mut file1 = NamedTempFile::new().unwrap();
        writeln!(file1, "Test").unwrap();
        let mut config = Config::new();
        config.checks.file_check.files = vec![
            file1.path().to_str().unwrap().to_string(),
            "non_existing_file.txt".to_string(),
        ];
        let check = FileCheck::new(&config);
        assert!(check.run().await.is_err());
    }
}
