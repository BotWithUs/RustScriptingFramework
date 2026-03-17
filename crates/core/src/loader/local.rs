use crate::loader::plugin::{LoadedPlugin, PluginError};
use bot_api::script::BotScript;
use std::path::{Path, PathBuf};
use thiserror::Error;
use tracing;

#[derive(Error, Debug)]
pub enum LoaderError {
    #[error("Scripts directory not found: {0}")]
    DirNotFound(PathBuf),
    #[error("IO error: {0}")]
    Io(#[from] std::io::Error),
    #[error("Plugin error: {0}")]
    Plugin(#[from] PluginError),
}

/// Discovers and loads script plugins from dynamic library files in a directory.
/// Equivalent to Java's LocalScriptLoader.
pub struct LocalScriptLoader {
    scripts_dir: PathBuf,
    /// Loaded plugin libraries. Must be kept alive as long as their scripts exist.
    loaded_plugins: Vec<LoadedPlugin>,
}

impl LocalScriptLoader {
    pub fn new(scripts_dir: impl Into<PathBuf>) -> Self {
        Self {
            scripts_dir: scripts_dir.into(),
            loaded_plugins: Vec::new(),
        }
    }

    /// Returns the appropriate file extension for dynamic libraries on the current platform.
    fn lib_extension() -> &'static str {
        if cfg!(target_os = "windows") {
            "dll"
        } else if cfg!(target_os = "macos") {
            "dylib"
        } else {
            "so"
        }
    }

    /// Scan the scripts directory and load all script plugins.
    /// Returns the created script instances.
    ///
    /// # Safety
    /// Loading dynamic libraries is inherently unsafe. Scripts must be compiled
    /// with a compatible Rust toolchain and API crate version.
    pub unsafe fn load_scripts(&mut self) -> Result<Vec<Box<dyn BotScript>>, LoaderError> {
        if !self.scripts_dir.exists() {
            return Err(LoaderError::DirNotFound(self.scripts_dir.clone()));
        }

        let ext = Self::lib_extension();
        let mut scripts = Vec::new();

        let entries = std::fs::read_dir(&self.scripts_dir)?;
        for entry in entries {
            let entry = entry?;
            let path = entry.path();

            if path.extension().and_then(|e| e.to_str()) != Some(ext) {
                continue;
            }

            let file_name = path
                .file_name()
                .unwrap_or_default()
                .to_string_lossy()
                .to_string();

            tracing::info!("Loading script plugin: {}", file_name);

            match self.load_plugin(&path, &file_name) {
                Ok(script) => {
                    let manifest = script.manifest();
                    tracing::info!(
                        "Loaded script '{}' v{} by {}",
                        manifest.name,
                        manifest.version,
                        manifest.author
                    );
                    scripts.push(script);
                }
                Err(e) => {
                    tracing::error!("Failed to load plugin '{}': {}", file_name, e);
                }
            }
        }

        tracing::info!("Loaded {} script(s) from {:?}", scripts.len(), self.scripts_dir);
        Ok(scripts)
    }

    /// Load a single plugin from a library file.
    unsafe fn load_plugin(
        &mut self,
        path: &Path,
        file_name: &str,
    ) -> Result<Box<dyn BotScript>, LoaderError> {
        let library = libloading::Library::new(path)
            .map_err(|e| PluginError::LoadFailed(e.to_string()))?;

        let plugin = LoadedPlugin {
            library,
            file_name: file_name.to_string(),
        };

        let script = plugin.create_script()?;
        self.loaded_plugins.push(plugin);
        Ok(script)
    }

    /// Unload all plugins and clear the loaded list.
    pub fn unload_all(&mut self) {
        tracing::info!("Unloading {} plugin(s)", self.loaded_plugins.len());
        self.loaded_plugins.clear();
    }

    /// Reload scripts from the directory. Unloads existing plugins first.
    ///
    /// # Safety
    /// Same safety requirements as `load_scripts`.
    pub unsafe fn reload(&mut self) -> Result<Vec<Box<dyn BotScript>>, LoaderError> {
        self.unload_all();
        self.load_scripts()
    }
}
