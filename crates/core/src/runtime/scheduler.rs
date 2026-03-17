use bot_api::schedule::{ScheduleId, ScheduledEntry, ScriptScheduler};
use std::collections::HashMap;
use std::sync::{Arc, Mutex};
use std::time::{Duration, Instant};
use uuid::Uuid;

struct ScheduleState {
    entry: ScheduledEntry,
    cancel_tx: tokio::sync::watch::Sender<bool>,
}

/// Script scheduler implementation using tokio timers.
/// Equivalent to Java's ScriptSchedulerImpl.
pub struct ScriptSchedulerImpl {
    schedules: Arc<Mutex<HashMap<String, ScheduleState>>>,
    /// Callback to start a script by name. Provided by the runtime.
    start_callback: Arc<dyn Fn(&str) + Send + Sync>,
}

impl ScriptSchedulerImpl {
    pub fn new(start_callback: Arc<dyn Fn(&str) + Send + Sync>) -> Self {
        Self {
            schedules: Arc::new(Mutex::new(HashMap::new())),
            start_callback,
        }
    }

    fn schedule_oneshot(
        &self,
        script_name: &str,
        delay: Duration,
    ) -> ScheduleId {
        let id = Uuid::new_v4().to_string();
        let schedule_id = ScheduleId(id.clone());
        let (cancel_tx, mut cancel_rx) = tokio::sync::watch::channel(false);

        let entry = ScheduledEntry {
            id: schedule_id.clone(),
            script_name: script_name.to_string(),
            next_run: Instant::now() + delay,
            interval: None,
            max_duration: None,
        };

        {
            let mut schedules = self.schedules.lock().unwrap();
            schedules.insert(id.clone(), ScheduleState { entry, cancel_tx });
        }

        let callback = self.start_callback.clone();
        let name = script_name.to_string();
        let schedules = self.schedules.clone();
        let id_clone = id.clone();

        tokio::spawn(async move {
            tokio::select! {
                _ = tokio::time::sleep(delay) => {
                    (callback)(&name);
                }
                _ = cancel_rx.changed() => {}
            }
            let mut s = schedules.lock().unwrap();
            s.remove(&id_clone);
        });

        schedule_id
    }

    fn schedule_recurring(
        &self,
        script_name: &str,
        interval: Duration,
        max_duration: Option<Duration>,
    ) -> ScheduleId {
        let id = Uuid::new_v4().to_string();
        let schedule_id = ScheduleId(id.clone());
        let (cancel_tx, mut cancel_rx) = tokio::sync::watch::channel(false);

        let entry = ScheduledEntry {
            id: schedule_id.clone(),
            script_name: script_name.to_string(),
            next_run: Instant::now() + interval,
            interval: Some(interval),
            max_duration,
        };

        {
            let mut schedules = self.schedules.lock().unwrap();
            schedules.insert(id.clone(), ScheduleState { entry, cancel_tx });
        }

        let callback = self.start_callback.clone();
        let name = script_name.to_string();
        let schedules = self.schedules.clone();
        let id_clone = id.clone();

        tokio::spawn(async move {
            let start = Instant::now();
            let mut timer = tokio::time::interval(interval);
            timer.tick().await; // skip first immediate tick

            loop {
                tokio::select! {
                    _ = timer.tick() => {
                        if let Some(max) = max_duration {
                            if start.elapsed() >= max {
                                break;
                            }
                        }
                        (callback)(&name);
                    }
                    _ = cancel_rx.changed() => {
                        break;
                    }
                }
            }
            let mut s = schedules.lock().unwrap();
            s.remove(&id_clone);
        });

        schedule_id
    }
}

impl ScriptScheduler for ScriptSchedulerImpl {
    fn run_after(&self, script_name: &str, delay: Duration) -> ScheduleId {
        self.schedule_oneshot(script_name, delay)
    }

    fn run_after_with_config(
        &self,
        script_name: &str,
        delay: Duration,
        _config: HashMap<String, serde_json::Value>,
    ) -> ScheduleId {
        // TODO: pass config to the start callback
        self.schedule_oneshot(script_name, delay)
    }

    fn run_at(&self, script_name: &str, at: Instant) -> ScheduleId {
        let now = Instant::now();
        let delay = if at > now {
            at - now
        } else {
            Duration::ZERO
        };
        self.schedule_oneshot(script_name, delay)
    }

    fn run_every(&self, script_name: &str, interval: Duration) -> ScheduleId {
        self.schedule_recurring(script_name, interval, None)
    }

    fn run_every_for(
        &self,
        script_name: &str,
        interval: Duration,
        max_duration: Duration,
    ) -> ScheduleId {
        self.schedule_recurring(script_name, interval, Some(max_duration))
    }

    fn cancel(&self, id: &ScheduleId) -> bool {
        let mut schedules = self.schedules.lock().unwrap();
        if let Some(state) = schedules.remove(&id.0) {
            let _ = state.cancel_tx.send(true);
            true
        } else {
            false
        }
    }

    fn cancel_all(&self) {
        let mut schedules = self.schedules.lock().unwrap();
        for (_, state) in schedules.drain() {
            let _ = state.cancel_tx.send(true);
        }
    }

    fn list_scheduled(&self) -> Vec<ScheduledEntry> {
        let schedules = self.schedules.lock().unwrap();
        schedules.values().map(|s| s.entry.clone()).collect()
    }
}
