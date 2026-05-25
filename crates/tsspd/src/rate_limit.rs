//! Login rate limiting with token bucket per IP.

use std::collections::HashMap;
use std::net::IpAddr;
use std::sync::Arc;
use tokio::sync::RwLock;
use std::time::{SystemTime, UNIX_EPOCH};

const MAX_ATTEMPTS: u32 = 5;
const LOCKOUT_SECONDS: u64 = 1800; // 30 minutes
const BUCKET_DECAY_SECONDS: u64 = 30; // Refill bucket every 30s

/// Rate limiter state for a single IP.
#[derive(Debug, Clone)]
struct BucketState {
    failed_attempts: u32,
    last_reset: u64,
}

impl BucketState {
    fn new() -> Self {
        Self {
            failed_attempts: 0,
            last_reset: now_seconds(),
        }
    }

    /// Check if enough time has passed to reset the bucket.
    fn should_reset(&self) -> bool {
        now_seconds().saturating_sub(self.last_reset) >= BUCKET_DECAY_SECONDS
    }

    /// Check if this IP is currently locked out.
    fn is_locked_out(&self) -> bool {
        self.failed_attempts >= MAX_ATTEMPTS
            && now_seconds().saturating_sub(self.last_reset) < LOCKOUT_SECONDS
    }

    /// Record a failed attempt.
    fn record_failure(&mut self) {
        if self.should_reset() {
            self.failed_attempts = 0;
            self.last_reset = now_seconds();
        }
        self.failed_attempts += 1;
    }

    /// Reset on successful login.
    fn reset(&mut self) {
        self.failed_attempts = 0;
        self.last_reset = now_seconds();
    }
}

fn now_seconds() -> u64 {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map(|d| d.as_secs())
        .unwrap_or(0)
}

/// In-memory rate limiter for login attempts.
pub struct RateLimiter {
    buckets: Arc<RwLock<HashMap<IpAddr, BucketState>>>,
}

impl RateLimiter {
    /// Create a new rate limiter.
    pub fn new() -> Self {
        Self {
            buckets: Arc::new(RwLock::new(HashMap::new())),
        }
    }

    /// Check if an IP is allowed to attempt login.
    /// Returns true if the attempt is allowed, false if rate-limited.
    pub async fn check_and_record_attempt(&self, ip: IpAddr) -> bool {
        let mut buckets = self.buckets.write().await;
        let bucket = buckets
            .entry(ip)
            .or_insert_with(BucketState::new);

        if bucket.should_reset() {
            bucket.failed_attempts = 0;
            bucket.last_reset = now_seconds();
        }

        if bucket.is_locked_out() {
            return false;
        }

        true
    }

    /// Record a failed login attempt.
    pub async fn record_failure(&self, ip: IpAddr) {
        let mut buckets = self.buckets.write().await;
        buckets
            .entry(ip)
            .or_insert_with(BucketState::new)
            .record_failure();
    }

    /// Clear the failed attempts on successful login.
    pub async fn record_success(&self, ip: IpAddr) {
        let mut buckets = self.buckets.write().await;
        if let Some(bucket) = buckets.get_mut(&ip) {
            bucket.reset();
        }
    }

    /// Get remaining attempts for diagnostics (None if locked out).
    #[allow(dead_code)]
    pub async fn remaining_attempts(&self, ip: IpAddr) -> Option<u32> {
        let buckets = self.buckets.read().await;
        buckets.get(&ip).and_then(|bucket| {
            if bucket.is_locked_out() {
                None
            } else {
                Some(MAX_ATTEMPTS.saturating_sub(bucket.failed_attempts))
            }
        })
    }
}

impl Clone for RateLimiter {
    fn clone(&self) -> Self {
        Self {
            buckets: self.buckets.clone(),
        }
    }
}

impl Default for RateLimiter {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_allows_initial_attempts() {
        let limiter = RateLimiter::new();
        let ip = "127.0.0.1".parse().unwrap();

        for _ in 0..5 {
            assert!(limiter.check_and_record_attempt(ip).await);
        }
    }

    #[tokio::test]
    async fn test_blocks_after_max_attempts() {
        let limiter = RateLimiter::new();
        let ip = "127.0.0.1".parse().unwrap();

        for _ in 0..5 {
            assert!(limiter.check_and_record_attempt(ip).await);
            limiter.record_failure(ip).await;
        }

        assert!(!limiter.check_and_record_attempt(ip).await);
    }

    #[tokio::test]
    async fn test_resets_on_success() {
        let limiter = RateLimiter::new();
        let ip = "127.0.0.1".parse().unwrap();

        for _ in 0..3 {
            assert!(limiter.check_and_record_attempt(ip).await);
            limiter.record_failure(ip).await;
        }

        limiter.record_success(ip).await;

        assert_eq!(limiter.remaining_attempts(ip).await, Some(5));
    }

    #[tokio::test]
    async fn test_returns_none_when_locked() {
        let limiter = RateLimiter::new();
        let ip = "127.0.0.1".parse().unwrap();

        for _ in 0..5 {
            assert!(limiter.check_and_record_attempt(ip).await);
            limiter.record_failure(ip).await;
        }

        assert_eq!(limiter.remaining_attempts(ip).await, None);
    }
}
