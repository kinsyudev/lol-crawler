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
        dotenv::dotenv().ok();

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
        let valid_regions = ["na1", "euw1", "eun1", "kr", "br1", "jp1", "ru", "oc1", "tr1", "la1", "la2"];
        for region in &config.regions {
            if !valid_regions.contains(&region.as_str()) {
                anyhow::bail!("Invalid region '{}'. Valid regions: {}", region, valid_regions.join(", "));
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
