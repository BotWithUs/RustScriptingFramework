use bot_api::events::{EventBus, GameEvent, SubscriptionId};
use std::collections::HashMap;
use std::sync::atomic::{AtomicU64, Ordering};
use std::sync::RwLock;

type Callback = Box<dyn Fn(&dyn GameEvent) + Send + Sync>;

struct Subscription {
    id: SubscriptionId,
    callback: Callback,
}

/// Event bus implementation using type-erased callbacks keyed by event type string.
pub struct EventBusImpl {
    subscribers: RwLock<HashMap<&'static str, Vec<Subscription>>>,
    next_id: AtomicU64,
}

impl EventBusImpl {
    pub fn new() -> Self {
        Self {
            subscribers: RwLock::new(HashMap::new()),
            next_id: AtomicU64::new(1),
        }
    }
}

impl Default for EventBusImpl {
    fn default() -> Self {
        Self::new()
    }
}

impl EventBus for EventBusImpl {
    fn subscribe(
        &self,
        event_type: &'static str,
        callback: Box<dyn Fn(&dyn GameEvent) + Send + Sync>,
    ) -> SubscriptionId {
        let id = SubscriptionId(self.next_id.fetch_add(1, Ordering::Relaxed));
        let sub = Subscription {
            id,
            callback,
        };

        let mut subs = self.subscribers.write().unwrap();
        subs.entry(event_type).or_default().push(sub);
        id
    }

    fn unsubscribe(&self, id: SubscriptionId) {
        let mut subs = self.subscribers.write().unwrap();
        for listeners in subs.values_mut() {
            listeners.retain(|s| s.id != id);
        }
    }

    fn publish(&self, event: Box<dyn GameEvent>) {
        let subs = self.subscribers.read().unwrap();
        if let Some(listeners) = subs.get(event.event_type()) {
            for listener in listeners {
                (listener.callback)(event.as_ref());
            }
        }
    }
}
