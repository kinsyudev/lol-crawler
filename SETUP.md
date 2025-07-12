# League of Legends Crawler Setup Guide

## Prerequisites

1. **Rust**: Install Rust from [rustup.rs](https://rustup.rs/)
2. **Riot API Key**: Get your API key from [Riot Developer Portal](https://developer.riotgames.com/)

## Quick Start

1. **Clone and setup the project:**
   ```bash
   git clone <your-repo-url>
   cd lol-crawler
   ```

2. **Create environment configuration:**
   ```bash
   cp .env.example .env
   ```

3. **Edit `.env` file and add your Riot API key:**
   ```bash
   RIOT_API_KEY=your_actual_api_key_here
   REGIONS=na1,euw1,kr,eun1
   DATABASE_URL=./data/lol_crawler.db
   LOG_LEVEL=info
   ```

4. **Build and run:**
   ```bash
   cargo run
   ```

## Configuration Options

### Required Environment Variables

| Variable | Description | Example |
|----------|-------------|---------|
| `RIOT_API_KEY` | Your Riot Games API key | `RGAPI-12345...` |

### Optional Environment Variables

| Variable | Default | Description |
|----------|---------|-------------|
| `DATABASE_URL` | `./data/lol_crawler.db` | Path to DuckDB database file |
| `REGIONS` | `na1,euw1,kr,eun1` | Comma-separated list of regions to crawl |
| `LOG_LEVEL` | `info` | Log level (error, warn, info, debug, trace) |

### Advanced Configuration

Rate limiting and crawler behavior can be fine-tuned with additional environment variables. See `.env.example` for the complete list.

## Regions

Available regions:
- **Americas**: `na1` (North America), `br1` (Brazil), `la1` (Latin America North), `la2` (Latin America South)
- **Europe**: `euw1` (EU West), `eun1` (EU Nordic & East), `tr1` (Turkey), `ru` (Russia)
- **Asia**: `kr` (Korea), `jp1` (Japan)
- **Oceania**: `oc1` (Oceania)

## API Key Types

### Development Key
- **Rate Limits**: 20 requests/second, 100 requests/2 minutes
- **Duration**: 24 hours (renewable)
- **Use**: Testing and development

### Production Key
- **Rate Limits**: Much higher (varies by tier)
- **Duration**: Long-term
- **Use**: Production applications
- **Requirements**: Application approval process

## Running the Crawler

### Development Mode
```bash
# Run with debug logging
LOG_LEVEL=debug cargo run

# Run for specific regions only
REGIONS=na1,kr cargo run
```

### Production Mode
```bash
# Build optimized release
cargo build --release

# Run the release binary
./target/release/lol-crawler
```

### Using Docker
```bash
# Build Docker image
docker build -t lol-crawler .

# Run with environment file
docker run --env-file .env -v $(pwd)/data:/app/data lol-crawler
```

## Monitoring

The crawler provides extensive logging and health monitoring:

### Health Check Logs
Every 60 seconds, the crawler logs:
- Queue sizes (high/medium/low priority)
- Database statistics (matches/summoners/participants)
- Rate limit status

### Example Health Log
```
Health Check - Queue: 150H/2500M/45000L, DB: 50000M/25000S/500000P, Rate Limits: 18/95
```

## Database

The crawler uses DuckDB for local storage. The database file will be created automatically at the specified path.

### Database Schema
- **matches**: Core match data
- **summoners**: Player information
- **participants**: Player performance in matches
- **teams**: Team-level statistics
- **bans**: Champion bans
- **active_games**: Currently active games
- **crawler_state**: Crawler status and progress
- **api_calls**: API request logging

### Querying Data
```sql
-- Connect to database
.open ./data/lol_crawler.db

-- Example queries
SELECT COUNT(*) FROM matches;
SELECT region, COUNT(*) FROM summoners GROUP BY region;
SELECT champion_id, AVG(kills), AVG(deaths), AVG(assists) 
FROM participants 
GROUP BY champion_id 
ORDER BY AVG(kills) DESC 
LIMIT 10;
```

## Troubleshooting

### Common Issues

1. **API Key Invalid**
   ```
   Error: Authentication failed
   ```
   - Verify your API key is correct and not expired
   - Ensure no extra spaces in the `.env` file

2. **Rate Limit Exceeded**
   ```
   Warning: Rate limit hit, waiting...
   ```
   - Normal behavior - the crawler will automatically retry
   - Consider using a production API key for higher limits

3. **Database Permission Error**
   ```
   Error: Failed to initialize database
   ```
   - Ensure the data directory exists and is writable
   - Check file permissions

4. **No Featured Games**
   ```
   Warning: No featured games found for region
   ```
   - Normal during off-peak hours
   - The crawler will retry periodically

### Performance Tuning

For production deployments:
1. Use a production API key
2. Adjust `MAX_CONCURRENT_REQUESTS` based on your rate limits
3. Consider running multiple instances for different regions
4. Monitor disk space for database growth

## Stopping the Crawler

The crawler supports graceful shutdown:
- Press `Ctrl+C` to trigger shutdown
- The crawler will finish processing current tasks
- State is automatically saved

## Data Retention

The crawler stores all data indefinitely. For production use, consider:
- Implementing data retention policies
- Regular database backups
- Monitoring disk usage