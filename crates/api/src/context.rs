use crate::client::ClientProvider;
use crate::events::EventBus;
use crate::game::GameApi;
use crate::message::MessageBus;
use crate::schedule::ScriptScheduler;
use crate::state::SharedState;
use std::sync::Arc;

/// Context provided to scripts on startup, giving access to all framework services.
/// Equivalent to Java's ScriptContext interface.
pub struct ScriptContext {
    pub game: Arc<dyn GameApi>,
    pub events: Arc<dyn EventBus>,
    pub messages: Arc<dyn MessageBus>,
    pub state: Arc<dyn SharedState>,
    pub clients: Arc<dyn ClientProvider>,
    pub scheduler: Arc<dyn ScriptScheduler>,
}

impl ScriptContext {
    pub fn new(
        game: Arc<dyn GameApi>,
        events: Arc<dyn EventBus>,
        messages: Arc<dyn MessageBus>,
        state: Arc<dyn SharedState>,
        clients: Arc<dyn ClientProvider>,
        scheduler: Arc<dyn ScriptScheduler>,
    ) -> Self {
        Self {
            game,
            events,
            messages,
            state,
            clients,
            scheduler,
        }
    }

    pub fn game_api(&self) -> &dyn GameApi {
        self.game.as_ref()
    }

    pub fn event_bus(&self) -> &dyn EventBus {
        self.events.as_ref()
    }

    pub fn message_bus(&self) -> &dyn MessageBus {
        self.messages.as_ref()
    }

    pub fn shared_state(&self) -> &dyn SharedState {
        self.state.as_ref()
    }

    pub fn client_provider(&self) -> &dyn ClientProvider {
        self.clients.as_ref()
    }

    pub fn script_scheduler(&self) -> &dyn ScriptScheduler {
        self.scheduler.as_ref()
    }
}
