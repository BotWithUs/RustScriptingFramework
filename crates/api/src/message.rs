use serde::{Deserialize, Serialize};
use std::time::Duration;

/// A message sent between scripts via the message bus.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ScriptMessage {
    pub sender: String,
    pub channel: String,
    pub payload: serde_json::Value,
    pub request_id: Option<String>,
}

/// A unique identifier for a message bus subscription.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct MessageSubscriptionId(pub u64);

/// Inter-script communication bus with named channels.
/// Supports fire-and-forget publishing and request-response patterns.
/// Equivalent to Java's MessageBus interface.
pub trait MessageBus: Send + Sync {
    /// Subscribe to messages on a named channel.
    fn subscribe(
        &self,
        channel: &str,
        handler: Box<dyn Fn(&ScriptMessage) + Send + Sync>,
    ) -> MessageSubscriptionId;

    /// Unsubscribe from a channel.
    fn unsubscribe(&self, id: MessageSubscriptionId);

    /// Publish a fire-and-forget message to a channel.
    fn publish(&self, channel: &str, sender: &str, payload: serde_json::Value);

    /// Send a request and wait for a response with a timeout.
    /// Returns None if no response is received within the timeout.
    fn request(
        &self,
        channel: &str,
        sender: &str,
        payload: serde_json::Value,
        timeout: Duration,
    ) -> Option<ScriptMessage>;

    /// Respond to a request identified by its request ID.
    fn respond(
        &self,
        request_id: &str,
        channel: &str,
        sender: &str,
        payload: serde_json::Value,
    );
}
