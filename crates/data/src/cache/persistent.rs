//! File-based persistent cache implementation.
//!
//! This module provides a file-based cache that persists data to disk.

use super::{Cache, CacheEntry};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::fs;
use std::path::PathBuf;
use std::sync::RwLock;
use std::time::Duration;

/// File-based persistent cache.
#[derive(Debug)]
pub struct FileCache {
    /// Directory where cache files are stored.
    cache_dir: PathBuf,
    /// In-memory index of cache entries.
    index: RwLock<HashMap<String, CacheEntryMetadata>>,
    /// Whether to use a single file or multiple files.
    single_file: bool,
}

/// Metadata for a cache entry (stored in index).
#[derive(Debug, Clone, Serialize, Deserialize)]
struct CacheEntryMetadata {
    /// Expiration timestamp in seconds since epoch.
    expires_at: u64,
    /// File path for multi-file mode, or key for single-file mode.
    location: String,
}

/// Single-file cache format.
#[derive(Debug, Serialize, Deserialize, Default)]
struct CacheFile {
    entries: HashMap<String, SerializedEntry>,
}

/// Serialized cache entry.
#[derive(Debug, Serialize, Deserialize)]
struct SerializedEntry {
    data: Vec<u8>,
    expires_at: u64,
}

impl FileCache {
    /// Creates a new file cache in the specified directory.
    ///
    /// # Arguments
    /// * `cache_dir` - Directory to store cache files
    ///
    /// # Errors
    /// Returns an error if the directory cannot be created
    pub fn new(cache_dir: PathBuf) -> std::io::Result<Self> {
        fs::create_dir_all(&cache_dir)?;

        let cache = Self {
            cache_dir,
            index: RwLock::new(HashMap::new()),
            single_file: true,
        };

        // Load existing cache
        cache.load_index();

        Ok(cache)
    }

    /// Creates a file cache using multiple files (one per key).
    pub fn multi_file(cache_dir: PathBuf) -> std::io::Result<Self> {
        fs::create_dir_all(&cache_dir)?;

        let cache = Self {
            cache_dir,
            index: RwLock::new(HashMap::new()),
            single_file: false,
        };

        cache.load_index();

        Ok(cache)
    }

    /// Returns the cache directory path.
    #[must_use]
    pub fn cache_dir(&self) -> &PathBuf {
        &self.cache_dir
    }

    /// Loads the cache index from disk.
    fn load_index(&self) {
        if self.single_file {
            self.load_single_file_index();
        } else {
            self.load_multi_file_index();
        }
    }

    /// Loads index from single cache file.
    fn load_single_file_index(&self) {
        let cache_file = self.cache_dir.join("cache.json");
        if let Ok(content) = fs::read_to_string(&cache_file)
            && let Ok(cache_data) = serde_json::from_str::<CacheFile>(&content)
        {
            let mut index = self.index.write().unwrap();
            for (key, entry) in cache_data.entries {
                index.insert(
                    key.clone(),
                    CacheEntryMetadata {
                        expires_at: entry.expires_at,
                        location: key,
                    },
                );
            }
        }
    }

    /// Loads index from multiple cache files.
    fn load_multi_file_index(&self) {
        if let Ok(entries) = fs::read_dir(&self.cache_dir) {
            let mut index = self.index.write().unwrap();
            for entry in entries.flatten() {
                if let Some(filename) = entry.file_name().to_str()
                    && filename.ends_with(".cache")
                {
                    let key = filename.trim_end_matches(".cache").to_string();
                    if let Ok(content) = fs::read_to_string(entry.path())
                        && let Ok(cached) = serde_json::from_str::<SerializedEntry>(&content)
                    {
                        index.insert(
                            key.clone(),
                            CacheEntryMetadata {
                                expires_at: cached.expires_at,
                                location: entry.path().to_string_lossy().to_string(),
                            },
                        );
                    }
                }
            }
        }
    }

    /// Saves the cache to disk (single-file mode).
    fn save_single_file(&self) {
        let cache_file = self.cache_dir.join("cache.json");

        // Read existing cache
        let mut cache_data = if let Ok(content) = fs::read_to_string(&cache_file) {
            serde_json::from_str(&content).unwrap_or_default()
        } else {
            CacheFile::default()
        };

        // Update with current index
        let index = self.index.read().unwrap();
        cache_data.entries.retain(|k, _| index.contains_key(k));

        // Write back
        if let Ok(content) = serde_json::to_string_pretty(&cache_data) {
            let _ = fs::write(&cache_file, content);
        }
    }

    /// Generates a safe filename from a cache key.
    fn key_to_filename(&self, key: &str) -> String {
        // Replace unsafe characters
        let safe_key: String = key
            .chars()
            .map(|c| {
                if c.is_alphanumeric() || c == '-' || c == '_' {
                    c
                } else {
                    '_'
                }
            })
            .collect();
        format!("{}.cache", safe_key)
    }

    /// Removes expired entries from disk.
    pub fn cleanup_expired(&self) {
        let now = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap_or_default()
            .as_secs();

        let mut index = self.index.write().unwrap();
        let expired_keys: Vec<String> = index
            .iter()
            .filter(|(_, meta)| meta.expires_at <= now)
            .map(|(k, _)| k.clone())
            .collect();

        for key in expired_keys {
            if !self.single_file {
                let filename = self.key_to_filename(&key);
                let file_path = self.cache_dir.join(&filename);
                let _ = fs::remove_file(file_path);
            }
            index.remove(&key);
        }

        if self.single_file {
            drop(index);
            self.save_single_file();
        }
    }
}

impl Cache for FileCache {
    fn get(&self, key: &str) -> Option<Vec<u8>> {
        let index = self.index.read().ok()?;
        let meta = index.get(key)?;

        // Check expiration
        let now = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap_or_default()
            .as_secs();

        if meta.expires_at <= now {
            return None;
        }

        if self.single_file {
            // Read from single cache file
            let cache_file = self.cache_dir.join("cache.json");
            let content = fs::read_to_string(&cache_file).ok()?;
            let cache_data: CacheFile = serde_json::from_str(&content).ok()?;
            cache_data.entries.get(key).map(|e| e.data.clone())
        } else {
            // Read from individual file
            let filename = self.key_to_filename(key);
            let file_path = self.cache_dir.join(&filename);
            let content = fs::read_to_string(&file_path).ok()?;
            let entry: SerializedEntry = serde_json::from_str(&content).ok()?;
            Some(entry.data)
        }
    }

    fn set(&self, key: &str, value: Vec<u8>, ttl: Duration) {
        let entry = CacheEntry::new(value.clone(), ttl);

        // Update index
        {
            let mut index = self.index.write().unwrap();
            index.insert(
                key.to_string(),
                CacheEntryMetadata {
                    expires_at: entry.expires_at,
                    location: key.to_string(),
                },
            );
        }

        if self.single_file {
            // Update single cache file
            let cache_file = self.cache_dir.join("cache.json");
            let mut cache_data = if let Ok(content) = fs::read_to_string(&cache_file) {
                serde_json::from_str(&content).unwrap_or_default()
            } else {
                CacheFile::default()
            };

            cache_data.entries.insert(
                key.to_string(),
                SerializedEntry {
                    data: value,
                    expires_at: entry.expires_at,
                },
            );

            if let Ok(content) = serde_json::to_string_pretty(&cache_data) {
                let _ = fs::write(&cache_file, content);
            }
        } else {
            // Write to individual file
            let filename = self.key_to_filename(key);
            let file_path = self.cache_dir.join(&filename);
            let serialized = SerializedEntry {
                data: value,
                expires_at: entry.expires_at,
            };

            if let Ok(content) = serde_json::to_string_pretty(&serialized) {
                let _ = fs::write(&file_path, content);
            }
        }
    }

    fn remove(&self, key: &str) {
        {
            let mut index = self.index.write().unwrap();
            index.remove(key);
        }

        if self.single_file {
            self.save_single_file();
        } else {
            let filename = self.key_to_filename(key);
            let file_path = self.cache_dir.join(&filename);
            let _ = fs::remove_file(file_path);
        }
    }

    fn clear(&self) {
        {
            let mut index = self.index.write().unwrap();
            index.clear();
        }

        if self.single_file {
            let cache_file = self.cache_dir.join("cache.json");
            let _ = fs::write(&cache_file, "{}");
        } else if let Ok(entries) = fs::read_dir(&self.cache_dir) {
            for entry in entries.flatten() {
                if let Some(filename) = entry.file_name().to_str()
                    && filename.ends_with(".cache")
                {
                    let _ = fs::remove_file(entry.path());
                }
            }
        }
    }

    fn len(&self) -> usize {
        self.index.read().map(|i| i.len()).unwrap_or(0)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::tempdir;

    #[test]
    fn test_file_cache_basic() {
        let dir = tempdir().unwrap();
        let cache = FileCache::new(dir.path().to_path_buf()).unwrap();

        cache.set("key1", vec![1, 2, 3], Duration::from_secs(3600));
        assert_eq!(cache.get("key1"), Some(vec![1, 2, 3]));
        assert_eq!(cache.len(), 1);
    }

    #[test]
    fn test_file_cache_miss() {
        let dir = tempdir().unwrap();
        let cache = FileCache::new(dir.path().to_path_buf()).unwrap();

        assert_eq!(cache.get("nonexistent"), None);
    }

    #[test]
    fn test_file_cache_remove() {
        let dir = tempdir().unwrap();
        let cache = FileCache::new(dir.path().to_path_buf()).unwrap();

        cache.set("key1", vec![1, 2, 3], Duration::from_secs(3600));
        assert!(cache.contains("key1"));

        cache.remove("key1");
        assert!(!cache.contains("key1"));
    }

    #[test]
    fn test_file_cache_clear() {
        let dir = tempdir().unwrap();
        let cache = FileCache::new(dir.path().to_path_buf()).unwrap();

        cache.set("key1", vec![1], Duration::from_secs(3600));
        cache.set("key2", vec![2], Duration::from_secs(3600));

        assert_eq!(cache.len(), 2);

        cache.clear();
        assert!(cache.is_empty());
    }

    #[test]
    fn test_file_cache_expiration() {
        let dir = tempdir().unwrap();
        let cache = FileCache::new(dir.path().to_path_buf()).unwrap();

        // Set with 0 TTL (immediately expired)
        cache.set("expired", vec![1, 2, 3], Duration::from_secs(0));

        // Should not return expired entry
        assert_eq!(cache.get("expired"), None);
    }

    #[test]
    fn test_file_cache_multi_file() {
        let dir = tempdir().unwrap();
        let cache = FileCache::multi_file(dir.path().to_path_buf()).unwrap();

        cache.set("key1", vec![1, 2, 3], Duration::from_secs(3600));
        cache.set("key2", vec![4, 5, 6], Duration::from_secs(3600));

        assert_eq!(cache.get("key1"), Some(vec![1, 2, 3]));
        assert_eq!(cache.get("key2"), Some(vec![4, 5, 6]));

        // Check that individual files were created
        let files: Vec<_> = fs::read_dir(dir.path())
            .unwrap()
            .filter_map(|e| e.ok())
            .filter(|e| e.file_name().to_string_lossy().ends_with(".cache"))
            .collect();

        assert_eq!(files.len(), 2);
    }

    #[test]
    fn test_file_cache_persistence() {
        let dir = tempdir().unwrap();
        let cache_path = dir.path().to_path_buf();

        // Create cache and add data
        {
            let cache = FileCache::new(cache_path.clone()).unwrap();
            cache.set("persistent", vec![1, 2, 3], Duration::from_secs(3600));
        }

        // Create new cache instance and verify data persists
        {
            let cache = FileCache::new(cache_path).unwrap();
            assert_eq!(cache.get("persistent"), Some(vec![1, 2, 3]));
        }
    }
}
