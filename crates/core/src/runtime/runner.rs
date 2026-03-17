use bot_api::config::ScriptConfig;
use bot_api::context::ScriptContext;
use bot_api::script::{BotScript, LoopAction, ScriptManifest};

use crate::config::ScriptConfigStore;
use crate::runtime::profiler::LoopProfiler;

use std::sync::Arc;
use tokio::sync::watch;
use tokio::task::JoinHandle;
use tracing;

/// The running state of a script.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum RunnerState {
    Registered,
    Starting,
    Running,
    Stopping,
    Stopped,
    Errored,
}

/// Manages the execution of a single script on its own async task.
/// Equivalent to Java's ScriptRunner.
pub struct ScriptRunner {
    manifest: ScriptManifest,
    state: RunnerState,
    cancel_tx: watch::Sender<bool>,
    cancel_rx: watch::Receiver<bool>,
    handle: Option<JoinHandle<()>>,
}

impl ScriptRunner {
    pub fn new(manifest: ScriptManifest) -> Self {
        let (cancel_tx, cancel_rx) = watch::channel(false);
        Self {
            manifest,
            state: RunnerState::Registered,
            cancel_tx,
            cancel_rx,
            handle: None,
        }
    }

    pub fn manifest(&self) -> &ScriptManifest {
        &self.manifest
    }

    pub fn state(&self) -> RunnerState {
        self.state
    }

    pub fn is_running(&self) -> bool {
        self.state == RunnerState::Running || self.state == RunnerState::Starting
    }

    /// Start executing the script on a background task.
    pub fn start(
        &mut self,
        mut script: Box<dyn BotScript>,
        ctx: ScriptContext,
        config_store: Arc<ScriptConfigStore>,
    ) {
        self.state = RunnerState::Starting;
        let script_name = self.manifest.name.clone();
        let mut cancel_rx = self.cancel_rx.clone();

        let handle = tokio::task::spawn(async move {
            let mut profiler = LoopProfiler::new(1000);

            // Load config
            let mut config = config_store
                .load(&script_name)
                .unwrap_or_else(|e| {
                    tracing::warn!("Failed to load config for '{}': {}", script_name, e);
                    ScriptConfig::new()
                });
            config.apply_defaults(&script.config_fields());
            script.on_config_update(&config);

            // onStart phase
            tracing::info!("Starting script '{}'", script_name);
            {
                let start_result = script.on_start(&ctx);
                if let Err(e) = start_result {
                    tracing::error!("Script '{}' onStart failed: {}", script_name, e);
                    return;
                }
            }

            // Main loop
            tracing::info!("Script '{}' entering main loop", script_name);
            loop {
                // Check cancellation
                if *cancel_rx.borrow() {
                    tracing::info!("Script '{}' cancelled", script_name);
                    break;
                }

                let loop_start = std::time::Instant::now();

                // Run onLoop - extract the result immediately so the non-Send
                // error type doesn't live across an await point.
                let action = match script.on_loop() {
                    Ok(action) => action,
                    Err(e) => {
                        tracing::error!("Script '{}' onLoop error: {}", script_name, e);
                        break;
                    }
                };

                let elapsed = loop_start.elapsed();
                profiler.record(elapsed);

                match action {
                    LoopAction::Sleep(ms) => {
                        tokio::select! {
                            _ = tokio::time::sleep(std::time::Duration::from_millis(ms)) => {}
                            _ = cancel_rx.changed() => {
                                tracing::info!("Script '{}' cancelled during sleep", script_name);
                                break;
                            }
                        }
                    }
                    LoopAction::Stop => {
                        tracing::info!("Script '{}' requested stop", script_name);
                        break;
                    }
                }
            }

            // onStop phase
            tracing::info!("Stopping script '{}'", script_name);
            script.on_stop();

            if let Some(avg) = profiler.avg() {
                tracing::info!(
                    "Script '{}' profiler: avg={:?}, min={:?}, max={:?}, iterations={}",
                    script_name,
                    avg,
                    profiler.min().unwrap(),
                    profiler.max().unwrap(),
                    profiler.count()
                );
            }
        });

        self.handle = Some(handle);
        self.state = RunnerState::Running;
    }

    /// Signal the script to stop.
    pub fn stop(&mut self) {
        if self.is_running() {
            self.state = RunnerState::Stopping;
            let _ = self.cancel_tx.send(true);
        }
    }

    /// Wait for the script task to complete with a timeout.
    pub async fn await_stop(&mut self, timeout: std::time::Duration) -> bool {
        if let Some(handle) = self.handle.take() {
            match tokio::time::timeout(timeout, handle).await {
                Ok(Ok(())) => {
                    self.state = RunnerState::Stopped;
                    true
                }
                Ok(Err(e)) => {
                    tracing::error!("Script '{}' task panicked: {}", self.manifest.name, e);
                    self.state = RunnerState::Errored;
                    false
                }
                Err(_) => {
                    tracing::warn!("Script '{}' did not stop within timeout", self.manifest.name);
                    false
                }
            }
        } else {
            self.state = RunnerState::Stopped;
            true
        }
    }
}
