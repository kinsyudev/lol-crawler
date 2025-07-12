use rusqlite::{Connection, Result as SqliteResult};

/// Current database schema version
pub const SCHEMA_VERSION: i32 = 1;

/// Database schema management for League of Legends crawler
pub struct Schema;

impl Schema {
    /// Initialize the complete database schema
    pub fn initialize(conn: &Connection) -> SqliteResult<()> {
        log::info!("Initializing database schema version {}", SCHEMA_VERSION);
        
        // Create all tables
        Self::create_summoners_table(conn)?;
        Self::create_matches_table(conn)?;
        Self::create_participants_table(conn)?;
        Self::create_teams_table(conn)?;
        Self::create_bans_table(conn)?;
        Self::create_timeline_events_table(conn)?;
        Self::create_crawler_state_table(conn)?;
        Self::create_api_calls_table(conn)?;
        Self::create_active_games_table(conn)?;
        
        // Create indexes for performance
        Self::create_indexes(conn)?;
        
        // Initialize default data
        Self::initialize_default_data(conn)?;
        
        log::info!("Database schema initialized successfully");
        Ok(())
    }

    /// Create summoners table - stores player profile information
    fn create_summoners_table(conn: &Connection) -> SqliteResult<()> {
        conn.execute(
            "CREATE TABLE IF NOT EXISTS summoners (
                puuid TEXT PRIMARY KEY,
                summoner_id TEXT UNIQUE,
                account_id TEXT,
                summoner_name TEXT,
                profile_icon_id INTEGER,
                summoner_level INTEGER,
                region TEXT,
                created_at TEXT DEFAULT CURRENT_TIMESTAMP,
                updated_at TEXT DEFAULT CURRENT_TIMESTAMP
            )",
            [],
        )?;
        Ok(())
    }

    /// Create matches table - stores core match metadata
    fn create_matches_table(conn: &Connection) -> SqliteResult<()> {
        conn.execute(
            "CREATE TABLE IF NOT EXISTS matches (
                match_id TEXT PRIMARY KEY,
                game_creation INTEGER,
                game_duration INTEGER,
                game_end_timestamp INTEGER,
                game_id INTEGER,
                game_mode TEXT,
                game_name TEXT,
                game_type TEXT,
                game_version TEXT,
                map_id INTEGER,
                platform_id TEXT,
                queue_id INTEGER,
                tournament_code TEXT,
                region TEXT,
                created_at TEXT DEFAULT CURRENT_TIMESTAMP
            )",
            [],
        )?;
        Ok(())
    }

    /// Create participants table - stores individual player performance data
    fn create_participants_table(conn: &Connection) -> SqliteResult<()> {
        conn.execute(
            "CREATE TABLE IF NOT EXISTS participants (
                id INTEGER PRIMARY KEY AUTOINCREMENT,
                match_id TEXT,
                puuid TEXT,
                summoner_name TEXT,
                champion_id INTEGER,
                champion_name TEXT,
                team_id INTEGER,
                position TEXT,
                individual_position TEXT,
                kills INTEGER,
                deaths INTEGER,
                assists INTEGER,
                total_damage_dealt INTEGER,
                total_damage_dealt_to_champions INTEGER,
                total_damage_taken INTEGER,
                gold_earned INTEGER,
                gold_spent INTEGER,
                turret_kills INTEGER,
                inhibitor_kills INTEGER,
                total_minions_killed INTEGER,
                neutral_minions_killed INTEGER,
                champion_level INTEGER,
                items_0 INTEGER,
                items_1 INTEGER,
                items_2 INTEGER,
                items_3 INTEGER,
                items_4 INTEGER,
                items_5 INTEGER,
                items_6 INTEGER,
                summoner_spell_1 INTEGER,
                summoner_spell_2 INTEGER,
                primary_rune_tree INTEGER,
                secondary_rune_tree INTEGER,
                win BOOLEAN,
                first_blood_kill BOOLEAN,
                first_tower_kill BOOLEAN,
                UNIQUE(match_id, puuid)
            )",
            [],
        )?;
        Ok(())
    }

    /// Create teams table - stores team-level statistics and objectives
    fn create_teams_table(conn: &Connection) -> SqliteResult<()> {
        conn.execute(
            "CREATE TABLE IF NOT EXISTS teams (
                id INTEGER PRIMARY KEY AUTOINCREMENT,
                match_id TEXT,
                team_id INTEGER,
                win BOOLEAN,
                first_baron BOOLEAN,
                first_dragon BOOLEAN,
                first_inhibitor BOOLEAN,
                first_rift_herald BOOLEAN,
                first_tower BOOLEAN,
                baron_kills INTEGER,
                dragon_kills INTEGER,
                inhibitor_kills INTEGER,
                rift_herald_kills INTEGER,
                tower_kills INTEGER,
                UNIQUE(match_id, team_id)
            )",
            [],
        )?;
        Ok(())
    }

    /// Create bans table - stores champion bans for each team
    fn create_bans_table(conn: &Connection) -> SqliteResult<()> {
        conn.execute(
            "CREATE TABLE IF NOT EXISTS bans (
                id INTEGER PRIMARY KEY AUTOINCREMENT,
                match_id TEXT,
                team_id INTEGER,
                champion_id INTEGER,
                pick_turn INTEGER
            )",
            [],
        )?;
        Ok(())
    }

    /// Create timeline_events table - stores detailed match timeline events
    fn create_timeline_events_table(conn: &Connection) -> SqliteResult<()> {
        conn.execute(
            "CREATE TABLE IF NOT EXISTS timeline_events (
                id INTEGER PRIMARY KEY AUTOINCREMENT,
                match_id TEXT,
                timestamp INTEGER,
                event_type TEXT,
                participant_id INTEGER,
                position_x INTEGER,
                position_y INTEGER,
                item_id INTEGER,
                skill_slot INTEGER,
                level_up_type TEXT,
                ward_type TEXT,
                creator_id INTEGER,
                killer_id INTEGER,
                victim_id INTEGER,
                assisting_participant_ids TEXT,
                team_id INTEGER,
                monster_type TEXT,
                monster_sub_type TEXT,
                lane_type TEXT,
                tower_type TEXT,
                building_type TEXT
            )",
            [],
        )?;
        Ok(())
    }

    /// Create crawler_state table - tracks crawler progress and state
    fn create_crawler_state_table(conn: &Connection) -> SqliteResult<()> {
        conn.execute(
            "CREATE TABLE IF NOT EXISTS crawler_state (
                id INTEGER PRIMARY KEY,
                last_processed_summoner TEXT,
                total_summoners_processed INTEGER,
                total_matches_processed INTEGER,
                queue_size INTEGER,
                last_update TEXT DEFAULT CURRENT_TIMESTAMP
            )",
            [],
        )?;
        Ok(())
    }

    /// Create api_calls table - logs API requests for rate limit monitoring
    fn create_api_calls_table(conn: &Connection) -> SqliteResult<()> {
        conn.execute(
            "CREATE TABLE IF NOT EXISTS api_calls (
                id INTEGER PRIMARY KEY AUTOINCREMENT,
                endpoint TEXT,
                region TEXT,
                timestamp TEXT DEFAULT CURRENT_TIMESTAMP,
                response_code INTEGER,
                rate_limit_remaining INTEGER
            )",
            [],
        )?;
        Ok(())
    }

    /// Create active_games table - stores currently ongoing games discovered during crawling
    fn create_active_games_table(conn: &Connection) -> SqliteResult<()> {
        conn.execute(
            "CREATE TABLE IF NOT EXISTS active_games (
                game_id INTEGER PRIMARY KEY,
                game_type TEXT,
                game_start_time INTEGER,
                map_id INTEGER,
                queue_id INTEGER,
                platform_id TEXT,
                game_mode TEXT,
                participants TEXT,
                discovered_at TEXT DEFAULT CURRENT_TIMESTAMP
            )",
            [],
        )?;
        Ok(())
    }

    /// Create database indexes for optimal query performance
    fn create_indexes(conn: &Connection) -> SqliteResult<()> {
        log::debug!("Creating database indexes");
        
        // Participants table indexes
        conn.execute(
            "CREATE INDEX IF NOT EXISTS idx_participants_match_id ON participants(match_id)",
            [],
        )?;
        conn.execute(
            "CREATE INDEX IF NOT EXISTS idx_participants_puuid ON participants(puuid)",
            [],
        )?;
        
        // Matches table indexes
        conn.execute(
            "CREATE INDEX IF NOT EXISTS idx_matches_game_creation ON matches(game_creation)",
            [],
        )?;
        conn.execute(
            "CREATE INDEX IF NOT EXISTS idx_matches_queue_id ON matches(queue_id)",
            [],
        )?;
        
        // Summoners table indexes
        conn.execute(
            "CREATE INDEX IF NOT EXISTS idx_summoners_region ON summoners(region)",
            [],
        )?;
        
        Ok(())
    }

    /// Initialize default data required for crawler operation
    fn initialize_default_data(conn: &Connection) -> SqliteResult<()> {
        // Initialize crawler state if not exists
        conn.execute(
            "INSERT OR IGNORE INTO crawler_state (id, total_summoners_processed, total_matches_processed, queue_size) VALUES (1, 0, 0, 0)",
            [],
        )?;
        Ok(())
    }

    /// Get the current schema version from the database
    pub fn get_version(_conn: &Connection) -> SqliteResult<i32> {
        // For now, we assume version 1. In future versions, we'd store this in a schema_info table
        Ok(SCHEMA_VERSION)
    }

    /// Check if the database needs migration
    pub fn needs_migration(conn: &Connection) -> SqliteResult<bool> {
        let current_version = Self::get_version(conn)?;
        Ok(current_version < SCHEMA_VERSION)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use rusqlite::Connection;

    #[test]
    fn test_schema_initialization() {
        let conn = Connection::open_in_memory().unwrap();
        Schema::initialize(&conn).unwrap();
        
        // Verify tables exist
        let table_count: i32 = conn
            .query_row(
                "SELECT COUNT(*) FROM sqlite_master WHERE type='table'",
                [],
                |row| row.get(0),
            )
            .unwrap();
        
        // Should have 9 tables (8 data tables + sqlite_sequence)
        assert!(table_count >= 8);
    }

    #[test]
    fn test_schema_version() {
        assert_eq!(SCHEMA_VERSION, 1);
    }
}