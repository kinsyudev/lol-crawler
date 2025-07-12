# League of Legends Ranked Solo/Duo Data Crawler

A Rust-based data crawler that systematically collects comprehensive **ranked solo/duo** League of Legends match data using Riot Games' API.

## Overview

This crawler operates by starting with Master+ tier players and expanding outward through the player network to build a comprehensive dataset of **ranked solo/duo queue matches only** (Queue ID 420). The system is designed to gather and store detailed competitive game information in a local SQLite database for analysis and research purposes.

> **🎯 Data Focus**: This crawler exclusively collects ranked solo/duo queue data, filtering out all other game modes (ARAM, normals, flex queue, etc.) to provide a clean dataset of competitive 5v5 gameplay.

## How It Works

### 1. Initial Seed Collection
- Fetches Master+ tier ranked players from the League API
- Falls back to featured games if spectator endpoints are restricted
- Extracts player lists from these high-tier sources as initial crawl targets

### 2. Recursive Player Discovery
- For each discovered player, retrieves their match history
- **Filters matches to ranked solo/duo queue only** (Queue ID 420)
- Extracts unique players from qualifying matches to expand the crawl frontier
- Continues this process to build an ever-growing network of competitive players and matches

### 3. Comprehensive Data Storage
- Stores complete **ranked solo/duo match data** including:
  - Match metadata (duration, game mode, patch version, etc.)
  - Player performance statistics (KDA, damage, gold, CS, etc.)
  - Champion selections and item builds
  - Team objectives and achievements
  - Participant runes and summoner spells
- All competitive data persisted in a local SQLite database for efficient querying

## Architecture

The crawler is built in Rust for performance and reliability, with the following key components:

- **API Client**: Handles rate-limited requests to Riot Games API with automatic retry logic
- **Data Models**: Structured representations of Match-v5 and Summoner-v4 API responses
- **Database Layer**: SQLite integration for local data persistence
- **Crawler Engine**: Manages the breadth-first exploration of the player network with priority queues
- **Rate Limiter**: Token bucket implementation ensuring compliance with Riot API rate limits

## Getting Started

### Prerequisites
- Rust (latest stable version)
- Riot Games API key from [developer.riotgames.com](https://developer.riotgames.com/)

### Setup

1. **Clone the repository**
   ```bash
   git clone https://github.com/your-username/lol-crawler.git
   cd lol-crawler
   ```

2. **Configure environment**
   ```bash
   cp .env.example .env
   # Edit .env and add your Riot API key
   ```

3. **Build and run**
   ```bash
   cargo build --release
   cargo run
   ```

### Configuration

The crawler is configured via environment variables in the `.env` file:

- `RIOT_API_KEY`: Your Riot Games API key (required)
- `REGIONS`: Comma-separated list of regions to crawl (e.g., "na1,euw1,kr")
- `DATABASE_URL`: Path to SQLite database file
- `LOG_LEVEL`: Logging level (debug, info, warn, error)

See `.env.example` for all available configuration options.

### Available Regions

- **Americas**: `na1` (North America), `br1` (Brazil), `la1` (Latin America North), `la2` (Latin America South)
- **Europe**: `euw1` (EU West), `eun1` (EU Nordic & East), `tr1` (Turkey), `ru` (Russia)  
- **Asia**: `kr` (Korea), `jp1` (Japan)
- **Oceania**: `oc1` (Oceania)

### API Key Types

**Development Key** (Free):
- Rate Limits: 20 requests/second, 100 requests/2 minutes
- Duration: 24 hours (renewable)
- Perfect for testing and development

**Production Key** (Application Required):
- Much higher rate limits
- Long-term duration
- Required for production deployments

## Database Schema

The SQLite database stores data across multiple tables:

- **matches**: Core match metadata (game_id, duration, mode, version, etc.)
- **participants**: Individual player performance data (KDA, damage, items, etc.)
- **summoners**: Player profile information (PUUID, level, region)
- **teams**: Team-level statistics and objectives
- **bans**: Champion bans for each team
- **active_games**: Currently ongoing games discovered during crawling
- **api_calls**: Request logging for rate limit monitoring

## Features

- **Ranked-only data collection**: Exclusively collects ranked solo/duo queue matches (Queue ID 420)
- **Automatic API compliance**: Built-in rate limiting respects Riot API limits
- **Robust error handling**: Automatic retries and fallback strategies
- **Efficient storage**: Optimized SQLite schema for fast queries
- **Multi-region support**: Crawl ranked data from any Riot Games region
- **Priority queues**: Smart crawling prioritizes high-value targets
- **Real-time monitoring**: Health checks and progress tracking

## Use Cases

This ranked solo/duo dataset enables analysis of:
- **Competitive meta trends** and champion balance in ranked play
- **High-level player behavior** patterns and decision-making
- **Ranked game outcome prediction** models
- **Skill progression tracking** through the ranked ladder
- **Team composition effectiveness** in competitive environments
- **Patch impact analysis** on ranked gameplay

## Monitoring & Health Checks

The crawler provides extensive logging and health monitoring:

### Health Check Logs
Every 60 seconds, the crawler logs:
- Queue sizes (High/Medium/Low priority summoners)
- Database statistics (Matches/Summoners/Participants)  
- Rate limit status

**Example Health Log:**
```
Health Check - Queue: 48H/1M/0L, DB: 991M/1S/9511P, Rate Limits: 19/99
```

## Troubleshooting

### Common Issues

**API Key Invalid**
```
Error: Authentication failed
```
- Verify your API key is correct and not expired
- Ensure no extra spaces in the `.env` file
- Check that your key hasn't been rate-limited or banned

**Rate Limit Exceeded**  
```
Warning: Rate limit hit, waiting 33s before retry
```
- Normal behavior - the crawler automatically retries
- Consider using a production API key for higher limits
- The crawler respects Riot's rate limits to avoid bans

**Database Permission Error**
```  
Error: Failed to initialize database
```
- Ensure the `data/` directory exists and is writable
- Check file permissions for the database path
- Verify sufficient disk space for database growth

### Performance Tuning

For production deployments:
1. Use a production API key for higher rate limits
2. Adjust `MAX_CONCURRENT_REQUESTS` based on your API tier
3. Monitor disk space - ranked match data grows quickly  
4. Consider multiple instances for different regions

### Graceful Shutdown

The crawler supports graceful shutdown:
- Press `Ctrl+C` to trigger shutdown
- Current tasks complete before stopping
- Crawler state is automatically saved

## Important Notes

- This project is not affiliated with Riot Games
- You must comply with [Riot Games API Terms of Service](https://developer.riotgames.com/terms)
- API keys have rate limits - respect them to avoid being banned
- Use collected data responsibly and in accordance with Riot's policies

## License

MIT License - see [LICENSE](LICENSE) file for details.