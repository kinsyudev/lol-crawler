use crate::Result;
use rusqlite::{Connection, Result as SqliteResult};
use std::path::Path;
use std::sync::{Arc, Mutex};

#[derive(Clone)]
pub struct Database {
    connection: Arc<Mutex<Connection>>,
}

impl Database {
    pub fn new(database_url: &str) -> Result<Self> {
        // Ensure the parent directory exists
        if let Some(parent) = Path::new(database_url).parent() {
            std::fs::create_dir_all(parent)?;
        }

        let conn = Connection::open(database_url)?;
        let database = Database {
            connection: Arc::new(Mutex::new(conn)),
        };

        // Initialize schema
        database.initialize_schema()?;

        Ok(database)
    }

    pub fn execute(&self, sql: &str, params: &[&dyn rusqlite::ToSql]) -> Result<usize> {
        let conn = self.connection.lock().unwrap();
        Ok(conn.execute(sql, params)?)
    }

    fn initialize_schema(&self) -> Result<()> {
        let conn = self.connection.lock().unwrap();

        // Create summoners table
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

        // Create matches table
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

        // Create participants table
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

        // Create teams table
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

        // Create bans table
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

        // Create timeline_events table
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

        // Create crawler_state table
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

        // Create api_calls table
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

        // Create active_games table
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

        // Create indexes
        conn.execute(
            "CREATE INDEX IF NOT EXISTS idx_participants_match_id ON participants(match_id)",
            [],
        )?;
        conn.execute(
            "CREATE INDEX IF NOT EXISTS idx_participants_puuid ON participants(puuid)",
            [],
        )?;
        conn.execute(
            "CREATE INDEX IF NOT EXISTS idx_matches_game_creation ON matches(game_creation)",
            [],
        )?;
        conn.execute(
            "CREATE INDEX IF NOT EXISTS idx_matches_queue_id ON matches(queue_id)",
            [],
        )?;
        conn.execute(
            "CREATE INDEX IF NOT EXISTS idx_summoners_region ON summoners(region)",
            [],
        )?;

        // Initialize crawler state if not exists
        conn.execute(
            "INSERT OR IGNORE INTO crawler_state (id, total_summoners_processed, total_matches_processed, queue_size) VALUES (1, 0, 0, 0)",
            [],
        )?;

        log::info!("Database schema initialized successfully");
        Ok(())
    }

    pub fn query_row<T, F>(&self, sql: &str, params: &[&dyn rusqlite::ToSql], f: F) -> Result<T>
    where
        F: FnOnce(&rusqlite::Row) -> SqliteResult<T>,
    {
        let conn = self.connection.lock().unwrap();
        Ok(conn.query_row(sql, params, f)?)
    }

    pub fn query_map<T, F>(
        &self,
        sql: &str,
        params: &[&dyn rusqlite::ToSql],
        mut f: F,
    ) -> Result<Vec<T>>
    where
        F: FnMut(&rusqlite::Row) -> SqliteResult<T>,
    {
        let conn = self.connection.lock().unwrap();
        let mut stmt = conn.prepare(sql)?;
        let rows = stmt.query_map(params, &mut f)?;

        let mut results = Vec::new();
        for row in rows {
            results.push(row?);
        }

        Ok(results)
    }
}
