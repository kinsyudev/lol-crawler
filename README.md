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

## Important Notes

- This project is not affiliated with Riot Games
- You must comply with [Riot Games API Terms of Service](https://developer.riotgames.com/terms)
- API keys have rate limits - respect them to avoid being banned
- Use collected data responsibly and in accordance with Riot's policies

## License

MIT License - see [LICENSE](LICENSE) file for details.