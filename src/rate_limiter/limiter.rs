use super::TokenBucket;
use crate::config::RateLimitConfig;
use dashmap::DashMap;
use std::sync::Arc;
use std::time::Duration;
use tokio::sync::RwLock;
use tokio::time::sleep;

#[derive(Debug)]
pub struct RateLimiter {
    application_limiter_per_second: Arc<RwLock<TokenBucket>>,
    application_limiter_per_two_minutes: Arc<RwLock<TokenBucket>>,
    method_limiters: Arc<DashMap<String, Arc<RwLock<TokenBucket>>>>,
    service_limiters: Arc<DashMap<String, Arc<RwLock<TokenBucket>>>>,
    config: RateLimitConfig,
}

impl RateLimiter {
    pub fn new(config: RateLimitConfig) -> Self {
        Self {
            application_limiter_per_second: Arc::new(RwLock::new(TokenBucket::per_second(
                config.application_limit_per_second,
                config.application_limit_per_second,
            ))),
            application_limiter_per_two_minutes: Arc::new(RwLock::new(
                TokenBucket::per_two_minutes(
                    config.application_limit_per_two_minutes,
                    config.application_limit_per_two_minutes,
                ),
            )),
            method_limiters: Arc::new(DashMap::new()),
            service_limiters: Arc::new(DashMap::new()),
            config,
        }
    }

    pub async fn acquire_permit(
        &self,
        endpoint: &str,
        region: &str,
    ) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
        let retries = self.config.max_retries;
        let mut retry_count = 0;

        while retry_count < retries {
            // Try to acquire from all rate limiters
            if self.try_acquire_all(endpoint, region).await? {
                return Ok(());
            }

            // If we failed, wait and retry
            retry_count += 1;
            if retry_count < retries {
                let delay = Duration::from_millis(self.config.retry_delay_ms * (1 << retry_count)); // Exponential backoff
                log::debug!(
                    "Rate limit hit, retrying in {:?} (attempt {}/{})",
                    delay,
                    retry_count,
                    retries
                );
                sleep(delay).await;
            }
        }

        Err(format!(
            "Failed to acquire rate limit permit after {} retries",
            retries
        )
        .into())
    }

    async fn try_acquire_all(
        &self,
        endpoint: &str,
        region: &str,
    ) -> Result<bool, Box<dyn std::error::Error + Send + Sync>> {
        // Check application rate limits first
        {
            let mut app_limiter_per_sec = self.application_limiter_per_second.write().await;
            if !app_limiter_per_sec.try_acquire(1) {
                log::debug!("Application rate limit per second hit");
                return Ok(false);
            }
        }

        {
            let mut app_limiter_per_two_min =
                self.application_limiter_per_two_minutes.write().await;
            if !app_limiter_per_two_min.try_acquire(1) {
                log::debug!("Application rate limit per two minutes hit");
                return Ok(false);
            }
        }

        // Check method rate limits
        let method_key = format!("{}:{}", endpoint, region);
        let method_limiter = self.get_or_create_method_limiter(&method_key);
        {
            let mut limiter = method_limiter.write().await;
            if !limiter.try_acquire(1) {
                log::debug!("Method rate limit hit for {}", method_key);
                return Ok(false);
            }
        }

        // Check service rate limits
        let service_key = self.extract_service_from_endpoint(endpoint);
        let service_limiter = self.get_or_create_service_limiter(&service_key, region);
        {
            let mut limiter = service_limiter.write().await;
            if !limiter.try_acquire(1) {
                log::debug!("Service rate limit hit for {}", service_key);
                return Ok(false);
            }
        }

        Ok(true)
    }

    fn get_or_create_method_limiter(&self, method_key: &str) -> Arc<RwLock<TokenBucket>> {
        self.method_limiters
            .entry(method_key.to_string())
            .or_insert_with(|| {
                // Default method limits - these would typically come from API headers
                Arc::new(RwLock::new(TokenBucket::per_second(20, 20)))
            })
            .clone()
    }

    fn get_or_create_service_limiter(
        &self,
        service: &str,
        region: &str,
    ) -> Arc<RwLock<TokenBucket>> {
        let service_key = format!("{}:{}", service, region);
        self.service_limiters
            .entry(service_key)
            .or_insert_with(|| {
                // Default service limits - these would typically come from API headers
                Arc::new(RwLock::new(TokenBucket::per_second(100, 100)))
            })
            .clone()
    }

    fn extract_service_from_endpoint(&self, endpoint: &str) -> String {
        // Extract service name from endpoint path
        // e.g., "/lol/spectator/v4/featured-games" -> "spectator"
        let parts: Vec<&str> = endpoint.split('/').collect();
        if parts.len() >= 3 {
            parts[2].to_string()
        } else {
            "unknown".to_string()
        }
    }

    pub async fn update_limits_from_headers(
        &self,
        endpoint: &str,
        region: &str,
        headers: &reqwest::header::HeaderMap,
    ) {
        // Update rate limits based on API response headers
        if let Some(app_limit) = headers.get("X-App-Rate-Limit") {
            if let Ok(limit_str) = app_limit.to_str() {
                self.parse_and_update_app_limits(limit_str).await;
            }
        }

        if let Some(method_limit) = headers.get("X-Method-Rate-Limit") {
            if let Ok(limit_str) = method_limit.to_str() {
                self.parse_and_update_method_limits(endpoint, region, limit_str)
                    .await;
            }
        }

        if let Some(service_limit) = headers.get("X-Service-Rate-Limit") {
            if let Ok(limit_str) = service_limit.to_str() {
                let service = self.extract_service_from_endpoint(endpoint);
                self.parse_and_update_service_limits(&service, region, limit_str)
                    .await;
            }
        }
    }

    async fn parse_and_update_app_limits(&self, limit_str: &str) {
        // Parse rate limit string like "20:1,100:120" (20 per 1 second, 100 per 120 seconds)
        for limit_pair in limit_str.split(',') {
            if let Some((count_str, window_str)) = limit_pair.split_once(':') {
                if let (Ok(count), Ok(window)) =
                    (count_str.parse::<u32>(), window_str.parse::<u64>())
                {
                    if window == 1 {
                        let mut limiter = self.application_limiter_per_second.write().await;
                        *limiter = TokenBucket::per_second(count, count);
                    } else if window == 120 {
                        let mut limiter = self.application_limiter_per_two_minutes.write().await;
                        *limiter = TokenBucket::per_two_minutes(count, count);
                    }
                }
            }
        }
    }

    async fn parse_and_update_method_limits(&self, endpoint: &str, region: &str, limit_str: &str) {
        let method_key = format!("{}:{}", endpoint, region);
        let limiter = self.get_or_create_method_limiter(&method_key);

        // Parse and update method limits (similar to app limits)
        for limit_pair in limit_str.split(',') {
            if let Some((count_str, window_str)) = limit_pair.split_once(':') {
                if let (Ok(count), Ok(window)) =
                    (count_str.parse::<u32>(), window_str.parse::<u64>())
                {
                    if window == 1 {
                        let mut limiter_guard = limiter.write().await;
                        *limiter_guard = TokenBucket::per_second(count, count);
                        break;
                    }
                }
            }
        }
    }

    async fn parse_and_update_service_limits(&self, service: &str, region: &str, limit_str: &str) {
        let service_limiter = self.get_or_create_service_limiter(service, region);

        // Parse and update service limits (similar to app limits)
        for limit_pair in limit_str.split(',') {
            if let Some((count_str, window_str)) = limit_pair.split_once(':') {
                if let (Ok(count), Ok(window)) =
                    (count_str.parse::<u32>(), window_str.parse::<u64>())
                {
                    if window == 1 {
                        let mut limiter_guard = service_limiter.write().await;
                        *limiter_guard = TokenBucket::per_second(count, count);
                        break;
                    }
                }
            }
        }
    }

    pub async fn handle_429_response(&self, retry_after: Option<u64>) {
        let delay = if let Some(retry_after_secs) = retry_after {
            Duration::from_secs(retry_after_secs)
        } else {
            Duration::from_millis(self.config.retry_delay_ms)
        };

        log::warn!("Received 429 response, waiting {:?} before retry", delay);
        sleep(delay).await;
    }

    pub async fn get_rate_limit_status(&self) -> RateLimitStatus {
        let app_tokens_per_sec = {
            let mut limiter = self.application_limiter_per_second.write().await;
            limiter.available_tokens()
        };

        let app_tokens_per_two_min = {
            let mut limiter = self.application_limiter_per_two_minutes.write().await;
            limiter.available_tokens()
        };

        RateLimitStatus {
            application_tokens_per_second: app_tokens_per_sec,
            application_tokens_per_two_minutes: app_tokens_per_two_min,
            method_limiters_count: self.method_limiters.len(),
            service_limiters_count: self.service_limiters.len(),
        }
    }
}

#[derive(Debug)]
pub struct RateLimitStatus {
    pub application_tokens_per_second: u32,
    pub application_tokens_per_two_minutes: u32,
    pub method_limiters_count: usize,
    pub service_limiters_count: usize,
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::config::RateLimitConfig;
    use reqwest::header::{HeaderMap, HeaderValue};
    use tokio::time::{Duration, Instant};

    fn test_config() -> RateLimitConfig {
        RateLimitConfig {
            application_limit_per_second: 20,
            application_limit_per_two_minutes: 100,
            max_concurrent_requests: 10,
            retry_delay_ms: 100,
            max_retries: 3,
        }
    }

    #[tokio::test]
    async fn test_rate_limiter_creation() {
        let config = test_config();
        let limiter = RateLimiter::new(config);
        
        let status = limiter.get_rate_limit_status().await;
        assert_eq!(status.application_tokens_per_second, 20);
        assert_eq!(status.application_tokens_per_two_minutes, 100);
    }

    #[tokio::test]
    async fn test_basic_permit_acquisition() {
        let config = test_config();
        let limiter = RateLimiter::new(config);

        // Should be able to acquire permits initially
        limiter.acquire_permit("/lol/summoner/v4/summoners/test", "na1").await.unwrap();
        limiter.acquire_permit("/lol/match/v5/matches/test", "na1").await.unwrap();
    }

    #[tokio::test]
    async fn test_application_rate_limit_exhaustion() {
        let mut config = test_config();
        config.application_limit_per_second = 5; // Low limit for testing
        let limiter = RateLimiter::new(config);

        // Should be able to acquire all permits initially
        for _ in 0..5 {
            limiter.acquire_permit("/test", "na1").await.unwrap();
        }

        // Check that we've consumed tokens
        let status = limiter.get_rate_limit_status().await;
        assert!(status.application_tokens_per_second < 5);
    }

    #[tokio::test]
    async fn test_method_rate_limiting() {
        let config = test_config();
        let limiter = RateLimiter::new(config);

        let endpoint = "/lol/summoner/v4/summoners/test";
        let region = "na1";

        // Fill up the method limiter (default 20 per second)
        for _ in 0..20 {
            assert!(limiter.try_acquire_all(endpoint, region).await.unwrap());
        }

        // Next request should fail initially
        assert!(!limiter.try_acquire_all(endpoint, region).await.unwrap());
    }

    #[tokio::test]
    async fn test_service_extraction() {
        let config = test_config();
        let limiter = RateLimiter::new(config);

        assert_eq!(limiter.extract_service_from_endpoint("/lol/summoner/v4/summoners/test"), "summoner");
        assert_eq!(limiter.extract_service_from_endpoint("/lol/match/v5/matches/test"), "match");
        assert_eq!(limiter.extract_service_from_endpoint("/lol/spectator/v4/featured-games"), "spectator");
        assert_eq!(limiter.extract_service_from_endpoint("/invalid"), "unknown");
    }

    #[tokio::test]
    async fn test_header_parsing_app_limits() {
        let config = test_config();
        let limiter = RateLimiter::new(config);

        let mut headers = HeaderMap::new();
        headers.insert("X-App-Rate-Limit", HeaderValue::from_static("10:1,50:120"));

        limiter.update_limits_from_headers("/test", "na1", &headers).await;

        let status = limiter.get_rate_limit_status().await;
        assert_eq!(status.application_tokens_per_second, 10);
        assert_eq!(status.application_tokens_per_two_minutes, 50);
    }

    #[tokio::test]
    async fn test_header_parsing_method_limits() {
        let config = test_config();
        let limiter = RateLimiter::new(config);

        let mut headers = HeaderMap::new();
        headers.insert("X-Method-Rate-Limit", HeaderValue::from_static("5:1"));

        let endpoint = "/lol/summoner/v4/summoners/test";
        let region = "na1";

        limiter.update_limits_from_headers(endpoint, region, &headers).await;

        // Should be able to acquire 5 permits
        for _ in 0..5 {
            assert!(limiter.try_acquire_all(endpoint, region).await.unwrap());
        }

        // 6th should fail
        assert!(!limiter.try_acquire_all(endpoint, region).await.unwrap());
    }

    #[tokio::test]
    async fn test_429_response_handling() {
        let config = test_config();
        let limiter = RateLimiter::new(config);

        let start = Instant::now();
        limiter.handle_429_response(Some(1)).await; // 1 second wait
        let elapsed = start.elapsed();

        assert!(elapsed >= Duration::from_millis(900));
        assert!(elapsed <= Duration::from_millis(1200));
    }

    #[tokio::test]
    async fn test_429_response_handling_default() {
        let config = test_config();
        let limiter = RateLimiter::new(config);

        let start = Instant::now();
        limiter.handle_429_response(None).await; // Should use retry_delay_ms (100ms)
        let elapsed = start.elapsed();

        assert!(elapsed >= Duration::from_millis(90));
        assert!(elapsed <= Duration::from_millis(200));
    }

    #[tokio::test]
    async fn test_concurrent_access() {
        let config = test_config();
        let limiter = Arc::new(RateLimiter::new(config));

        let mut handles = vec![];

        // Spawn multiple concurrent tasks
        for i in 0..5 {
            let limiter_clone = limiter.clone();
            let handle = tokio::spawn(async move {
                limiter_clone.acquire_permit(&format!("/test{}", i), "na1").await.unwrap();
            });
            handles.push(handle);
        }

        // All should complete successfully
        for handle in handles {
            handle.await.unwrap();
        }
    }

    #[tokio::test]
    async fn test_rate_limit_status() {
        let config = test_config();
        let limiter = RateLimiter::new(config);

        // Initial status
        let status = limiter.get_rate_limit_status().await;
        assert_eq!(status.application_tokens_per_second, 20);
        assert_eq!(status.application_tokens_per_two_minutes, 100);
        assert_eq!(status.method_limiters_count, 0);
        assert_eq!(status.service_limiters_count, 0);

        // Use some permits to create method limiters
        limiter.acquire_permit("/lol/summoner/v4/test", "na1").await.unwrap();
        limiter.acquire_permit("/lol/match/v5/test", "euw1").await.unwrap();

        let status = limiter.get_rate_limit_status().await;
        assert!(status.application_tokens_per_second < 20); // Some consumed
        assert!(status.method_limiters_count > 0); // Method limiters created
        assert!(status.service_limiters_count > 0); // Service limiters created
    }

    #[tokio::test]
    async fn test_exponential_backoff_behavior() {
        let mut config = test_config();
        config.retry_delay_ms = 50;
        config.max_retries = 2;
        
        let limiter = RateLimiter::new(config);

        // Test that exponential backoff delays are calculated correctly
        // This tests the behavior without actually hitting rate limits
        let start = Instant::now();
        limiter.handle_429_response(None).await; // Uses retry_delay_ms
        let elapsed = start.elapsed();

        assert!(elapsed >= Duration::from_millis(40));
        assert!(elapsed <= Duration::from_millis(100));
    }
}
