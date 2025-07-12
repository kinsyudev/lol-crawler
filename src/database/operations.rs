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
            |row| Ok(row.get::<_, String>(0)?),
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

    pub fn get_participants_count(&self) -> Result<i64> {
        let count: i64 =
            self.query_row("SELECT COUNT(*) FROM participants", &[], |row| row.get(0))?;
        Ok(count)
    }
}
