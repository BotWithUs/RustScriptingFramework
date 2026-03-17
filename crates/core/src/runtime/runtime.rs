use bot_api::client::ClientProvider;
use bot_api::events::EventBus;
use bot_api::game::GameApi;
use bot_api::message::MessageBus;
use bot_api::schedule::ScriptScheduler;
use bot_api::script::{BotScript, ScriptInfo};
use bot_api::state::SharedState;
use bot_api::context::ScriptContext;

use crate::config::ScriptConfigStore;
use crate::runtime::runner::ScriptRunner;

use std::collections::HashMap;
use std::sync::Arc;
use tracing;

/// Manages the collection of script runners and their lifecycle.
/// Equivalent to Java's ScriptRuntime.
pub struct ScriptRuntime {
    runners: HashMap<String, ScriptRunner>,
    /// Scripts that have been loaded but not yet started.
    scripts: HashMap<String, Box<dyn BotScript>>,
    config_store: Arc<ScriptConfigStore>,

    // Shared services that are passed to each script's context
    game: Arc<dyn GameApi>,
    events: Arc<dyn EventBus>,
    messages: Arc<dyn MessageBus>,
    state: Arc<dyn SharedState>,
    clients: Arc<dyn ClientProvider>,
    scheduler: Arc<dyn ScriptScheduler>,
}

impl ScriptRuntime {
    pub fn new(
        config_store: ScriptConfigStore,
        game: Arc<dyn GameApi>,
        events: Arc<dyn EventBus>,
        messages: Arc<dyn MessageBus>,
        state: Arc<dyn SharedState>,
        clients: Arc<dyn ClientProvider>,
        scheduler: Arc<dyn ScriptScheduler>,
    ) -> Self {
        Self {
            runners: HashMap::new(),
            scripts: HashMap::new(),
            config_store: Arc::new(config_store),
            game,
            events,
            messages,
            state,
            clients,
            scheduler,
        }
    }

    /// Register a script without starting it.
    pub fn register(&mut self, script: Box<dyn BotScript>) {
        let manifest = script.manifest();
        let name = manifest.name.clone();
        tracing::info!("Registering script '{}'", name);

        let runner = ScriptRunner::new(manifest);
        self.runners.insert(name.clone(), runner);
        self.scripts.insert(name, script);
    }

    /// Start a registered script by name.
    pub fn start(&mut self, name: &str) -> bool {
        let script = match self.scripts.remove(name) {
            Some(s) => s,
            None => {
                tracing::warn!("Script '{}' not found or already running", name);
                return false;
            }
        };

        let runner = match self.runners.get_mut(name) {
            Some(r) => r,
            None => {
                tracing::error!("No runner found for script '{}'", name);
                self.scripts.insert(name.to_string(), script);
                return false;
            }
        };

        let ctx = ScriptContext::new(
            self.game.clone(),
            self.events.clone(),
            self.messages.clone(),
            self.state.clone(),
            self.clients.clone(),
            self.scheduler.clone(),
        );

        runner.start(script, ctx, self.config_store.clone());
        true
    }

    /// Stop a running script by name.
    pub fn stop(&mut self, name: &str) -> bool {
        if let Some(runner) = self.runners.get_mut(name) {
            if runner.is_running() {
                runner.stop();
                return true;
            }
        }
        false
    }

    /// Stop all running scripts.
    pub fn stop_all(&mut self) {
        for runner in self.runners.values_mut() {
            if runner.is_running() {
                runner.stop();
            }
        }
    }

    /// Wait for all scripts to stop.
    pub async fn await_all_stopped(&mut self, timeout: std::time::Duration) {
        for runner in self.runners.values_mut() {
            runner.await_stop(timeout).await;
        }
    }

    /// List all registered scripts with their current state.
    pub fn list_all(&self) -> Vec<ScriptInfo> {
        self.runners
            .values()
            .map(|runner| {
                ScriptInfo::from_manifest(runner.manifest(), runner.is_running())
            })
            .collect()
    }

    /// List only running scripts.
    pub fn list_running(&self) -> Vec<ScriptInfo> {
        self.runners
            .values()
            .filter(|runner| runner.is_running())
            .map(|runner| ScriptInfo::from_manifest(runner.manifest(), true))
            .collect()
    }

    /// Count of running scripts (no allocation).
    pub fn running_count(&self) -> usize {
        self.runners.values().filter(|r| r.is_running()).count()
    }

    /// Count of all registered scripts (no allocation).
    pub fn total_count(&self) -> usize {
        self.runners.len()
    }

    /// Check if a script is running.
    pub fn is_running(&self, name: &str) -> bool {
        self.runners
            .get(name)
            .map(|r| r.is_running())
            .unwrap_or(false)
    }

    /// Get info about a specific script.
    pub fn get_info(&self, name: &str) -> Option<ScriptInfo> {
        self.runners.get(name).map(|runner| {
            ScriptInfo::from_manifest(runner.manifest(), runner.is_running())
        })
    }
}
