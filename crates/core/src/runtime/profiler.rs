use std::time::Duration;

/// Tracks loop execution timing for performance profiling.
/// Equivalent to Java's ScriptProfiler.
pub struct LoopProfiler {
    durations: Vec<Duration>,
    max_samples: usize,
}

impl LoopProfiler {
    pub fn new(max_samples: usize) -> Self {
        Self {
            durations: Vec::with_capacity(max_samples),
            max_samples,
        }
    }

    /// Record a loop iteration duration.
    pub fn record(&mut self, duration: Duration) {
        if self.durations.len() >= self.max_samples {
            self.durations.remove(0);
        }
        self.durations.push(duration);
    }

    /// Minimum recorded loop duration.
    pub fn min(&self) -> Option<Duration> {
        self.durations.iter().min().copied()
    }

    /// Maximum recorded loop duration.
    pub fn max(&self) -> Option<Duration> {
        self.durations.iter().max().copied()
    }

    /// Average loop duration.
    pub fn avg(&self) -> Option<Duration> {
        if self.durations.is_empty() {
            return None;
        }
        let total: Duration = self.durations.iter().sum();
        Some(total / self.durations.len() as u32)
    }

    /// Number of recorded samples.
    pub fn count(&self) -> usize {
        self.durations.len()
    }

    /// Reset all recorded data.
    pub fn reset(&mut self) {
        self.durations.clear();
    }
}
