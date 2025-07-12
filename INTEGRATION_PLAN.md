# League of Legends Crawler - Comprehensive Integration Plan

## 1. Database Schema Design (DuckDB)

### Core Tables

```sql
-- Summoners/Players table
CREATE TABLE summoners (
    puuid VARCHAR PRIMARY KEY,
    summoner_id VARCHAR UNIQUE,
    account_id VARCHAR,
    summoner_name VARCHAR,
    profile_icon_id INTEGER,
    summoner_level INTEGER,
    region VARCHAR,
    created_at TIMESTAMP DEFAULT CURRENT_TIMESTAMP,
    updated_at TIMESTAMP DEFAULT CURRENT_TIMESTAMP
);

-- Matches table (main match metadata)
CREATE TABLE matches (
    match_id VARCHAR PRIMARY KEY,
    game_creation BIGINT,
    game_duration INTEGER,
    game_end_timestamp BIGINT,
    game_id BIGINT,
    game_mode VARCHAR,
    game_name VARCHAR,
    game_type VARCHAR,
    game_version VARCHAR,
    map_id INTEGER,
    platform_id VARCHAR,
    queue_id INTEGER,
    tournament_code VARCHAR,
    region VARCHAR,
    created_at TIMESTAMP DEFAULT CURRENT_TIMESTAMP
);

-- Participants table (player performance in specific matches)
CREATE TABLE participants (
    id BIGINT PRIMARY KEY,
    match_id VARCHAR REFERENCES matches(match_id),
    puuid VARCHAR REFERENCES summoners(puuid),
    summoner_name VARCHAR,
    champion_id INTEGER,
    champion_name VARCHAR,
    team_id INTEGER,
    position VARCHAR,
    individual_position VARCHAR,
    kills INTEGER,
    deaths INTEGER,
    assists INTEGER,
    total_damage_dealt INTEGER,
    total_damage_dealt_to_champions INTEGER,
    total_damage_taken INTEGER,
    gold_earned INTEGER,
    gold_spent INTEGER,
    turret_kills INTEGER,
    inhibitor_kills INTEGER,
    total_minions_killed INTEGER,
    neutral_minions_killed INTEGER,
    champion_level INTEGER,
    items_0 INTEGER,
    items_1 INTEGER,
    items_2 INTEGER,
    items_3 INTEGER,
    items_4 INTEGER,
    items_5 INTEGER,
    items_6 INTEGER,
    summoner_spell_1 INTEGER,
    summoner_spell_2 INTEGER,
    primary_rune_tree INTEGER,
    secondary_rune_tree INTEGER,
    win BOOLEAN,
    first_blood_kill BOOLEAN,
    first_tower_kill BOOLEAN
);

-- Teams table (team-level statistics)
CREATE TABLE teams (
    id BIGINT PRIMARY KEY,
    match_id VARCHAR REFERENCES matches(match_id),
    team_id INTEGER,
    win BOOLEAN,
    first_baron BOOLEAN,
    first_dragon BOOLEAN,
    first_inhibitor BOOLEAN,
    first_rift_herald BOOLEAN,
    first_tower BOOLEAN,
    baron_kills INTEGER,
    dragon_kills INTEGER,
    inhibitor_kills INTEGER,
    rift_herald_kills INTEGER,
    tower_kills INTEGER
);

-- Bans table
CREATE TABLE bans (
    id BIGINT PRIMARY KEY,
    match_id VARCHAR REFERENCES matches(match_id),
    team_id INTEGER,
    champion_id INTEGER,
    pick_turn INTEGER
);

-- Timeline events (detailed game events)
CREATE TABLE timeline_events (
    id BIGINT PRIMARY KEY,
    match_id VARCHAR REFERENCES matches(match_id),
    timestamp BIGINT,
    event_type VARCHAR,
    participant_id INTEGER,
    position_x INTEGER,
    position_y INTEGER,
    item_id INTEGER,
    skill_slot INTEGER,
    level_up_type VARCHAR,
    ward_type VARCHAR,
    creator_id INTEGER,
    killer_id INTEGER,
    victim_id INTEGER,
    assisting_participant_ids INTEGER[],
    team_id INTEGER,
    monster_type VARCHAR,
    monster_sub_type VARCHAR,
    lane_type VARCHAR,
    tower_type VARCHAR,
    building_type VARCHAR
);

-- Crawler state management
CREATE TABLE crawler_state (
    id INTEGER PRIMARY KEY,
    last_processed_summoner VARCHAR,
    total_summoners_processed INTEGER,
    total_matches_processed INTEGER,
    queue_size INTEGER,
    last_update TIMESTAMP DEFAULT CURRENT_TIMESTAMP
);

-- API rate limiting tracking
CREATE TABLE api_calls (
    id BIGINT PRIMARY KEY,
    endpoint VARCHAR,
    region VARCHAR,
    timestamp TIMESTAMP DEFAULT CURRENT_TIMESTAMP,
    response_code INTEGER,
    rate_limit_remaining INTEGER
);

-- Active games tracking (for seed generation)
CREATE TABLE active_games (
    game_id BIGINT PRIMARY KEY,
    game_type VARCHAR,
    game_start_time BIGINT,
    map_id INTEGER,
    queue_id INTEGER,
    platform_id VARCHAR,
    game_mode VARCHAR,
    participants JSON,
    discovered_at TIMESTAMP DEFAULT CURRENT_TIMESTAMP
);
```

### Indexes for Performance

```sql
CREATE INDEX idx_participants_match_id ON participants(match_id);
CREATE INDEX idx_participants_puuid ON participants(puuid);
CREATE INDEX idx_matches_game_creation ON matches(game_creation);
CREATE INDEX idx_matches_queue_id ON matches(queue_id);
CREATE INDEX idx_timeline_match_id ON timeline_events(match_id);
CREATE INDEX idx_summoners_region ON summoners(region);
```

## 2. Rate Limiting Strategy

### API Rate Limits (Riot Games)
- **Development API**: 20 calls/second, 100 calls/2 minutes
- **Production API**: ~300 calls/second per region (estimated)
- **Rate limit types**: Application, Method, Service
- **Error handling**: 429 status codes with retry-after headers

### Implementation Strategy
```rust
pub struct RateLimiter {
    application_limiter: TokenBucket,
    method_limiters: HashMap<String, TokenBucket>,
    service_limiters: HashMap<String, TokenBucket>,
    retry_queue: VecDeque<PendingRequest>,
}

// Token bucket algorithm with leaky bucket fallback
pub struct TokenBucket {
    capacity: u32,
    tokens: u32,
    refill_rate: u32,
    last_refill: Instant,
}
```

### Rate Limiting Rules
- Monitor X-Rate-Limit headers in responses
- Implement exponential backoff for 429 responses
- Distribute requests across multiple regions
- Prioritize active game fetching over historical data
- Queue non-urgent requests during peak times

## 3. Crawler Architecture

### Core Components

```rust
// Main crawler orchestrator
pub struct CrawlerEngine {
    api_client: RiotApiClient,
    database: DuckDbConnection,
    rate_limiter: RateLimiter,
    summoner_queue: SummonerQueue,
    crawler_state: CrawlerState,
}

// API client with built-in rate limiting
pub struct RiotApiClient {
    client: reqwest::Client,
    rate_limiter: Arc<RateLimiter>,
    base_urls: HashMap<Region, String>,
    api_key: String,
}

// Priority-based summoner processing queue
pub struct SummonerQueue {
    high_priority: VecDeque<SummonerTask>,  // Master+ players
    medium_priority: VecDeque<SummonerTask>, // Diamond players
    low_priority: VecDeque<SummonerTask>,   // Others
}
```

### Data Flow Design

1. **Seed Generation Phase**
   - Fetch featured games every 5 minutes
   - Extract Master+ summoners from active games
   - Add to high-priority queue

2. **Expansion Phase**
   - Process summoners from queue by priority
   - Fetch match history (last 20 matches)
   - Extract unique summoners from matches
   - Store match data and add new summoners to queue

3. **Continuous Operation**
   - Monitor queue sizes and adjust processing rates
   - Implement circuit breaker for API failures
   - Periodic health checks and state persistence

### Processing Priorities
```rust
pub enum SummonerPriority {
    High,    // Master+ tier, recently active
    Medium,  // Diamond tier, active within 7 days
    Low,     // Other tiers, older activity
}
```

## 4. Deployment Infrastructure

### Server Requirements
- **CPU**: 4+ cores (concurrent request processing)
- **RAM**: 8GB+ (queue management and caching)
- **Storage**: 500GB+ SSD (DuckDB growth)
- **Network**: Stable connection with low latency to Riot servers

### Containerization (Docker)
```dockerfile
FROM rust:1.70-slim as builder
WORKDIR /app
COPY . .
RUN cargo build --release

FROM debian:bookworm-slim
RUN apt-get update && apt-get install -y ca-certificates
COPY --from=builder /app/target/release/lol-crawler /usr/local/bin/
CMD ["lol-crawler"]
```

### Environment Configuration
```env
RIOT_API_KEY=your_production_api_key
DATABASE_URL=./data/lol_crawler.db
LOG_LEVEL=info
REGIONS=na1,euw1,kr,eun1
MAX_CONCURRENT_REQUESTS=50
QUEUE_SIZE_LIMIT=100000
HEALTH_CHECK_INTERVAL=60
```

## 5. Monitoring and Observability

### Metrics to Track
- API calls per minute/hour
- Rate limit utilization
- Queue sizes by priority
- Database growth rate
- Error rates by endpoint
- Match processing latency
- Unique summoners discovered

### Logging Strategy
```rust
// Structured logging with serde_json
log::info!(
    "match_processed";
    "match_id" => match_id,
    "duration_ms" => processing_time.as_millis(),
    "participants" => participant_count,
    "new_summoners" => new_summoners_found
);
```

### Health Endpoints
```rust
// Health check endpoint
GET /health
{
    "status": "healthy",
    "uptime_seconds": 86400,
    "queue_sizes": {
        "high": 150,
        "medium": 2500,
        "low": 45000
    },
    "api_rate_limit_remaining": 85,
    "last_successful_request": "2025-01-12T10:30:00Z"
}
```

## 6. Error Handling and Recovery

### Failure Scenarios
- API rate limit exceeded → Exponential backoff with jitter
- Network timeouts → Retry with circuit breaker
- Database lock contention → Queue requests with timeout
- Invalid API responses → Log and skip with alerting
- Disk space exhaustion → Cleanup old data, alert operators

### State Persistence
- Save queue state every 5 minutes
- Checkpoint processed summoner lists
- Track API call history for rate limit recovery
- Maintain resumable crawl position

## 7. Performance Optimizations

### Database Optimizations
- Batch inserts for match data (100-500 records)
- Prepared statements for frequent queries
- Connection pooling for concurrent access
- Periodic VACUUM and ANALYZE operations

### Memory Management
- LRU cache for frequently accessed summoners
- Streaming JSON parsing for large responses
- Bounded queues with backpressure
- Periodic garbage collection tuning

### Network Optimizations
- HTTP/2 connection reuse
- Compression for API responses
- Regional load balancing
- CDN caching for static data (champions, items)

## 8. Security Considerations

### API Key Management
- Environment variable injection
- Key rotation capability
- Request signing validation
- Rate limit monitoring for abuse

### Data Protection
- No PII storage beyond game usernames
- Secure database file permissions
- Encrypted backups if cloud storage used
- Audit logging for data access

This comprehensive plan provides the foundation for a production-ready League of Legends data crawler that can run continuously while respecting Riot's API constraints.