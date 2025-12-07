//! Caching layer for market data.
//!
//! This module provides caching functionality to reduce API calls
//! and improve performance for repeated data requests.

mod memory;
mod persistent;

pub use memory::MemoryCache;
pub use persistent::FileCache;

use anyhow::Result;
use std::time::Duration;

/// Trait for cache implementations.
pub trait Cache: Send + Sync {
    /// Gets a value from the cache.
    ///
    /// # Arguments
    /// * `key` - The cache key
    ///
    /// # Returns
    /// The cached value if present and not expired, None otherwise
    fn get(&self, key: &str) -> Option<Vec<u8>>;

    /// Sets a value in the cache with a TTL.
    ///
    /// # Arguments
    /// * `key` - The cache key
    /// * `value` - The value to cache
    /// * `ttl` - Time-to-live for the cached value
    fn set(&self, key: &str, value: Vec<u8>, ttl: Duration);

    /// Removes a value from the cache.
    ///
    /// # Arguments
    /// * `key` - The cache key
    fn remove(&self, key: &str);

    /// Clears all values from the cache.
    fn clear(&self);

    /// Returns the number of items in the cache.
    fn len(&self) -> usize;

    /// Returns true if the cache is empty.
    fn is_empty(&self) -> bool {
        self.len() == 0
    }

    /// Checks if a key exists in the cache.
    fn contains(&self, key: &str) -> bool {
        self.get(key).is_some()
    }
}

/// Cache entry with expiration.
#[derive(Debug, Clone)]
pub struct CacheEntry {
    /// The cached data.
    pub data: Vec<u8>,
    /// Expiration timestamp in seconds since epoch.
    pub expires_at: u64,
}

impl CacheEntry {
    /// Creates a new cache entry.
    #[must_use]
    pub fn new(data: Vec<u8>, ttl: Duration) -> Self {
        let now = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap_or_default()
            .as_secs();

        Self {
            data,
            expires_at: now + ttl.as_secs(),
        }
    }

    /// Checks if the entry has expired.
    #[must_use]
    pub fn is_expired(&self) -> bool {
        let now = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap_or_default()
            .as_secs();

        now >= self.expires_at
    }
}

/// Cache key builder for consistent key generation.
#[derive(Debug, Default)]
pub struct CacheKeyBuilder {
    parts: Vec<String>,
}

impl CacheKeyBuilder {
    /// Creates a new cache key builder.
    #[must_use]
    pub fn new() -> Self {
        Self { parts: Vec::new() }
    }

    /// Adds a part to the key.
    #[must_use]
    pub fn with(mut self, part: impl Into<String>) -> Self {
        self.parts.push(part.into());
        self
    }

    /// Builds the final cache key.
    #[must_use]
    pub fn build(self) -> String {
        self.parts.join(":")
    }
}

/// Cached data provider wrapper.
///
/// Wraps any data provider with caching functionality.
pub struct CachedProvider<P, C> {
    /// The underlying provider.
    provider: P,
    /// The cache implementation.
    cache: C,
    /// Default TTL for cached data.
    default_ttl: Duration,
}

impl<P, C: Cache> CachedProvider<P, C> {
    /// Creates a new cached provider.
    #[must_use]
    pub fn new(provider: P, cache: C, default_ttl: Duration) -> Self {
        Self {
            provider,
            cache,
            default_ttl,
        }
    }

    /// Gets the underlying provider.
    #[must_use]
    pub fn provider(&self) -> &P {
        &self.provider
    }

    /// Gets the cache.
    #[must_use]
    pub fn cache(&self) -> &C {
        &self.cache
    }

    /// Gets data from cache or fetches it.
    pub fn get_or_fetch<T, F>(&self, key: &str, fetch: F) -> Result<T>
    where
        T: serde::de::DeserializeOwned + serde::Serialize,
        F: FnOnce(&P) -> Result<T>,
    {
        // Try cache first
        if let Some(data) = self.cache.get(key)
            && let Ok(value) = serde_json::from_slice(&data)
        {
            return Ok(value);
        }

        // Fetch from provider
        let value = fetch(&self.provider)?;

        // Cache the result
        if let Ok(data) = serde_json::to_vec(&value) {
            self.cache.set(key, data, self.default_ttl);
        }

        Ok(value)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_cache_entry_expiration() {
        let entry = CacheEntry::new(vec![1, 2, 3], Duration::from_secs(3600));
        assert!(!entry.is_expired());

        let expired_entry = CacheEntry {
            data: vec![1, 2, 3],
            expires_at: 0, // Already expired
        };
        assert!(expired_entry.is_expired());
    }

    #[test]
    fn test_cache_key_builder() {
        let key = CacheKeyBuilder::new()
            .with("price")
            .with("SOL")
            .with("USDC")
            .with("1h")
            .build();

        assert_eq!(key, "price:SOL:USDC:1h");
    }

    #[test]
    fn test_cache_key_builder_empty() {
        let key = CacheKeyBuilder::new().build();
        assert_eq!(key, "");
    }
}
