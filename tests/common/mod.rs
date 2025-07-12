use chrono::Utc;
use lol_crawler::config::{Config, CrawlerConfig, LoggingConfig, RateLimitConfig};
use lol_crawler::models::database::{DbMatch, DbParticipant, DbSummoner};

pub fn test_config() -> Config {
    Config {
        riot_api_key: "RGAPI-test-integration-key".to_string(),
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
            batch_size: 50,
            health_check_interval_seconds: 60,
            state_save_interval_seconds: 300,
        },
        logging: LoggingConfig {
            level: "info".to_string(),
            format: "json".to_string(),
        },
    }
}

pub fn create_test_summoner(puuid: &str) -> DbSummoner {
    DbSummoner {
        puuid: puuid.to_string(),
        summoner_id: format!("summoner-id-{}", puuid),
        account_id: format!("account-id-{}", puuid),
        summoner_name: format!("TestPlayer_{}", &puuid[..8]),
        profile_icon_id: 1234,
        summoner_level: 100,
        region: "na1".to_string(),
        created_at: Utc::now(),
        updated_at: Utc::now(),
    }
}

pub fn create_test_match(match_id: &str, queue_id: i32) -> DbMatch {
    DbMatch {
        match_id: match_id.to_string(),
        game_creation: 1640000000000,
        game_duration: 1800,
        game_end_timestamp: Some(1640001800000),
        game_id: match_id.len() as i64, // Simple ID generation
        game_mode: "CLASSIC".to_string(),
        game_name: Some("Test Game".to_string()),
        game_type: "MATCHED_GAME".to_string(),
        game_version: "13.1.1".to_string(),
        map_id: 11,
        platform_id: "NA1".to_string(),
        queue_id,
        tournament_code: None,
        region: "na1".to_string(),
        created_at: Utc::now(),
    }
}

pub fn create_test_participant(match_id: &str, puuid: &str) -> DbParticipant {
    DbParticipant {
        id: None,
        match_id: match_id.to_string(),
        puuid: puuid.to_string(),
        summoner_name: format!("Player_{}", &puuid[..8]),
        champion_id: 157, // Yasuo
        champion_name: Some("Yasuo".to_string()),
        team_id: 100,
        position: Some("MIDDLE".to_string()),
        individual_position: Some("MIDDLE".to_string()),
        kills: 5,
        deaths: 3,
        assists: 10,
        total_damage_dealt: 150000,
        total_damage_dealt_to_champions: 25000,
        total_damage_taken: 20000,
        gold_earned: 15000,
        gold_spent: 14000,
        turret_kills: 2,
        inhibitor_kills: 1,
        total_minions_killed: 200,
        neutral_minions_killed: 50,
        champion_level: 18,
        items_0: 3006,
        items_1: 3087,
        items_2: 3031,
        items_3: 3036,
        items_4: 3072,
        items_5: 3026,
        items_6: 3340,
        summoner_spell_1: 4,             // Flash
        summoner_spell_2: 12,            // Teleport
        primary_rune_tree: Some(8000),   // Precision
        secondary_rune_tree: Some(8300), // Inspiration
        win: true,
        first_blood_kill: false,
        first_tower_kill: true,
    }
}
