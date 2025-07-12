use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SummonerResponse {
    #[serde(rename = "accountId")]
    pub account_id: Option<String>,
    #[serde(rename = "profileIconId")]
    pub profile_icon_id: u32,
    #[serde(rename = "revisionDate")]
    pub revision_date: u64,
    pub name: Option<String>,
    pub id: Option<String>,
    pub puuid: String,
    #[serde(rename = "summonerLevel")]
    pub summoner_level: u32,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MatchListResponse {
    pub matches: Vec<MatchReference>,
    #[serde(rename = "startIndex")]
    pub start_index: u32,
    #[serde(rename = "endIndex")]
    pub end_index: u32,
    #[serde(rename = "totalGames")]
    pub total_games: u32,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MatchReference {
    #[serde(rename = "gameId")]
    pub game_id: u64,
    pub role: Option<String>,
    pub season: Option<u32>,
    #[serde(rename = "platformId")]
    pub platform_id: String,
    pub champion: u32,
    pub queue: u32,
    pub lane: Option<String>,
    pub timestamp: u64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MatchResponse {
    #[serde(rename = "gameId")]
    pub game_id: u64,
    #[serde(rename = "platformId")]
    pub platform_id: String,
    #[serde(rename = "gameCreation")]
    pub game_creation: u64,
    #[serde(rename = "gameDuration")]
    pub game_duration: u32,
    #[serde(rename = "queueId")]
    pub queue_id: u32,
    #[serde(rename = "mapId")]
    pub map_id: u32,
    #[serde(rename = "seasonId")]
    pub season_id: u32,
    #[serde(rename = "gameVersion")]
    pub game_version: String,
    #[serde(rename = "gameMode")]
    pub game_mode: String,
    #[serde(rename = "gameType")]
    pub game_type: String,
    pub teams: Vec<Team>,
    pub participants: Vec<MatchParticipant>,
    #[serde(rename = "participantIdentities")]
    pub participant_identities: Vec<ParticipantIdentity>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Team {
    #[serde(rename = "teamId")]
    pub team_id: u32,
    pub win: String,
    #[serde(rename = "firstBlood")]
    pub first_blood: bool,
    #[serde(rename = "firstTower")]
    pub first_tower: bool,
    #[serde(rename = "firstInhibitor")]
    pub first_inhibitor: bool,
    #[serde(rename = "firstBaron")]
    pub first_baron: bool,
    #[serde(rename = "firstDragon")]
    pub first_dragon: bool,
    #[serde(rename = "firstRiftHerald")]
    pub first_rift_herald: bool,
    #[serde(rename = "towerKills")]
    pub tower_kills: u32,
    #[serde(rename = "inhibitorKills")]
    pub inhibitor_kills: u32,
    #[serde(rename = "baronKills")]
    pub baron_kills: u32,
    #[serde(rename = "dragonKills")]
    pub dragon_kills: u32,
    #[serde(rename = "vilemawKills")]
    pub vilemaw_kills: Option<u32>,
    #[serde(rename = "riftHeraldKills")]
    pub rift_herald_kills: u32,
    #[serde(rename = "dominionVictoryScore")]
    pub dominion_victory_score: Option<u32>,
    pub bans: Vec<TeamBan>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TeamBan {
    #[serde(rename = "championId")]
    pub champion_id: i32,
    #[serde(rename = "pickTurn")]
    pub pick_turn: u32,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MatchParticipant {
    #[serde(rename = "participantId")]
    pub participant_id: u32,
    #[serde(rename = "teamId")]
    pub team_id: u32,
    #[serde(rename = "championId")]
    pub champion_id: u32,
    #[serde(rename = "spell1Id")]
    pub spell1_id: u32,
    #[serde(rename = "spell2Id")]
    pub spell2_id: u32,
    pub stats: ParticipantStats,
    pub timeline: Option<ParticipantTimeline>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ParticipantStats {
    #[serde(rename = "participantId")]
    pub participant_id: u32,
    pub win: bool,
    pub item0: u32,
    pub item1: u32,
    pub item2: u32,
    pub item3: u32,
    pub item4: u32,
    pub item5: u32,
    pub item6: u32,
    pub kills: u32,
    pub deaths: u32,
    pub assists: u32,
    #[serde(rename = "largestKillingSpree")]
    pub largest_killing_spree: u32,
    #[serde(rename = "largestMultiKill")]
    pub largest_multi_kill: u32,
    #[serde(rename = "killingSprees")]
    pub killing_sprees: u32,
    #[serde(rename = "longestTimeSpentLiving")]
    pub longest_time_spent_living: u32,
    #[serde(rename = "doubleKills")]
    pub double_kills: u32,
    #[serde(rename = "tripleKills")]
    pub triple_kills: u32,
    #[serde(rename = "quadraKills")]
    pub quadra_kills: u32,
    #[serde(rename = "pentaKills")]
    pub penta_kills: u32,
    #[serde(rename = "unrealKills")]
    pub unreal_kills: u32,
    #[serde(rename = "totalDamageDealt")]
    pub total_damage_dealt: u32,
    #[serde(rename = "magicDamageDealt")]
    pub magic_damage_dealt: u32,
    #[serde(rename = "physicalDamageDealt")]
    pub physical_damage_dealt: u32,
    #[serde(rename = "trueDamageDealt")]
    pub true_damage_dealt: u32,
    #[serde(rename = "largestCriticalStrike")]
    pub largest_critical_strike: u32,
    #[serde(rename = "totalDamageDealtToChampions")]
    pub total_damage_dealt_to_champions: u32,
    #[serde(rename = "magicDamageDealtToChampions")]
    pub magic_damage_dealt_to_champions: u32,
    #[serde(rename = "physicalDamageDealtToChampions")]
    pub physical_damage_dealt_to_champions: u32,
    #[serde(rename = "trueDamageDealtToChampions")]
    pub true_damage_dealt_to_champions: u32,
    #[serde(rename = "totalHeal")]
    pub total_heal: u32,
    #[serde(rename = "totalUnitsHealed")]
    pub total_units_healed: u32,
    #[serde(rename = "damageSelfMitigated")]
    pub damage_self_mitigated: u32,
    #[serde(rename = "damageDealtToObjectives")]
    pub damage_dealt_to_objectives: u32,
    #[serde(rename = "damageDealtToTurrets")]
    pub damage_dealt_to_turrets: u32,
    #[serde(rename = "visionScore")]
    pub vision_score: u32,
    #[serde(rename = "timeCCingOthers")]
    pub time_ccing_others: u32,
    #[serde(rename = "totalDamageTaken")]
    pub total_damage_taken: u32,
    #[serde(rename = "magicalDamageTaken")]
    pub magical_damage_taken: u32,
    #[serde(rename = "physicalDamageTaken")]
    pub physical_damage_taken: u32,
    #[serde(rename = "trueDamageTaken")]
    pub true_damage_taken: u32,
    #[serde(rename = "goldEarned")]
    pub gold_earned: u32,
    #[serde(rename = "goldSpent")]
    pub gold_spent: u32,
    #[serde(rename = "turretKills")]
    pub turret_kills: u32,
    #[serde(rename = "inhibitorKills")]
    pub inhibitor_kills: u32,
    #[serde(rename = "totalMinionsKilled")]
    pub total_minions_killed: u32,
    #[serde(rename = "neutralMinionsKilled")]
    pub neutral_minions_killed: u32,
    #[serde(rename = "neutralMinionsKilledTeamJungle")]
    pub neutral_minions_killed_team_jungle: u32,
    #[serde(rename = "neutralMinionsKilledEnemyJungle")]
    pub neutral_minions_killed_enemy_jungle: u32,
    #[serde(rename = "totalTimeCrowdControlDealt")]
    pub total_time_crowd_control_dealt: u32,
    #[serde(rename = "champLevel")]
    pub champ_level: u32,
    #[serde(rename = "visionWardsBoughtInGame")]
    pub vision_wards_bought_in_game: u32,
    #[serde(rename = "sightWardsBoughtInGame")]
    pub sight_wards_bought_in_game: u32,
    #[serde(rename = "wardsPlaced")]
    pub wards_placed: u32,
    #[serde(rename = "wardsKilled")]
    pub wards_killed: u32,
    #[serde(rename = "firstBloodKill")]
    pub first_blood_kill: bool,
    #[serde(rename = "firstBloodAssist")]
    pub first_blood_assist: bool,
    #[serde(rename = "firstTowerKill")]
    pub first_tower_kill: bool,
    #[serde(rename = "firstTowerAssist")]
    pub first_tower_assist: bool,
    #[serde(rename = "firstInhibitorKill")]
    pub first_inhibitor_kill: bool,
    #[serde(rename = "firstInhibitorAssist")]
    pub first_inhibitor_assist: bool,
    #[serde(rename = "combatPlayerScore")]
    pub combat_player_score: Option<u32>,
    #[serde(rename = "objectivePlayerScore")]
    pub objective_player_score: Option<u32>,
    #[serde(rename = "totalPlayerScore")]
    pub total_player_score: Option<u32>,
    #[serde(rename = "totalScoreRank")]
    pub total_score_rank: Option<u32>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ParticipantTimeline {
    #[serde(rename = "participantId")]
    pub participant_id: u32,
    #[serde(rename = "csDiffPerMinDeltas")]
    pub cs_diff_per_min_deltas: Option<std::collections::HashMap<String, f64>>,
    #[serde(rename = "damageTakenPerMinDeltas")]
    pub damage_taken_per_min_deltas: Option<std::collections::HashMap<String, f64>>,
    pub role: String,
    pub lane: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ParticipantIdentity {
    #[serde(rename = "participantId")]
    pub participant_id: u32,
    pub player: Player,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Player {
    #[serde(rename = "platformId")]
    pub platform_id: String,
    #[serde(rename = "accountId")]
    pub account_id: String,
    #[serde(rename = "summonerName")]
    pub summoner_name: String,
    #[serde(rename = "summonerId")]
    pub summoner_id: String,
    #[serde(rename = "currentPlatformId")]
    pub current_platform_id: String,
    #[serde(rename = "currentAccountId")]
    pub current_account_id: String,
    #[serde(rename = "matchHistoryUri")]
    pub match_history_uri: String,
    #[serde(rename = "profileIcon")]
    pub profile_icon: u32,
    pub puuid: Option<String>,
}
