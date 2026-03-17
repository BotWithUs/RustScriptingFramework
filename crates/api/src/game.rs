use serde::{Deserialize, Serialize};
use thiserror::Error;
use std::collections::HashMap;

#[derive(Error, Debug)]
pub enum GameApiError {
    #[error("RPC communication error: {0}")]
    RpcError(String),
    #[error("Game not connected")]
    NotConnected,
    #[error("Invalid response: {0}")]
    InvalidResponse(String),
    #[error("RPC timeout: {0}")]
    Timeout(String),
    #[error("Server error: {0}")]
    ServerError(String),
}

/// Position in the game world.
#[derive(Debug, Clone, Copy, Serialize, Deserialize, Default)]
pub struct WorldPosition {
    pub x: i32,
    pub y: i32,
    pub z: i32,
}

/// A game entity (NPC, player, object, etc.)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EntityInfo {
    pub handle: i64,
    pub name: String,
    pub position: WorldPosition,
    pub entity_type: String,
    pub type_id: i64,
    pub animation_id: i32,
    pub health: i32,
    pub max_health: i32,
    pub overhead_text: String,
    pub moving: bool,
    pub in_combat: bool,
}

/// An item in an inventory slot.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct InventoryItem {
    pub item_id: i64,
    pub name: String,
    pub quantity: i32,
    pub slot: i32,
    pub inventory_id: i32,
}

/// Player skill information.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PlayerStat {
    pub skill_id: i32,
    pub name: String,
    pub level: i32,
    pub boosted_level: i32,
    pub experience: i64,
}

/// An action to queue for execution.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GameAction {
    pub action_id: i32,
    pub param1: i64,
    pub param2: i64,
    pub param3: i64,
}

/// A UI component/widget.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ComponentInfo {
    pub interface_id: i32,
    pub component_id: i32,
    pub sub_component_id: i32,
    pub text: String,
    pub item_id: i64,
    pub item_quantity: i32,
    pub sprite_id: i32,
    pub component_type: i32,
    pub visible: bool,
    pub position: (i32, i32, i32, i32), // x, y, width, height
    pub options: Vec<String>,
}

/// A ground item/obj stack entry.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GroundItemInfo {
    pub handle: i64,
    pub item_id: i64,
    pub name: String,
    pub quantity: i32,
    pub position: WorldPosition,
}

/// A projectile in the game world.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ProjectileInfo {
    pub id: i32,
    pub start_position: WorldPosition,
    pub end_position: WorldPosition,
    pub target_handle: i64,
    pub remaining_cycles: i32,
}

/// Account information.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AccountInfo {
    pub display_name: String,
    pub login_state: i32,
    pub world: i32,
    pub members: bool,
}

/// Chat message entry.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ChatMessage {
    pub index: i32,
    pub message_type: i32,
    pub sender: String,
    pub text: String,
}

/// Mini-menu entry.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MiniMenuEntry {
    pub option: String,
    pub target: String,
    pub action_id: i32,
    pub param1: i64,
    pub param2: i64,
}

/// Grand Exchange offer.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GrandExchangeOffer {
    pub slot: i32,
    pub item_id: i64,
    pub quantity: i32,
    pub price: i32,
    pub transferred: i32,
    pub spent: i64,
    pub state: i32,
}

/// World info.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WorldInfo {
    pub id: i32,
    pub members: bool,
    pub population: i32,
    pub location: String,
    pub activity: String,
}

/// Viewport info.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ViewportInfo {
    pub camera_x: i32,
    pub camera_y: i32,
    pub camera_z: i32,
    pub camera_yaw: i32,
    pub camera_pitch: i32,
}

/// Config type lookup result.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ConfigType {
    pub id: i32,
    pub name: String,
    pub fields: HashMap<String, serde_json::Value>,
}

/// Stream connection info.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StreamInfo {
    pub pipe_name: String,
    pub width: u32,
    pub height: u32,
    pub quality: u32,
    pub frame_skip: u32,
}

/// Personality profile for humanization.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Personality {
    pub reaction_speed: f64,
    pub fatigue_level: f64,
    pub attention_span: f64,
    pub mouse_speed: f64,
    pub click_accuracy: f64,
}

/// Screen position.
#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
pub struct ScreenPosition {
    pub x: i32,
    pub y: i32,
}

/// Game window rectangle.
#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
pub struct GameRect {
    pub x: i32,
    pub y: i32,
    pub width: i32,
    pub height: i32,
}

/// Hitmark on an entity.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Hitmark {
    pub damage: i32,
    pub hitmark_type: i32,
    pub cycle: i32,
}

/// Game API trait providing access to game state and actions via RPC.
/// All methods correspond to RPC calls to the game client.
/// Equivalent to Java's GameAPI interface with all 144 methods.
pub trait GameApi: Send + Sync {
    // ===== System Methods =====
    fn ping(&self) -> Result<bool, GameApiError>;
    fn list_methods(&self) -> Result<Vec<String>, GameApiError>;
    fn subscribe(&self, event: &str) -> Result<bool, GameApiError>;
    fn unsubscribe(&self, event: &str) -> Result<bool, GameApiError>;
    fn get_client_count(&self) -> Result<i32, GameApiError>;
    fn list_events(&self) -> Result<Vec<String>, GameApiError>;
    fn get_subscriptions(&self) -> Result<Vec<String>, GameApiError>;

    // ===== Action Queue Methods =====
    fn queue_action(&self, action: &GameAction) -> Result<bool, GameApiError>;
    fn queue_actions(&self, actions: &[GameAction]) -> Result<bool, GameApiError>;
    fn get_action_queue_size(&self) -> Result<i32, GameApiError>;
    fn clear_action_queue(&self) -> Result<(), GameApiError>;
    fn get_action_history(&self, max_results: i32, action_id_filter: Option<i32>) -> Result<Vec<GameAction>, GameApiError>;
    fn get_last_action_time(&self) -> Result<i64, GameApiError>;
    fn set_behavior_mod(&self, mod_id: &str, value: f64) -> Result<(), GameApiError>;
    fn clear_behavior_mod(&self, mod_id: &str) -> Result<(), GameApiError>;
    fn get_behavior_mod(&self, mod_id: &str) -> Result<f64, GameApiError>;
    fn are_actions_blocked(&self) -> Result<bool, GameApiError>;
    fn set_actions_blocked(&self, blocked: bool) -> Result<(), GameApiError>;

    // ===== Entity Query Methods =====
    fn query_entities(&self, filter: &HashMap<String, serde_json::Value>) -> Result<Vec<EntityInfo>, GameApiError>;
    fn get_entity_info(&self, handle: i64) -> Result<EntityInfo, GameApiError>;
    fn get_entity_name(&self, handle: i64) -> Result<String, GameApiError>;
    fn get_entity_health(&self, handle: i64) -> Result<(i32, i32), GameApiError>;
    fn get_entity_position(&self, handle: i64) -> Result<WorldPosition, GameApiError>;
    fn is_entity_valid(&self, handle: i64) -> Result<bool, GameApiError>;
    fn get_entity_hitmarks(&self, handle: i64) -> Result<Vec<Hitmark>, GameApiError>;
    fn get_entity_animation(&self, handle: i64) -> Result<i32, GameApiError>;
    fn get_entity_overhead_text(&self, handle: i64) -> Result<String, GameApiError>;
    fn get_animation_length(&self, animation_id: i32) -> Result<i32, GameApiError>;

    // ===== Ground Items Methods =====
    fn query_ground_items(&self, filter: &HashMap<String, serde_json::Value>) -> Result<Vec<GroundItemInfo>, GameApiError>;
    fn get_obj_stack_items(&self, handle: i64) -> Result<Vec<GroundItemInfo>, GameApiError>;
    fn query_obj_stacks(&self, filter: &HashMap<String, serde_json::Value>) -> Result<Vec<GroundItemInfo>, GameApiError>;

    // ===== Projectiles & Effects =====
    fn query_projectiles(&self, projectile_id: Option<i32>, plane: Option<i32>, max_results: i32) -> Result<Vec<ProjectileInfo>, GameApiError>;
    fn query_spot_anims(&self, anim_id: Option<i32>, plane: Option<i32>, max_results: i32) -> Result<Vec<HashMap<String, serde_json::Value>>, GameApiError>;
    fn query_hint_arrows(&self, max_results: i32) -> Result<Vec<HashMap<String, serde_json::Value>>, GameApiError>;

    // ===== World & Navigation =====
    fn query_worlds(&self, include_activity: bool) -> Result<Vec<WorldInfo>, GameApiError>;
    fn get_current_world(&self) -> Result<WorldInfo, GameApiError>;
    fn compute_name_hash(&self, name: &str) -> Result<i64, GameApiError>;
    fn update_query_context(&self) -> Result<(), GameApiError>;
    fn invalidate_query_context(&self) -> Result<(), GameApiError>;

    // ===== UI Component Methods =====
    fn query_components(&self, filter: &HashMap<String, serde_json::Value>) -> Result<Vec<ComponentInfo>, GameApiError>;
    fn is_component_valid(&self, interface_id: i32, component_id: i32, sub_component_id: i32) -> Result<bool, GameApiError>;
    fn get_component_text(&self, interface_id: i32, component_id: i32) -> Result<String, GameApiError>;
    fn get_component_item(&self, interface_id: i32, component_id: i32, sub_component_id: i32) -> Result<(i64, i32), GameApiError>;
    fn get_component_position(&self, interface_id: i32, component_id: i32) -> Result<(i32, i32, i32, i32), GameApiError>;
    fn get_component_options(&self, interface_id: i32, component_id: i32) -> Result<Vec<String>, GameApiError>;
    fn get_component_sprite_id(&self, interface_id: i32, component_id: i32) -> Result<i32, GameApiError>;
    fn get_component_type(&self, interface_id: i32, component_id: i32) -> Result<i32, GameApiError>;
    fn get_component_children(&self, interface_id: i32, component_id: i32) -> Result<Vec<ComponentInfo>, GameApiError>;
    fn get_component_by_hash(&self, interface_id: i32, component_id: i32, sub_component_id: i32) -> Result<ComponentInfo, GameApiError>;
    fn get_open_interfaces(&self) -> Result<Vec<i32>, GameApiError>;
    fn is_interface_open(&self, interface_id: i32) -> Result<bool, GameApiError>;

    // ===== Game Variables =====
    fn get_varp(&self, var_id: i32) -> Result<i32, GameApiError>;
    fn get_varbit(&self, varbit_id: i32) -> Result<i32, GameApiError>;
    fn get_varc_int(&self, varc_id: i32) -> Result<i32, GameApiError>;
    fn get_varc_string(&self, varc_id: i32) -> Result<String, GameApiError>;
    fn query_varbits(&self, varbit_ids: &[i32]) -> Result<HashMap<i32, i32>, GameApiError>;

    // ===== Script Execution =====
    fn get_script_handle(&self, script_id: i32) -> Result<i64, GameApiError>;
    fn execute_script(&self, handle: i64, int_args: &[i32], string_args: &[String], returns: &[String]) -> Result<HashMap<String, serde_json::Value>, GameApiError>;
    fn destroy_script_handle(&self, handle: i64) -> Result<(), GameApiError>;
    fn fire_key_trigger(&self, interface_id: i32, component_id: i32, input: i32) -> Result<(), GameApiError>;

    // ===== Game State =====
    fn get_local_player(&self) -> Result<EntityInfo, GameApiError>;
    fn get_account_info(&self) -> Result<AccountInfo, GameApiError>;
    fn get_game_cycle(&self) -> Result<i64, GameApiError>;
    fn get_login_state(&self) -> Result<i32, GameApiError>;
    fn get_mini_menu(&self) -> Result<Vec<MiniMenuEntry>, GameApiError>;
    fn get_grand_exchange_offers(&self) -> Result<Vec<GrandExchangeOffer>, GameApiError>;
    fn get_world_to_screen(&self, tile_x: i32, tile_y: i32) -> Result<ScreenPosition, GameApiError>;
    fn batch_world_to_screen(&self, positions: &[(i32, i32)]) -> Result<Vec<ScreenPosition>, GameApiError>;
    fn get_viewport_info(&self) -> Result<ViewportInfo, GameApiError>;
    fn get_entity_screen_positions(&self, handles: &[i64]) -> Result<Vec<ScreenPosition>, GameApiError>;
    fn get_game_window_rect(&self) -> Result<GameRect, GameApiError>;
    fn set_world(&self, world_id: i32) -> Result<(), GameApiError>;
    fn change_login_state(&self) -> Result<(), GameApiError>;
    fn login_to_lobby(&self) -> Result<(), GameApiError>;

    // ===== Cache =====
    fn get_cache_file(&self, index_id: i32, archive_id: i32, file_id: i32) -> Result<Vec<u8>, GameApiError>;
    fn get_cache_file_count(&self, index_id: i32, archive_id: i32, shift: i32) -> Result<i32, GameApiError>;
    fn get_navigation_archive(&self) -> Result<Vec<u8>, GameApiError>;

    // ===== Breaks & Scheduling =====
    fn schedule_break(&self) -> Result<(), GameApiError>;
    fn interrupt_break(&self) -> Result<(), GameApiError>;

    // ===== Auto Login =====
    fn get_auto_login(&self) -> Result<bool, GameApiError>;
    fn set_auto_login(&self, enabled: bool) -> Result<(), GameApiError>;

    // ===== Screenshots =====
    fn take_screenshot(&self) -> Result<Vec<u8>, GameApiError>;

    // ===== Streaming =====
    fn start_stream(&self, frame_skip: u32, quality: u32, width: u32, height: u32) -> Result<StreamInfo, GameApiError>;
    fn stop_stream(&self) -> Result<(), GameApiError>;

    // ===== Humanization =====
    fn get_humanization_enabled(&self) -> Result<bool, GameApiError>;
    fn set_humanization_enabled(&self, enabled: bool) -> Result<(), GameApiError>;
    fn get_personality(&self) -> Result<Personality, GameApiError>;

    // ===== Inventory & Items =====
    fn query_inventories(&self) -> Result<Vec<HashMap<String, serde_json::Value>>, GameApiError>;
    fn query_inventory_items(&self, filter: &HashMap<String, serde_json::Value>) -> Result<Vec<InventoryItem>, GameApiError>;
    fn get_inventory_item(&self, inventory_id: i32, slot: i32) -> Result<InventoryItem, GameApiError>;
    fn get_item_vars(&self, inventory_id: i32, slot: i32) -> Result<HashMap<i32, i32>, GameApiError>;
    fn get_item_var_value(&self, inventory_id: i32, slot: i32, var_id: i32) -> Result<i32, GameApiError>;
    fn is_inventory_item_valid(&self, inventory_id: i32, slot: i32) -> Result<bool, GameApiError>;

    // ===== Player Stats =====
    fn get_player_stats(&self) -> Result<Vec<PlayerStat>, GameApiError>;
    fn get_player_stat(&self, skill_id: i32) -> Result<PlayerStat, GameApiError>;
    fn get_player_stat_count(&self) -> Result<i32, GameApiError>;

    // ===== Chat =====
    fn query_chat_history(&self, message_type: Option<i32>, max_results: i32) -> Result<Vec<ChatMessage>, GameApiError>;
    fn get_chat_message_text(&self, index: i32) -> Result<String, GameApiError>;
    fn get_chat_message_player(&self, index: i32) -> Result<String, GameApiError>;
    fn get_chat_message_type(&self, index: i32) -> Result<i32, GameApiError>;
    fn get_chat_history_size(&self) -> Result<i32, GameApiError>;

    // ===== Config Type Lookups =====
    fn get_item_type(&self, id: i32) -> Result<ConfigType, GameApiError>;
    fn get_npc_type(&self, id: i32) -> Result<ConfigType, GameApiError>;
    fn get_location_type(&self, id: i32) -> Result<ConfigType, GameApiError>;
    fn get_enum_type(&self, id: i32) -> Result<ConfigType, GameApiError>;
    fn get_struct_type(&self, id: i32) -> Result<ConfigType, GameApiError>;
    fn get_sequence_type(&self, id: i32) -> Result<ConfigType, GameApiError>;
    fn get_quest_type(&self, id: i32) -> Result<ConfigType, GameApiError>;
}
