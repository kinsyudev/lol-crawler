use super::{queue::SummonerQueue, worker::CrawlerWorker};
use crate::api::RiotApiClient;
use crate::config::Config;
use crate::database::Database;
use crate::models::database::{DbActiveGame, DbCrawlerState, SummonerPriority, SummonerTask};
use crate::rate_limiter::RateLimiter;
use chrono::Utc;
use std::sync::Arc;
use std::time::Duration;
use tokio::time::{interval, sleep};

pub struct CrawlerEngine {
    api_client: RiotApiClient,
    database: Database,
    summoner_queue: SummonerQueue,
    worker: CrawlerWorker,
    config: Config,
    running: Arc<tokio::sync::RwLock<bool>>,
}

impl CrawlerEngine {
    pub fn new(config: Config, database: Database) -> crate::Result<Self> {
        let rate_limiter = Arc::new(RateLimiter::new(config.rate_limits.clone()));
        let api_client = RiotApiClient::new(config.clone(), rate_limiter, database.clone())?;
        let worker = CrawlerWorker::new(api_client.clone(), database.clone());
        let summoner_queue = SummonerQueue::new();

        Ok(Self {
            api_client,
            database,
            summoner_queue,
            worker,
            config,
            running: Arc::new(tokio::sync::RwLock::new(false)),
        })
    }

    pub async fn start(&self) -> crate::Result<()> {
        {
            let mut running = self.running.write().await;
            if *running {
                log::warn!("Crawler is already running");
                return Ok(());
            }
            *running = true;
        }

        log::info!("Starting League of Legends crawler");

        // Initialize with featured games from all regions
        self.seed_with_featured_games().await?;

        // Spawn background tasks
        let featured_games_task = self.spawn_featured_games_task();
        let crawler_task = self.spawn_crawler_task();
        let health_check_task = self.spawn_health_check_task();
        let state_save_task = self.spawn_state_save_task();

        // Wait for all tasks
        tokio::try_join!(
            featured_games_task,
            crawler_task,
            health_check_task,
            state_save_task
        )?;

        Ok(())
    }

    pub async fn stop(&self) {
        log::info!("Stopping crawler");
        let mut running = self.running.write().await;
        *running = false;
    }

    async fn is_running(&self) -> bool {
        *self.running.read().await
    }

    async fn seed_with_featured_games(&self) -> crate::Result<()> {
        log::info!("Seeding crawler with featured games from all regions");

        for region in &self.config.regions {
            match self.process_featured_games_for_region(region).await {
                Ok(count) => {
                    log::info!("Added {} high-priority summoners from {} featured games", count, region);
                }
                Err(e) => {
                    log::error!("Failed to process featured games for region {}: {}", region, e);
                }
            }
        }

        let total_size = self.summoner_queue.total_size().await;
        log::info!("Initial queue size: {}", total_size);

        Ok(())
    }

    async fn process_featured_games_for_region(&self, region: &str) -> crate::Result<usize> {
        // Try featured games first, fallback to master league if not accessible
        let summoner_tasks = match self.api_client.get_featured_games(region).await {
            Ok(featured_games) => {
                log::info!("Using featured games for seeding in region {}", region);
                self.extract_summoners_from_featured_games(featured_games, region).await?
            }
            Err(e) => {
                log::warn!("Featured games not accessible ({}), falling back to master league", e);
                self.extract_summoners_from_master_league(region).await?
            }
        };

        let count = summoner_tasks.len();
        if !summoner_tasks.is_empty() {
            self.summoner_queue.push_batch(summoner_tasks).await;
        }

        Ok(count)
    }

    async fn extract_summoners_from_featured_games(&self, featured_games: crate::models::riot::FeaturedGamesResponse, region: &str) -> crate::Result<Vec<SummonerTask>> {
        let mut summoner_tasks = Vec::new();

        for game in featured_games.game_list {
            // Store active game
            let active_game = DbActiveGame {
                game_id: game.game_id as i64,
                game_type: game.game_type,
                game_start_time: game.game_start_time as i64,
                map_id: game.map_id as i32,
                queue_id: game.game_queue_config_id.unwrap_or(0) as i32,
                platform_id: game.platform_id,
                game_mode: game.game_mode,
                participants: serde_json::to_string(&game.participants)?,
                discovered_at: Utc::now(),
            };

            if let Err(e) = self.database.insert_active_game(&active_game) {
                log::warn!("Failed to store active game {}: {}", game.game_id, e);
            }

            // Extract summoners from participants
            for participant in game.participants {
                if let Some(puuid) = participant.puuid {
                    // Check if we already have this summoner
                    match self.database.summoner_exists(&puuid) {
                        Ok(true) => continue, // Skip existing summoners
                        Ok(false) => {
                            // New summoner - add to high priority queue
                            summoner_tasks.push(SummonerTask {
                                puuid,
                                summoner_name: participant.summoner_name,
                                region: region.to_string(),
                                priority: SummonerPriority::High,
                                added_at: Utc::now(),
                                retries: 0,
                            });
                        }
                        Err(e) => {
                            log::warn!("Failed to check if summoner exists: {}", e);
                            // Add anyway to be safe
                            summoner_tasks.push(SummonerTask {
                                puuid,
                                summoner_name: participant.summoner_name,
                                region: region.to_string(),
                                priority: SummonerPriority::High,
                                added_at: Utc::now(),
                                retries: 0,
                            });
                        }
                    }
                }
            }
        }

        Ok(summoner_tasks)
    }

    async fn extract_summoners_from_master_league(&self, region: &str) -> crate::Result<Vec<SummonerTask>> {
        log::info!("Fetching master league players for region {}", region);
        
        let master_league = self.api_client.get_master_league(region, "RANKED_SOLO_5x5").await?;
        let mut summoner_tasks = Vec::new();

        for entry in master_league.entries.into_iter().take(50) { // Limit to 50 for initial seeding
            // Check if we already have this summoner
            match self.database.summoner_exists(&entry.puuid) {
                Ok(true) => continue, // Skip existing summoners
                Ok(false) => {
                    // New summoner - add to high priority queue
                    summoner_tasks.push(SummonerTask {
                        puuid: entry.puuid.clone(),
                        summoner_name: format!("Master_Player_{}", &entry.puuid[..8]), // Temporary name, will be resolved later
                        region: region.to_string(),
                        priority: SummonerPriority::High,
                        added_at: Utc::now(),
                        retries: 0,
                    });
                }
                Err(e) => {
                    log::warn!("Failed to check if summoner exists: {}", e);
                    // Add anyway to be safe
                    summoner_tasks.push(SummonerTask {
                        puuid: entry.puuid.clone(),
                        summoner_name: format!("Master_Player_{}", &entry.puuid[..8]),
                        region: region.to_string(),
                        priority: SummonerPriority::High,
                        added_at: Utc::now(),
                        retries: 0,
                    });
                }
            }
        }

        log::info!("Found {} master league players in region {}", summoner_tasks.len(), region);
        Ok(summoner_tasks)
    }

    async fn spawn_featured_games_task(&self) -> crate::Result<()> {
        let mut interval = interval(Duration::from_secs(self.config.crawler.featured_games_interval_seconds));
        let _api_client = self.api_client.clone();
        let _database = self.database.clone();
        let _queue = &self.summoner_queue;
        let regions = self.config.regions.clone();
        let running = self.running.clone();

        loop {
            interval.tick().await;

            if !*running.read().await {
                break;
            }

            log::debug!("Refreshing featured games");

            for region in &regions {
                match self.process_featured_games_for_region(region).await {
                    Ok(count) => {
                        if count > 0 {
                            log::info!("Added {} new summoners from {} featured games", count, region);
                        }
                    }
                    Err(e) => {
                        log::error!("Failed to refresh featured games for region {}: {}", region, e);
                    }
                }
            }
        }

        Ok(())
    }

    async fn spawn_crawler_task(&self) -> crate::Result<()> {
        let running = self.running.clone();
        let mut processed_count = 0;
        let mut matches_processed = 0;

        while *running.read().await {
            // Check if queue is empty
            if self.summoner_queue.is_empty().await {
                log::debug!("Queue is empty, waiting for new summoners");
                sleep(Duration::from_secs(30)).await;
                continue;
            }

            // Process next summoner
            if let Some(task) = self.summoner_queue.pop().await {
                match self.worker.process_summoner(&task).await {
                    Ok(new_tasks) => {
                        processed_count += 1;
                        let match_count = new_tasks.len();
                        matches_processed += match_count;

                        log::info!(
                            "Processed summoner {} ({}), discovered {} new summoners",
                            task.summoner_name,
                            task.puuid,
                            match_count
                        );

                        // Add new summoners to queue
                        if !new_tasks.is_empty() {
                            self.summoner_queue.push_batch(new_tasks).await;
                        }

                        // Periodic queue cleanup
                        if processed_count % 100 == 0 {
                            self.summoner_queue.remove_duplicates().await;
                            let (high, medium, low) = self.summoner_queue.size().await;
                            log::info!(
                                "Queue status: {} high, {} medium, {} low priority summoners",
                                high, medium, low
                            );
                        }
                    }
                    Err(e) => {
                        log::error!("Failed to process summoner {}: {}", task.summoner_name, e);

                        // Retry logic
                        if task.retries < 3 {
                            let mut retry_task = task.clone();
                            retry_task.retries += 1;
                            retry_task.priority = SummonerPriority::Low; // Demote on retry
                            self.summoner_queue.push(retry_task).await;
                        }
                    }
                }

                // Rate limiting - small delay between requests
                sleep(Duration::from_millis(100)).await;
            }
        }

        log::info!("Crawler task completed. Processed {} summoners, {} matches", processed_count, matches_processed);
        Ok(())
    }

    async fn spawn_health_check_task(&self) -> crate::Result<()> {
        let mut interval = interval(Duration::from_secs(self.config.crawler.health_check_interval_seconds));
        let running = self.running.clone();

        loop {
            interval.tick().await;

            if !*running.read().await {
                break;
            }

            // Get current stats
            let (high, medium, low) = self.summoner_queue.size().await;
            let rate_limit_status = self.api_client.get_rate_limit_status().await;

            let matches_count = self.database.get_matches_count().unwrap_or(0);
            let summoners_count = self.database.get_summoners_count().unwrap_or(0);
            let participants_count = self.database.get_participants_count().unwrap_or(0);

            log::info!(
                "Health Check - Queue: {}H/{}M/{}L, DB: {}M/{}S/{}P, Rate Limits: {}/{}",
                high, medium, low,
                matches_count, summoners_count, participants_count,
                rate_limit_status.application_tokens_per_second,
                rate_limit_status.application_tokens_per_two_minutes
            );
        }

        Ok(())
    }

    async fn spawn_state_save_task(&self) -> crate::Result<()> {
        let mut interval = interval(Duration::from_secs(self.config.crawler.state_save_interval_seconds));
        let running = self.running.clone();

        loop {
            interval.tick().await;

            if !*running.read().await {
                break;
            }

            // Save crawler state
            let total_queue_size = self.summoner_queue.total_size().await;
            let matches_count = self.database.get_matches_count().unwrap_or(0);
            let summoners_count = self.database.get_summoners_count().unwrap_or(0);

            let state = DbCrawlerState {
                id: 1,
                last_processed_summoner: None, // Could track this if needed
                total_summoners_processed: summoners_count as i32,
                total_matches_processed: matches_count as i32,
                queue_size: total_queue_size as i32,
                last_update: Utc::now(),
            };

            if let Err(e) = self.database.update_crawler_state(&state) {
                log::error!("Failed to save crawler state: {}", e);
            } else {
                log::debug!("Crawler state saved");
            }
        }

        Ok(())
    }

    pub async fn get_status(&self) -> CrawlerStatus {
        let (high, medium, low) = self.summoner_queue.size().await;
        let rate_limit_status = self.api_client.get_rate_limit_status().await;

        CrawlerStatus {
            running: self.is_running().await,
            queue_sizes: QueueSizes { high, medium, low },
            rate_limit_status,
            database_stats: DatabaseStats {
                matches: self.database.get_matches_count().unwrap_or(0),
                summoners: self.database.get_summoners_count().unwrap_or(0),
                participants: self.database.get_participants_count().unwrap_or(0),
            },
        }
    }
}

#[derive(Debug)]
pub struct CrawlerStatus {
    pub running: bool,
    pub queue_sizes: QueueSizes,
    pub rate_limit_status: crate::rate_limiter::RateLimitStatus,
    pub database_stats: DatabaseStats,
}

#[derive(Debug)]
pub struct QueueSizes {
    pub high: usize,
    pub medium: usize,
    pub low: usize,
}

#[derive(Debug)]
pub struct DatabaseStats {
    pub matches: i64,
    pub summoners: i64,
    pub participants: i64,
}