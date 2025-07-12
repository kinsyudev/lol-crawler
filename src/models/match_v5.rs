use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MatchDto {
    pub metadata: MetadataDto,
    pub info: InfoDto,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MetadataDto {
    #[serde(rename = "dataVersion")]
    pub data_version: String,
    #[serde(rename = "matchId")]
    pub match_id: String,
    pub participants: Vec<String>, // List of participant PUUIDs
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct InfoDto {
    #[serde(rename = "endOfGameResult")]
    pub end_of_game_result: Option<String>,
    #[serde(rename = "gameCreation")]
    pub game_creation: i64,
    #[serde(rename = "gameDuration")]
    pub game_duration: i64,
    #[serde(rename = "gameEndTimestamp")]
    pub game_end_timestamp: Option<i64>,
    #[serde(rename = "gameId")]
    pub game_id: i64,
    #[serde(rename = "gameMode")]
    pub game_mode: String,
    #[serde(rename = "gameName")]
    pub game_name: Option<String>,
    #[serde(rename = "gameStartTimestamp")]
    pub game_start_timestamp: i64,
    #[serde(rename = "gameType")]
    pub game_type: String,
    #[serde(rename = "gameVersion")]
    pub game_version: String,
    #[serde(rename = "mapId")]
    pub map_id: i32,
    pub participants: Vec<ParticipantDto>,
    #[serde(rename = "platformId")]
    pub platform_id: String,
    #[serde(rename = "queueId")]
    pub queue_id: i32,
    pub teams: Vec<TeamDto>,
    #[serde(rename = "tournamentCode")]
    pub tournament_code: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ParticipantDto {
    #[serde(rename = "allInPings")]
    pub all_in_pings: Option<i32>,
    #[serde(rename = "assistMePings")]
    pub assist_me_pings: Option<i32>,
    pub assists: i32,
    #[serde(rename = "baronKills")]
    pub baron_kills: i32,
    #[serde(rename = "bountyLevel")]
    pub bounty_level: Option<i32>,
    #[serde(rename = "champExperience")]
    pub champ_experience: i32,
    #[serde(rename = "champLevel")]
    pub champ_level: i32,
    #[serde(rename = "championId")]
    pub champion_id: i32,
    #[serde(rename = "championName")]
    pub champion_name: String,
    #[serde(rename = "commandPings")]
    pub command_pings: Option<i32>,
    #[serde(rename = "championTransform")]
    pub champion_transform: Option<i32>,
    #[serde(rename = "consumablesPurchased")]
    pub consumables_purchased: i32,
    pub challenges: Option<ChallengesDto>,
    #[serde(rename = "damageDealtToBuildings")]
    pub damage_dealt_to_buildings: i32,
    #[serde(rename = "damageDealtToObjectives")]
    pub damage_dealt_to_objectives: i32,
    #[serde(rename = "damageDealtToTurrets")]
    pub damage_dealt_to_turrets: i32,
    #[serde(rename = "damageSelfMitigated")]
    pub damage_self_mitigated: i32,
    pub deaths: i32,
    #[serde(rename = "detectorWardsPlaced")]
    pub detector_wards_placed: i32,
    #[serde(rename = "doubleKills")]
    pub double_kills: i32,
    #[serde(rename = "dragonKills")]
    pub dragon_kills: i32,
    #[serde(rename = "eligibleForProgression")]
    pub eligible_for_progression: Option<bool>,
    #[serde(rename = "enemyMissingPings")]
    pub enemy_missing_pings: Option<i32>,
    #[serde(rename = "enemyVisionPings")]
    pub enemy_vision_pings: Option<i32>,
    #[serde(rename = "firstBloodAssist")]
    pub first_blood_assist: bool,
    #[serde(rename = "firstBloodKill")]
    pub first_blood_kill: bool,
    #[serde(rename = "firstTowerAssist")]
    pub first_tower_assist: bool,
    #[serde(rename = "firstTowerKill")]
    pub first_tower_kill: bool,
    #[serde(rename = "gameEndedInEarlySurrender")]
    pub game_ended_in_early_surrender: bool,
    #[serde(rename = "gameEndedInSurrender")]
    pub game_ended_in_surrender: bool,
    #[serde(rename = "holdPings")]
    pub hold_pings: Option<i32>,
    #[serde(rename = "getBackPings")]
    pub get_back_pings: Option<i32>,
    #[serde(rename = "goldEarned")]
    pub gold_earned: i32,
    #[serde(rename = "goldSpent")]
    pub gold_spent: i32,
    #[serde(rename = "individualPosition")]
    pub individual_position: String,
    #[serde(rename = "inhibitorKills")]
    pub inhibitor_kills: i32,
    #[serde(rename = "inhibitorTakedowns")]
    pub inhibitor_takedowns: i32,
    #[serde(rename = "inhibitorsLost")]
    pub inhibitors_lost: i32,
    pub item0: i32,
    pub item1: i32,
    pub item2: i32,
    pub item3: i32,
    pub item4: i32,
    pub item5: i32,
    pub item6: i32,
    #[serde(rename = "itemsPurchased")]
    pub items_purchased: i32,
    #[serde(rename = "killingSprees")]
    pub killing_sprees: i32,
    pub kills: i32,
    pub lane: String,
    #[serde(rename = "largestCriticalStrike")]
    pub largest_critical_strike: i32,
    #[serde(rename = "largestKillingSpree")]
    pub largest_killing_spree: i32,
    #[serde(rename = "largestMultiKill")]
    pub largest_multi_kill: i32,
    #[serde(rename = "longestTimeSpentLiving")]
    pub longest_time_spent_living: i32,
    #[serde(rename = "magicDamageDealt")]
    pub magic_damage_dealt: i32,
    #[serde(rename = "magicDamageDealtToChampions")]
    pub magic_damage_dealt_to_champions: i32,
    #[serde(rename = "magicDamageTaken")]
    pub magic_damage_taken: i32,
    pub missions: Option<MissionsDto>,
    #[serde(rename = "neutralMinionsKilled")]
    pub neutral_minions_killed: i32,
    #[serde(rename = "needVisionPings")]
    pub need_vision_pings: Option<i32>,
    #[serde(rename = "nexusKills")]
    pub nexus_kills: i32,
    #[serde(rename = "nexusTakedowns")]
    pub nexus_takedowns: i32,
    #[serde(rename = "nexusLost")]
    pub nexus_lost: i32,
    #[serde(rename = "objectivesStolen")]
    pub objectives_stolen: i32,
    #[serde(rename = "objectivesStolenAssists")]
    pub objectives_stolen_assists: i32,
    #[serde(rename = "onMyWayPings")]
    pub on_my_way_pings: Option<i32>,
    #[serde(rename = "participantId")]
    pub participant_id: i32,
    #[serde(rename = "pentaKills")]
    pub penta_kills: i32,
    pub perks: Option<PerksDto>,
    #[serde(rename = "physicalDamageDealt")]
    pub physical_damage_dealt: i32,
    #[serde(rename = "physicalDamageDealtToChampions")]
    pub physical_damage_dealt_to_champions: i32,
    #[serde(rename = "physicalDamageTaken")]
    pub physical_damage_taken: i32,
    pub placement: Option<i32>,
    #[serde(rename = "playerAugment1")]
    pub player_augment1: Option<i32>,
    #[serde(rename = "playerAugment2")]
    pub player_augment2: Option<i32>,
    #[serde(rename = "playerAugment3")]
    pub player_augment3: Option<i32>,
    #[serde(rename = "playerAugment4")]
    pub player_augment4: Option<i32>,
    #[serde(rename = "playerSubteamId")]
    pub player_subteam_id: Option<i32>,
    #[serde(rename = "pushPings")]
    pub push_pings: Option<i32>,
    #[serde(rename = "profileIcon")]
    pub profile_icon: i32,
    pub puuid: String,
    #[serde(rename = "quadraKills")]
    pub quadra_kills: i32,
    #[serde(rename = "riotIdGameName")]
    pub riot_id_game_name: Option<String>,
    #[serde(rename = "riotIdTagline")]
    pub riot_id_tagline: Option<String>,
    pub role: String,
    #[serde(rename = "sightWardsBoughtInGame")]
    pub sight_wards_bought_in_game: i32,
    #[serde(rename = "spell1Casts")]
    pub spell1_casts: i32,
    #[serde(rename = "spell2Casts")]
    pub spell2_casts: i32,
    #[serde(rename = "spell3Casts")]
    pub spell3_casts: i32,
    #[serde(rename = "spell4Casts")]
    pub spell4_casts: i32,
    #[serde(rename = "subteamPlacement")]
    pub subteam_placement: Option<i32>,
    #[serde(rename = "summoner1Casts")]
    pub summoner1_casts: i32,
    #[serde(rename = "summoner1Id")]
    pub summoner1_id: i32,
    #[serde(rename = "summoner2Casts")]
    pub summoner2_casts: i32,
    #[serde(rename = "summoner2Id")]
    pub summoner2_id: i32,
    #[serde(rename = "summonerId")]
    pub summoner_id: String,
    #[serde(rename = "summonerLevel")]
    pub summoner_level: i32,
    #[serde(rename = "summonerName")]
    pub summoner_name: String,
    #[serde(rename = "teamEarlySurrendered")]
    pub team_early_surrendered: bool,
    #[serde(rename = "teamId")]
    pub team_id: i32,
    #[serde(rename = "teamPosition")]
    pub team_position: String,
    #[serde(rename = "timeCCingOthers")]
    pub time_ccing_others: i32,
    #[serde(rename = "timePlayed")]
    pub time_played: i32,
    #[serde(rename = "totalAllyJungleMinionsKilled")]
    pub total_ally_jungle_minions_killed: i32,
    #[serde(rename = "totalDamageDealt")]
    pub total_damage_dealt: i32,
    #[serde(rename = "totalDamageDealtToChampions")]
    pub total_damage_dealt_to_champions: i32,
    #[serde(rename = "totalDamageShieldedOnTeammates")]
    pub total_damage_shielded_on_teammates: i32,
    #[serde(rename = "totalDamageTaken")]
    pub total_damage_taken: i32,
    #[serde(rename = "totalEnemyJungleMinionsKilled")]
    pub total_enemy_jungle_minions_killed: i32,
    #[serde(rename = "totalHeal")]
    pub total_heal: i32,
    #[serde(rename = "totalHealsOnTeammates")]
    pub total_heals_on_teammates: i32,
    #[serde(rename = "totalMinionsKilled")]
    pub total_minions_killed: i32,
    #[serde(rename = "totalTimeCCDealt")]
    pub total_time_cc_dealt: i32,
    #[serde(rename = "totalTimeSpentDead")]
    pub total_time_spent_dead: i32,
    #[serde(rename = "totalUnitsHealed")]
    pub total_units_healed: i32,
    #[serde(rename = "tripleKills")]
    pub triple_kills: i32,
    #[serde(rename = "trueDamageDealt")]
    pub true_damage_dealt: i32,
    #[serde(rename = "trueDamageDealtToChampions")]
    pub true_damage_dealt_to_champions: i32,
    #[serde(rename = "trueDamageTaken")]
    pub true_damage_taken: i32,
    #[serde(rename = "turretKills")]
    pub turret_kills: i32,
    #[serde(rename = "turretTakedowns")]
    pub turret_takedowns: i32,
    #[serde(rename = "turretsLost")]
    pub turrets_lost: i32,
    #[serde(rename = "unrealKills")]
    pub unreal_kills: i32,
    #[serde(rename = "visionScore")]
    pub vision_score: i32,
    #[serde(rename = "visionClearedPings")]
    pub vision_cleared_pings: Option<i32>,
    #[serde(rename = "visionWardsBoughtInGame")]
    pub vision_wards_bought_in_game: i32,
    #[serde(rename = "wardsKilled")]
    pub wards_killed: i32,
    #[serde(rename = "wardsPlaced")]
    pub wards_placed: i32,
    pub win: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TeamDto {
    pub bans: Vec<BanDto>,
    pub objectives: ObjectivesDto,
    #[serde(rename = "teamId")]
    pub team_id: i32,
    pub win: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BanDto {
    #[serde(rename = "championId")]
    pub champion_id: i32,
    #[serde(rename = "pickTurn")]
    pub pick_turn: i32,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ObjectivesDto {
    pub baron: ObjectiveDto,
    pub champion: ObjectiveDto,
    pub dragon: ObjectiveDto,
    pub horde: Option<ObjectiveDto>,
    pub inhibitor: ObjectiveDto,
    #[serde(rename = "riftHerald")]
    pub rift_herald: ObjectiveDto,
    pub tower: ObjectiveDto,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ObjectiveDto {
    pub first: bool,
    pub kills: i32,
}

// Simplified versions of complex nested structures
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ChallengesDto {
    // Only including a few key fields for now
    #[serde(rename = "kda")]
    pub kda: Option<f64>,
    #[serde(rename = "killParticipation")]
    pub kill_participation: Option<f64>,
    #[serde(flatten)]
    pub other: serde_json::Map<String, serde_json::Value>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MissionsDto {
    #[serde(rename = "playerScore0")]
    pub player_score0: Option<i32>,
    #[serde(rename = "playerScore1")]
    pub player_score1: Option<i32>,
    #[serde(flatten)]
    pub other: serde_json::Map<String, serde_json::Value>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PerksDto {
    #[serde(rename = "statPerks")]
    pub stat_perks: PerkStatsDto,
    pub styles: Vec<PerkStyleDto>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PerkStatsDto {
    pub defense: i32,
    pub flex: i32,
    pub offense: i32,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PerkStyleDto {
    pub description: String,
    pub selections: Vec<PerkStyleSelectionDto>,
    pub style: i32,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PerkStyleSelectionDto {
    pub perk: i32,
    pub var1: i32,
    pub var2: i32,
    pub var3: i32,
}
