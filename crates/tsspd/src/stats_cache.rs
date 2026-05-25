//! Cached repository statistics to avoid expensive COUNT(*) queries.

use std::sync::Arc;
use tokio::sync::RwLock;
use std::time::{SystemTime, UNIX_EPOCH};
use tssp_ports::RepositoryStats;

/// How long to cache stats before refreshing (5 seconds).
const CACHE_TTL_SECONDS: u64 = 5;

#[derive(Clone)]
struct CachedStats {
    stats: RepositoryStats,
    cached_at: u64,
}

/// In-memory statistics cache to avoid expensive `SQLite` COUNT(*) queries.
pub struct StatsCache {
    cached: Arc<RwLock<Option<CachedStats>>>,
}

impl StatsCache {
    /// Create a new stats cache.
    pub fn new() -> Self {
        Self {
            cached: Arc::new(RwLock::new(None)),
        }
    }

    /// Get cached stats if available and fresh, otherwise None.
    pub async fn get(&self) -> Option<RepositoryStats> {
        let cached = self.cached.read().await;
        cached.as_ref().and_then(|entry| {
            let now = now_seconds();
            if now.saturating_sub(entry.cached_at) < CACHE_TTL_SECONDS {
                Some(entry.stats)
            } else {
                None
            }
        })
    }

    /// Store stats in the cache.
    pub async fn store(&self, stats: RepositoryStats) {
        let mut cached = self.cached.write().await;
        *cached = Some(CachedStats {
            stats,
            cached_at: now_seconds(),
        });
    }

    /// Clear the cache.
    #[allow(dead_code)]
    pub async fn clear(&self) {
        let mut cached = self.cached.write().await;
        *cached = None;
    }
}

impl Clone for StatsCache {
    fn clone(&self) -> Self {
        Self {
            cached: self.cached.clone(),
        }
    }
}

impl Default for StatsCache {
    fn default() -> Self {
        Self::new()
    }
}

fn now_seconds() -> u64 {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map(|d| d.as_secs())
        .unwrap_or(0)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_stores_and_retrieves_stats() {
        let cache = StatsCache::new();
        let stats = RepositoryStats {
            file_count: 10,
            note_count: 5,
            tag_count: 3,
            pinned_count: 2,
            recent_upload_count: 1,
            recent_note_count: 0,
            storage_bytes_used: 1024,
        };

        cache.store(stats).await;

        let retrieved = cache.get().await;
        assert!(retrieved.is_some());
        let retrieved = retrieved.unwrap();
        assert_eq!(retrieved.file_count, 10);
    }

    #[tokio::test]
    async fn test_returns_none_when_empty() {
        let cache = StatsCache::new();
        assert!(cache.get().await.is_none());
    }

    #[tokio::test]
    async fn test_clear_clears_cache() {
        let cache = StatsCache::new();
        let stats = RepositoryStats {
            file_count: 10,
            note_count: 5,
            tag_count: 3,
            pinned_count: 2,
            recent_upload_count: 1,
            recent_note_count: 0,
            storage_bytes_used: 1024,
        };

        cache.store(stats).await;
        cache.clear().await;

        assert!(cache.get().await.is_none());
    }
}
