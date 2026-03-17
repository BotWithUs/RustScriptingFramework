use crate::config::{ConfigField, ScriptConfig};
use crate::context::ScriptContext;
use crate::ui::ScriptUi;

/// Metadata describing a script, equivalent to Java's @ScriptManifest annotation.
#[derive(Debug, Clone)]
pub struct ScriptManifest {
    pub name: String,
    pub version: String,
    pub author: String,
    pub description: String,
}

/// What a script's loop iteration should do next.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum LoopAction {
    /// Sleep for the given number of milliseconds before the next loop iteration.
    Sleep(u64),
    /// Stop the script.
    Stop,
}

/// Core script trait that all bot scripts must implement.
/// Equivalent to Java's BotScript SPI interface.
///
/// Scripts are loaded as dynamic libraries and instantiated by the runtime.
/// The lifecycle is: `on_start` -> repeated `on_loop` -> `on_stop`.
pub trait BotScript: Send {
    /// Returns the script's metadata manifest.
    fn manifest(&self) -> ScriptManifest;

    /// Called once when the script starts. Receives the context providing
    /// access to game API, event bus, message bus, shared state, etc.
    fn on_start(&mut self, ctx: &ScriptContext) -> Result<(), Box<dyn std::error::Error>>;

    /// Called repeatedly in a loop. Returns a `LoopAction` indicating
    /// how long to sleep before the next iteration, or to stop.
    fn on_loop(&mut self) -> Result<LoopAction, Box<dyn std::error::Error>>;

    /// Called once when the script is stopping (either by request or error).
    fn on_stop(&mut self);

    /// Returns the configuration fields this script supports.
    /// The runtime uses these to build configuration UI and set defaults.
    fn config_fields(&self) -> Vec<ConfigField> {
        Vec::new()
    }

    /// Called when configuration is loaded or updated.
    fn on_config_update(&mut self, _config: &ScriptConfig) {}

    /// Returns an optional UI renderer for this script.
    fn get_ui(&self) -> Option<Box<dyn ScriptUi>> {
        None
    }
}

/// Information about a registered script (snapshot of its state).
/// Equivalent to Java's ScriptInfo record.
#[derive(Debug, Clone)]
pub struct ScriptInfo {
    pub name: String,
    pub version: String,
    pub author: String,
    pub description: String,
    pub running: bool,
    pub class_name: String,
}

impl ScriptInfo {
    pub fn from_manifest(manifest: &ScriptManifest, running: bool) -> Self {
        Self {
            name: manifest.name.clone(),
            version: manifest.version.clone(),
            author: manifest.author.clone(),
            description: manifest.description.clone(),
            running,
            class_name: manifest.name.clone(),
        }
    }
}

/// A task within a task-based script. Scripts can organize their logic
/// as a priority-sorted list of tasks that are checked each loop iteration.
/// Equivalent to Java's Task interface.
pub trait Task: Send {
    /// Display name for logging.
    fn name(&self) -> &str;

    /// Returns true if this task should execute this iteration.
    fn validate(&self) -> bool;

    /// Execute the task. Returns the delay in milliseconds before the next loop.
    fn execute(&mut self) -> Result<LoopAction, Box<dyn std::error::Error>>;

    /// Priority (higher = checked first). Default is 0.
    fn priority(&self) -> i32 {
        0
    }
}
