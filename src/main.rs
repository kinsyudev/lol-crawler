use lol_crawler::{Config, CrawlerEngine, Database};
use std::process;

#[tokio::main]
async fn main() {
    // Initialize logging
    env_logger::init();

    // Load configuration
    let config = match Config::from_env() {
        Ok(config) => config,
        Err(e) => {
            eprintln!("Failed to load configuration: {}", e);
            process::exit(1);
        }
    };

    log::info!("Starting League of Legends crawler with config:");
    log::info!("- Regions: {:?}", config.regions);
    log::info!("- Database: {}", config.database_url);
    log::info!(
        "- Rate limits: {} per second, {} per 2 minutes",
        config.rate_limits.application_limit_per_second,
        config.rate_limits.application_limit_per_two_minutes
    );

    // Initialize database
    let database = match Database::new(&config.database_url) {
        Ok(db) => db,
        Err(e) => {
            eprintln!("Failed to initialize database: {}", e);
            process::exit(1);
        }
    };

    log::info!("Database initialized successfully");

    // Create and start crawler
    let crawler = match CrawlerEngine::new(config, database) {
        Ok(crawler) => crawler,
        Err(e) => {
            eprintln!("Failed to create crawler engine: {}", e);
            process::exit(1);
        }
    };

    // Handle graceful shutdown
    let crawler_ref = &crawler;
    let shutdown_task = async {
        tokio::signal::ctrl_c()
            .await
            .expect("Failed to listen for ctrl+c");
        log::info!("Received shutdown signal");
        crawler_ref.stop().await;
    };

    // Run crawler and shutdown handler
    tokio::select! {
        result = crawler.start() => {
            match result {
                Ok(_) => log::info!("Crawler finished successfully"),
                Err(e) => {
                    log::error!("Crawler failed: {}", e);
                    process::exit(1);
                }
            }
        }
        _ = shutdown_task => {
            log::info!("Shutdown completed");
        }
    }
}
