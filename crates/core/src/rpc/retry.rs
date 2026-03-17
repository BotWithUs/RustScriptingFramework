use std::time::Duration;

/// Retry policy with exponential backoff for RPC calls.
/// Equivalent to Java's RetryPolicy.
#[derive(Debug, Clone)]
pub struct RetryPolicy {
    pub max_retries: u32,
    pub initial_delay: Duration,
    pub backoff_multiplier: f64,
    pub max_delay: Duration,
}

impl RetryPolicy {
    pub const NONE: RetryPolicy = RetryPolicy {
        max_retries: 0,
        initial_delay: Duration::ZERO,
        backoff_multiplier: 1.0,
        max_delay: Duration::ZERO,
    };

    pub fn default_policy() -> Self {
        Self {
            max_retries: 3,
            initial_delay: Duration::from_millis(100),
            backoff_multiplier: 2.0,
            max_delay: Duration::from_secs(5),
        }
    }

    pub fn new(max_retries: u32, initial_delay: Duration, backoff_multiplier: f64, max_delay: Duration) -> Self {
        Self {
            max_retries,
            initial_delay,
            backoff_multiplier,
            max_delay,
        }
    }

    /// Calculate the delay for a given attempt (0-indexed).
    pub fn delay_for_attempt(&self, attempt: u32) -> Duration {
        if attempt == 0 {
            return self.initial_delay;
        }
        let delay_ms = self.initial_delay.as_millis() as f64
            * self.backoff_multiplier.powi(attempt as i32);
        let clamped = Duration::from_millis(delay_ms as u64).min(self.max_delay);
        clamped
    }
}
