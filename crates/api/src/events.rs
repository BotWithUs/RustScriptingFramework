use std::any::Any;
use std::fmt;

/// Trait for all game events that can be published through the event bus.
pub trait GameEvent: Any + Send + Sync + fmt::Debug {
    /// Returns the event type identifier.
    fn event_type(&self) -> &'static str;

    /// Upcast to Any for type-erased dispatch.
    fn as_any(&self) -> &dyn Any;
}

/// A unique identifier for an event subscription.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct SubscriptionId(pub u64);

/// Event bus for publishing and subscribing to game events.
/// Equivalent to Java's EventBus interface.
pub trait EventBus: Send + Sync {
    /// Subscribe to events of a specific type. Returns a subscription ID
    /// that can be used to unsubscribe later.
    fn subscribe(
        &self,
        event_type: &'static str,
        callback: Box<dyn Fn(&dyn GameEvent) + Send + Sync>,
    ) -> SubscriptionId;

    /// Remove a subscription by its ID.
    fn unsubscribe(&self, id: SubscriptionId);

    /// Publish an event to all subscribers of its type.
    fn publish(&self, event: Box<dyn GameEvent>);
}

// --- Predefined game event types ---

#[derive(Debug, Clone)]
pub struct ChatMessageEvent {
    pub sender: String,
    pub text: String,
    pub channel: String,
}

impl GameEvent for ChatMessageEvent {
    fn event_type(&self) -> &'static str { "ChatMessage" }
    fn as_any(&self) -> &dyn Any { self }
}

#[derive(Debug, Clone)]
pub struct TickEvent {
    pub tick: u64,
}

impl GameEvent for TickEvent {
    fn event_type(&self) -> &'static str { "Tick" }
    fn as_any(&self) -> &dyn Any { self }
}

#[derive(Debug, Clone)]
pub struct ActionExecutedEvent {
    pub action_id: i32,
    pub entity_id: i64,
}

impl GameEvent for ActionExecutedEvent {
    fn event_type(&self) -> &'static str { "ActionExecuted" }
    fn as_any(&self) -> &dyn Any { self }
}

#[derive(Debug, Clone)]
pub struct LoginStateChangeEvent {
    pub old_state: i32,
    pub new_state: i32,
}

impl GameEvent for LoginStateChangeEvent {
    fn event_type(&self) -> &'static str { "LoginStateChange" }
    fn as_any(&self) -> &dyn Any { self }
}

#[derive(Debug, Clone)]
pub struct VarChangeEvent {
    pub var_id: i32,
    pub old_value: i32,
    pub new_value: i32,
}

impl GameEvent for VarChangeEvent {
    fn event_type(&self) -> &'static str { "VarChange" }
    fn as_any(&self) -> &dyn Any { self }
}

#[derive(Debug, Clone)]
pub struct VarbitChangeEvent {
    pub varbit_id: i32,
    pub old_value: i32,
    pub new_value: i32,
}

impl GameEvent for VarbitChangeEvent {
    fn event_type(&self) -> &'static str { "VarbitChange" }
    fn as_any(&self) -> &dyn Any { self }
}

#[derive(Debug, Clone)]
pub struct KeyInputEvent {
    pub key_code: i32,
    pub pressed: bool,
}

impl GameEvent for KeyInputEvent {
    fn event_type(&self) -> &'static str { "KeyInput" }
    fn as_any(&self) -> &dyn Any { self }
}
