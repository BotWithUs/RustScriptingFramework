use bot_api::client::ClientProvider;
use bot_api::message::MessageBus;
use bot_api::state::SharedState;
use std::collections::{HashMap, HashSet};
use std::sync::{Arc, RwLock};

/// Result of an operation on a client.
#[derive(Debug, Clone)]
pub struct OpResult {
    pub success: bool,
    pub client_name: String,
    pub script_name: String,
    pub message: String,
}

/// Status of a script on a specific client.
#[derive(Debug, Clone)]
pub struct ScriptStatusEntry {
    pub client_name: String,
    pub script_name: String,
    pub version: String,
    pub running: bool,
    pub client_alive: bool,
}

/// Context for management scripts that orchestrate across multiple clients.
/// Equivalent to Java's ManagementContext.
pub struct ManagementContext {
    pub orchestrator: Arc<ClientOrchestrator>,
    pub clients: Arc<dyn ClientProvider>,
    pub messages: Arc<dyn MessageBus>,
    pub state: Arc<dyn SharedState>,
}

/// Manages client groups and cross-client script operations.
/// Equivalent to Java's ClientOrchestrator interface.
pub struct ClientOrchestrator {
    clients: Arc<dyn ClientProvider>,
    groups: RwLock<HashMap<String, GroupInfo>>,
}

struct GroupInfo {
    #[allow(dead_code)]
    description: String,
    members: HashSet<String>,
}

impl ClientOrchestrator {
    pub fn new(clients: Arc<dyn ClientProvider>) -> Self {
        Self {
            clients,
            groups: RwLock::new(HashMap::new()),
        }
    }

    // --- Client queries ---

    pub fn get_client_names(&self) -> Vec<String> {
        self.clients
            .clients()
            .iter()
            .map(|c| c.name().to_string())
            .collect()
    }

    pub fn is_client_alive(&self, name: &str) -> bool {
        self.clients
            .get_client(name)
            .map(|c| c.is_alive())
            .unwrap_or(false)
    }

    // --- Group management ---

    pub fn create_group(&self, name: &str, description: &str) -> bool {
        let mut groups = self.groups.write().unwrap();
        if groups.contains_key(name) {
            return false;
        }
        groups.insert(
            name.to_string(),
            GroupInfo {
                description: description.to_string(),
                members: HashSet::new(),
            },
        );
        true
    }

    pub fn delete_group(&self, name: &str) -> bool {
        let mut groups = self.groups.write().unwrap();
        groups.remove(name).is_some()
    }

    pub fn get_group_names(&self) -> Vec<String> {
        let groups = self.groups.read().unwrap();
        groups.keys().cloned().collect()
    }

    pub fn get_group_members(&self, group_name: &str) -> Vec<String> {
        let groups = self.groups.read().unwrap();
        groups
            .get(group_name)
            .map(|g| g.members.iter().cloned().collect())
            .unwrap_or_default()
    }

    pub fn add_to_group(&self, group_name: &str, client_name: &str) -> bool {
        let mut groups = self.groups.write().unwrap();
        if let Some(group) = groups.get_mut(group_name) {
            group.members.insert(client_name.to_string())
        } else {
            false
        }
    }

    pub fn remove_from_group(&self, group_name: &str, client_name: &str) -> bool {
        let mut groups = self.groups.write().unwrap();
        if let Some(group) = groups.get_mut(group_name) {
            group.members.remove(client_name)
        } else {
            false
        }
    }
}

/// Trait for management scripts that orchestrate across multiple clients.
/// Equivalent to Java's ManagementScript interface.
pub trait ManagementScript: Send {
    fn on_start(&mut self, ctx: &ManagementContext) -> Result<(), Box<dyn std::error::Error>>;
    fn on_loop(&mut self) -> Result<bot_api::script::LoopAction, Box<dyn std::error::Error>>;
    fn on_stop(&mut self);
}
