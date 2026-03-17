use std::collections::HashMap;

/// Thread-safe key-value store for sharing data between scripts.
/// Equivalent to Java's SharedState interface.
pub trait SharedState: Send + Sync {
    /// Store a value under the given key.
    fn put(&self, key: &str, value: serde_json::Value);

    /// Retrieve a value by key.
    fn get(&self, key: &str) -> Option<serde_json::Value>;

    /// Remove a value by key, returning it if it existed.
    fn remove(&self, key: &str) -> Option<serde_json::Value>;

    /// Check if a key exists.
    fn contains_key(&self, key: &str) -> bool;

    /// Get a snapshot of all key-value pairs.
    fn snapshot(&self) -> HashMap<String, serde_json::Value>;

    /// Remove all entries.
    fn clear(&self);
}
