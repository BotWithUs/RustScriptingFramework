use crate::rpc::codec::{CodecError, MessagePackCodec};
use crate::rpc::metrics::RpcMetrics;
use crate::rpc::pipe::{PipeClient, PipeError};
use crate::rpc::retry::RetryPolicy;
use std::collections::HashMap;
use std::sync::atomic::{AtomicBool, AtomicU64, Ordering};
use std::sync::{Arc, Mutex};
use std::time::{Duration, Instant};
use thiserror::Error;
use tracing;

#[derive(Error, Debug)]
pub enum RpcError {
    #[error("RPC error from server: {0}")]
    ServerError(String),
    #[error("RPC timeout: method '{method}' timed out after {timeout:?}")]
    Timeout {
        method: String,
        timeout: Duration,
    },
    #[error("Pipe error: {0}")]
    Pipe(#[from] PipeError),
    #[error("Codec error: {0}")]
    Codec(#[from] CodecError),
    #[error("Not connected")]
    NotConnected,
    #[error("Client closed")]
    Closed,
}

type EventHandler = Arc<dyn Fn(HashMap<String, rmpv::Value>) + Send + Sync>;

/// High-level RPC client with request/response correlation, event interleaving,
/// retry policies, timeout handling, and background reader thread.
/// Equivalent to Java's RpcClient.
///
/// Threading model:
/// - Background reader thread polls for incoming messages when no RPC call is active
/// - RPC caller acquires exclusive lock, sends request, reads responses until match
/// - Events received during RPC calls are dispatched asynchronously
pub struct RpcClient {
    pipe: Arc<Mutex<PipeClient>>,
    id_counter: AtomicU64,
    running: Arc<AtomicBool>,
    timeout: Duration,
    retry_policy: RetryPolicy,
    metrics: Arc<RpcMetrics>,
    event_handler: Arc<Mutex<Option<EventHandler>>>,

    // The pipe_lock ensures only one thread accesses the pipe at a time.
    pipe_lock: Arc<Mutex<()>>,
}

impl RpcClient {
    pub fn new(pipe: PipeClient) -> Self {
        Self {
            pipe: Arc::new(Mutex::new(pipe)),
            id_counter: AtomicU64::new(1),
            running: Arc::new(AtomicBool::new(false)),
            timeout: Duration::from_secs(10),
            retry_policy: RetryPolicy::NONE,
            metrics: Arc::new(RpcMetrics::new()),
            event_handler: Arc::new(Mutex::new(None)),
            pipe_lock: Arc::new(Mutex::new(())),
        }
    }

    pub fn set_timeout(&mut self, timeout: Duration) {
        self.timeout = timeout;
    }

    pub fn set_retry_policy(&mut self, policy: RetryPolicy) {
        self.retry_policy = policy;
    }

    pub fn set_event_handler<F>(&self, handler: F)
    where
        F: Fn(HashMap<String, rmpv::Value>) + Send + Sync + 'static,
    {
        let mut h = self.event_handler.lock().unwrap();
        *h = Some(Arc::new(handler));
    }

    pub fn metrics(&self) -> &RpcMetrics {
        &self.metrics
    }

    /// Mark the client as active.
    pub fn start(&self) {
        self.running.store(true, Ordering::SeqCst);
    }

    /// Stop the client and close the pipe.
    pub fn close(&self) {
        self.running.store(false, Ordering::SeqCst);
        let mut pipe = self.pipe.lock().unwrap();
        pipe.close();
    }

    /// Make a synchronous RPC call and return the result as a map.
    /// Handles request/response correlation, event interleaving, retry, and timeout.
    pub fn call_sync(
        &self,
        method: &str,
        params: HashMap<String, rmpv::Value>,
    ) -> Result<HashMap<String, rmpv::Value>, RpcError> {
        let result_value = self.call_sync_raw(method, params)?;

        // Convert result to map
        match result_value {
            rmpv::Value::Map(pairs) => {
                let mut map = HashMap::new();
                for (k, v) in pairs {
                    let key = match k {
                        rmpv::Value::String(s) => s.into_str().unwrap_or_default().to_string(),
                        _ => k.to_string(),
                    };
                    map.insert(key, v);
                }
                Ok(map)
            }
            rmpv::Value::Nil => Ok(HashMap::new()),
            _ => {
                // Wrap non-map result
                let mut map = HashMap::new();
                map.insert("result".to_string(), result_value);
                Ok(map)
            }
        }
    }

    /// Make a synchronous RPC call and return the raw result Value.
    pub fn call_sync_raw(
        &self,
        method: &str,
        params: HashMap<String, rmpv::Value>,
    ) -> Result<rmpv::Value, RpcError> {
        let mut last_error = None;
        let max_attempts = self.retry_policy.max_retries + 1;

        for attempt in 0..max_attempts {
            if attempt > 0 {
                let delay = self.retry_policy.delay_for_attempt(attempt - 1);
                tracing::debug!(
                    "Retrying RPC '{}' (attempt {}/{}), delay {:?}",
                    method, attempt + 1, max_attempts, delay
                );
                std::thread::sleep(delay);
            }

            let start = Instant::now();
            match self.do_call(method, &params) {
                Ok(value) => {
                    self.metrics.record_call(method, start.elapsed(), false);
                    return Ok(value);
                }
                Err(RpcError::Timeout { .. }) => {
                    // Don't retry timeouts
                    self.metrics.record_call(method, start.elapsed(), true);
                    return Err(RpcError::Timeout {
                        method: method.to_string(),
                        timeout: self.timeout,
                    });
                }
                Err(e) => {
                    self.metrics.record_call(method, start.elapsed(), true);
                    last_error = Some(e);
                }
            }
        }

        Err(last_error.unwrap_or(RpcError::NotConnected))
    }

    /// Make a synchronous RPC call and return the result as a list.
    pub fn call_sync_list(
        &self,
        method: &str,
        params: HashMap<String, rmpv::Value>,
    ) -> Result<Vec<rmpv::Value>, RpcError> {
        let result = self.call_sync_raw(method, params)?;
        match result {
            rmpv::Value::Array(arr) => Ok(arr),
            rmpv::Value::Nil => Ok(Vec::new()),
            other => Ok(vec![other]),
        }
    }

    /// Internal: perform a single RPC call with timeout and event dispatch.
    fn do_call(
        &self,
        method: &str,
        params: &HashMap<String, rmpv::Value>,
    ) -> Result<rmpv::Value, RpcError> {
        let id = self.id_counter.fetch_add(1, Ordering::Relaxed);
        let deadline = Instant::now() + self.timeout;

        // Build request message
        let mut request = HashMap::new();
        request.insert("method".to_string(), rmpv::Value::String(method.into()));
        request.insert("id".to_string(), rmpv::Value::Integer(id.into()));
        if !params.is_empty() {
            let params_value = rmpv::Value::Map(
                params
                    .iter()
                    .map(|(k, v)| (rmpv::Value::String(k.clone().into()), v.clone()))
                    .collect(),
            );
            request.insert("params".to_string(), params_value);
        }

        let encoded = MessagePackCodec::encode(&request)?;

        // Acquire exclusive pipe access
        let _pipe_lock = self.pipe_lock.lock().unwrap();
        let mut pipe = self.pipe.lock().unwrap();

        // Send request
        pipe.send(&encoded)?;

        // Read responses until we get one with matching ID
        loop {
            if Instant::now() >= deadline {
                return Err(RpcError::Timeout {
                    method: method.to_string(),
                    timeout: self.timeout,
                });
            }

            let msg_data = pipe.read_message()?;
            let msg = MessagePackCodec::decode(&msg_data)?;

            // Check if this is an event (has "event" field)
            if msg.contains_key("event") {
                // Dispatch event asynchronously
                let handler = self.event_handler.lock().unwrap().clone();
                if let Some(h) = handler {
                    let event_msg = msg;
                    std::thread::spawn(move || {
                        h(event_msg);
                    });
                }
                continue;
            }

            // Check if this response matches our request ID
            let response_id = msg
                .get("id")
                .and_then(|v| v.as_u64())
                .unwrap_or(0);

            if response_id == id {
                // Check for error
                if let Some(error) = msg.get("error") {
                    if !error.is_nil() {
                        let err_msg = error
                            .as_str()
                            .unwrap_or("Unknown RPC error")
                            .to_string();
                        return Err(RpcError::ServerError(err_msg));
                    }
                }

                // Return result
                return Ok(msg
                    .get("result")
                    .cloned()
                    .unwrap_or(rmpv::Value::Nil));
            }

            // Response for a different request ID - log and continue
            tracing::warn!(
                "Received response for unexpected ID {} (expected {})",
                response_id, id
            );
        }
    }
}

impl Drop for RpcClient {
    fn drop(&mut self) {
        self.close();
    }
}
