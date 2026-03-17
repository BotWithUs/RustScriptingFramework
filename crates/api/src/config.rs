use serde::{Deserialize, Serialize};
use std::collections::HashMap;

/// The type and constraints of a configuration field.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum ConfigFieldType {
    Int {
        min: i64,
        max: i64,
        default: i64,
    },
    String {
        default: String,
    },
    Bool {
        default: bool,
    },
    Choice {
        options: Vec<String>,
        default: String,
    },
    ItemId {
        default: i64,
    },
}

/// A single configuration field definition.
/// Scripts declare these to specify what configuration they accept.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ConfigField {
    pub key: String,
    pub label: String,
    pub field_type: ConfigFieldType,
}

impl ConfigField {
    pub fn int(key: impl Into<String>, label: impl Into<String>, min: i64, max: i64, default: i64) -> Self {
        Self {
            key: key.into(),
            label: label.into(),
            field_type: ConfigFieldType::Int { min, max, default },
        }
    }

    pub fn string(key: impl Into<String>, label: impl Into<String>, default: impl Into<String>) -> Self {
        Self {
            key: key.into(),
            label: label.into(),
            field_type: ConfigFieldType::String { default: default.into() },
        }
    }

    pub fn bool(key: impl Into<String>, label: impl Into<String>, default: bool) -> Self {
        Self {
            key: key.into(),
            label: label.into(),
            field_type: ConfigFieldType::Bool { default },
        }
    }

    pub fn choice(key: impl Into<String>, label: impl Into<String>, options: Vec<String>, default: impl Into<String>) -> Self {
        Self {
            key: key.into(),
            label: label.into(),
            field_type: ConfigFieldType::Choice {
                options,
                default: default.into(),
            },
        }
    }

    pub fn item_id(key: impl Into<String>, label: impl Into<String>, default: i64) -> Self {
        Self {
            key: key.into(),
            label: label.into(),
            field_type: ConfigFieldType::ItemId { default },
        }
    }
}

/// Runtime configuration values for a script.
/// Backed by a string-keyed map of JSON values.
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct ScriptConfig {
    values: HashMap<String, serde_json::Value>,
}

impl ScriptConfig {
    pub fn new() -> Self {
        Self {
            values: HashMap::new(),
        }
    }

    pub fn from_map(values: HashMap<String, serde_json::Value>) -> Self {
        Self { values }
    }

    pub fn get_int(&self, key: &str, default: i64) -> i64 {
        self.values
            .get(key)
            .and_then(|v| v.as_i64())
            .unwrap_or(default)
    }

    pub fn get_string(&self, key: &str, default: &str) -> String {
        self.values
            .get(key)
            .and_then(|v| v.as_str())
            .map(|s| s.to_string())
            .unwrap_or_else(|| default.to_string())
    }

    pub fn get_bool(&self, key: &str, default: bool) -> bool {
        self.values
            .get(key)
            .and_then(|v| v.as_bool())
            .unwrap_or(default)
    }

    pub fn set(&mut self, key: impl Into<String>, value: serde_json::Value) {
        self.values.insert(key.into(), value);
    }

    pub fn as_map(&self) -> &HashMap<String, serde_json::Value> {
        &self.values
    }

    /// Apply defaults from config field definitions for any missing keys.
    pub fn apply_defaults(&mut self, fields: &[ConfigField]) {
        for field in fields {
            if !self.values.contains_key(&field.key) {
                let default_value = match &field.field_type {
                    ConfigFieldType::Int { default, .. } => serde_json::json!(default),
                    ConfigFieldType::String { default } => serde_json::json!(default),
                    ConfigFieldType::Bool { default } => serde_json::json!(default),
                    ConfigFieldType::Choice { default, .. } => serde_json::json!(default),
                    ConfigFieldType::ItemId { default } => serde_json::json!(default),
                };
                self.values.insert(field.key.clone(), default_value);
            }
        }
    }
}
