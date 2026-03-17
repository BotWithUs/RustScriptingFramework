use bot_api::state::SharedState;
use dashmap::DashMap;
use std::collections::HashMap;

/// Thread-safe shared state implementation backed by DashMap.
pub struct SharedStateImpl {
    map: DashMap<String, serde_json::Value>,
}

impl SharedStateImpl {
    pub fn new() -> Self {
        Self {
            map: DashMap::new(),
        }
    }
}

impl Default for SharedStateImpl {
    fn default() -> Self {
        Self::new()
    }
}

impl SharedState for SharedStateImpl {
    fn put(&self, key: &str, value: serde_json::Value) {
        self.map.insert(key.to_string(), value);
    }

    fn get(&self, key: &str) -> Option<serde_json::Value> {
        self.map.get(key).map(|v| v.value().clone())
    }

    fn remove(&self, key: &str) -> Option<serde_json::Value> {
        self.map.remove(key).map(|(_, v)| v)
    }

    fn contains_key(&self, key: &str) -> bool {
        self.map.contains_key(key)
    }

    fn snapshot(&self) -> HashMap<String, serde_json::Value> {
        self.map
            .iter()
            .map(|entry| (entry.key().clone(), entry.value().clone()))
            .collect()
    }

    fn clear(&self) {
        self.map.clear();
    }
}
