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
