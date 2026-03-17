use crate::game::GameApi;
use std::sync::Arc;

/// Unique identifier for a connected game client.
pub type ClientId = String;

/// Represents a connected game client instance.
pub trait Client: Send + Sync {
    /// Returns the unique identifier for this client.
    fn id(&self) -> &ClientId;

    /// Returns the display name for this client.
    fn name(&self) -> &str;

    /// Returns true if this client is still connected.
    fn is_alive(&self) -> bool;

    /// Access this client's game API for interaction.
    fn game_api(&self) -> Arc<dyn GameApi>;
}

/// Provides access to all connected game clients.
/// Equivalent to Java's ClientProvider interface.
pub trait ClientProvider: Send + Sync {
    /// Get all currently connected clients.
    fn clients(&self) -> Vec<Arc<dyn Client>>;

    /// Get a specific client by ID.
    fn get_client(&self, id: &str) -> Option<Arc<dyn Client>>;

    /// Get the number of connected clients.
    fn client_count(&self) -> usize;
}
