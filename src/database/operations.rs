use super::Database;
use crate::models::database::*;
use crate::Result;

impl Database {
    pub fn insert_summoner(&self, summoner: &DbSummoner) -> Result<()> {
        self.execute(
            "INSERT OR REPLACE INTO summoners 
             (puuid, summoner_id, account_id, summoner_name, profile_icon_id, summoner_level, region, created_at, updated_at) 
             VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9)",
            &[
                &summoner.puuid,
                &summoner.summoner_id,
                &summoner.account_id,
                &summoner.summoner_name,
                &summoner.profile_icon_id,
                &summoner.summoner_level,
                &summoner.region,
                &summoner.created_at.to_rfc3339(),
                &summoner.updated_at.to_rfc3339(),
            ],
        )?;
        Ok(())
    }

    pub fn insert_match(&self, match_data: &DbMatch) -> Result<()> {
        self.execute(
            "INSERT OR REPLACE INTO matches 
             (match_id, game_creation, game_duration, game_end_timestamp, game_id, game_mode, game_name, game_type, game_version, map_id, platform_id, queue_id, tournament_code, region, created_at) 
             VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9, ?10, ?11, ?12, ?13, ?14, ?15)",
            &[
                &match_data.match_id,
                &match_data.game_creation,
                &match_data.game_duration,
                &match_data.game_end_timestamp,
                &match_data.game_id,
                &match_data.game_mode,
                &match_data.game_name,
                &match_data.game_type,
                &match_data.game_version,
                &match_data.map_id,
                &match_data.platform_id,
                &match_data.queue_id,
                &match_data.tournament_code,
                &match_data.region,
                &match_data.created_at.to_rfc3339(),
            ],
        )?;
        Ok(())
    }

    pub fn insert_participant(&self, participant: &DbParticipant) -> Result<()> {
        self.execute(
            "INSERT OR REPLACE INTO participants 
             (match_id, puuid, summoner_name, champion_id, champion_name, team_id, position, individual_position, 
              kills, deaths, assists, total_damage_dealt, total_damage_dealt_to_champions, total_damage_taken, 
              gold_earned, gold_spent, turret_kills, inhibitor_kills, total_minions_killed, neutral_minions_killed, 
              champion_level, items_0, items_1, items_2, items_3, items_4, items_5, items_6, 
              summoner_spell_1, summoner_spell_2, primary_rune_tree, secondary_rune_tree, 
              win, first_blood_kill, first_tower_kill) 
             VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9, ?10, ?11, ?12, ?13, ?14, ?15, ?16, ?17, ?18, ?19, ?20, ?21, ?22, ?23, ?24, ?25, ?26, ?27, ?28, ?29, ?30, ?31, ?32, ?33, ?34, ?35)",
            &[
                &participant.match_id,
                &participant.puuid,
                &participant.summoner_name,
                &participant.champion_id,
                &participant.champion_name,
                &participant.team_id,
                &participant.position,
                &participant.individual_position,
                &participant.kills,
                &participant.deaths,
                &participant.assists,
                &participant.total_damage_dealt,
                &participant.total_damage_dealt_to_champions,
                &participant.total_damage_taken,
                &participant.gold_earned,
                &participant.gold_spent,
                &participant.turret_kills,
                &participant.inhibitor_kills,
                &participant.total_minions_killed,
                &participant.neutral_minions_killed,
                &participant.champion_level,
                &participant.items_0,
                &participant.items_1,
                &participant.items_2,
                &participant.items_3,
                &participant.items_4,
                &participant.items_5,
                &participant.items_6,
                &participant.summoner_spell_1,
                &participant.summoner_spell_2,
                &participant.primary_rune_tree,
                &participant.secondary_rune_tree,
                &participant.win,
                &participant.first_blood_kill,
                &participant.first_tower_kill,
            ],
        )?;
        Ok(())
    }

    pub fn insert_team(&self, team: &DbTeam) -> Result<()> {
        self.execute(
            "INSERT OR REPLACE INTO teams 
             (match_id, team_id, win, first_baron, first_dragon, first_inhibitor, first_rift_herald, first_tower, 
              baron_kills, dragon_kills, inhibitor_kills, rift_herald_kills, tower_kills) 
             VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9, ?10, ?11, ?12, ?13)",
            &[
                &team.match_id,
                &team.team_id,
                &team.win,
                &team.first_baron,
                &team.first_dragon,
                &team.first_inhibitor,
                &team.first_rift_herald,
                &team.first_tower,
                &team.baron_kills,
                &team.dragon_kills,
                &team.inhibitor_kills,
                &team.rift_herald_kills,
                &team.tower_kills,
            ],
        )?;
        Ok(())
    }

    pub fn insert_ban(&self, ban: &DbBan) -> Result<()> {
        self.execute(
            "INSERT INTO bans (match_id, team_id, champion_id, pick_turn) VALUES (?1, ?2, ?3, ?4)",
            &[
                &ban.match_id,
                &ban.team_id,
                &ban.champion_id,
                &ban.pick_turn,
            ],
        )?;
        Ok(())
    }

    pub fn insert_active_game(&self, game: &DbActiveGame) -> Result<()> {
        self.execute(
            "INSERT OR REPLACE INTO active_games 
             (game_id, game_type, game_start_time, map_id, queue_id, platform_id, game_mode, participants, discovered_at) 
             VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9)",
            &[
                &game.game_id,
                &game.game_type,
                &game.game_start_time,
                &game.map_id,
                &game.queue_id,
                &game.platform_id,
                &game.game_mode,
                &game.participants,
                &game.discovered_at.to_rfc3339(),
            ],
        )?;
        Ok(())
    }

    pub fn log_api_call(&self, call: &DbApiCall) -> Result<()> {
        self.execute(
            "INSERT INTO api_calls (endpoint, region, timestamp, response_code, rate_limit_remaining) 
             VALUES (?1, ?2, ?3, ?4, ?5)",
            &[
                &call.endpoint,
                &call.region,
                &call.timestamp.to_rfc3339(),
                &call.response_code,
                &call.rate_limit_remaining,
            ],
        )?;
        Ok(())
    }

    pub fn update_crawler_state(&self, state: &DbCrawlerState) -> Result<()> {
        self.execute(
            "UPDATE crawler_state SET 
             last_processed_summoner = ?1, total_summoners_processed = ?2, total_matches_processed = ?3, 
             queue_size = ?4, last_update = ?5 
             WHERE id = 1",
            &[
                &state.last_processed_summoner,
                &state.total_summoners_processed,
                &state.total_matches_processed,
                &state.queue_size,
                &state.last_update.to_rfc3339(),
            ],
        )?;
        Ok(())
    }

    pub fn get_crawler_state(&self) -> Result<Option<DbCrawlerState>> {
        let result = self.query_row(
            "SELECT id, last_processed_summoner, total_summoners_processed, total_matches_processed, queue_size, last_update FROM crawler_state WHERE id = 1",
            &[],
            |row| {
                let last_update_str: String = row.get(5)?;
                let last_update = last_update_str.parse().map_err(|_| rusqlite::Error::InvalidColumnType(5, "TEXT".to_string(), rusqlite::types::Type::Text))?;
                Ok(DbCrawlerState {
                    id: row.get(0)?,
                    last_processed_summoner: row.get(1)?,
                    total_summoners_processed: row.get(2)?,
                    total_matches_processed: row.get(3)?,
                    queue_size: row.get(4)?,
                    last_update,
                })
            }
        );

        match result {
            Ok(state) => Ok(Some(state)),
            Err(_) => Ok(None),
        }
    }

    pub fn summoner_exists(&self, puuid: &str) -> Result<bool> {
        let count: i64 = self.query_row(
            "SELECT COUNT(*) FROM summoners WHERE puuid = ?1",
            &[&puuid],
            |row| row.get(0),
        )?;
        Ok(count > 0)
    }

    pub fn match_exists(&self, match_id: &str) -> Result<bool> {
        let count: i64 = self.query_row(
            "SELECT COUNT(*) FROM matches WHERE match_id = ?1",
            &[&match_id],
            |row| row.get(0),
        )?;
        Ok(count > 0)
    }

    pub fn get_recent_api_calls(&self, endpoint: &str, region: &str, minutes: i32) -> Result<i32> {
        let count: i32 = self.query_row(
            "SELECT COUNT(*) FROM api_calls 
             WHERE endpoint = ?1 AND region = ?2 AND timestamp > datetime('now', '-' || ?3 || ' minutes')",
            &[&endpoint, &region, &minutes],
            |row| row.get(0)
        )?;
        Ok(count)
    }

    pub fn get_unique_summoners_from_matches(&self, limit: i32) -> Result<Vec<String>> {
        let puuids = self.query_map(
            "SELECT DISTINCT puuid FROM participants 
             WHERE puuid NOT IN (SELECT puuid FROM summoners) 
             LIMIT ?1",
            &[&limit],
            |row| row.get::<_, String>(0),
        )?;

        Ok(puuids)
    }

    pub fn get_matches_count(&self) -> Result<i64> {
        let count: i64 = self.query_row("SELECT COUNT(*) FROM matches", &[], |row| row.get(0))?;
        Ok(count)
    }

    pub fn get_summoners_count(&self) -> Result<i64> {
        let count: i64 = self.query_row("SELECT COUNT(*) FROM summoners", &[], |row| row.get(0))?;
        Ok(count)
    }

    pub fn get_existing_summoners_for_update(&self, limit: i32) -> Result<Vec<(String, String)>> {
        let summoners = self.query_map(
            "SELECT puuid, region FROM summoners 
             ORDER BY updated_at ASC 
             LIMIT ?1",
            &[&limit],
            |row| Ok((row.get::<_, String>(0)?, row.get::<_, String>(1)?)),
        )?;
        Ok(summoners)
    }

    pub fn get_participants_count(&self) -> Result<i64> {
        let count: i64 =
            self.query_row("SELECT COUNT(*) FROM participants", &[], |row| row.get(0))?;
        Ok(count)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::database::Database;
    use chrono::{DateTime, Utc};
    use std::str::FromStr;

    fn create_test_database() -> Database {
        Database::new(":memory:").expect("Failed to create test database")
    }

    fn test_summoner() -> DbSummoner {
        let now = Utc::now();
        let unique_id = now.timestamp_nanos_opt().unwrap_or(0);
        DbSummoner {
            puuid: format!("test-puuid-{}", unique_id),
            summoner_id: format!("test-summoner-id-{}", unique_id),
            account_id: format!("test-account-id-{}", unique_id),
            summoner_name: format!("TestSummoner{}", unique_id),
            profile_icon_id: 1234,
            summoner_level: 100,
            region: "na1".to_string(),
            created_at: now,
            updated_at: now,
        }
    }

    fn test_match() -> DbMatch {
        let now = Utc::now();
        let unique_id = now.timestamp_nanos_opt().unwrap_or(0);
        DbMatch {
            match_id: format!("NA1_{}", unique_id),
            game_creation: 1640000000000,
            game_duration: 1800,
            game_end_timestamp: Some(1640001800000),
            game_id: unique_id as i64,
            game_mode: "CLASSIC".to_string(),
            game_name: Some("Test Game".to_string()),
            game_type: "MATCHED_GAME".to_string(),
            game_version: "12.1.1".to_string(),
            map_id: 11,
            platform_id: "NA1".to_string(),
            queue_id: 420,
            tournament_code: None,
            region: "na1".to_string(),
            created_at: now,
        }
    }

    fn test_participant_for_match(match_id: &str, puuid: &str) -> DbParticipant {
        DbParticipant {
            id: None,
            match_id: match_id.to_string(),
            puuid: puuid.to_string(),
            summoner_name: "TestSummoner".to_string(),
            champion_id: 157,
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
            summoner_spell_1: 4,
            summoner_spell_2: 12,
            primary_rune_tree: Some(8000),
            secondary_rune_tree: Some(8300),
            win: true,
            first_blood_kill: false,
            first_tower_kill: true,
        }
    }

    fn test_team_for_match(match_id: &str) -> DbTeam {
        DbTeam {
            id: None,
            match_id: match_id.to_string(),
            team_id: 100,
            win: true,
            first_baron: true,
            first_dragon: false,
            first_inhibitor: true,
            first_rift_herald: false,
            first_tower: true,
            baron_kills: 2,
            dragon_kills: 3,
            inhibitor_kills: 1,
            rift_herald_kills: 1,
            tower_kills: 8,
        }
    }

    fn test_ban_for_match(match_id: &str) -> DbBan {
        DbBan {
            id: None,
            match_id: match_id.to_string(),
            team_id: 100,
            champion_id: 157,
            pick_turn: 1,
        }
    }

    fn test_active_game() -> DbActiveGame {
        DbActiveGame {
            game_id: 1234567890,
            game_type: "MATCHED_GAME".to_string(),
            game_start_time: 1640000000000,
            map_id: 11,
            queue_id: 420,
            platform_id: "NA1".to_string(),
            game_mode: "CLASSIC".to_string(),
            participants: r#"[{"puuid":"test-puuid-1"},{"puuid":"test-puuid-2"}]"#.to_string(),
            discovered_at: Utc::now(),
        }
    }

    fn test_api_call() -> DbApiCall {
        DbApiCall {
            id: None,
            endpoint: "/lol/summoner/v4/summoners/by-name/test".to_string(),
            region: "na1".to_string(),
            timestamp: Utc::now(),
            response_code: 200,
            rate_limit_remaining: Some(100),
        }
    }

    fn test_crawler_state() -> DbCrawlerState {
        DbCrawlerState {
            id: 1,
            last_processed_summoner: Some("test-puuid-123".to_string()),
            total_summoners_processed: 50,
            total_matches_processed: 200,
            queue_size: 10,
            last_update: Utc::now(),
        }
    }

    #[test]
    fn test_summoner_crud_operations() {
        let db = create_test_database();
        let summoner = test_summoner();

        // Test insertion
        assert!(db.insert_summoner(&summoner).is_ok());

        // Test existence check
        assert!(db.summoner_exists(&summoner.puuid).unwrap());
        assert!(!db.summoner_exists("non-existent-puuid").unwrap());

        // Test count
        assert_eq!(db.get_summoners_count().unwrap(), 1);

        // Test update via INSERT OR REPLACE
        let mut updated_summoner = summoner.clone();
        updated_summoner.summoner_level = 200;
        updated_summoner.summoner_name = "UpdatedSummoner".to_string();
        assert!(db.insert_summoner(&updated_summoner).is_ok());

        // Count should still be 1 (replace, not insert)
        assert_eq!(db.get_summoners_count().unwrap(), 1);
    }

    #[test]
    fn test_match_crud_operations() {
        let db = create_test_database();
        let match_data = test_match();

        // Test insertion
        assert!(db.insert_match(&match_data).is_ok());

        // Test existence check
        assert!(db.match_exists(&match_data.match_id).unwrap());
        assert!(!db.match_exists("non-existent-match").unwrap());

        // Test count
        assert_eq!(db.get_matches_count().unwrap(), 1);

        // Test update via INSERT OR REPLACE
        let mut updated_match = match_data.clone();
        updated_match.game_duration = 3600;
        assert!(db.insert_match(&updated_match).is_ok());

        // Count should still be 1 (replace, not insert)
        assert_eq!(db.get_matches_count().unwrap(), 1);
    }

    #[test]
    fn test_participant_crud_operations() {
        let db = create_test_database();
        let match_data = test_match();
        let summoner = test_summoner();
        let participant = test_participant_for_match(&match_data.match_id, &summoner.puuid);

        // Insert match first (foreign key requirement)
        assert!(db.insert_match(&match_data).is_ok());

        // Test insertion
        assert!(db.insert_participant(&participant).is_ok());

        // Test count
        assert_eq!(db.get_participants_count().unwrap(), 1);

        // Test update via INSERT OR REPLACE
        let mut updated_participant = participant.clone();
        updated_participant.kills = 10;
        updated_participant.champion_level = 18;
        assert!(db.insert_participant(&updated_participant).is_ok());

        // Count should still be 1 (replace, not insert)
        assert_eq!(db.get_participants_count().unwrap(), 1);
    }

    #[test]
    fn test_team_operations() {
        let db = create_test_database();
        let match_data = test_match();
        let team = test_team_for_match(&match_data.match_id);

        // Insert match first (foreign key requirement)
        assert!(db.insert_match(&match_data).is_ok());

        // Test insertion
        assert!(db.insert_team(&team).is_ok());

        // Test update via INSERT OR REPLACE
        let mut updated_team = team.clone();
        updated_team.baron_kills = 5;
        assert!(db.insert_team(&updated_team).is_ok());
    }

    #[test]
    fn test_ban_operations() {
        let db = create_test_database();
        let match_data = test_match();
        let ban = test_ban_for_match(&match_data.match_id);

        // Insert match first (foreign key requirement)
        assert!(db.insert_match(&match_data).is_ok());

        // Test insertion (INSERT, not INSERT OR REPLACE)
        assert!(db.insert_ban(&ban).is_ok());

        // Test multiple bans for same match
        let mut ban2 = ban.clone();
        ban2.champion_id = 238; // Different champion
        ban2.pick_turn = 2;
        assert!(db.insert_ban(&ban2).is_ok());
    }

    #[test]
    fn test_active_game_operations() {
        let db = create_test_database();
        let game = test_active_game();

        // Test insertion
        assert!(db.insert_active_game(&game).is_ok());

        // Test update via INSERT OR REPLACE
        let mut updated_game = game.clone();
        updated_game.queue_id = 440; // Ranked Flex
        assert!(db.insert_active_game(&updated_game).is_ok());
    }

    #[test]
    fn test_api_call_logging() {
        let db = create_test_database();
        let api_call = test_api_call();

        // Test logging
        assert!(db.log_api_call(&api_call).is_ok());

        // Test recent API calls query
        let recent_calls = db
            .get_recent_api_calls(&api_call.endpoint, &api_call.region, 60)
            .unwrap();
        assert_eq!(recent_calls, 1);

        // Test with different endpoint
        let no_calls = db
            .get_recent_api_calls("different-endpoint", &api_call.region, 60)
            .unwrap();
        assert_eq!(no_calls, 0);
    }

    #[test]
    fn test_crawler_state_operations() {
        let db = create_test_database();
        let state = test_crawler_state();

        // Initially no state
        assert!(db.get_crawler_state().unwrap().is_none());

        // First we need to insert a row (using INSERT OR IGNORE to avoid conflicts in test runs)
        let insert_result = db.execute(
            "INSERT OR IGNORE INTO crawler_state (id, last_processed_summoner, total_summoners_processed, total_matches_processed, queue_size, last_update) VALUES (1, NULL, 0, 0, 0, ?1)",
            &[&Utc::now().to_rfc3339()],
        );

        // If it returns an error due to existing data, that's fine for testing
        let _ = insert_result;

        // Now test update
        assert!(db.update_crawler_state(&state).is_ok());

        // Test retrieval
        let retrieved_state = db.get_crawler_state().unwrap();
        assert!(retrieved_state.is_some());
        let retrieved_state = retrieved_state.unwrap();
        assert_eq!(retrieved_state.total_summoners_processed, 50);
        assert_eq!(retrieved_state.total_matches_processed, 200);
        assert_eq!(retrieved_state.queue_size, 10);
    }

    #[test]
    fn test_get_unique_summoners_from_matches() {
        let db = create_test_database();
        let match_data = test_match();
        let summoner = test_summoner();
        let participant = test_participant_for_match(&match_data.match_id, &summoner.puuid);

        // Insert match and participant
        assert!(db.insert_match(&match_data).is_ok());
        assert!(db.insert_participant(&participant).is_ok());

        // Should find the participant's puuid since no summoner exists
        let unique_summoners = db.get_unique_summoners_from_matches(10).unwrap();
        assert_eq!(unique_summoners.len(), 1);
        assert_eq!(unique_summoners[0], participant.puuid);

        // Insert summoner
        assert!(db.insert_summoner(&summoner).is_ok());

        // Now should find no unique summoners
        let unique_summoners = db.get_unique_summoners_from_matches(10).unwrap();
        assert_eq!(unique_summoners.len(), 0);
    }

    #[test]
    fn test_get_existing_summoners_for_update() {
        let db = create_test_database();
        let summoner = test_summoner();

        // Insert summoner
        assert!(db.insert_summoner(&summoner).is_ok());

        // Test retrieval
        let summoners_for_update = db.get_existing_summoners_for_update(10).unwrap();
        assert_eq!(summoners_for_update.len(), 1);
        assert_eq!(summoners_for_update[0].0, summoner.puuid);
        assert_eq!(summoners_for_update[0].1, summoner.region);
    }

    #[test]
    fn test_data_referential_integrity() {
        let db = create_test_database();

        let match_data = test_match();
        let summoner = test_summoner();
        let participant = test_participant_for_match(&match_data.match_id, &summoner.puuid);

        // Current schema doesn't enforce foreign key constraints at database level
        // but application logic should ensure referential integrity

        // This demonstrates that participants can be inserted without matches
        // (no foreign key constraints defined in schema)
        let result = db.insert_participant(&participant);
        assert!(result.is_ok()); // No database-level constraint enforcement

        // Verify the data was inserted
        assert_eq!(db.get_participants_count().unwrap(), 1);

        // Insert the related match
        assert!(db.insert_match(&match_data).is_ok());

        // Query should work correctly even with the referential data
        let unique_summoners = db.get_unique_summoners_from_matches(10).unwrap();
        assert_eq!(unique_summoners.len(), 1);
        assert_eq!(unique_summoners[0], participant.puuid);
    }

    #[test]
    fn test_data_integrity_constraints() {
        let db = create_test_database();

        // Test duplicate primary key handling
        let summoner1 = test_summoner();
        let mut summoner2 = summoner1.clone();
        summoner2.summoner_name = "DifferentName".to_string();

        // First insert should succeed
        assert!(db.insert_summoner(&summoner1).is_ok());

        // Second insert with same puuid should replace (INSERT OR REPLACE)
        assert!(db.insert_summoner(&summoner2).is_ok());

        // Should still only have 1 summoner
        assert_eq!(db.get_summoners_count().unwrap(), 1);
    }

    #[test]
    fn test_empty_database_queries() {
        let db = create_test_database();

        // Test queries on empty database
        assert_eq!(db.get_summoners_count().unwrap(), 0);
        assert_eq!(db.get_matches_count().unwrap(), 0);
        assert_eq!(db.get_participants_count().unwrap(), 0);
        assert!(!db.summoner_exists("any-puuid").unwrap());
        assert!(!db.match_exists("any-match-id").unwrap());
        assert_eq!(db.get_unique_summoners_from_matches(10).unwrap().len(), 0);
        assert_eq!(db.get_existing_summoners_for_update(10).unwrap().len(), 0);
        assert_eq!(
            db.get_recent_api_calls("endpoint", "region", 60).unwrap(),
            0
        );
    }

    #[test]
    fn test_large_data_handling() {
        let db = create_test_database();

        // Test with very long strings
        let mut summoner = test_summoner();
        summoner.summoner_name = "A".repeat(500); // Very long name

        assert!(db.insert_summoner(&summoner).is_ok());
        assert!(db.summoner_exists(&summoner.puuid).unwrap());

        // Test with many records
        for i in 0..50 {
            let mut test_summoner = test_summoner();
            test_summoner.puuid = format!("puuid-{}", i);
            test_summoner.summoner_name = format!("Summoner{}", i);
            assert!(db.insert_summoner(&test_summoner).is_ok());
        }

        assert_eq!(db.get_summoners_count().unwrap(), 51); // 50 + 1 from earlier test
    }

    #[test]
    fn test_datetime_handling() {
        let db = create_test_database();

        // Test with specific datetime
        let specific_time = DateTime::<Utc>::from_str("2023-01-01T12:00:00Z").unwrap();
        let mut summoner = test_summoner();
        summoner.created_at = specific_time;
        summoner.updated_at = specific_time;

        assert!(db.insert_summoner(&summoner).is_ok());

        // Test with current datetime
        let mut summoner2 = test_summoner();
        summoner2.puuid = "different-puuid".to_string();
        summoner2.created_at = Utc::now();
        summoner2.updated_at = Utc::now();

        assert!(db.insert_summoner(&summoner2).is_ok());
    }

    #[test]
    fn test_optional_fields() {
        let db = create_test_database();
        let match_data = test_match();
        let summoner = test_summoner();

        // Insert match first
        assert!(db.insert_match(&match_data).is_ok());

        // Test participant with all optional fields as None
        let mut participant = test_participant_for_match(&match_data.match_id, &summoner.puuid);
        participant.champion_name = None;
        participant.position = None;
        participant.individual_position = None;
        participant.primary_rune_tree = None;
        participant.secondary_rune_tree = None;

        assert!(db.insert_participant(&participant).is_ok());

        // Test match with optional fields as None
        let mut match2 = test_match();
        match2.game_end_timestamp = None;
        match2.game_name = None;
        match2.tournament_code = None;

        assert!(db.insert_match(&match2).is_ok());
    }

    #[test]
    fn test_json_field_handling() {
        let db = create_test_database();
        let active_game = test_active_game();

        // Test with valid JSON
        assert!(db.insert_active_game(&active_game).is_ok());

        // Test with different JSON structure
        let mut game2 = active_game.clone();
        game2.game_id = 9999999999;
        game2.participants = r#"{"players":[{"name":"test1"},{"name":"test2"}]}"#.to_string();

        assert!(db.insert_active_game(&game2).is_ok());

        // Test with empty JSON
        let mut game3 = active_game.clone();
        game3.game_id = 8888888888;
        game3.participants = "{}".to_string();

        assert!(db.insert_active_game(&game3).is_ok());
    }

    #[test]
    fn test_limit_parameters() {
        let db = create_test_database();

        // Insert multiple summoners
        for i in 0..10 {
            let mut summoner = test_summoner();
            summoner.puuid = format!("puuid-{}", i);
            summoner.summoner_name = format!("Summoner{}", i);
            assert!(db.insert_summoner(&summoner).is_ok());
        }

        // Test limit functionality
        let limited_summoners = db.get_existing_summoners_for_update(5).unwrap();
        assert_eq!(limited_summoners.len(), 5);

        let all_summoners = db.get_existing_summoners_for_update(20).unwrap();
        assert_eq!(all_summoners.len(), 10);

        // Test zero limit
        let no_summoners = db.get_existing_summoners_for_update(0).unwrap();
        assert_eq!(no_summoners.len(), 0);
    }
}
