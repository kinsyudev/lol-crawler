use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone)]
pub struct DbSummoner {
    pub puuid: String,
    pub summoner_id: String,
    pub account_id: String,
    pub summoner_name: String,
    pub profile_icon_id: i32,
    pub summoner_level: i32,
    pub region: String,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

#[derive(Debug, Clone)]
pub struct DbMatch {
    pub match_id: String,
    pub game_creation: i64,
    pub game_duration: i32,
    pub game_end_timestamp: Option<i64>,
    pub game_id: i64,
    pub game_mode: String,
    pub game_name: Option<String>,
    pub game_type: String,
    pub game_version: String,
    pub map_id: i32,
    pub platform_id: String,
    pub queue_id: i32,
    pub tournament_code: Option<String>,
    pub region: String,
    pub created_at: DateTime<Utc>,
}

#[derive(Debug, Clone)]
pub struct DbParticipant {
    pub id: Option<i64>,
    pub match_id: String,
    pub puuid: String,
    pub summoner_name: String,
    pub champion_id: i32,
    pub champion_name: Option<String>,
    pub team_id: i32,
    pub position: Option<String>,
    pub individual_position: Option<String>,
    pub kills: i32,
    pub deaths: i32,
    pub assists: i32,
    pub total_damage_dealt: i32,
    pub total_damage_dealt_to_champions: i32,
    pub total_damage_taken: i32,
    pub gold_earned: i32,
    pub gold_spent: i32,
    pub turret_kills: i32,
    pub inhibitor_kills: i32,
    pub total_minions_killed: i32,
    pub neutral_minions_killed: i32,
    pub champion_level: i32,
    pub items_0: i32,
    pub items_1: i32,
    pub items_2: i32,
    pub items_3: i32,
    pub items_4: i32,
    pub items_5: i32,
    pub items_6: i32,
    pub summoner_spell_1: i32,
    pub summoner_spell_2: i32,
    pub primary_rune_tree: Option<i32>,
    pub secondary_rune_tree: Option<i32>,
    pub win: bool,
    pub first_blood_kill: bool,
    pub first_tower_kill: bool,
}

#[derive(Debug, Clone)]
pub struct DbTeam {
    pub id: Option<i64>,
    pub match_id: String,
    pub team_id: i32,
    pub win: bool,
    pub first_baron: bool,
    pub first_dragon: bool,
    pub first_inhibitor: bool,
    pub first_rift_herald: bool,
    pub first_tower: bool,
    pub baron_kills: i32,
    pub dragon_kills: i32,
    pub inhibitor_kills: i32,
    pub rift_herald_kills: i32,
    pub tower_kills: i32,
}

#[derive(Debug, Clone)]
pub struct DbBan {
    pub id: Option<i64>,
    pub match_id: String,
    pub team_id: i32,
    pub champion_id: i32,
    pub pick_turn: i32,
}

#[derive(Debug, Clone)]
pub struct DbActiveGame {
    pub game_id: i64,
    pub game_type: String,
    pub game_start_time: i64,
    pub map_id: i32,
    pub queue_id: i32,
    pub platform_id: String,
    pub game_mode: String,
    pub participants: String, // JSON
    pub discovered_at: DateTime<Utc>,
}

#[derive(Debug, Clone)]
pub struct DbCrawlerState {
    pub id: i32,
    pub last_processed_summoner: Option<String>,
    pub total_summoners_processed: i32,
    pub total_matches_processed: i32,
    pub queue_size: i32,
    pub last_update: DateTime<Utc>,
}

#[derive(Debug, Clone)]
pub struct DbApiCall {
    pub id: Option<i64>,
    pub endpoint: String,
    pub region: String,
    pub timestamp: DateTime<Utc>,
    pub response_code: i32,
    pub rate_limit_remaining: Option<i32>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum SummonerPriority {
    High,   // Master+ tier, recently active
    Medium, // Diamond tier, active within 7 days
    Low,    // Other tiers, older activity
}

#[derive(Debug, Clone)]
pub struct SummonerTask {
    pub puuid: String,
    pub summoner_name: String,
    pub region: String,
    pub priority: SummonerPriority,
    pub added_at: DateTime<Utc>,
    pub retries: u32,
}
