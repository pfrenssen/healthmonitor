use crate::checks::HealthCheck;
use crate::config::Config;
use async_trait::async_trait;
use log::debug;
use reqwest::Client;

pub struct UrlCheck {
    name: &'static str,
    interval: usize,
    timeout: usize,
    is_quick_check: bool,
    urls: Vec<String>,
}

impl UrlCheck {
    pub fn new(config: &Config) -> Self {
        Self {
            name: "UrlCheck",
            interval: config.checks.url_check.interval,
            timeout: config.checks.url_check.timeout,
            is_quick_check: false,
            urls: config.checks.url_check.urls.clone(),
        }
    }
}

#[async_trait]
impl HealthCheck for UrlCheck {
    fn name(&self) -> &str {
        self.name
    }

    fn interval(&self) -> usize {
        self.interval
    }

    fn is_quick_check(&self) -> bool {
        self.is_quick_check
    }

    fn is_enabled(&self) -> bool {
        !self.urls.is_empty()
    }

    async fn run(&self) -> Result<(), String> {
        debug!("Running URL checks");

        let client = Client::builder()
            .timeout(std::time::Duration::from_secs(self.timeout as u64))
            .connect_timeout(std::time::Duration::from_secs(self.timeout as u64))
            .build()
            .map_err(|e| format!("Failed to create HTTP client: {}", e))?;
        for url in &self.urls {
            let response = client
                .get(url)
                .send()
                .await
                .map_err(|e| format!("Failed to access {}: {}", url, e))?;
            if response.status() != 200 {
                return Err(format!("URL {} returned status {}", url, response.status()));
            }
        }
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use wiremock::matchers::{method, path};
    use wiremock::{Mock, MockServer, ResponseTemplate};

    #[test]
    fn test_name() {
        let check = UrlCheck::new(&Config::new());
        assert_eq!(check.name(), "UrlCheck");
    }

    #[test]
    fn test_is_quick_check() {
        let check = UrlCheck::new(&Config::new());
        assert!(!check.is_quick_check());
    }

    #[tokio::test]
    async fn test_without_urls_is_disabled() {
        let mut config = Config::new();
        config.checks.url_check.urls = vec![];
        let check = UrlCheck::new(&config);
        assert!(!check.is_enabled());
    }

    #[tokio::test]
    async fn test_with_urls_is_enabled() {
        let mut config = Config::new();
        config.checks.url_check.urls = vec!["http://example.com".to_string()];
        let check = UrlCheck::new(&config);
        assert!(check.is_enabled());
    }

    #[tokio::test]
    async fn test_run_with_successful_response() {
        let mock_server = MockServer::start().await;

        Mock::given(method("GET"))
            .respond_with(ResponseTemplate::new(200))
            .mount(&mock_server)
            .await;

        let mut config = Config::new();
        config.checks.url_check.urls = vec![mock_server.uri()];
        let check = UrlCheck::new(&config);

        assert!(check.run().await.is_ok());
    }

    #[tokio::test]
    async fn test_run_with_error_response() {
        let mock_server = MockServer::start().await;

        Mock::given(method("GET"))
            .respond_with(ResponseTemplate::new(500))
            .mount(&mock_server)
            .await;

        let mut config = Config::new();
        config.checks.url_check.urls = vec![mock_server.uri()];
        let check = UrlCheck::new(&config);

        assert!(check.run().await.is_err());
    }

    #[tokio::test]
    async fn test_run_with_timeout() {
        let mock_server = MockServer::start().await;

        Mock::given(method("GET"))
            .respond_with(ResponseTemplate::new(200).set_delay(std::time::Duration::from_secs(2)))
            .mount(&mock_server)
            .await;

        let mut config = Config::new();
        config.checks.url_check.timeout = 1;
        config.checks.url_check.urls = vec![mock_server.uri()];
        let check = UrlCheck::new(&config);

        assert!(check.run().await.is_err());
    }

    #[tokio::test]
    async fn test_run_with_multiple_urls() {
        let mock_server = MockServer::start().await;

        Mock::given(method("GET"))
            .respond_with(ResponseTemplate::new(200))
            .mount(&mock_server)
            .await;

        let mut config = Config::new();
        config.checks.url_check.urls =
            vec![mock_server.uri() + "/first", mock_server.uri() + "/second"];
        let check = UrlCheck::new(&config);

        assert!(check.run().await.is_ok());
    }

    #[tokio::test]
    async fn test_run_with_multiple_urls_second_url_returning_503() {
        let mock_server = MockServer::start().await;

        Mock::given(method("GET"))
            .and(path("/good"))
            .respond_with(ResponseTemplate::new(200))
            .mount(&mock_server)
            .await;

        Mock::given(method("GET"))
            .and(path("/bad"))
            .respond_with(ResponseTemplate::new(503))
            .mount(&mock_server)
            .await;

        let mut config = Config::new();
        config.checks.url_check.urls =
            vec![mock_server.uri() + "/good", mock_server.uri() + "/bad"];
        let check = UrlCheck::new(&config);

        assert!(check.run().await.is_err());
    }
}
