# CLAUDE.md

This file provides guidance to Claude Code (claude.ai/code) when working with code in this repository.

## Project Overview

This is a League of Legends ranked solo/duo data crawler built in Rust. The system starts by fetching Master+ tier players, then recursively expands through the player network to build a comprehensive dataset of competitive matches. All match data is stored in a local SQLite database.

## Common Commands

### Development
- `cargo run` - Run the crawler application
- `cargo build` - Build the project
- `cargo build --release` - Build optimized release version
- `cargo check` - Quick compile check without building
- `cargo test` - Run all tests
- `cargo test <test_name>` - Run specific test

### Code Quality
- `cargo fmt` - Format code
- `cargo clippy` - Run linter
- `cargo clippy -- -D warnings` - Fail on warnings

## Architecture

The crawler follows a modular design with these key components:

### Core Components (to be implemented)
- **API Client**: Rate-limited HTTP client for Riot Games API requests
- **Data Models**: Rust structs representing matches, players, champions, and game events
- **Database Layer**: SQLite integration for persistent storage and querying
- **Crawler Engine**: BFS-based player network exploration starting from Master+ games
- **Rate Limiter**: Ensures compliance with Riot API rate limits

### Data Flow
1. **Database-first approach**: Prioritize existing summoners for match updates
2. **Master+ league seeding**: When queue is low, fetch Master+ tier players from ranked ladder
3. **Match filtering**: Only process ranked solo/duo queue matches (Queue ID 420)
4. **Data storage**: Store comprehensive ranked match data in SQLite
5. **Network expansion**: Extract new players from ranked matches to expand crawl frontier
6. **Continuous crawling**: Repeat process to build growing competitive dataset

### Key Design Considerations
- Riot API rate limiting compliance is critical
- SQLite chosen for local analytics-optimized storage with excellent Rust ecosystem support
- Breadth-first expansion prevents getting stuck in small player clusters
- **Ranked-only focus**: Exclusively collects ranked solo/duo queue data for competitive analysis
- Focus on Master+ tier ensures high-quality competitive gameplay data