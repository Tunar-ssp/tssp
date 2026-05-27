//! Login rate limiting with token bucket per IP.
//!
//! Uses a bounded `HashMap`: evicts expired entries when the map exceeds `MAX_TRACKED_IPS`
//! to prevent OOM from bots spamming random source IPs.

use std::collections::HashMap;
use std::net::IpAddr;
use std::sync::Arc;
use std::time::{SystemTime, UNIX_EPOCH};
use tokio::sync::RwLock;

const MAX_ATTEMPTS: u32 = 5;
const LOCKOUT_SECONDS: u64 = 1800; // 30 minutes

/// Hard cap on tracked IPs. When exceeded, expired entries are purged first;
/// if still over capacity the write is dropped (fail-open for new IPs).
const MAX_TRACKED_IPS: usize = 10_000;

/// Rate limiter state for a single IP.
#[derive(Debug, Clone)]
struct BucketState {
    failed_attempts: u32,
    first_failure_time: u64,
}

impl BucketState {
    fn new() -> Self {
        Self {
            failed_attempts: 0,
            first_failure_time: 0,
        }
    }

    /// Check if this IP is currently locked out.
    fn is_locked_out(&self) -> bool {
        if self.failed_attempts < MAX_ATTEMPTS {
            return false;
        }
        let elapsed = now_seconds().saturating_sub(self.first_failure_time);
        elapsed < LOCKOUT_SECONDS
    }

    /// Check if this entry is expired (lockout period passed).
    #[allow(dead_code)]
    fn is_expired(&self) -> bool {
        if self.failed_attempts < MAX_ATTEMPTS {
            return false;
        }
        let elapsed = now_seconds().saturating_sub(self.first_failure_time);
        elapsed >= LOCKOUT_SECONDS
    }

    /// Record a failed attempt.
    fn record_failure(&mut self) {
        if self.failed_attempts == 0 {
            self.first_failure_time = now_seconds();
        }
        self.failed_attempts += 1;
    }

    /// Reset on successful login.
    #[allow(dead_code)]
    fn reset(&mut self) {
        self.failed_attempts = 0;
        self.first_failure_time = 0;
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
        let buckets = self.buckets.read().await;
        !matches!(buckets.get(&ip), Some(bucket) if bucket.is_locked_out())
    }

    /// Record a failed login attempt.
    pub async fn record_failure(&self, ip: IpAddr) {
        let mut buckets = self.buckets.write().await;

        // Enforce capacity: purge expired entries first.
        if buckets.len() >= MAX_TRACKED_IPS && !buckets.contains_key(&ip) {
            buckets.retain(|_, b| !b.is_expired());
            // Still over cap after eviction — drop this entry (fail-open).
            if buckets.len() >= MAX_TRACKED_IPS {
                return;
            }
        }

        buckets
            .entry(ip)
            .or_insert_with(BucketState::new)
            .record_failure();
    }

    /// Clear the failed attempts on successful login.
    pub async fn record_success(&self, ip: IpAddr) {
        let mut buckets = self.buckets.write().await;
        buckets.remove(&ip);
    }

    /// Get remaining attempts for diagnostics (None if locked out).
    #[allow(dead_code)]
    pub async fn remaining_attempts(&self, ip: IpAddr) -> Option<u32> {
        let buckets = self.buckets.read().await;
        match buckets.get(&ip) {
            Some(bucket) => {
                if bucket.is_locked_out() {
                    None
                } else {
                    Some(MAX_ATTEMPTS.saturating_sub(bucket.failed_attempts))
                }
            }
            None => Some(MAX_ATTEMPTS),
        }
    }

    /// Clean up expired entries to prevent memory leak.
    /// Should be called periodically (e.g., every hour).
    #[allow(dead_code)]
    pub async fn cleanup_expired(&self) {
        let mut buckets = self.buckets.write().await;
        buckets.retain(|_, bucket| !bucket.is_expired());
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
