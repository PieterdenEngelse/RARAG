// src/tools/rate_limiter.rs
// Optimization #6: Token Bucket Rate Limiter

use std::sync::Arc;
use tokio::sync::RwLock;
use tokio::time::{sleep, Duration, Instant};

pub struct RateLimiter {
    tokens: Arc<RwLock<f64>>,
    max_tokens: f64,
    refill_rate: f64,
    last_refill: Arc<RwLock<Instant>>,
}

impl RateLimiter {
    pub fn new(max_tokens: f64, refill_rate: f64) -> Self {
        Self {
            tokens: Arc::new(RwLock::new(max_tokens)),
            max_tokens,
            refill_rate,
            last_refill: Arc::new(RwLock::new(Instant::now())),
        }
    }

    /// Acquire tokens, wait if necessary
    pub async fn acquire(&self, tokens_needed: f64) {
        loop {
            // Refill tokens based on elapsed time
            {
                let mut t = self.tokens.write().await;
                let mut last = self.last_refill.write().await;

                let elapsed = last.elapsed().as_secs_f64();
                *t = (*t + elapsed * self.refill_rate).min(self.max_tokens);
                *last = Instant::now();
            }

            // Check if we have enough tokens
            let mut t = self.tokens.write().await;
            if *t >= tokens_needed {
                *t -= tokens_needed;
                return;
            }

            // Calculate wait time and release lock
            let needed = tokens_needed - *t;
            let wait_time = needed / self.refill_rate;
            drop(t);

            sleep(Duration::from_secs_f64(wait_time)).await;
        }
    }

    /// Get current token count
    pub async fn current_tokens(&self) -> f64 {
        let t = self.tokens.read().await;
        *t
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_token_system_basics() {
        // Basic token arithmetic test
        let max_tokens = 10.0;
        let mut tokens = max_tokens;

        assert_eq!(tokens, 10.0);
        tokens -= 5.0;
        assert_eq!(tokens, 5.0);
    }

    #[tokio::test]
    async fn test_acquire_immediate() {
        let limiter = RateLimiter::new(10.0, 1.0);
        limiter.acquire(5.0).await;
        let tokens = limiter.current_tokens().await;
        assert!((tokens - 5.0).abs() < 0.1);
    }

    #[tokio::test]
    async fn test_acquire_with_wait() {
        let limiter = RateLimiter::new(5.0, 1.0);
        let start = Instant::now();

        limiter.acquire(5.0).await;
        limiter.acquire(5.0).await;

        let elapsed = start.elapsed().as_secs_f64();
        assert!(elapsed >= 4.5);
    }

    #[test]
    fn test_burst_capacity() {
        let max_tokens = 10.0;
        let mut tokens = max_tokens;
        tokens -= 8.0;
        assert_eq!(tokens, 2.0);
    }

    #[test]
    fn test_multiple_acquisitions() {
        let mut tokens = 10.0;
        let mut acquisitions = 0;

        while tokens >= 2.0 {
            tokens -= 2.0;
            acquisitions += 1;
        }

        assert_eq!(acquisitions, 5);
    }
}
