use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Config {
    pub riot_api_key: String,
    pub database_url: String,
    pub regions: Vec<String>,
    pub rate_limits: RateLimitConfig,
    pub crawler: CrawlerConfig,
    pub logging: LoggingConfig,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RateLimitConfig {
    pub application_limit_per_second: u32,
    pub application_limit_per_two_minutes: u32,
    pub max_concurrent_requests: u32,
    pub retry_delay_ms: u64,
    pub max_retries: u32,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CrawlerConfig {
    pub queue_size_limit: usize,
    pub batch_size: usize,
    pub health_check_interval_seconds: u64,
    pub state_save_interval_seconds: u64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LoggingConfig {
    pub level: String,
    pub format: String,
}

impl Default for Config {
    fn default() -> Self {
        Self {
            riot_api_key: String::new(),
            database_url: "./data/lol_crawler.db".to_string(),
            regions: vec![
                "na1".to_string(),
                "euw1".to_string(),
                "kr".to_string(),
                "eun1".to_string(),
            ],
            rate_limits: RateLimitConfig {
                application_limit_per_second: 20,
                application_limit_per_two_minutes: 100,
                max_concurrent_requests: 10,
                retry_delay_ms: 1000,
                max_retries: 3,
            },
            crawler: CrawlerConfig {
                queue_size_limit: 100_000,
                batch_size: 100,
                health_check_interval_seconds: 60,
                state_save_interval_seconds: 300,
            },
            logging: LoggingConfig {
                level: "info".to_string(),
                format: "json".to_string(),
            },
        }
    }
}

impl Config {
    pub fn from_env() -> crate::Result<Self> {
        Self::from_env_with_dotenv(true)
    }

    #[cfg(test)]
    pub fn from_env_no_dotenv() -> crate::Result<Self> {
        Self::from_env_with_dotenv(false)
    }

    fn from_env_with_dotenv(load_dotenv: bool) -> crate::Result<Self> {
        if load_dotenv {
            dotenv::dotenv().ok();
        }

        let mut config = Config::default();

        if let Ok(api_key) = std::env::var("RIOT_API_KEY") {
            config.riot_api_key = api_key;
        }

        if let Ok(db_url) = std::env::var("DATABASE_URL") {
            config.database_url = db_url;
        }

        if let Ok(regions) = std::env::var("REGIONS") {
            config.regions = regions.split(',').map(|s| s.trim().to_string()).collect();
        }

        if let Ok(log_level) = std::env::var("LOG_LEVEL") {
            config.logging.level = log_level;
        }

        // Rate limiting configuration
        if let Ok(app_limit_per_second) = std::env::var("APPLICATION_LIMIT_PER_SECOND") {
            if let Ok(limit) = app_limit_per_second.parse::<u32>() {
                config.rate_limits.application_limit_per_second = limit;
            }
        }

        if let Ok(app_limit_per_two_minutes) = std::env::var("APPLICATION_LIMIT_PER_TWO_MINUTES") {
            if let Ok(limit) = app_limit_per_two_minutes.parse::<u32>() {
                config.rate_limits.application_limit_per_two_minutes = limit;
            }
        }

        if let Ok(max_concurrent) = std::env::var("MAX_CONCURRENT_REQUESTS") {
            if let Ok(limit) = max_concurrent.parse::<u32>() {
                config.rate_limits.max_concurrent_requests = limit;
            }
        }

        // Crawler configuration
        if let Ok(queue_limit) = std::env::var("QUEUE_SIZE_LIMIT") {
            if let Ok(limit) = queue_limit.parse::<usize>() {
                config.crawler.queue_size_limit = limit;
            }
        }

        if let Ok(batch_size) = std::env::var("BATCH_SIZE") {
            if let Ok(size) = batch_size.parse::<usize>() {
                config.crawler.batch_size = size;
            }
        }

        if let Ok(health_interval) = std::env::var("HEALTH_CHECK_INTERVAL_SECONDS") {
            if let Ok(seconds) = health_interval.parse::<u64>() {
                config.crawler.health_check_interval_seconds = seconds;
            }
        }

        if let Ok(save_interval) = std::env::var("STATE_SAVE_INTERVAL_SECONDS") {
            if let Ok(seconds) = save_interval.parse::<u64>() {
                config.crawler.state_save_interval_seconds = seconds;
            }
        }

        // Validation
        if config.riot_api_key.is_empty() {
            anyhow::bail!("RIOT_API_KEY environment variable is required");
        }

        if !config.riot_api_key.starts_with("RGAPI-") {
            anyhow::bail!("RIOT_API_KEY must start with 'RGAPI-'");
        }

        // Validate regions
        let valid_regions = [
            "na1", "euw1", "eun1", "kr", "br1", "jp1", "ru", "oc1", "tr1", "la1", "la2",
        ];
        for region in &config.regions {
            if !valid_regions.contains(&region.as_str()) {
                anyhow::bail!(
                    "Invalid region '{}'. Valid regions: {}",
                    region,
                    valid_regions.join(", ")
                );
            }
        }

        // Validate rate limits
        if config.rate_limits.application_limit_per_second == 0 {
            anyhow::bail!("APPLICATION_LIMIT_PER_SECOND must be greater than 0");
        }

        if config.rate_limits.max_concurrent_requests == 0 {
            anyhow::bail!("MAX_CONCURRENT_REQUESTS must be greater than 0");
        }

        // Validate crawler config
        if config.crawler.queue_size_limit == 0 {
            anyhow::bail!("QUEUE_SIZE_LIMIT must be greater than 0");
        }

        Ok(config)
    }

    pub fn base_url_for_region(&self, region: &str) -> String {
        match region {
            "na1" => "https://na1.api.riotgames.com".to_string(),
            "euw1" => "https://euw1.api.riotgames.com".to_string(),
            "eun1" => "https://eun1.api.riotgames.com".to_string(),
            "kr" => "https://kr.api.riotgames.com".to_string(),
            "br1" => "https://br1.api.riotgames.com".to_string(),
            "jp1" => "https://jp1.api.riotgames.com".to_string(),
            "ru" => "https://ru.api.riotgames.com".to_string(),
            "oc1" => "https://oc1.api.riotgames.com".to_string(),
            "tr1" => "https://tr1.api.riotgames.com".to_string(),
            "la1" => "https://la1.api.riotgames.com".to_string(),
            "la2" => "https://la2.api.riotgames.com".to_string(),
            _ => format!("https://{}.api.riotgames.com", region),
        }
    }

    pub fn regional_base_url_for_region(&self, region: &str) -> String {
        match region {
            "na1" | "br1" | "la1" | "la2" => "https://americas.api.riotgames.com".to_string(),
            "euw1" | "eun1" | "tr1" | "ru" => "https://europe.api.riotgames.com".to_string(),
            "kr" | "jp1" => "https://asia.api.riotgames.com".to_string(),
            "oc1" => "https://sea.api.riotgames.com".to_string(),
            _ => "https://americas.api.riotgames.com".to_string(),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::env;

    fn setup_clean_env() {
        // Clean environment variables that might affect tests
        let env_vars = [
            "RIOT_API_KEY",
            "DATABASE_URL",
            "REGIONS",
            "LOG_LEVEL",
            "APPLICATION_LIMIT_PER_SECOND",
            "APPLICATION_LIMIT_PER_TWO_MINUTES",
            "MAX_CONCURRENT_REQUESTS",
            "QUEUE_SIZE_LIMIT",
            "BATCH_SIZE",
            "HEALTH_CHECK_INTERVAL_SECONDS",
            "STATE_SAVE_INTERVAL_SECONDS",
        ];

        for var in &env_vars {
            env::remove_var(var);
        }
    }

    fn set_minimal_valid_env() {
        env::set_var("RIOT_API_KEY", "RGAPI-test-key-123");
    }

    #[test]
    fn test_default_config() {
        let config = Config::default();

        // Test default values
        assert_eq!(config.riot_api_key, "");
        assert_eq!(config.database_url, "./data/lol_crawler.db");
        assert_eq!(config.regions, vec!["na1", "euw1", "kr", "eun1"]);

        // Test rate limit defaults
        assert_eq!(config.rate_limits.application_limit_per_second, 20);
        assert_eq!(config.rate_limits.application_limit_per_two_minutes, 100);
        assert_eq!(config.rate_limits.max_concurrent_requests, 10);
        assert_eq!(config.rate_limits.retry_delay_ms, 1000);
        assert_eq!(config.rate_limits.max_retries, 3);

        // Test crawler defaults
        assert_eq!(config.crawler.queue_size_limit, 100_000);
        assert_eq!(config.crawler.batch_size, 100);
        assert_eq!(config.crawler.health_check_interval_seconds, 60);
        assert_eq!(config.crawler.state_save_interval_seconds, 300);

        // Test logging defaults
        assert_eq!(config.logging.level, "info");
        assert_eq!(config.logging.format, "json");
    }

    #[test]
    fn test_config_from_env_minimal_valid() {
        setup_clean_env();
        set_minimal_valid_env();

        let config = Config::from_env_no_dotenv().unwrap();

        assert_eq!(config.riot_api_key, "RGAPI-test-key-123");
        // Should use defaults for everything else
        assert_eq!(config.database_url, "./data/lol_crawler.db");
        assert_eq!(config.regions, vec!["na1", "euw1", "kr", "eun1"]);

        setup_clean_env(); // Clean up after test
    }

    #[test]
    fn test_config_from_env_missing_api_key() {
        setup_clean_env();

        let result = Config::from_env_no_dotenv();
        assert!(
            result.is_err(),
            "Expected error for missing API key, but got success"
        );
        assert!(result
            .unwrap_err()
            .to_string()
            .contains("RIOT_API_KEY environment variable is required"));

        setup_clean_env(); // Clean up after test
    }

    #[test]
    fn test_config_from_env_invalid_api_key_format() {
        setup_clean_env();
        env::set_var("RIOT_API_KEY", "invalid-key-format");

        let result = Config::from_env_no_dotenv();
        assert!(result.is_err());
        assert!(result
            .unwrap_err()
            .to_string()
            .contains("RIOT_API_KEY must start with 'RGAPI-'"));

        setup_clean_env(); // Clean up after test
    }

    #[test]
    fn test_config_from_env_all_variables() {
        setup_clean_env();

        // Set all environment variables
        env::set_var("RIOT_API_KEY", "RGAPI-complete-test-key");
        env::set_var("DATABASE_URL", "./test_data/custom.db");
        env::set_var("REGIONS", "na1,euw1,kr");
        env::set_var("LOG_LEVEL", "debug");
        env::set_var("APPLICATION_LIMIT_PER_SECOND", "50");
        env::set_var("APPLICATION_LIMIT_PER_TWO_MINUTES", "500");
        env::set_var("MAX_CONCURRENT_REQUESTS", "25");
        env::set_var("QUEUE_SIZE_LIMIT", "50000");
        env::set_var("BATCH_SIZE", "200");
        env::set_var("HEALTH_CHECK_INTERVAL_SECONDS", "120");
        env::set_var("STATE_SAVE_INTERVAL_SECONDS", "600");

        let config = Config::from_env_no_dotenv().unwrap();

        // Verify all values were parsed correctly
        assert_eq!(config.riot_api_key, "RGAPI-complete-test-key");
        assert_eq!(config.database_url, "./test_data/custom.db");
        assert_eq!(config.regions, vec!["na1", "euw1", "kr"]);
        assert_eq!(config.logging.level, "debug");
        assert_eq!(config.rate_limits.application_limit_per_second, 50);
        assert_eq!(config.rate_limits.application_limit_per_two_minutes, 500);
        assert_eq!(config.rate_limits.max_concurrent_requests, 25);
        assert_eq!(config.crawler.queue_size_limit, 50000);
        assert_eq!(config.crawler.batch_size, 200);
        assert_eq!(config.crawler.health_check_interval_seconds, 120);
        assert_eq!(config.crawler.state_save_interval_seconds, 600);

        setup_clean_env(); // Clean up after test
    }

    #[test]
    fn test_regions_parsing() {
        setup_clean_env();
        set_minimal_valid_env();

        // Test single region
        env::set_var("REGIONS", "na1");
        let config = Config::from_env_no_dotenv().unwrap();
        assert_eq!(config.regions, vec!["na1"]);

        // Test multiple regions with spaces
        env::set_var("REGIONS", " na1 , euw1 , kr ");
        let config = Config::from_env_no_dotenv().unwrap();
        assert_eq!(config.regions, vec!["na1", "euw1", "kr"]);

        // Test all valid regions
        env::set_var("REGIONS", "na1,euw1,eun1,kr,br1,jp1,ru,oc1,tr1,la1,la2");
        let config = Config::from_env_no_dotenv().unwrap();
        assert_eq!(config.regions.len(), 11);

        setup_clean_env(); // Clean up after test
    }

    #[test]
    fn test_invalid_regions() {
        setup_clean_env();
        set_minimal_valid_env();

        env::set_var("REGIONS", "na1,invalid_region,kr");
        let result = Config::from_env_no_dotenv();
        assert!(result.is_err());
        assert!(result
            .unwrap_err()
            .to_string()
            .contains("Invalid region 'invalid_region'"));

        setup_clean_env(); // Clean up after test
    }

    #[test]
    fn test_invalid_numeric_parsing() {
        setup_clean_env();
        set_minimal_valid_env();

        // Test invalid application limit - should use default
        env::set_var("APPLICATION_LIMIT_PER_SECOND", "not_a_number");
        let config = Config::from_env_no_dotenv().unwrap();
        assert_eq!(config.rate_limits.application_limit_per_second, 20); // default

        // Test invalid queue size - should use default
        env::set_var("QUEUE_SIZE_LIMIT", "invalid");
        let config = Config::from_env_no_dotenv().unwrap();
        assert_eq!(config.crawler.queue_size_limit, 100_000); // default

        setup_clean_env(); // Clean up after test
    }

    #[test]
    fn test_validation_zero_rate_limits() {
        setup_clean_env();
        set_minimal_valid_env();

        env::set_var("APPLICATION_LIMIT_PER_SECOND", "0");
        let result = Config::from_env_no_dotenv();
        assert!(result.is_err());
        assert!(result
            .unwrap_err()
            .to_string()
            .contains("APPLICATION_LIMIT_PER_SECOND must be greater than 0"));

        setup_clean_env(); // Clean up after test
    }

    #[test]
    fn test_validation_zero_concurrent_requests() {
        setup_clean_env();
        set_minimal_valid_env();

        env::set_var("MAX_CONCURRENT_REQUESTS", "0");
        let result = Config::from_env_no_dotenv();
        assert!(result.is_err());
        assert!(result
            .unwrap_err()
            .to_string()
            .contains("MAX_CONCURRENT_REQUESTS must be greater than 0"));

        setup_clean_env(); // Clean up after test
    }

    #[test]
    fn test_validation_zero_queue_size() {
        setup_clean_env();
        set_minimal_valid_env();

        env::set_var("QUEUE_SIZE_LIMIT", "0");
        let result = Config::from_env_no_dotenv();
        assert!(result.is_err());
        assert!(result
            .unwrap_err()
            .to_string()
            .contains("QUEUE_SIZE_LIMIT must be greater than 0"));

        setup_clean_env(); // Clean up after test
    }

    #[test]
    fn test_base_url_for_region() {
        let config = Config::default();

        // Test all known regions
        assert_eq!(
            config.base_url_for_region("na1"),
            "https://na1.api.riotgames.com"
        );
        assert_eq!(
            config.base_url_for_region("euw1"),
            "https://euw1.api.riotgames.com"
        );
        assert_eq!(
            config.base_url_for_region("kr"),
            "https://kr.api.riotgames.com"
        );
        assert_eq!(
            config.base_url_for_region("br1"),
            "https://br1.api.riotgames.com"
        );
        assert_eq!(
            config.base_url_for_region("jp1"),
            "https://jp1.api.riotgames.com"
        );

        // Test unknown region fallback
        assert_eq!(
            config.base_url_for_region("unknown"),
            "https://unknown.api.riotgames.com"
        );
    }

    #[test]
    fn test_regional_base_url_for_region() {
        let config = Config::default();

        // Test Americas
        assert_eq!(
            config.regional_base_url_for_region("na1"),
            "https://americas.api.riotgames.com"
        );
        assert_eq!(
            config.regional_base_url_for_region("br1"),
            "https://americas.api.riotgames.com"
        );
        assert_eq!(
            config.regional_base_url_for_region("la1"),
            "https://americas.api.riotgames.com"
        );
        assert_eq!(
            config.regional_base_url_for_region("la2"),
            "https://americas.api.riotgames.com"
        );

        // Test Europe
        assert_eq!(
            config.regional_base_url_for_region("euw1"),
            "https://europe.api.riotgames.com"
        );
        assert_eq!(
            config.regional_base_url_for_region("eun1"),
            "https://europe.api.riotgames.com"
        );
        assert_eq!(
            config.regional_base_url_for_region("tr1"),
            "https://europe.api.riotgames.com"
        );
        assert_eq!(
            config.regional_base_url_for_region("ru"),
            "https://europe.api.riotgames.com"
        );

        // Test Asia
        assert_eq!(
            config.regional_base_url_for_region("kr"),
            "https://asia.api.riotgames.com"
        );
        assert_eq!(
            config.regional_base_url_for_region("jp1"),
            "https://asia.api.riotgames.com"
        );

        // Test SEA
        assert_eq!(
            config.regional_base_url_for_region("oc1"),
            "https://sea.api.riotgames.com"
        );

        // Test unknown region fallback
        assert_eq!(
            config.regional_base_url_for_region("unknown"),
            "https://americas.api.riotgames.com"
        );
    }

    #[test]
    fn test_config_serialization() {
        let config = Config::default();

        // Test that config can be serialized to JSON
        let json = serde_json::to_string(&config);
        assert!(json.is_ok());

        // Test that it can be deserialized back
        let deserialized: Result<Config, _> = serde_json::from_str(&json.unwrap());
        assert!(deserialized.is_ok());

        let restored_config = deserialized.unwrap();
        assert_eq!(restored_config.database_url, config.database_url);
        assert_eq!(restored_config.regions, config.regions);
        assert_eq!(
            restored_config.rate_limits.application_limit_per_second,
            config.rate_limits.application_limit_per_second
        );
    }

    #[test]
    fn test_edge_cases_parsing() {
        setup_clean_env();
        set_minimal_valid_env();

        // Test empty regions string
        env::set_var("REGIONS", "");
        let result = Config::from_env_no_dotenv();

        // Empty regions string creates a single empty region which should fail validation
        assert!(result.is_err());
        assert!(result.unwrap_err().to_string().contains("Invalid region"));

        setup_clean_env(); // Clean up after test
    }

    #[test]
    fn test_extremely_large_values() {
        setup_clean_env();
        set_minimal_valid_env();

        // Test very large but valid values
        env::set_var("APPLICATION_LIMIT_PER_SECOND", "999999");
        env::set_var("QUEUE_SIZE_LIMIT", "9999999999");
        env::set_var("HEALTH_CHECK_INTERVAL_SECONDS", "86400"); // 1 day

        let config = Config::from_env_no_dotenv().unwrap();
        assert_eq!(config.rate_limits.application_limit_per_second, 999999);
        assert_eq!(config.crawler.queue_size_limit, 9999999999);
        assert_eq!(config.crawler.health_check_interval_seconds, 86400);

        setup_clean_env(); // Clean up after test
    }

    #[test]
    fn test_boundary_values() {
        setup_clean_env();
        set_minimal_valid_env();

        // Test boundary value of 1 (minimum allowed)
        env::set_var("APPLICATION_LIMIT_PER_SECOND", "1");
        env::set_var("MAX_CONCURRENT_REQUESTS", "1");
        env::set_var("QUEUE_SIZE_LIMIT", "1");

        let config = Config::from_env_no_dotenv().unwrap();
        assert_eq!(config.rate_limits.application_limit_per_second, 1);
        assert_eq!(config.rate_limits.max_concurrent_requests, 1);
        assert_eq!(config.crawler.queue_size_limit, 1);

        setup_clean_env(); // Clean up after test
    }

    #[test]
    fn test_special_api_key_formats() {
        setup_clean_env();

        // Test minimal valid API key
        env::set_var("RIOT_API_KEY", "RGAPI-");
        let result = Config::from_env_no_dotenv();
        assert!(result.is_ok());

        // Test very long API key
        env::set_var(
            "RIOT_API_KEY",
            "RGAPI-abcdef123456789012345678901234567890abcdef123456789012345678901234567890",
        );
        let result = Config::from_env_no_dotenv();
        assert!(result.is_ok());

        // Test API key with special characters
        env::set_var("RIOT_API_KEY", "RGAPI-abc_def-123.456");
        let result = Config::from_env_no_dotenv();
        assert!(result.is_ok());

        setup_clean_env(); // Clean up after test
    }

    #[test]
    fn test_env_var_case_sensitivity() {
        setup_clean_env();

        // Environment variables should be case sensitive
        env::set_var("riot_api_key", "RGAPI-lowercase"); // Wrong case
        env::set_var("RIOT_API_KEY", "RGAPI-correct-case");

        let config = Config::from_env_no_dotenv().unwrap();
        assert_eq!(config.riot_api_key, "RGAPI-correct-case");

        setup_clean_env(); // Clean up after test
    }

    #[test]
    fn test_partial_env_override() {
        setup_clean_env();
        set_minimal_valid_env();

        // Set only some environment variables
        env::set_var("DATABASE_URL", "/custom/path/db.sqlite");
        env::set_var("APPLICATION_LIMIT_PER_SECOND", "25");
        // Leave others as defaults

        let config = Config::from_env_no_dotenv().unwrap();

        // Overridden values
        assert_eq!(config.database_url, "/custom/path/db.sqlite");
        assert_eq!(config.rate_limits.application_limit_per_second, 25);

        // Default values
        assert_eq!(config.regions, vec!["na1", "euw1", "kr", "eun1"]);
        assert_eq!(config.rate_limits.application_limit_per_two_minutes, 100);
        assert_eq!(config.crawler.batch_size, 100);

        setup_clean_env(); // Clean up after test
    }
}
