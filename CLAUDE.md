# CLAUDE.md

This file provides guidance to Claude Code (claude.ai/code) when working with code in this repository.

## Project Overview

This is a League of Legends game data crawler built in Rust. The system starts by fetching active Master+ tier games, then recursively expands through the player network to build a comprehensive dataset. All match data is stored in a local DuckDB instance.

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
- **Database Layer**: DuckDB integration for persistent storage and querying
- **Crawler Engine**: BFS-based player network exploration starting from Master+ games
- **Rate Limiter**: Ensures compliance with Riot API rate limits

### Data Flow
1. Fetch active Master+ games as initial seed
2. Extract unique players from these games
3. For each player, retrieve match history via Riot API
4. Store comprehensive match data in DuckDB
5. Extract new players from matches to expand crawl frontier
6. Repeat process to build growing dataset

### Key Design Considerations
- Riot API rate limiting compliance is critical
- DuckDB chosen for local analytics-optimized storage
- Breadth-first expansion prevents getting stuck in small player clusters
- Focus on Master+ tier ensures high-quality gameplay data