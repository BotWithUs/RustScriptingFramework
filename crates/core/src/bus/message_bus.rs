use bot_api::message::{MessageBus, MessageSubscriptionId, ScriptMessage};
use std::collections::HashMap;
use std::sync::atomic::{AtomicU64, Ordering};
use std::sync::{Arc, Mutex, RwLock};
use std::time::Duration;
use uuid::Uuid;

type Handler = Box<dyn Fn(&ScriptMessage) + Send + Sync>;

struct ChannelSubscription {
    id: MessageSubscriptionId,
    handler: Handler,
}

/// Message bus implementation supporting fire-and-forget and request-response patterns.
pub struct MessageBusImpl {
    subscribers: RwLock<HashMap<String, Vec<ChannelSubscription>>>,
    pending_requests: Arc<Mutex<HashMap<String, std::sync::mpsc::Sender<ScriptMessage>>>>,
    next_id: AtomicU64,
}

impl MessageBusImpl {
    pub fn new() -> Self {
        Self {
            subscribers: RwLock::new(HashMap::new()),
            pending_requests: Arc::new(Mutex::new(HashMap::new())),
            next_id: AtomicU64::new(1),
        }
    }
}

impl Default for MessageBusImpl {
    fn default() -> Self {
        Self::new()
    }
}

impl MessageBus for MessageBusImpl {
    fn subscribe(
        &self,
        channel: &str,
        handler: Box<dyn Fn(&ScriptMessage) + Send + Sync>,
    ) -> MessageSubscriptionId {
        let id = MessageSubscriptionId(self.next_id.fetch_add(1, Ordering::Relaxed));
        let sub = ChannelSubscription { id, handler };

        let mut subs = self.subscribers.write().unwrap();
        subs.entry(channel.to_string()).or_default().push(sub);
        id
    }

    fn unsubscribe(&self, id: MessageSubscriptionId) {
        let mut subs = self.subscribers.write().unwrap();
        for listeners in subs.values_mut() {
            listeners.retain(|s| s.id != id);
        }
    }

    fn publish(&self, channel: &str, sender: &str, payload: serde_json::Value) {
        let msg = ScriptMessage {
            sender: sender.to_string(),
            channel: channel.to_string(),
            payload,
            request_id: None,
        };

        let subs = self.subscribers.read().unwrap();
        if let Some(listeners) = subs.get(channel) {
            for listener in listeners {
                (listener.handler)(&msg);
            }
        }
    }

    fn request(
        &self,
        channel: &str,
        sender: &str,
        payload: serde_json::Value,
        timeout: Duration,
    ) -> Option<ScriptMessage> {
        let request_id = Uuid::new_v4().to_string();
        let (tx, rx) = std::sync::mpsc::channel();

        // Register pending request
        {
            let mut pending = self.pending_requests.lock().unwrap();
            pending.insert(request_id.clone(), tx);
        }

        // Publish the request message
        let msg = ScriptMessage {
            sender: sender.to_string(),
            channel: channel.to_string(),
            payload,
            request_id: Some(request_id.clone()),
        };

        let subs = self.subscribers.read().unwrap();
        if let Some(listeners) = subs.get(channel) {
            for listener in listeners {
                (listener.handler)(&msg);
            }
        }
        drop(subs);

        // Wait for response
        let result = rx.recv_timeout(timeout).ok();

        // Clean up pending request
        let mut pending = self.pending_requests.lock().unwrap();
        pending.remove(&request_id);

        result
    }

    fn respond(
        &self,
        request_id: &str,
        channel: &str,
        sender: &str,
        payload: serde_json::Value,
    ) {
        let pending = self.pending_requests.lock().unwrap();
        if let Some(tx) = pending.get(request_id) {
            let msg = ScriptMessage {
                sender: sender.to_string(),
                channel: channel.to_string(),
                payload,
                request_id: Some(request_id.to_string()),
            };
            let _ = tx.send(msg);
        }
    }
}
