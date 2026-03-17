use std::time::{Duration, Instant};

/// Unique identifier for a scheduled task.
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct ScheduleId(pub String);

/// Information about a scheduled script execution.
#[derive(Debug, Clone)]
pub struct ScheduledEntry {
    pub id: ScheduleId,
    pub script_name: String,
    pub next_run: Instant,
    /// None for one-shot schedules.
    pub interval: Option<Duration>,
    /// Maximum duration to run the script. None for unlimited.
    pub max_duration: Option<Duration>,
}

/// Script scheduler for delayed and recurring script execution.
/// Equivalent to Java's ScriptScheduler interface.
pub trait ScriptScheduler: Send + Sync {
    /// Schedule a script to run after a delay.
    fn run_after(&self, script_name: &str, delay: Duration) -> ScheduleId;

    /// Schedule a script to run after a delay with configuration.
    fn run_after_with_config(
        &self,
        script_name: &str,
        delay: Duration,
        config: std::collections::HashMap<String, serde_json::Value>,
    ) -> ScheduleId;

    /// Schedule a script to run at a specific instant.
    fn run_at(&self, script_name: &str, at: Instant) -> ScheduleId;

    /// Schedule a script to run at a recurring interval.
    fn run_every(&self, script_name: &str, interval: Duration) -> ScheduleId;

    /// Schedule a recurring script with a maximum total duration.
    fn run_every_for(
        &self,
        script_name: &str,
        interval: Duration,
        max_duration: Duration,
    ) -> ScheduleId;

    /// Cancel a scheduled execution.
    fn cancel(&self, id: &ScheduleId) -> bool;

    /// Cancel all scheduled executions.
    fn cancel_all(&self);

    /// List all pending scheduled entries.
    fn list_scheduled(&self) -> Vec<ScheduledEntry>;
}
