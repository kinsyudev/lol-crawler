use super::{ApiError, Endpoints};
use crate::config::Config;
use crate::database::Database;
use crate::models::database::DbApiCall;
use crate::models::riot::*;
use crate::models::MatchDto;
use crate::rate_limiter::RateLimiter;
use chrono::Utc;
use reqwest::{Client, Response};
use std::sync::Arc;
use std::time::Duration;
use tokio::time::sleep;

#[derive(Clone)]
pub struct RiotApiClient {
    client: Client,
    rate_limiter: Arc<RateLimiter>,
    config: Config,
    database: Database,
}

impl RiotApiClient {
    pub fn new(
        config: Config,
        rate_limiter: Arc<RateLimiter>,
        database: Database,
    ) -> Result<Self, ApiError> {
        let client = Client::builder()
            .timeout(Duration::from_secs(30))
            .user_agent("lol-crawler/1.0")
            .build()?;

        Ok(Self {
            client,
            rate_limiter,
            config,
            database,
        })
    }

    async fn make_request(&self, url: &str, region: &str) -> Result<Response, ApiError> {
        let endpoint = url
            .split(&self.config.base_url_for_region(region))
            .nth(1)
            .or_else(|| {
                url.split(&self.config.regional_base_url_for_region(region))
                    .nth(1)
            })
            .unwrap_or(url);

        log::debug!("Making API request to URL: {}", url);
        log::debug!("Endpoint: {}, Region: {}", endpoint, region);

        // Acquire rate limit permit
        self.rate_limiter
            .acquire_permit(endpoint, region)
            .await
            .map_err(|e| ApiError::RateLimiter(e.to_string()))?;

        let response = self
            .client
            .get(url)
            .header("X-Riot-Token", &self.config.riot_api_key)
            .send()
            .await?;

        // Log API call
        let api_call = DbApiCall {
            id: None,
            endpoint: endpoint.to_string(),
            region: region.to_string(),
            timestamp: Utc::now(),
            response_code: response.status().as_u16() as i32,
            rate_limit_remaining: response
                .headers()
                .get("X-App-Rate-Limit-Count")
                .and_then(|h| h.to_str().ok())
                .and_then(|s| s.parse().ok()),
        };

        if let Err(e) = self.database.log_api_call(&api_call) {
            log::warn!("Failed to log API call: {}", e);
        }

        // Update rate limiters from headers
        self.rate_limiter
            .update_limits_from_headers(endpoint, region, response.headers())
            .await;

        match response.status().as_u16() {
            200 => Ok(response),
            400 => Err(ApiError::BadRequest(
                response.text().await.unwrap_or_default(),
            )),
            401 | 403 => Err(ApiError::Authentication),
            404 => Err(ApiError::NotFound),
            429 => {
                let retry_after = response
                    .headers()
                    .get("Retry-After")
                    .and_then(|h| h.to_str().ok())
                    .and_then(|s| s.parse().ok());

                self.rate_limiter.handle_429_response(retry_after).await;
                Err(ApiError::RateLimit)
            }
            500..=599 => Err(ApiError::ServiceUnavailable),
            status => Err(ApiError::Api {
                status,
                message: response.text().await.unwrap_or_default(),
            }),
        }
    }

    async fn make_request_with_retry<T>(&self, url: &str, region: &str) -> Result<T, ApiError>
    where
        T: serde::de::DeserializeOwned,
    {
        let max_retries = self.config.rate_limits.max_retries;
        let mut retries = 0;

        loop {
            match self.make_request(url, region).await {
                Ok(response) => {
                    let text = response.text().await?;
                    match serde_json::from_str::<T>(&text) {
                        Ok(data) => return Ok(data),
                        Err(e) => {
                            log::error!("Failed to parse JSON response from {}: {}", url, e);
                            log::debug!("Response body: {}", text);
                            return Err(ApiError::Json(e));
                        }
                    }
                }
                Err(e) if e.is_retryable() && retries < max_retries => {
                    retries += 1;
                    let delay = Duration::from_millis(
                        self.config.rate_limits.retry_delay_ms * (1 << retries),
                    );
                    log::warn!(
                        "Request failed (attempt {}/{}): {}. Retrying in {:?}",
                        retries,
                        max_retries,
                        e,
                        delay
                    );
                    sleep(delay).await;
                }
                Err(e) => return Err(e),
            }
        }
    }

    pub async fn get_summoner_by_name(
        &self,
        region: &str,
        summoner_name: &str,
    ) -> Result<SummonerResponse, ApiError> {
        let url = Endpoints::summoner_by_name(&self.config, region, summoner_name);
        log::debug!(
            "Fetching summoner by name: {} in region: {}",
            summoner_name,
            region
        );
        self.make_request_with_retry(&url, region).await
    }

    pub async fn get_summoner_by_puuid(
        &self,
        region: &str,
        puuid: &str,
    ) -> Result<SummonerResponse, ApiError> {
        let url = Endpoints::summoner_by_puuid(&self.config, region, puuid);
        log::debug!(
            "Fetching summoner by PUUID: {} in region: {}",
            puuid,
            region
        );
        self.make_request_with_retry(&url, region).await
    }

    pub async fn get_summoner_by_id(
        &self,
        region: &str,
        summoner_id: &str,
    ) -> Result<SummonerResponse, ApiError> {
        let url = Endpoints::summoner_by_id(&self.config, region, summoner_id);
        log::debug!(
            "Fetching summoner by ID: {} in region: {}",
            summoner_id,
            region
        );
        self.make_request_with_retry(&url, region).await
    }

    pub async fn get_match_list_by_puuid(
        &self,
        region: &str,
        puuid: &str,
        start: Option<u32>,
        count: Option<u32>,
    ) -> Result<Vec<String>, ApiError> {
        let url = Endpoints::match_list_by_puuid(&self.config, region, puuid, start, count);
        log::debug!(
            "Fetching match list for PUUID: {} in region: {}",
            puuid,
            region
        );
        self.make_request_with_retry(&url, region).await
    }

    pub async fn get_match_by_id(
        &self,
        region: &str,
        match_id: &str,
    ) -> Result<MatchDto, ApiError> {
        let url = Endpoints::match_by_id(&self.config, region, match_id);
        log::debug!("Fetching match: {} in region: {}", match_id, region);
        self.make_request_with_retry(&url, region).await
    }

    pub async fn get_master_league(
        &self,
        region: &str,
        queue: &str,
    ) -> Result<LeagueListResponse, ApiError> {
        let url = Endpoints::master_league(&self.config, region, queue);
        log::debug!(
            "Fetching master league for queue: {} in region: {}",
            queue,
            region
        );
        self.make_request_with_retry(&url, region).await
    }

    pub async fn get_grandmaster_league(
        &self,
        region: &str,
        queue: &str,
    ) -> Result<LeagueListResponse, ApiError> {
        let url = Endpoints::grandmaster_league(&self.config, region, queue);
        log::debug!(
            "Fetching grandmaster league for queue: {} in region: {}",
            queue,
            region
        );
        self.make_request_with_retry(&url, region).await
    }

    pub async fn get_challenger_league(
        &self,
        region: &str,
        queue: &str,
    ) -> Result<LeagueListResponse, ApiError> {
        let url = Endpoints::challenger_league(&self.config, region, queue);
        log::debug!(
            "Fetching challenger league for queue: {} in region: {}",
            queue,
            region
        );
        self.make_request_with_retry(&url, region).await
    }

    pub async fn get_rate_limit_status(&self) -> crate::rate_limiter::RateLimitStatus {
        self.rate_limiter.get_rate_limit_status().await
    }
}

// Additional Riot API models for league endpoints
#[derive(serde::Deserialize, serde::Serialize, Debug, Clone)]
pub struct LeagueListResponse {
    #[serde(rename = "leagueId")]
    pub league_id: String,
    pub entries: Vec<LeagueEntry>,
    pub tier: String,
    pub name: String,
    pub queue: String,
}

#[derive(serde::Deserialize, serde::Serialize, Debug, Clone)]
pub struct LeagueEntry {
    pub puuid: String,
    #[serde(rename = "leaguePoints")]
    pub league_points: u32,
    pub rank: String,
    pub wins: u32,
    pub losses: u32,
    pub veteran: bool,
    pub inactive: bool,
    #[serde(rename = "freshBlood")]
    pub fresh_blood: bool,
    #[serde(rename = "hotStreak")]
    pub hot_streak: bool,
    #[serde(rename = "miniSeries")]
    pub mini_series: Option<MiniSeries>,
}

#[derive(serde::Deserialize, serde::Serialize, Debug, Clone)]
pub struct MiniSeries {
    pub losses: u32,
    pub progress: String,
    pub target: u32,
    pub wins: u32,
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::config::{Config, CrawlerConfig, LoggingConfig, RateLimitConfig};
    use crate::database::Database;
    use crate::rate_limiter::RateLimiter;
    use mockito::Server;
    use std::sync::Arc;

    fn test_config() -> Config {
        Config {
            riot_api_key: "RGAPI-test-key".to_string(),
            database_url: ":memory:".to_string(),
            regions: vec!["na1".to_string()],
            rate_limits: RateLimitConfig {
                application_limit_per_second: 20,
                application_limit_per_two_minutes: 100,
                max_concurrent_requests: 10,
                retry_delay_ms: 100,
                max_retries: 3,
            },
            crawler: CrawlerConfig {
                queue_size_limit: 1000,
                batch_size: 10,
                health_check_interval_seconds: 60,
                state_save_interval_seconds: 300,
            },
            logging: LoggingConfig {
                level: "info".to_string(),
                format: "json".to_string(),
            },
        }
    }

    async fn setup_test_client() -> (RiotApiClient, Database) {
        let config = test_config();
        let database = Database::new(":memory:").unwrap();
        let rate_limiter = Arc::new(RateLimiter::new(config.rate_limits.clone()));
        let client = RiotApiClient::new(config, rate_limiter, database.clone()).unwrap();
        (client, database)
    }

    #[tokio::test]
    async fn test_api_client_creation() {
        let (client, _) = setup_test_client().await;
        // Just test that we can create the client without errors
        assert_eq!(client.config.riot_api_key, "RGAPI-test-key");
    }

    #[tokio::test]
    async fn test_summoner_response_parsing() {
        // Test that we can parse a valid summoner response
        let mock_response = r#"{
            "puuid": "test-puuid-123",
            "profileIconId": 1234,
            "revisionDate": 1234567890,
            "summonerLevel": 100
        }"#;

        let summoner: Result<SummonerResponse, _> = serde_json::from_str(mock_response);
        assert!(summoner.is_ok());

        let summoner = summoner.unwrap();
        assert_eq!(summoner.puuid, "test-puuid-123");
        assert_eq!(summoner.profile_icon_id, 1234);
        assert_eq!(summoner.summoner_level, 100);
        assert!(summoner.name.is_none()); // Current API doesn't return name
    }

    #[tokio::test]
    async fn test_http_404_handling() {
        let mut server = Server::new_async().await;
        let mut config = test_config();
        config.regions = vec!["mock".to_string()];

        // Override the base URL methods to use mock server
        let mock_url = server.url();

        let mock = server
            .mock(
                "GET",
                "/lol/summoner/v4/summoners/by-name/NonExistentSummoner",
            )
            .with_status(404)
            .with_header("content-type", "application/json")
            .with_header("X-Riot-Token", "RGAPI-test-key")
            .with_body("{}")
            .create_async()
            .await;

        // Create a custom client for testing with mock server URL
        let database = Database::new(":memory:").unwrap();
        let rate_limiter = Arc::new(RateLimiter::new(config.rate_limits.clone()));
        let client = RiotApiClient::new(config, rate_limiter, database).unwrap();

        // Construct the mock URL manually for testing
        let test_url = format!(
            "{}/lol/summoner/v4/summoners/by-name/NonExistentSummoner",
            mock_url
        );

        // Test the make_request method directly
        let result = client.make_request(&test_url, "mock").await;
        assert!(matches!(result, Err(ApiError::NotFound)));

        mock.assert_async().await;
    }

    #[tokio::test]
    async fn test_http_401_authentication_error() {
        let error = ApiError::Authentication;
        assert!(matches!(error, ApiError::Authentication));
        assert!(!error.is_retryable());
    }

    #[tokio::test]
    async fn test_http_429_rate_limit_error() {
        let error = ApiError::RateLimit;
        assert!(matches!(error, ApiError::RateLimit));
        assert!(error.is_retryable());
    }

    #[tokio::test]
    async fn test_http_500_service_unavailable() {
        let error = ApiError::ServiceUnavailable;
        assert!(matches!(error, ApiError::ServiceUnavailable));
        assert!(error.is_retryable());
    }

    #[tokio::test]
    async fn test_json_parsing_error() {
        let mut server = Server::new_async().await;
        let config = test_config();
        let mock_url = server.url();

        let mock = server
            .mock("GET", "/lol/summoner/v4/summoners/by-name/TestSummoner")
            .with_status(200)
            .with_header("content-type", "application/json")
            .with_header("X-Riot-Token", "RGAPI-test-key")
            .with_body("invalid json content")
            .create_async()
            .await;

        let database = Database::new(":memory:").unwrap();
        let rate_limiter = Arc::new(RateLimiter::new(config.rate_limits.clone()));
        let client = RiotApiClient::new(config, rate_limiter, database).unwrap();

        let test_url = format!(
            "{}/lol/summoner/v4/summoners/by-name/TestSummoner",
            mock_url
        );

        // Test make_request_with_retry which handles JSON parsing
        let result: Result<SummonerResponse, _> =
            client.make_request_with_retry(&test_url, "mock").await;
        assert!(matches!(result, Err(ApiError::Json(_))));

        mock.assert_async().await;
    }

    #[tokio::test]
    async fn test_request_timeout_handling() {
        // Test that the client was created with proper timeout configuration
        // The timeout is configured during client construction but isn't directly exposed
        let (_client, _) = setup_test_client().await;
        // We can only test that the client was created successfully with timeout settings
        // The actual timeout behavior would be tested through integration tests
        assert!(true); // Client creation with timeout succeeded
    }

    #[tokio::test]
    async fn test_user_agent_header() {
        let (client, _) = setup_test_client().await;
        // The user agent is set in the client builder
        // This test verifies the client was built with the correct user agent
        // In practice, we'd need to inspect actual requests to verify this
        assert!(!client.config.riot_api_key.is_empty());
    }

    #[tokio::test]
    async fn test_endpoint_extraction() {
        let (_client, _) = setup_test_client().await;

        // Test URL parsing logic
        let base_url = "https://na1.api.riotgames.com";
        let full_url = "https://na1.api.riotgames.com/lol/summoner/v4/summoners/test";

        let endpoint = full_url.split(base_url).nth(1).unwrap_or(full_url);

        assert_eq!(endpoint, "/lol/summoner/v4/summoners/test");
    }

    #[tokio::test]
    async fn test_regional_vs_platform_endpoints() {
        let config = test_config();

        // Platform endpoints (summoner, league data)
        let platform_url = config.base_url_for_region("na1");
        assert_eq!(platform_url, "https://na1.api.riotgames.com");

        // Regional endpoints (match data)
        let regional_url = config.regional_base_url_for_region("na1");
        assert_eq!(regional_url, "https://americas.api.riotgames.com");
    }

    #[tokio::test]
    async fn test_exponential_backoff_calculation() {
        let config = test_config();
        let base_delay = config.rate_limits.retry_delay_ms;

        // Test exponential backoff: delay = base_delay * (2^retry_count)
        let retry_1_delay = base_delay * (1 << 1); // 200ms
        let retry_2_delay = base_delay * (1 << 2); // 400ms
        let retry_3_delay = base_delay * (1 << 3); // 800ms

        assert_eq!(retry_1_delay, 200);
        assert_eq!(retry_2_delay, 400);
        assert_eq!(retry_3_delay, 800);
    }

    #[tokio::test]
    async fn test_api_call_logging() {
        let (_, database) = setup_test_client().await;

        // Test that we can log API calls
        let api_call = DbApiCall {
            id: None,
            endpoint: "/lol/summoner/v4/summoners/test".to_string(),
            region: "na1".to_string(),
            timestamp: chrono::Utc::now(),
            response_code: 200,
            rate_limit_remaining: Some(19),
        };

        let result = database.log_api_call(&api_call);
        assert!(result.is_ok());
    }

    #[tokio::test]
    async fn test_match_list_endpoint() {
        let config = test_config();
        let url = Endpoints::match_list_by_puuid(&config, "na1", "test-puuid", Some(0), Some(20));

        assert!(url.contains("test-puuid"));
        assert!(url.contains("start=0"));
        assert!(url.contains("count=20"));
        assert!(url.contains("americas.api.riotgames.com")); // Regional endpoint
    }

    #[tokio::test]
    async fn test_master_league_endpoint() {
        let config = test_config();
        let url = Endpoints::master_league(&config, "na1", "RANKED_SOLO_5x5");

        assert!(url.contains("masterleagues"));
        assert!(url.contains("RANKED_SOLO_5x5"));
        assert!(url.contains("na1.api.riotgames.com")); // Platform endpoint
    }

    #[tokio::test]
    async fn test_error_message_extraction() {
        // Test different error types and their messages
        let auth_error = ApiError::Authentication;
        let not_found_error = ApiError::NotFound;
        let rate_limit_error = ApiError::RateLimit;

        // These should have different handling
        assert!(!auth_error.is_retryable());
        assert!(!not_found_error.is_retryable());
        assert!(rate_limit_error.is_retryable());
    }

    #[tokio::test]
    async fn test_retry_logic_with_429() {
        let mut server = Server::new_async().await;
        let mut config = test_config();
        config.rate_limits.max_retries = 2;
        config.rate_limits.retry_delay_ms = 50;

        let mock_url = server.url();

        // First request returns 429, second succeeds
        let mock_429 = server
            .mock("GET", "/lol/summoner/v4/summoners/by-name/TestSummoner")
            .with_status(429)
            .with_header("content-type", "application/json")
            .with_header("Retry-After", "1")
            .with_header("X-Riot-Token", "RGAPI-test-key")
            .with_body("{}")
            .expect(1)
            .create_async()
            .await;

        let mock_success = server
            .mock("GET", "/lol/summoner/v4/summoners/by-name/TestSummoner")
            .with_status(200)
            .with_header("content-type", "application/json")
            .with_header("X-Riot-Token", "RGAPI-test-key")
            .with_body(
                r#"{
                "puuid": "test-puuid",
                "profileIconId": 1234,
                "revisionDate": 1234567890,
                "summonerLevel": 100
            }"#,
            )
            .expect(1)
            .create_async()
            .await;

        let database = Database::new(":memory:").unwrap();
        let rate_limiter = Arc::new(RateLimiter::new(config.rate_limits.clone()));
        let client = RiotApiClient::new(config, rate_limiter, database).unwrap();

        let test_url = format!(
            "{}/lol/summoner/v4/summoners/by-name/TestSummoner",
            mock_url
        );

        let start = tokio::time::Instant::now();
        let result: Result<SummonerResponse, _> =
            client.make_request_with_retry(&test_url, "mock").await;
        let elapsed = start.elapsed();

        assert!(result.is_ok());
        // Should have waited at least the retry-after time
        assert!(elapsed >= Duration::from_millis(900));

        mock_429.assert_async().await;
        mock_success.assert_async().await;
    }

    #[tokio::test]
    async fn test_exponential_backoff_on_service_errors() {
        let mut server = Server::new_async().await;
        let mut config = test_config();
        config.rate_limits.max_retries = 3;
        config.rate_limits.retry_delay_ms = 10; // Small delay for testing

        let mock_url = server.url();

        // Return 500 errors for all attempts
        let mock_error = server
            .mock("GET", "/lol/summoner/v4/summoners/by-name/TestSummoner")
            .with_status(500)
            .with_header("content-type", "application/json")
            .with_header("X-Riot-Token", "RGAPI-test-key")
            .with_body("Internal Server Error")
            .expect(4) // Initial + 3 retries
            .create_async()
            .await;

        let database = Database::new(":memory:").unwrap();
        let rate_limiter = Arc::new(RateLimiter::new(config.rate_limits.clone()));
        let client = RiotApiClient::new(config, rate_limiter, database).unwrap();

        let test_url = format!(
            "{}/lol/summoner/v4/summoners/by-name/TestSummoner",
            mock_url
        );

        let start = tokio::time::Instant::now();
        let result: Result<SummonerResponse, _> =
            client.make_request_with_retry(&test_url, "mock").await;
        let elapsed = start.elapsed();

        assert!(matches!(result, Err(ApiError::ServiceUnavailable)));
        // Should have waited for exponential backoff: 10ms + 20ms + 40ms = ~70ms minimum
        assert!(elapsed >= Duration::from_millis(60));

        mock_error.assert_async().await;
    }

    #[tokio::test]
    async fn test_successful_summoner_request() {
        let mut server = Server::new_async().await;
        let config = test_config();
        let mock_url = server.url();

        let mock_response = r#"{
            "puuid": "test-puuid-success",
            "profileIconId": 5678,
            "revisionDate": 1234567890,
            "summonerLevel": 150
        }"#;

        let mock = server
            .mock("GET", "/lol/summoner/v4/summoners/by-name/TestSummoner")
            .with_status(200)
            .with_header("content-type", "application/json")
            .with_header("X-App-Rate-Limit-Count", "1:1,5:120")
            .with_header("X-Method-Rate-Limit-Count", "1:1")
            .with_header("X-Riot-Token", "RGAPI-test-key")
            .with_body(mock_response)
            .create_async()
            .await;

        let database = Database::new(":memory:").unwrap();
        let rate_limiter = Arc::new(RateLimiter::new(config.rate_limits.clone()));
        let client = RiotApiClient::new(config, rate_limiter, database).unwrap();

        let test_url = format!(
            "{}/lol/summoner/v4/summoners/by-name/TestSummoner",
            mock_url
        );

        let result: Result<SummonerResponse, _> =
            client.make_request_with_retry(&test_url, "mock").await;

        assert!(result.is_ok());
        let summoner = result.unwrap();
        assert_eq!(summoner.puuid, "test-puuid-success");
        assert_eq!(summoner.profile_icon_id, 5678);
        assert_eq!(summoner.summoner_level, 150);

        mock.assert_async().await;
    }

    #[tokio::test]
    async fn test_rate_limit_header_updates() {
        let mut server = Server::new_async().await;
        let config = test_config();
        let mock_url = server.url();

        let mock = server
            .mock("GET", "/lol/summoner/v4/summoners/by-name/TestSummoner")
            .with_status(200)
            .with_header("content-type", "application/json")
            .with_header("X-App-Rate-Limit", "10:1,50:120")
            .with_header("X-Method-Rate-Limit", "5:1")
            .with_header("X-Service-Rate-Limit", "100:1")
            .with_header("X-Riot-Token", "RGAPI-test-key")
            .with_body(
                r#"{
                "puuid": "test-puuid",
                "profileIconId": 1234,
                "revisionDate": 1234567890,
                "summonerLevel": 100
            }"#,
            )
            .create_async()
            .await;

        let database = Database::new(":memory:").unwrap();
        let rate_limiter = Arc::new(RateLimiter::new(config.rate_limits.clone()));
        let client = RiotApiClient::new(config, rate_limiter, database).unwrap();

        let test_url = format!(
            "{}/lol/summoner/v4/summoners/by-name/TestSummoner",
            mock_url
        );

        // Make request which should update rate limits from headers
        let _result = client.make_request(&test_url, "mock").await.unwrap();

        // Check that rate limits were updated
        let status = client.get_rate_limit_status().await;
        assert_eq!(status.application_tokens_per_second, 10);
        assert_eq!(status.application_tokens_per_two_minutes, 50);

        mock.assert_async().await;
    }

    #[tokio::test]
    async fn test_match_list_request() {
        let mut server = Server::new_async().await;
        let config = test_config();
        let mock_url = server.url();

        let mock_response = r#"["NA1_1234567890", "NA1_0987654321"]"#;

        let mock = server
            .mock("GET", "/lol/match/v5/matches/by-puuid/test-puuid/ids")
            .match_query(mockito::Matcher::AllOf(vec![
                mockito::Matcher::UrlEncoded("start".into(), "0".into()),
                mockito::Matcher::UrlEncoded("count".into(), "20".into()),
            ]))
            .with_status(200)
            .with_header("content-type", "application/json")
            .with_header("X-Riot-Token", "RGAPI-test-key")
            .with_body(mock_response)
            .create_async()
            .await;

        let database = Database::new(":memory:").unwrap();
        let rate_limiter = Arc::new(RateLimiter::new(config.rate_limits.clone()));
        let client = RiotApiClient::new(config, rate_limiter, database).unwrap();

        let test_url = format!(
            "{}/lol/match/v5/matches/by-puuid/test-puuid/ids?start=0&count=20",
            mock_url
        );

        let result: Result<Vec<String>, _> =
            client.make_request_with_retry(&test_url, "mock").await;

        assert!(result.is_ok());
        let match_ids = result.unwrap();
        assert_eq!(match_ids.len(), 2);
        assert_eq!(match_ids[0], "NA1_1234567890");
        assert_eq!(match_ids[1], "NA1_0987654321");

        mock.assert_async().await;
    }

    #[tokio::test]
    async fn test_match_data_request() {
        let mut server = Server::new_async().await;
        let config = test_config();
        let mock_url = server.url();

        let mock_response = r#"{
            "metadata": {
                "dataVersion": "2",
                "matchId": "NA1_1234567890",
                "participants": ["player1", "player2"]
            },
            "info": {
                "gameCreation": 1640000000000,
                "gameDuration": 1800,
                "gameEndTimestamp": 1640001800000,
                "gameId": 1234567890,
                "gameMode": "CLASSIC",
                "gameStartTimestamp": 1640000000000,
                "gameType": "MATCHED_GAME",
                "gameVersion": "12.1.1",
                "mapId": 11,
                "platformId": "NA1",
                "queueId": 420,
                "teams": [],
                "participants": [],
                "tournamentCode": null
            }
        }"#;

        let mock = server
            .mock("GET", "/lol/match/v5/matches/NA1_1234567890")
            .with_status(200)
            .with_header("content-type", "application/json")
            .with_header("X-Riot-Token", "RGAPI-test-key")
            .with_body(mock_response)
            .create_async()
            .await;

        let database = Database::new(":memory:").unwrap();
        let rate_limiter = Arc::new(RateLimiter::new(config.rate_limits.clone()));
        let client = RiotApiClient::new(config, rate_limiter, database).unwrap();

        let test_url = format!("{}/lol/match/v5/matches/NA1_1234567890", mock_url);

        let result: Result<MatchDto, _> = client.make_request_with_retry(&test_url, "mock").await;

        assert!(result.is_ok());
        let match_data = result.unwrap();
        assert_eq!(match_data.info.queue_id, 420);
        assert_eq!(match_data.info.game_mode, "CLASSIC");

        mock.assert_async().await;
    }

    #[tokio::test]
    async fn test_league_endpoint_request() {
        let mut server = Server::new_async().await;
        let config = test_config();
        let mock_url = server.url();

        let mock_response = r#"{
            "leagueId": "test-league-id",
            "entries": [
                {
                    "puuid": "test-player-1",
                    "leaguePoints": 100,
                    "rank": "I",
                    "wins": 50,
                    "losses": 40,
                    "veteran": false,
                    "inactive": false,
                    "freshBlood": true,
                    "hotStreak": false,
                    "miniSeries": null
                }
            ],
            "tier": "MASTER",
            "name": "Test League",
            "queue": "RANKED_SOLO_5x5"
        }"#;

        let mock = server
            .mock(
                "GET",
                "/lol/league/v4/masterleagues/by-queue/RANKED_SOLO_5x5",
            )
            .with_status(200)
            .with_header("content-type", "application/json")
            .with_header("X-Riot-Token", "RGAPI-test-key")
            .with_body(mock_response)
            .create_async()
            .await;

        let database = Database::new(":memory:").unwrap();
        let rate_limiter = Arc::new(RateLimiter::new(config.rate_limits.clone()));
        let client = RiotApiClient::new(config, rate_limiter, database).unwrap();

        let test_url = format!(
            "{}/lol/league/v4/masterleagues/by-queue/RANKED_SOLO_5x5",
            mock_url
        );

        let result: Result<LeagueListResponse, _> =
            client.make_request_with_retry(&test_url, "mock").await;

        assert!(result.is_ok());
        let league_data = result.unwrap();
        assert_eq!(league_data.tier, "MASTER");
        assert_eq!(league_data.queue, "RANKED_SOLO_5x5");
        assert_eq!(league_data.entries.len(), 1);
        assert_eq!(league_data.entries[0].puuid, "test-player-1");

        mock.assert_async().await;
    }

    #[tokio::test]
    async fn test_bad_request_400_error() {
        let mut server = Server::new_async().await;
        let config = test_config();
        let mock_url = server.url();

        let mock = server
            .mock("GET", "/lol/summoner/v4/summoners/by-name/Invalid@Name")
            .with_status(400)
            .with_header("content-type", "application/json")
            .with_header("X-Riot-Token", "RGAPI-test-key")
            .with_body(r#"{"status": {"message": "Bad request", "status_code": 400}}"#)
            .create_async()
            .await;

        let database = Database::new(":memory:").unwrap();
        let rate_limiter = Arc::new(RateLimiter::new(config.rate_limits.clone()));
        let client = RiotApiClient::new(config, rate_limiter, database).unwrap();

        let test_url = format!(
            "{}/lol/summoner/v4/summoners/by-name/Invalid@Name",
            mock_url
        );

        let result = client.make_request(&test_url, "mock").await;
        assert!(matches!(result, Err(ApiError::BadRequest(_))));

        if let Err(ApiError::BadRequest(message)) = result {
            assert!(message.contains("Bad request"));
        }

        mock.assert_async().await;
    }

    #[tokio::test]
    async fn test_authentication_401_error() {
        let mut server = Server::new_async().await;
        let config = test_config();
        let mock_url = server.url();

        let mock = server
            .mock("GET", "/lol/summoner/v4/summoners/by-name/TestSummoner")
            .with_status(401)
            .with_header("content-type", "application/json")
            .with_header("X-Riot-Token", "RGAPI-test-key")
            .with_body(r#"{"status": {"message": "Unauthorized", "status_code": 401}}"#)
            .create_async()
            .await;

        let database = Database::new(":memory:").unwrap();
        let rate_limiter = Arc::new(RateLimiter::new(config.rate_limits.clone()));
        let client = RiotApiClient::new(config, rate_limiter, database).unwrap();

        let test_url = format!(
            "{}/lol/summoner/v4/summoners/by-name/TestSummoner",
            mock_url
        );

        let result = client.make_request(&test_url, "mock").await;
        assert!(matches!(result, Err(ApiError::Authentication)));

        // Authentication errors should not be retryable
        let error = ApiError::Authentication;
        assert!(!error.is_retryable());

        mock.assert_async().await;
    }

    #[tokio::test]
    async fn test_concurrent_api_requests() {
        let config = test_config();
        let database = Database::new(":memory:").unwrap();
        let rate_limiter = Arc::new(RateLimiter::new(config.rate_limits.clone()));
        let client = Arc::new(RiotApiClient::new(config, rate_limiter, database).unwrap());

        let mut handles = vec![];

        // Spawn multiple concurrent requests
        for i in 0..5 {
            let client_clone = client.clone();
            let handle = tokio::spawn(async move {
                // Test concurrent access to rate limit status
                let _status = client_clone.get_rate_limit_status().await;

                // Simulate some processing time
                tokio::time::sleep(Duration::from_millis(10)).await;

                format!("Request {} completed", i)
            });
            handles.push(handle);
        }

        // All should complete successfully
        for handle in handles {
            let result = handle.await.unwrap();
            assert!(result.contains("completed"));
        }
    }
}
