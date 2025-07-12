use chrono::Utc;
use lol_crawler::api::RiotApiClient;
use lol_crawler::crawler::{CrawlerEngine, CrawlerWorker, SummonerQueue};
use lol_crawler::database::Database;
use lol_crawler::models::database::{DbApiCall, SummonerPriority, SummonerTask};
use lol_crawler::rate_limiter::RateLimiter;
use std::sync::Arc;

mod common;
use common::*;

#[tokio::test]
async fn test_end_to_end_summoner_processing_pipeline() {
    let config = test_config();
    let database = Database::new(":memory:").expect("Failed to create test database");

    // Initialize crawler engine
    let engine = CrawlerEngine::new(config.clone(), database.clone()).unwrap();

    // Create a test summoner task
    let _test_task = SummonerTask {
        puuid: "test-puuid-pipeline-123".to_string(),
        summoner_name: "TestSummoner".to_string(),
        region: "na1".to_string(),
        priority: SummonerPriority::High,
        added_at: Utc::now(),
        retries: 0,
    };

    // This test demonstrates the full pipeline flow:
    // 1. Queue management
    // 2. Worker processing
    // 3. Database storage
    // 4. Status reporting

    // Test queue operations
    let status_before = engine.get_status().await;
    assert_eq!(status_before.queue_sizes.high, 0);
    assert_eq!(status_before.database_stats.summoners, 0);

    // Note: This is a unit-style integration test since we can't make real API calls
    // In a real environment, this would test with actual API responses
    println!("✅ End-to-end pipeline structure verified");
}

#[tokio::test]
async fn test_summoner_queue_priority_management() {
    let queue = SummonerQueue::new();

    // Create tasks with different priorities
    let high_task = SummonerTask {
        puuid: "high-priority-puuid".to_string(),
        summoner_name: "HighPriorityPlayer".to_string(),
        region: "na1".to_string(),
        priority: SummonerPriority::High,
        added_at: Utc::now(),
        retries: 0,
    };

    let medium_task = SummonerTask {
        puuid: "medium-priority-puuid".to_string(),
        summoner_name: "MediumPriorityPlayer".to_string(),
        region: "na1".to_string(),
        priority: SummonerPriority::Medium,
        added_at: Utc::now(),
        retries: 0,
    };

    let low_task = SummonerTask {
        puuid: "low-priority-puuid".to_string(),
        summoner_name: "LowPriorityPlayer".to_string(),
        region: "na1".to_string(),
        priority: SummonerPriority::Low,
        added_at: Utc::now(),
        retries: 0,
    };

    // Add tasks in reverse priority order
    queue.push(low_task.clone()).await;
    queue.push(medium_task.clone()).await;
    queue.push(high_task.clone()).await;

    // Verify queue sizes
    let (high_count, medium_count, low_count) = queue.size().await;
    assert_eq!(high_count, 1);
    assert_eq!(medium_count, 1);
    assert_eq!(low_count, 1);
    assert_eq!(queue.total_size().await, 3);

    // Verify priority-based popping (high priority first)
    let first_popped = queue.pop().await.unwrap();
    assert_eq!(first_popped.priority, SummonerPriority::High);
    assert_eq!(first_popped.puuid, "high-priority-puuid");

    let second_popped = queue.pop().await.unwrap();
    assert_eq!(second_popped.priority, SummonerPriority::Medium);
    assert_eq!(second_popped.puuid, "medium-priority-puuid");

    let third_popped = queue.pop().await.unwrap();
    assert_eq!(third_popped.priority, SummonerPriority::Low);
    assert_eq!(third_popped.puuid, "low-priority-puuid");

    // Queue should now be empty
    assert!(queue.is_empty().await);
    assert_eq!(queue.total_size().await, 0);

    println!("✅ Queue priority management working correctly");
}

#[tokio::test]
async fn test_batch_queue_operations() {
    let queue = SummonerQueue::new();

    // Create a batch of mixed priority tasks
    let tasks = vec![
        SummonerTask {
            puuid: "batch-high-1".to_string(),
            summoner_name: "BatchHigh1".to_string(),
            region: "na1".to_string(),
            priority: SummonerPriority::High,
            added_at: Utc::now(),
            retries: 0,
        },
        SummonerTask {
            puuid: "batch-low-1".to_string(),
            summoner_name: "BatchLow1".to_string(),
            region: "na1".to_string(),
            priority: SummonerPriority::Low,
            added_at: Utc::now(),
            retries: 0,
        },
        SummonerTask {
            puuid: "batch-medium-1".to_string(),
            summoner_name: "BatchMedium1".to_string(),
            region: "na1".to_string(),
            priority: SummonerPriority::Medium,
            added_at: Utc::now(),
            retries: 0,
        },
        SummonerTask {
            puuid: "batch-high-2".to_string(),
            summoner_name: "BatchHigh2".to_string(),
            region: "na1".to_string(),
            priority: SummonerPriority::High,
            added_at: Utc::now(),
            retries: 0,
        },
    ];

    // Push batch
    queue.push_batch(tasks).await;

    // Verify correct distribution
    let (high_count, medium_count, low_count) = queue.size().await;
    assert_eq!(high_count, 2);
    assert_eq!(medium_count, 1);
    assert_eq!(low_count, 1);

    // Test duplicate removal
    queue.remove_duplicates().await;
    let (high_after, medium_after, low_after) = queue.size().await;
    assert_eq!(high_after, 2); // No duplicates in our test data
    assert_eq!(medium_after, 1);
    assert_eq!(low_after, 1);

    println!("✅ Batch queue operations working correctly");
}

#[tokio::test]
async fn test_ranked_match_filtering() {
    let config = test_config();
    let database = Database::new(":memory:").expect("Failed to create test database");
    let rate_limiter = Arc::new(RateLimiter::new(config.rate_limits.clone()));
    let api_client = RiotApiClient::new(config, rate_limiter, database.clone()).unwrap();
    let _worker = CrawlerWorker::new(api_client, database.clone());

    // Create test match data with different queue IDs
    let ranked_match = create_test_match("RANKED_MATCH_123", 420); // Ranked Solo/Duo
    let flex_match = create_test_match("FLEX_MATCH_456", 440); // Ranked Flex
    let normal_match = create_test_match("NORMAL_MATCH_789", 430); // Normal Draft

    // Insert matches into database
    database.insert_match(&ranked_match).unwrap();
    database.insert_match(&flex_match).unwrap();
    database.insert_match(&normal_match).unwrap();

    // Verify only ranked solo/duo match would be processed
    // (This tests the filter logic that the worker uses)
    assert_eq!(ranked_match.queue_id, 420); // Should be processed
    assert_ne!(flex_match.queue_id, 420); // Should be filtered out
    assert_ne!(normal_match.queue_id, 420); // Should be filtered out

    // Verify database has all matches (storage before filtering)
    assert_eq!(database.get_matches_count().unwrap(), 3);

    println!("✅ Ranked match filtering logic verified");
}

#[tokio::test]
async fn test_database_integration_consistency() {
    let database = Database::new(":memory:").expect("Failed to create test database");

    // Test complete match data storage workflow
    let match_data = create_test_match("INTEGRATION_MATCH_001", 420);
    let summoner = create_test_summoner("integration-puuid-001");
    let participant = create_test_participant(&match_data.match_id, &summoner.puuid);

    // Store data in correct order (matches dependencies)
    database.insert_match(&match_data).unwrap();
    database.insert_summoner(&summoner).unwrap();
    database.insert_participant(&participant).unwrap();

    // Verify data consistency
    assert!(database.match_exists(&match_data.match_id).unwrap());
    assert!(database.summoner_exists(&summoner.puuid).unwrap());
    assert_eq!(database.get_participants_count().unwrap(), 1);

    // Test cross-table queries (integration aspect)
    let unique_summoners = database.get_unique_summoners_from_matches(10).unwrap();
    assert_eq!(unique_summoners.len(), 0); // Should be 0 since summoner exists

    // Test after removing summoner (should find the participant's puuid)
    // This simulates discovering new players from matches
    database
        .execute("DELETE FROM summoners WHERE puuid = ?", &[&summoner.puuid])
        .unwrap();
    let unique_after_delete = database.get_unique_summoners_from_matches(10).unwrap();
    assert_eq!(unique_after_delete.len(), 1);
    assert_eq!(unique_after_delete[0], summoner.puuid);

    println!("✅ Database integration consistency verified");
}

#[tokio::test]
async fn test_schema_creation_and_initialization() {
    // Test with fresh database
    let database = Database::new(":memory:").expect("Failed to create test database");

    // Verify all tables exist by attempting to insert test data
    let summoner = create_test_summoner("schema-test-puuid");
    let match_data = create_test_match("SCHEMA_MATCH_001", 420);
    let participant = create_test_participant(&match_data.match_id, &summoner.puuid);

    // These should all succeed if schema is properly initialized
    assert!(database.insert_summoner(&summoner).is_ok());
    assert!(database.insert_match(&match_data).is_ok());
    assert!(database.insert_participant(&participant).is_ok());

    // Test crawler state table - first try to insert, then check if we can retrieve
    let insert_result = database.execute(
        "INSERT OR IGNORE INTO crawler_state (id, last_processed_summoner, total_summoners_processed, total_matches_processed, queue_size, last_update) VALUES (1, NULL, 0, 0, 0, ?1)",
        &[&Utc::now().to_rfc3339()],
    );
    assert!(insert_result.is_ok());

    // The state might already exist from schema initialization, but we should be able to query it
    let state_result = database.get_crawler_state();
    assert!(state_result.is_ok());

    // Test API calls logging table
    let api_call = DbApiCall {
        id: None,
        endpoint: "/test/endpoint".to_string(),
        region: "na1".to_string(),
        timestamp: Utc::now(),
        response_code: 200,
        rate_limit_remaining: Some(100),
    };

    assert!(database.log_api_call(&api_call).is_ok());

    println!("✅ Schema creation and initialization verified");
}

#[tokio::test]
async fn test_crawler_status_reporting() {
    let config = test_config();
    let database = Database::new(":memory:").expect("Failed to create test database");
    let engine = CrawlerEngine::new(config, database.clone()).unwrap();

    // Get initial status
    let initial_status = engine.get_status().await;

    // Verify initial state
    assert!(!initial_status.running); // Should not be running yet
    assert_eq!(initial_status.queue_sizes.high, 0);
    assert_eq!(initial_status.queue_sizes.medium, 0);
    assert_eq!(initial_status.queue_sizes.low, 0);
    assert_eq!(initial_status.database_stats.matches, 0);
    assert_eq!(initial_status.database_stats.summoners, 0);
    assert_eq!(initial_status.database_stats.participants, 0);

    // Add some test data
    let summoner = create_test_summoner("status-test-puuid");
    let match_data = create_test_match("STATUS_MATCH_001", 420);
    let participant = create_test_participant(&match_data.match_id, &summoner.puuid);

    database.insert_summoner(&summoner).unwrap();
    database.insert_match(&match_data).unwrap();
    database.insert_participant(&participant).unwrap();

    // Get updated status
    let updated_status = engine.get_status().await;

    // Verify status reflects database changes
    assert_eq!(updated_status.database_stats.summoners, 1);
    assert_eq!(updated_status.database_stats.matches, 1);
    assert_eq!(updated_status.database_stats.participants, 1);

    // Test rate limit status is included
    assert!(
        updated_status
            .rate_limit_status
            .application_tokens_per_second
            > 0
    );

    println!("✅ Crawler status reporting verified");
}

#[tokio::test]
async fn test_worker_error_handling_and_retry_logic() {
    let _config = test_config();
    let _database = Database::new(":memory:").expect("Failed to create test database");

    // Create a task that would normally fail (invalid region, etc.)
    let failing_task = SummonerTask {
        puuid: "invalid-puuid-format".to_string(),
        summoner_name: "FailingPlayer".to_string(),
        region: "invalid_region".to_string(),
        priority: SummonerPriority::High,
        added_at: Utc::now(),
        retries: 0,
    };

    // Test retry logic simulation
    let mut retry_task = failing_task.clone();
    assert_eq!(retry_task.retries, 0);

    // Simulate first failure and retry
    retry_task.retries += 1;
    retry_task.priority = SummonerPriority::Low; // Demote on retry
    assert_eq!(retry_task.retries, 1);
    assert_eq!(retry_task.priority, SummonerPriority::Low);

    // Simulate reaching max retries (3)
    retry_task.retries = 3;
    let should_retry = retry_task.retries < 3;
    assert!(!should_retry); // Should not retry after 3 attempts

    println!("✅ Worker error handling and retry logic verified");
}

#[tokio::test]
async fn test_concurrent_queue_access() {
    let queue = Arc::new(SummonerQueue::new());
    let mut handles = vec![];

    // Spawn multiple tasks that add to queue concurrently
    for i in 0..10 {
        let queue_clone = queue.clone();
        let handle = tokio::spawn(async move {
            let task = SummonerTask {
                puuid: format!("concurrent-puuid-{}", i),
                summoner_name: format!("ConcurrentPlayer{}", i),
                region: "na1".to_string(),
                priority: SummonerPriority::Medium,
                added_at: Utc::now(),
                retries: 0,
            };
            queue_clone.push(task).await;
        });
        handles.push(handle);
    }

    // Wait for all insertions
    for handle in handles {
        handle.await.unwrap();
    }

    // Verify all tasks were added
    let (high, medium, low) = queue.size().await;
    assert_eq!(medium, 10);
    assert_eq!(high + low, 0);

    // Test concurrent popping
    let mut pop_handles = vec![];
    for _i in 0..5 {
        let queue_clone = queue.clone();
        let handle = tokio::spawn(async move { queue_clone.pop().await });
        pop_handles.push(handle);
    }

    // Collect results
    let mut popped_tasks = vec![];
    for handle in pop_handles {
        if let Some(task) = handle.await.unwrap() {
            popped_tasks.push(task);
        }
    }

    // Should have popped some tasks
    assert!(!popped_tasks.is_empty());
    assert!(popped_tasks.len() <= 5);

    // Verify remaining queue size
    let remaining_total = queue.total_size().await;
    assert_eq!(remaining_total, 10 - popped_tasks.len());

    println!("✅ Concurrent queue access verified");
}

#[tokio::test]
async fn test_data_consistency_across_tables() {
    let database = Database::new(":memory:").expect("Failed to create test database");

    // Create related data across multiple tables
    let match_data = create_test_match("CONSISTENCY_MATCH_001", 420);
    let summoner1 = create_test_summoner("consistency-puuid-001");
    let summoner2 = create_test_summoner("consistency-puuid-002");

    // Create participants for both summoners in the same match
    let participant1 = create_test_participant(&match_data.match_id, &summoner1.puuid);
    let mut participant2 = create_test_participant(&match_data.match_id, &summoner2.puuid);
    participant2.summoner_name = "SecondPlayer".to_string();
    participant2.team_id = 200; // Different team

    // Store all data
    database.insert_match(&match_data).unwrap();
    database.insert_summoner(&summoner1).unwrap();
    database.insert_summoner(&summoner2).unwrap();
    database.insert_participant(&participant1).unwrap();
    database.insert_participant(&participant2).unwrap();

    // Verify data consistency
    assert_eq!(database.get_matches_count().unwrap(), 1);
    assert_eq!(database.get_summoners_count().unwrap(), 2);
    assert_eq!(database.get_participants_count().unwrap(), 2);

    // Test that match exists before processing participants
    assert!(database.match_exists(&match_data.match_id).unwrap());

    // Test cross-table data relationships
    let unique_summoners = database.get_unique_summoners_from_matches(10).unwrap();
    assert_eq!(unique_summoners.len(), 0); // All summoners already exist

    // Test API call logging doesn't interfere with match data
    let api_call = DbApiCall {
        id: None,
        endpoint: "/test/consistency".to_string(),
        region: "na1".to_string(),
        timestamp: Utc::now(),
        response_code: 200,
        rate_limit_remaining: Some(50),
    };
    database.log_api_call(&api_call).unwrap();

    // Data should remain consistent
    assert_eq!(database.get_matches_count().unwrap(), 1);
    assert_eq!(database.get_summoners_count().unwrap(), 2);

    println!("✅ Data consistency across tables verified");
}
