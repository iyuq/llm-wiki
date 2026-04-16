use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};
use std::collections::HashMap;
use std::path::Path;

/// A single cache entry tracking a source file's ingest state.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CacheEntry {
    /// SHA256 hash of source content at time of ingest.
    pub hash: String,
    /// Unix timestamp when ingested.
    pub timestamp: u64,
    /// Wiki files written during this ingest.
    pub files_written: Vec<String>,
}

/// SHA256-based ingest cache with JSON file persistence.
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct IngestCache {
    pub entries: HashMap<String, CacheEntry>,
}

impl IngestCache {
    /// Load cache from JSON file, or return empty cache.
    pub fn load(state_dir: &Path) -> crate::Result<Self> {
        let cache_path = state_dir.join("ingest-cache.json");
        if !cache_path.exists() {
            return Ok(Self::default());
        }
        let content = std::fs::read_to_string(&cache_path)?;
        let cache: Self = serde_json::from_str(&content)?;
        Ok(cache)
    }

    /// Save cache to JSON file.
    pub fn save(&self, state_dir: &Path) -> crate::Result<()> {
        std::fs::create_dir_all(state_dir)?;
        let cache_path = state_dir.join("ingest-cache.json");
        let content = serde_json::to_string_pretty(self)?;
        std::fs::write(cache_path, content)?;
        Ok(())
    }

    /// Compute SHA256 hash of a file.
    pub fn compute_hash(path: &Path) -> crate::Result<String> {
        let bytes = std::fs::read(path)?;
        let mut hasher = Sha256::new();
        hasher.update(&bytes);
        let result = hasher.finalize();
        Ok(format!("{:x}", result))
    }

    /// Check if a source file is cached and unchanged.
    /// Returns Some(entry) if cached with same hash, None otherwise.
    pub fn lookup(&self, source_path: &str, current_hash: &str) -> Option<&CacheEntry> {
        self.entries.get(source_path).and_then(|entry| {
            if entry.hash == current_hash {
                Some(entry)
            } else {
                None
            }
        })
    }

    /// Get a cache entry by path regardless of hash.
    pub fn get(&self, source_path: &str) -> Option<&CacheEntry> {
        self.entries.get(source_path)
    }

    /// Insert or update a cache entry.
    pub fn insert(
        &mut self,
        source_path: String,
        hash: String,
        files_written: Vec<String>,
    ) {
        let timestamp = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap_or_default()
            .as_secs();
        self.entries.insert(
            source_path,
            CacheEntry {
                hash,
                timestamp,
                files_written,
            },
        );
    }

    /// Remove a cache entry.
    pub fn remove(&mut self, source_path: &str) -> bool {
        self.entries.remove(source_path).is_some()
    }

    /// Clear all cache entries.
    pub fn clear_all(&mut self) {
        self.entries.clear();
    }
}
