//! In-memory cache implementation.
//!
//! This module provides a thread-safe in-memory cache with TTL support.

use super::{Cache, CacheEntry};
use std::collections::HashMap;
use std::sync::RwLock;
use std::time::Duration;

/// Thread-safe in-memory cache.
#[derive(Debug)]
pub struct MemoryCache {
    /// The cache storage.
    entries: RwLock<HashMap<String, CacheEntry>>,
    /// Maximum number of entries (0 = unlimited).
    max_entries: usize,
}

impl MemoryCache {
    /// Creates a new memory cache with unlimited entries.
    #[must_use]
    pub fn new() -> Self {
        Self {
            entries: RwLock::new(HashMap::new()),
            max_entries: 0,
        }
    }

    /// Creates a new memory cache with a maximum number of entries.
    #[must_use]
    pub fn with_max_entries(max_entries: usize) -> Self {
        Self {
            entries: RwLock::new(HashMap::new()),
            max_entries,
        }
    }

    /// Removes expired entries from the cache.
    pub fn cleanup_expired(&self) {
        if let Ok(mut entries) = self.entries.write() {
            entries.retain(|_, entry| !entry.is_expired());
        }
    }

    /// Gets cache statistics.
    #[must_use]
    pub fn stats(&self) -> CacheStats {
        let entries = self.entries.read().unwrap();
        let total = entries.len();
        let expired = entries.values().filter(|e| e.is_expired()).count();

        CacheStats {
            total_entries: total,
            expired_entries: expired,
            active_entries: total - expired,
        }
    }
}

impl Default for MemoryCache {
    fn default() -> Self {
        Self::new()
    }
}

impl Cache for MemoryCache {
    fn get(&self, key: &str) -> Option<Vec<u8>> {
        let entries = self.entries.read().ok()?;
        let entry = entries.get(key)?;

        if entry.is_expired() {
            // Entry is expired, return None
            // Cleanup will happen on next set or explicit cleanup
            return None;
        }

        Some(entry.data.clone())
    }

    fn set(&self, key: &str, value: Vec<u8>, ttl: Duration) {
        if let Ok(mut entries) = self.entries.write() {
            // Check if we need to evict entries
            if self.max_entries > 0 && entries.len() >= self.max_entries {
                // Remove expired entries first
                entries.retain(|_, entry| !entry.is_expired());

                // If still at capacity, remove oldest entry
                if entries.len() >= self.max_entries {
                    // Find and remove the entry with the earliest expiration
                    if let Some(oldest_key) = entries
                        .iter()
                        .min_by_key(|(_, e)| e.expires_at)
                        .map(|(k, _)| k.clone())
                    {
                        entries.remove(&oldest_key);
                    }
                }
            }

            let entry = CacheEntry::new(value, ttl);
            entries.insert(key.to_string(), entry);
        }
    }

    fn remove(&self, key: &str) {
        if let Ok(mut entries) = self.entries.write() {
            entries.remove(key);
        }
    }

    fn clear(&self) {
        if let Ok(mut entries) = self.entries.write() {
            entries.clear();
        }
    }

    fn len(&self) -> usize {
        self.entries.read().map(|e| e.len()).unwrap_or(0)
    }
}

/// Cache statistics.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct CacheStats {
    /// Total number of entries.
    pub total_entries: usize,
    /// Number of expired entries.
    pub expired_entries: usize,
    /// Number of active (non-expired) entries.
    pub active_entries: usize,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_memory_cache_basic() {
        let cache = MemoryCache::new();

        cache.set("key1", vec![1, 2, 3], Duration::from_secs(3600));
        assert_eq!(cache.get("key1"), Some(vec![1, 2, 3]));
        assert_eq!(cache.len(), 1);
    }

    #[test]
    fn test_memory_cache_miss() {
        let cache = MemoryCache::new();
        assert_eq!(cache.get("nonexistent"), None);
    }

    #[test]
    fn test_memory_cache_remove() {
        let cache = MemoryCache::new();

        cache.set("key1", vec![1, 2, 3], Duration::from_secs(3600));
        assert!(cache.contains("key1"));

        cache.remove("key1");
        assert!(!cache.contains("key1"));
    }

    #[test]
    fn test_memory_cache_clear() {
        let cache = MemoryCache::new();

        cache.set("key1", vec![1], Duration::from_secs(3600));
        cache.set("key2", vec![2], Duration::from_secs(3600));
        cache.set("key3", vec![3], Duration::from_secs(3600));

        assert_eq!(cache.len(), 3);

        cache.clear();
        assert!(cache.is_empty());
    }

    #[test]
    fn test_memory_cache_expiration() {
        let cache = MemoryCache::new();

        // Set with 0 TTL (immediately expired)
        cache.set("expired", vec![1, 2, 3], Duration::from_secs(0));

        // Should not return expired entry
        assert_eq!(cache.get("expired"), None);
    }

    #[test]
    fn test_memory_cache_max_entries() {
        let cache = MemoryCache::with_max_entries(2);

        cache.set("key1", vec![1], Duration::from_secs(3600));
        cache.set("key2", vec![2], Duration::from_secs(3600));
        cache.set("key3", vec![3], Duration::from_secs(3600));

        // Should only have 2 entries
        assert_eq!(cache.len(), 2);
        // Most recent should be present
        assert!(cache.contains("key3"));
    }

    #[test]
    fn test_memory_cache_stats() {
        let cache = MemoryCache::new();

        cache.set("active", vec![1], Duration::from_secs(3600));
        cache.set("expired", vec![2], Duration::from_secs(0));

        let stats = cache.stats();
        assert_eq!(stats.total_entries, 2);
        assert_eq!(stats.expired_entries, 1);
        assert_eq!(stats.active_entries, 1);
    }

    #[test]
    fn test_memory_cache_cleanup() {
        let cache = MemoryCache::new();

        cache.set("active", vec![1], Duration::from_secs(3600));
        cache.set("expired", vec![2], Duration::from_secs(0));

        assert_eq!(cache.len(), 2);

        cache.cleanup_expired();

        assert_eq!(cache.len(), 1);
        assert!(cache.contains("active"));
        assert!(!cache.contains("expired"));
    }
}
