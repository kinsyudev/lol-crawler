use crate::api::RiotApiClient;
use crate::database::Database;
use crate::models::database::{DbMatch, DbParticipant, DbSummoner, DbTeam, DbBan, SummonerTask, SummonerPriority};
use chrono::Utc;
use std::collections::HashSet;

pub struct CrawlerWorker {
    api_client: RiotApiClient,
    database: Database,
}

impl CrawlerWorker {
    pub fn new(api_client: RiotApiClient, database: Database) -> Self {
        Self {
            api_client,
            database,
        }
    }

    pub async fn process_summoner(&self, task: &SummonerTask) -> crate::Result<Vec<SummonerTask>> {
        log::info!("Processing summoner: {} ({}) in region: {}", 
                  task.summoner_name, task.puuid, task.region);

        // First, fetch summoner details and store them
        match self.fetch_and_store_summoner(&task.puuid, &task.region).await {
            Ok(_) => log::debug!("Summoner {} stored successfully", task.puuid),
            Err(e) => {
                log::warn!("Failed to fetch summoner {}: {}", task.puuid, e);
                // Continue with match history even if summoner fetch fails
            }
        }

        // Fetch match history
        let match_ids = match self.api_client.get_match_list_by_puuid(
            &task.region,
            &task.puuid,
            Some(0),
            Some(20), // Fetch last 20 matches
        ).await {
            Ok(matches) => matches,
            Err(e) => {
                log::error!("Failed to fetch match list for {}: {}", task.puuid, e);
                return Ok(Vec::new());
            }
        };

        log::debug!("Found {} matches for summoner {}", match_ids.len(), task.puuid);

        let mut new_summoners = HashSet::new();

        // Process each match
        for match_id in match_ids {
            // Skip if match already exists
            if self.database.match_exists(&match_id)? {
                log::debug!("Match {} already exists, skipping", match_id);
                continue;
            }

            match self.fetch_and_store_match(&match_id, &task.region).await {
                Ok(discovered_summoners) => {
                    new_summoners.extend(discovered_summoners);
                    log::debug!("Successfully processed match {}", match_id);
                }
                Err(e) => {
                    log::warn!("Failed to process match {}: {}", match_id, e);
                }
            }
        }

        // Convert discovered summoners to tasks
        let new_tasks: Vec<SummonerTask> = new_summoners
            .into_iter()
            .filter(|(puuid, _)| {
                // Filter out summoners we already have
                match self.database.summoner_exists(puuid) {
                    Ok(exists) => !exists,
                    Err(_) => true, // Include if we can't check
                }
            })
            .map(|(puuid, summoner_name)| SummonerTask {
                puuid,
                summoner_name,
                region: task.region.clone(),
                priority: SummonerPriority::Low, // New discoveries start as low priority
                added_at: Utc::now(),
                retries: 0,
            })
            .collect();

        log::info!("Discovered {} new summoners from processing {}", 
                  new_tasks.len(), task.summoner_name);

        Ok(new_tasks)
    }

    async fn fetch_and_store_summoner(&self, puuid: &str, region: &str) -> crate::Result<()> {
        let summoner = self.api_client.get_summoner_by_puuid(region, puuid).await?;

        let db_summoner = DbSummoner {
            puuid: summoner.puuid.clone(),
            summoner_id: summoner.id.unwrap_or_else(|| "".to_string()),
            account_id: summoner.account_id.unwrap_or_else(|| "".to_string()),
            summoner_name: summoner.name.unwrap_or_else(|| format!("Player_{}", &summoner.puuid[..8])),
            profile_icon_id: summoner.profile_icon_id as i32,
            summoner_level: summoner.summoner_level as i32,
            region: region.to_string(),
            created_at: Utc::now(),
            updated_at: Utc::now(),
        };

        self.database.insert_summoner(&db_summoner)?;
        Ok(())
    }

    async fn fetch_and_store_match(&self, match_id: &str, region: &str) -> crate::Result<HashSet<(String, String)>> {
        let match_data = self.api_client.get_match_by_id(region, match_id).await?;
        
        // Store match metadata
        let db_match = DbMatch {
            match_id: match_data.metadata.match_id.clone(),
            game_creation: match_data.info.game_creation,
            game_duration: match_data.info.game_duration as i32,
            game_end_timestamp: match_data.info.game_end_timestamp,
            game_id: match_data.info.game_id,
            game_mode: match_data.info.game_mode.clone(),
            game_name: match_data.info.game_name.clone(),
            game_type: match_data.info.game_type.clone(),
            game_version: match_data.info.game_version.clone(),
            map_id: match_data.info.map_id,
            platform_id: match_data.info.platform_id.clone(),
            queue_id: match_data.info.queue_id,
            tournament_code: match_data.info.tournament_code.clone(),
            region: region.to_string(),
            created_at: Utc::now(),
        };

        self.database.insert_match(&db_match)?;

        // Store teams
        for team in &match_data.info.teams {
            let db_team = DbTeam {
                id: None,
                match_id: match_data.metadata.match_id.clone(),
                team_id: team.team_id,
                win: team.win,
                first_baron: team.objectives.baron.first,
                first_dragon: team.objectives.dragon.first,
                first_inhibitor: team.objectives.inhibitor.first,
                first_rift_herald: team.objectives.rift_herald.first,
                first_tower: team.objectives.tower.first,
                baron_kills: team.objectives.baron.kills,
                dragon_kills: team.objectives.dragon.kills,
                inhibitor_kills: team.objectives.inhibitor.kills,
                rift_herald_kills: team.objectives.rift_herald.kills,
                tower_kills: team.objectives.tower.kills,
            };

            self.database.insert_team(&db_team)?;

            // Store bans
            for ban in &team.bans {
                if ban.champion_id > 0 { // 0 or -1 indicates no ban
                    let db_ban = DbBan {
                        id: None,
                        match_id: match_data.metadata.match_id.clone(),
                        team_id: team.team_id,
                        champion_id: ban.champion_id,
                        pick_turn: ban.pick_turn,
                    };

                    self.database.insert_ban(&db_ban)?;
                }
            }
        }

        // Store participants and collect summoner info
        let mut discovered_summoners = HashSet::new();

        for participant in &match_data.info.participants {
            // In Match-v5, participant data includes PUUID directly
            discovered_summoners.insert((
                participant.puuid.clone(),
                participant.summoner_name.clone(),
            ));

            let db_participant = DbParticipant {
                id: None,
                match_id: match_data.metadata.match_id.clone(),
                puuid: participant.puuid.clone(),
                summoner_name: participant.summoner_name.clone(),
                champion_id: participant.champion_id,
                champion_name: Some(participant.champion_name.clone()),
                team_id: participant.team_id,
                position: Some(participant.lane.clone()),
                individual_position: Some(participant.individual_position.clone()),
                kills: participant.kills,
                deaths: participant.deaths,
                assists: participant.assists,
                total_damage_dealt: participant.total_damage_dealt,
                total_damage_dealt_to_champions: participant.total_damage_dealt_to_champions,
                total_damage_taken: participant.total_damage_taken,
                gold_earned: participant.gold_earned,
                gold_spent: participant.gold_spent,
                turret_kills: participant.turret_kills,
                inhibitor_kills: participant.inhibitor_kills,
                total_minions_killed: participant.total_minions_killed,
                neutral_minions_killed: participant.neutral_minions_killed,
                champion_level: participant.champ_level,
                items_0: participant.item0,
                items_1: participant.item1,
                items_2: participant.item2,
                items_3: participant.item3,
                items_4: participant.item4,
                items_5: participant.item5,
                items_6: participant.item6,
                summoner_spell_1: participant.summoner1_id,
                summoner_spell_2: participant.summoner2_id,
                primary_rune_tree: participant.perks.as_ref().and_then(|p| p.styles.get(0).map(|s| s.style)),
                secondary_rune_tree: participant.perks.as_ref().and_then(|p| p.styles.get(1).map(|s| s.style)),
                win: participant.win,
                first_blood_kill: participant.first_blood_kill,
                first_tower_kill: participant.first_tower_kill,
            };

            self.database.insert_participant(&db_participant)?;
        }

        Ok(discovered_summoners)
    }
}