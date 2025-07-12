use crate::config::Config;

pub struct Endpoints;

impl Endpoints {
    pub fn summoner_by_name(config: &Config, region: &str, summoner_name: &str) -> String {
        format!(
            "{}/lol/summoner/v4/summoners/by-name/{}",
            config.base_url_for_region(region),
            urlencoding::encode(summoner_name)
        )
    }

    pub fn summoner_by_puuid(config: &Config, region: &str, puuid: &str) -> String {
        format!(
            "{}/lol/summoner/v4/summoners/by-puuid/{}",
            config.base_url_for_region(region),
            puuid
        )
    }

    pub fn summoner_by_id(config: &Config, region: &str, summoner_id: &str) -> String {
        format!(
            "{}/lol/summoner/v4/summoners/{}",
            config.base_url_for_region(region),
            summoner_id
        )
    }


    pub fn match_list_by_puuid(
        config: &Config,
        region: &str,
        puuid: &str,
        start: Option<u32>,
        count: Option<u32>,
    ) -> String {
        let base_url = config.regional_base_url_for_region(region);
        let mut url = format!("{}/lol/match/v5/matches/by-puuid/{}/ids", base_url, puuid);

        let mut params = Vec::new();
        if let Some(start) = start {
            params.push(format!("start={}", start));
        }
        if let Some(count) = count {
            params.push(format!("count={}", count));
        }

        if !params.is_empty() {
            url.push('?');
            url.push_str(&params.join("&"));
        }

        url
    }

    pub fn match_by_id(config: &Config, region: &str, match_id: &str) -> String {
        format!(
            "{}/lol/match/v5/matches/{}",
            config.regional_base_url_for_region(region),
            match_id
        )
    }

    pub fn match_timeline(config: &Config, region: &str, match_id: &str) -> String {
        format!(
            "{}/lol/match/v5/matches/{}/timeline",
            config.regional_base_url_for_region(region),
            match_id
        )
    }

    pub fn league_entries_by_summoner(config: &Config, region: &str, summoner_id: &str) -> String {
        format!(
            "{}/lol/league/v4/entries/by-summoner/{}",
            config.base_url_for_region(region),
            summoner_id
        )
    }

    pub fn master_league(config: &Config, region: &str, queue: &str) -> String {
        format!(
            "{}/lol/league/v4/masterleagues/by-queue/{}",
            config.base_url_for_region(region),
            queue
        )
    }

    pub fn grandmaster_league(config: &Config, region: &str, queue: &str) -> String {
        format!(
            "{}/lol/league/v4/grandmasterleagues/by-queue/{}",
            config.base_url_for_region(region),
            queue
        )
    }

    pub fn challenger_league(config: &Config, region: &str, queue: &str) -> String {
        format!(
            "{}/lol/league/v4/challengerleagues/by-queue/{}",
            config.base_url_for_region(region),
            queue
        )
    }
}

// Queue IDs for ranked queues
pub mod queues {
    pub const RANKED_SOLO_5X5: &str = "RANKED_SOLO_5x5";
    pub const RANKED_FLEX_SR: &str = "RANKED_FLEX_SR";
    pub const RANKED_FLEX_TT: &str = "RANKED_FLEX_TT";

    // Numeric queue IDs for filtering matches
    pub const RANKED_SOLO_QUEUE_ID: i32 = 420;
}
