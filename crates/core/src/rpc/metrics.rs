use std::collections::HashMap;
use std::sync::Mutex;
use std::time::Duration;

/// Per-method RPC call statistics.
#[derive(Debug, Clone)]
pub struct MethodStats {
    pub call_count: u64,
    pub total_time: Duration,
    pub error_count: u64,
}

impl MethodStats {
    pub fn avg_latency(&self) -> Duration {
        if self.call_count == 0 {
            Duration::ZERO
        } else {
            self.total_time / self.call_count as u32
        }
    }
}

/// Tracks per-method RPC call metrics.
/// Equivalent to Java's RpcMetrics.
pub struct RpcMetrics {
    stats: Mutex<HashMap<String, MethodStats>>,
}

impl RpcMetrics {
    pub fn new() -> Self {
        Self {
            stats: Mutex::new(HashMap::new()),
        }
    }

    /// Record a completed RPC call.
    pub fn record_call(&self, method: &str, duration: Duration, is_error: bool) {
        let mut stats = self.stats.lock().unwrap();
        let entry = stats.entry(method.to_string()).or_insert_with(|| MethodStats {
            call_count: 0,
            total_time: Duration::ZERO,
            error_count: 0,
        });
        entry.call_count += 1;
        entry.total_time += duration;
        if is_error {
            entry.error_count += 1;
        }
    }

    /// Get a snapshot of all method statistics.
    pub fn snapshot(&self) -> HashMap<String, MethodStats> {
        self.stats.lock().unwrap().clone()
    }

    /// Reset all metrics.
    pub fn reset(&self) {
        self.stats.lock().unwrap().clear();
    }
}

impl Default for RpcMetrics {
    fn default() -> Self {
        Self::new()
    }
}
