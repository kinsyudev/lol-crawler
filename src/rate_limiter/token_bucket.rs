use std::time::{Duration, Instant};
use tokio::time::sleep;

#[derive(Debug)]
pub struct TokenBucket {
    capacity: u32,
    tokens: u32,
    refill_rate: u32,
    refill_interval: Duration,
    last_refill: Instant,
}

impl TokenBucket {
    pub fn new(capacity: u32, refill_rate: u32, refill_interval: Duration) -> Self {
        Self {
            capacity,
            tokens: capacity,
            refill_rate,
            refill_interval,
            last_refill: Instant::now(),
        }
    }

    pub fn per_second(capacity: u32, rate_per_second: u32) -> Self {
        Self::new(capacity, rate_per_second, Duration::from_secs(1))
    }

    pub fn per_two_minutes(capacity: u32, rate_per_two_minutes: u32) -> Self {
        Self::new(capacity, rate_per_two_minutes, Duration::from_secs(120))
    }

    pub async fn acquire(
        &mut self,
        tokens: u32,
    ) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
        self.refill();

        if self.tokens >= tokens {
            self.tokens -= tokens;
            return Ok(());
        }

        // Calculate wait time
        let tokens_needed = tokens - self.tokens;
        let wait_time = self.calculate_wait_time(tokens_needed);

        log::debug!(
            "Rate limit hit, waiting {:?} for {} tokens",
            wait_time,
            tokens_needed
        );
        sleep(wait_time).await;

        self.refill();
        if self.tokens >= tokens {
            self.tokens -= tokens;
            Ok(())
        } else {
            Err("Unable to acquire tokens after waiting".into())
        }
    }

    pub fn try_acquire(&mut self, tokens: u32) -> bool {
        self.refill();

        if self.tokens >= tokens {
            self.tokens -= tokens;
            true
        } else {
            false
        }
    }

    pub fn available_tokens(&mut self) -> u32 {
        self.refill();
        self.tokens
    }

    fn refill(&mut self) {
        let now = Instant::now();
        let elapsed = now.duration_since(self.last_refill);

        if elapsed >= self.refill_interval {
            let intervals_passed = elapsed.as_millis() / self.refill_interval.as_millis();
            let tokens_to_add = (intervals_passed as u32) * self.refill_rate;

            self.tokens = (self.tokens + tokens_to_add).min(self.capacity);
            self.last_refill = now;
        }
    }

    fn calculate_wait_time(&self, tokens_needed: u32) -> Duration {
        let intervals_needed = tokens_needed.div_ceil(self.refill_rate);
        Duration::from_millis(intervals_needed as u64 * self.refill_interval.as_millis() as u64)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use tokio::time::{sleep, Duration};

    #[tokio::test]
    async fn test_token_bucket_basic() {
        let mut bucket = TokenBucket::per_second(10, 10);

        // Should be able to acquire initial tokens
        assert!(bucket.try_acquire(5));
        assert_eq!(bucket.available_tokens(), 5);

        // Should fail to acquire more than available
        assert!(!bucket.try_acquire(10));

        // Should still have 5 tokens
        assert_eq!(bucket.available_tokens(), 5);
    }

    #[tokio::test]
    async fn test_token_bucket_refill() {
        let mut bucket = TokenBucket::new(10, 10, Duration::from_millis(100));

        // Consume all tokens
        assert!(bucket.try_acquire(10));
        assert_eq!(bucket.available_tokens(), 0);

        // Wait for refill
        sleep(Duration::from_millis(150)).await;

        // Should have refilled
        assert!(bucket.available_tokens() > 0);
    }

    #[tokio::test]
    async fn test_token_bucket_acquire_wait() {
        let mut bucket = TokenBucket::new(5, 5, Duration::from_millis(100));

        // Consume all tokens
        bucket.try_acquire(5);

        let start = Instant::now();
        bucket.acquire(3).await.unwrap();
        let elapsed = start.elapsed();

        // Should have waited approximately 100ms for refill
        assert!(elapsed >= Duration::from_millis(90));
        assert!(elapsed <= Duration::from_millis(200));
    }
}
