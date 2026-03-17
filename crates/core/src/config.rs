use bot_api::config::ScriptConfig;
use std::collections::HashMap;
use std::path::PathBuf;
use thiserror::Error;
use tracing;

#[derive(Error, Debug)]
pub enum ConfigError {
    #[error("IO error: {0}")]
    Io(#[from] std::io::Error),
    #[error("JSON error: {0}")]
    Json(#[from] serde_json::Error),
}

/// Handles loading and saving script configurations to disk as JSON files.
/// Equivalent to Java's ScriptConfigStore.
pub struct ScriptConfigStore {
    config_dir: PathBuf,
}

impl ScriptConfigStore {
    pub fn new(config_dir: impl Into<PathBuf>) -> Self {
        Self {
            config_dir: config_dir.into(),
        }
    }

    /// Creates a config store using the default directory (~/.botwithus/config/).
    pub fn default_location() -> Self {
        let home = dirs_next().unwrap_or_else(|| PathBuf::from("."));
        Self::new(home.join(".botwithus").join("config"))
    }

    /// Load configuration for a script by name.
    pub fn load(&self, script_name: &str) -> Result<ScriptConfig, ConfigError> {
        let path = self.config_path(script_name);
        if !path.exists() {
            return Ok(ScriptConfig::new());
        }

        let contents = std::fs::read_to_string(&path)?;
        let values: HashMap<String, serde_json::Value> = serde_json::from_str(&contents)?;
        Ok(ScriptConfig::from_map(values))
    }

    /// Save configuration for a script.
    pub fn save(&self, script_name: &str, config: &ScriptConfig) -> Result<(), ConfigError> {
        std::fs::create_dir_all(&self.config_dir)?;
        let path = self.config_path(script_name);
        let json = serde_json::to_string_pretty(config.as_map())?;
        std::fs::write(&path, json)?;
        tracing::debug!("Saved config for '{}' to {:?}", script_name, path);
        Ok(())
    }

    fn config_path(&self, script_name: &str) -> PathBuf {
        let safe_name = sanitize_filename(script_name);
        self.config_dir.join(format!("{}.json", safe_name))
    }
}

fn sanitize_filename(name: &str) -> String {
    name.chars()
        .map(|c| if c.is_alphanumeric() || c == '-' || c == '_' { c } else { '_' })
        .collect()
}

fn dirs_next() -> Option<PathBuf> {
    std::env::var_os("USERPROFILE")
        .or_else(|| std::env::var_os("HOME"))
        .map(PathBuf::from)
}
