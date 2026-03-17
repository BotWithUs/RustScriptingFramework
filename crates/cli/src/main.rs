mod command;
mod gui;

use anyhow::Result;
use bot_api::client::{Client, ClientProvider};
use bot_api::game::*;
use bot_api::schedule::ScriptScheduler;
use bot_core::bus::event_bus::EventBusImpl;
use bot_core::bus::message_bus::MessageBusImpl;
use bot_core::config::ScriptConfigStore;
use bot_core::loader::local::LocalScriptLoader;
use bot_core::rpc::client::RpcClient;
use bot_core::rpc::game_api::RpcGameApi;
use bot_core::rpc::pipe::PipeClient;
use bot_core::rpc::retry::RetryPolicy;
use bot_core::runtime::runtime::ScriptRuntime;
use bot_core::runtime::scheduler::ScriptSchedulerImpl;
use bot_core::state::SharedStateImpl;
use clap::Parser;
use gui::context::AppContext;
use std::collections::HashMap;
use std::path::PathBuf;
use std::sync::Arc;
use tracing;

#[derive(Parser, Debug)]
#[command(name = "botwithus", about = "Rust Scripting Framework for game automation")]
struct Args {
    /// Directory containing script plugins (.dll/.so/.dylib)
    #[arg(short, long, default_value = "scripts")]
    scripts_dir: PathBuf,

    /// Named pipe to connect to the game client
    #[arg(short, long, default_value = r"\\.\pipe\BotWithUs")]
    pipe: String,

    /// Configuration directory
    #[arg(short, long)]
    config_dir: Option<PathBuf>,

    /// Start all discovered scripts automatically
    #[arg(long, default_value_t = false)]
    auto_start: bool,

    /// Run in headless mode (no GUI, Ctrl+C to stop)
    #[arg(long, default_value_t = false)]
    headless: bool,
}

/// Stub GameApi for when no game is connected (used for testing/development).
/// Returns sensible defaults for all methods.
struct StubGameApi;

macro_rules! stub_ok { ($v:expr) => { Ok($v) }; }

impl GameApi for StubGameApi {
    fn ping(&self) -> Result<bool, GameApiError> { stub_ok!(true) }
    fn list_methods(&self) -> Result<Vec<String>, GameApiError> { stub_ok!(vec![]) }
    fn subscribe(&self, _: &str) -> Result<bool, GameApiError> { stub_ok!(true) }
    fn unsubscribe(&self, _: &str) -> Result<bool, GameApiError> { stub_ok!(true) }
    fn get_client_count(&self) -> Result<i32, GameApiError> { stub_ok!(0) }
    fn list_events(&self) -> Result<Vec<String>, GameApiError> { stub_ok!(vec![]) }
    fn get_subscriptions(&self) -> Result<Vec<String>, GameApiError> { stub_ok!(vec![]) }

    fn queue_action(&self, _: &GameAction) -> Result<bool, GameApiError> { stub_ok!(true) }
    fn queue_actions(&self, _: &[GameAction]) -> Result<bool, GameApiError> { stub_ok!(true) }
    fn get_action_queue_size(&self) -> Result<i32, GameApiError> { stub_ok!(0) }
    fn clear_action_queue(&self) -> Result<(), GameApiError> { stub_ok!(()) }
    fn get_action_history(&self, _: i32, _: Option<i32>) -> Result<Vec<GameAction>, GameApiError> { stub_ok!(vec![]) }
    fn get_last_action_time(&self) -> Result<i64, GameApiError> { stub_ok!(0) }
    fn set_behavior_mod(&self, _: &str, _: f64) -> Result<(), GameApiError> { stub_ok!(()) }
    fn clear_behavior_mod(&self, _: &str) -> Result<(), GameApiError> { stub_ok!(()) }
    fn get_behavior_mod(&self, _: &str) -> Result<f64, GameApiError> { stub_ok!(0.0) }
    fn are_actions_blocked(&self) -> Result<bool, GameApiError> { stub_ok!(false) }
    fn set_actions_blocked(&self, _: bool) -> Result<(), GameApiError> { stub_ok!(()) }

    fn query_entities(&self, _: &HashMap<String, serde_json::Value>) -> Result<Vec<EntityInfo>, GameApiError> { stub_ok!(vec![]) }
    fn get_entity_info(&self, _: i64) -> Result<EntityInfo, GameApiError> {
        stub_ok!(EntityInfo {
            handle: 0, name: String::new(), position: WorldPosition::default(),
            entity_type: String::new(), type_id: 0, animation_id: -1,
            health: 0, max_health: 0, overhead_text: String::new(),
            moving: false, in_combat: false,
        })
    }
    fn get_entity_name(&self, _: i64) -> Result<String, GameApiError> { stub_ok!(String::new()) }
    fn get_entity_health(&self, _: i64) -> Result<(i32, i32), GameApiError> { stub_ok!((0, 0)) }
    fn get_entity_position(&self, _: i64) -> Result<WorldPosition, GameApiError> { stub_ok!(WorldPosition::default()) }
    fn is_entity_valid(&self, _: i64) -> Result<bool, GameApiError> { stub_ok!(false) }
    fn get_entity_hitmarks(&self, _: i64) -> Result<Vec<Hitmark>, GameApiError> { stub_ok!(vec![]) }
    fn get_entity_animation(&self, _: i64) -> Result<i32, GameApiError> { stub_ok!(-1) }
    fn get_entity_overhead_text(&self, _: i64) -> Result<String, GameApiError> { stub_ok!(String::new()) }
    fn get_animation_length(&self, _: i32) -> Result<i32, GameApiError> { stub_ok!(0) }

    fn query_ground_items(&self, _: &HashMap<String, serde_json::Value>) -> Result<Vec<GroundItemInfo>, GameApiError> { stub_ok!(vec![]) }
    fn get_obj_stack_items(&self, _: i64) -> Result<Vec<GroundItemInfo>, GameApiError> { stub_ok!(vec![]) }
    fn query_obj_stacks(&self, _: &HashMap<String, serde_json::Value>) -> Result<Vec<GroundItemInfo>, GameApiError> { stub_ok!(vec![]) }

    fn query_projectiles(&self, _: Option<i32>, _: Option<i32>, _: i32) -> Result<Vec<ProjectileInfo>, GameApiError> { stub_ok!(vec![]) }
    fn query_spot_anims(&self, _: Option<i32>, _: Option<i32>, _: i32) -> Result<Vec<HashMap<String, serde_json::Value>>, GameApiError> { stub_ok!(vec![]) }
    fn query_hint_arrows(&self, _: i32) -> Result<Vec<HashMap<String, serde_json::Value>>, GameApiError> { stub_ok!(vec![]) }

    fn query_worlds(&self, _: bool) -> Result<Vec<WorldInfo>, GameApiError> { stub_ok!(vec![]) }
    fn get_current_world(&self) -> Result<WorldInfo, GameApiError> {
        stub_ok!(WorldInfo { id: 0, members: false, population: 0, location: String::new(), activity: String::new() })
    }
    fn compute_name_hash(&self, _: &str) -> Result<i64, GameApiError> { stub_ok!(0) }
    fn update_query_context(&self) -> Result<(), GameApiError> { stub_ok!(()) }
    fn invalidate_query_context(&self) -> Result<(), GameApiError> { stub_ok!(()) }

    fn query_components(&self, _: &HashMap<String, serde_json::Value>) -> Result<Vec<ComponentInfo>, GameApiError> { stub_ok!(vec![]) }
    fn is_component_valid(&self, _: i32, _: i32, _: i32) -> Result<bool, GameApiError> { stub_ok!(false) }
    fn get_component_text(&self, _: i32, _: i32) -> Result<String, GameApiError> { stub_ok!(String::new()) }
    fn get_component_item(&self, _: i32, _: i32, _: i32) -> Result<(i64, i32), GameApiError> { stub_ok!((0, 0)) }
    fn get_component_position(&self, _: i32, _: i32) -> Result<(i32, i32, i32, i32), GameApiError> { stub_ok!((0, 0, 0, 0)) }
    fn get_component_options(&self, _: i32, _: i32) -> Result<Vec<String>, GameApiError> { stub_ok!(vec![]) }
    fn get_component_sprite_id(&self, _: i32, _: i32) -> Result<i32, GameApiError> { stub_ok!(-1) }
    fn get_component_type(&self, _: i32, _: i32) -> Result<i32, GameApiError> { stub_ok!(0) }
    fn get_component_children(&self, _: i32, _: i32) -> Result<Vec<ComponentInfo>, GameApiError> { stub_ok!(vec![]) }
    fn get_component_by_hash(&self, _: i32, _: i32, _: i32) -> Result<ComponentInfo, GameApiError> {
        stub_ok!(ComponentInfo {
            interface_id: 0, component_id: 0, sub_component_id: 0,
            text: String::new(), item_id: 0, item_quantity: 0,
            sprite_id: -1, component_type: 0, visible: false,
            position: (0, 0, 0, 0), options: vec![],
        })
    }
    fn get_open_interfaces(&self) -> Result<Vec<i32>, GameApiError> { stub_ok!(vec![]) }
    fn is_interface_open(&self, _: i32) -> Result<bool, GameApiError> { stub_ok!(false) }

    fn get_varp(&self, _: i32) -> Result<i32, GameApiError> { stub_ok!(0) }
    fn get_varbit(&self, _: i32) -> Result<i32, GameApiError> { stub_ok!(0) }
    fn get_varc_int(&self, _: i32) -> Result<i32, GameApiError> { stub_ok!(0) }
    fn get_varc_string(&self, _: i32) -> Result<String, GameApiError> { stub_ok!(String::new()) }
    fn query_varbits(&self, _: &[i32]) -> Result<HashMap<i32, i32>, GameApiError> { stub_ok!(HashMap::new()) }

    fn get_script_handle(&self, _: i32) -> Result<i64, GameApiError> { stub_ok!(0) }
    fn execute_script(&self, _: i64, _: &[i32], _: &[String], _: &[String]) -> Result<HashMap<String, serde_json::Value>, GameApiError> { stub_ok!(HashMap::new()) }
    fn destroy_script_handle(&self, _: i64) -> Result<(), GameApiError> { stub_ok!(()) }
    fn fire_key_trigger(&self, _: i32, _: i32, _: i32) -> Result<(), GameApiError> { stub_ok!(()) }

    fn get_local_player(&self) -> Result<EntityInfo, GameApiError> {
        stub_ok!(EntityInfo {
            handle: 1, name: "LocalPlayer".into(), position: WorldPosition::default(),
            entity_type: "player".into(), type_id: 0, animation_id: -1,
            health: 100, max_health: 100, overhead_text: String::new(),
            moving: false, in_combat: false,
        })
    }
    fn get_account_info(&self) -> Result<AccountInfo, GameApiError> {
        stub_ok!(AccountInfo { display_name: String::new(), login_state: 0, world: 0, members: false })
    }
    fn get_game_cycle(&self) -> Result<i64, GameApiError> { stub_ok!(0) }
    fn get_login_state(&self) -> Result<i32, GameApiError> { stub_ok!(0) }
    fn get_mini_menu(&self) -> Result<Vec<MiniMenuEntry>, GameApiError> { stub_ok!(vec![]) }
    fn get_grand_exchange_offers(&self) -> Result<Vec<GrandExchangeOffer>, GameApiError> { stub_ok!(vec![]) }
    fn get_world_to_screen(&self, _: i32, _: i32) -> Result<ScreenPosition, GameApiError> { stub_ok!(ScreenPosition { x: 0, y: 0 }) }
    fn batch_world_to_screen(&self, _: &[(i32, i32)]) -> Result<Vec<ScreenPosition>, GameApiError> { stub_ok!(vec![]) }
    fn get_viewport_info(&self) -> Result<ViewportInfo, GameApiError> {
        stub_ok!(ViewportInfo { camera_x: 0, camera_y: 0, camera_z: 0, camera_yaw: 0, camera_pitch: 0 })
    }
    fn get_entity_screen_positions(&self, _: &[i64]) -> Result<Vec<ScreenPosition>, GameApiError> { stub_ok!(vec![]) }
    fn get_game_window_rect(&self) -> Result<GameRect, GameApiError> { stub_ok!(GameRect { x: 0, y: 0, width: 800, height: 600 }) }
    fn set_world(&self, _: i32) -> Result<(), GameApiError> { stub_ok!(()) }
    fn change_login_state(&self) -> Result<(), GameApiError> { stub_ok!(()) }
    fn login_to_lobby(&self) -> Result<(), GameApiError> { stub_ok!(()) }

    fn get_cache_file(&self, _: i32, _: i32, _: i32) -> Result<Vec<u8>, GameApiError> { stub_ok!(vec![]) }
    fn get_cache_file_count(&self, _: i32, _: i32, _: i32) -> Result<i32, GameApiError> { stub_ok!(0) }
    fn get_navigation_archive(&self) -> Result<Vec<u8>, GameApiError> { stub_ok!(vec![]) }

    fn schedule_break(&self) -> Result<(), GameApiError> { stub_ok!(()) }
    fn interrupt_break(&self) -> Result<(), GameApiError> { stub_ok!(()) }

    fn get_auto_login(&self) -> Result<bool, GameApiError> { stub_ok!(false) }
    fn set_auto_login(&self, _: bool) -> Result<(), GameApiError> { stub_ok!(()) }

    fn take_screenshot(&self) -> Result<Vec<u8>, GameApiError> { stub_ok!(vec![]) }

    fn start_stream(&self, _: u32, _: u32, _: u32, _: u32) -> Result<StreamInfo, GameApiError> {
        stub_ok!(StreamInfo { pipe_name: String::new(), width: 0, height: 0, quality: 0, frame_skip: 0 })
    }
    fn stop_stream(&self) -> Result<(), GameApiError> { stub_ok!(()) }

    fn get_humanization_enabled(&self) -> Result<bool, GameApiError> { stub_ok!(false) }
    fn set_humanization_enabled(&self, _: bool) -> Result<(), GameApiError> { stub_ok!(()) }
    fn get_personality(&self) -> Result<Personality, GameApiError> {
        stub_ok!(Personality { reaction_speed: 1.0, fatigue_level: 0.0, attention_span: 1.0, mouse_speed: 1.0, click_accuracy: 1.0 })
    }

    fn query_inventories(&self) -> Result<Vec<HashMap<String, serde_json::Value>>, GameApiError> { stub_ok!(vec![]) }
    fn query_inventory_items(&self, _: &HashMap<String, serde_json::Value>) -> Result<Vec<InventoryItem>, GameApiError> { stub_ok!(vec![]) }
    fn get_inventory_item(&self, _: i32, _: i32) -> Result<InventoryItem, GameApiError> {
        stub_ok!(InventoryItem { item_id: 0, name: String::new(), quantity: 0, slot: 0, inventory_id: 0 })
    }
    fn get_item_vars(&self, _: i32, _: i32) -> Result<HashMap<i32, i32>, GameApiError> { stub_ok!(HashMap::new()) }
    fn get_item_var_value(&self, _: i32, _: i32, _: i32) -> Result<i32, GameApiError> { stub_ok!(0) }
    fn is_inventory_item_valid(&self, _: i32, _: i32) -> Result<bool, GameApiError> { stub_ok!(false) }

    fn get_player_stats(&self) -> Result<Vec<PlayerStat>, GameApiError> { stub_ok!(vec![]) }
    fn get_player_stat(&self, _: i32) -> Result<PlayerStat, GameApiError> {
        stub_ok!(PlayerStat { skill_id: 0, name: String::new(), level: 1, boosted_level: 1, experience: 0 })
    }
    fn get_player_stat_count(&self) -> Result<i32, GameApiError> { stub_ok!(0) }

    fn query_chat_history(&self, _: Option<i32>, _: i32) -> Result<Vec<ChatMessage>, GameApiError> { stub_ok!(vec![]) }
    fn get_chat_message_text(&self, _: i32) -> Result<String, GameApiError> { stub_ok!(String::new()) }
    fn get_chat_message_player(&self, _: i32) -> Result<String, GameApiError> { stub_ok!(String::new()) }
    fn get_chat_message_type(&self, _: i32) -> Result<i32, GameApiError> { stub_ok!(0) }
    fn get_chat_history_size(&self) -> Result<i32, GameApiError> { stub_ok!(0) }

    fn get_item_type(&self, _: i32) -> Result<ConfigType, GameApiError> { stub_ok!(ConfigType { id: 0, name: String::new(), fields: HashMap::new() }) }
    fn get_npc_type(&self, _: i32) -> Result<ConfigType, GameApiError> { stub_ok!(ConfigType { id: 0, name: String::new(), fields: HashMap::new() }) }
    fn get_location_type(&self, _: i32) -> Result<ConfigType, GameApiError> { stub_ok!(ConfigType { id: 0, name: String::new(), fields: HashMap::new() }) }
    fn get_enum_type(&self, _: i32) -> Result<ConfigType, GameApiError> { stub_ok!(ConfigType { id: 0, name: String::new(), fields: HashMap::new() }) }
    fn get_struct_type(&self, _: i32) -> Result<ConfigType, GameApiError> { stub_ok!(ConfigType { id: 0, name: String::new(), fields: HashMap::new() }) }
    fn get_sequence_type(&self, _: i32) -> Result<ConfigType, GameApiError> { stub_ok!(ConfigType { id: 0, name: String::new(), fields: HashMap::new() }) }
    fn get_quest_type(&self, _: i32) -> Result<ConfigType, GameApiError> { stub_ok!(ConfigType { id: 0, name: String::new(), fields: HashMap::new() }) }
}

/// Stub ClientProvider for standalone operation.
struct StubClientProvider;

impl ClientProvider for StubClientProvider {
    fn clients(&self) -> Vec<Arc<dyn Client>> { vec![] }
    fn get_client(&self, _id: &str) -> Option<Arc<dyn Client>> { None }
    fn client_count(&self) -> usize { 0 }
}

/// Connect to the game via named pipe and return a live RpcGameApi.
fn connect_to_game(pipe_name: &str) -> Result<(RpcGameApi, Arc<RpcClient>)> {
    let mut pipe = PipeClient::new(pipe_name);
    pipe.connect().map_err(|e| anyhow::anyhow!("Pipe connect failed: {}", e))?;

    let mut rpc = RpcClient::new(pipe);
    rpc.set_timeout(std::time::Duration::from_secs(10));
    rpc.set_retry_policy(RetryPolicy::default_policy());

    let rpc = Arc::new(rpc);
    rpc.start();

    let api = RpcGameApi::new(rpc.clone());
    Ok((api, rpc))
}

fn main() -> Result<()> {
    // Initialize logging
    tracing_subscriber::fmt()
        .with_env_filter(
            tracing_subscriber::EnvFilter::try_from_default_env()
                .unwrap_or_else(|_| tracing_subscriber::EnvFilter::new("info")),
        )
        .init();

    let args = Args::parse();
    tracing::info!("BotWithUs Rust Scripting Framework starting...");
    tracing::info!("Scripts directory: {:?}", args.scripts_dir);

    // Create tokio runtime for script execution (background threads)
    let tokio_rt = tokio::runtime::Runtime::new()?;
    let _guard = tokio_rt.enter();

    // Create shared services
    let config_store = match &args.config_dir {
        Some(dir) => ScriptConfigStore::new(dir),
        None => ScriptConfigStore::default_location(),
    };

    // Try to connect to the game via named pipe
    let (game, rpc_client): (Arc<dyn GameApi>, Option<Arc<RpcClient>>) = match connect_to_game(&args.pipe) {
        Ok((api, rpc)) => {
            tracing::info!("Connected to game pipe: {}", args.pipe);
            (Arc::new(api), Some(rpc))
        }
        Err(e) => {
            tracing::warn!(
                "Could not connect to game pipe '{}': {}. Using stub API (offline mode).",
                args.pipe, e
            );
            (Arc::new(StubGameApi), None)
        }
    };

    let events: Arc<dyn bot_api::events::EventBus> = Arc::new(EventBusImpl::new());
    let messages: Arc<dyn bot_api::message::MessageBus> = Arc::new(MessageBusImpl::new());
    let state: Arc<dyn bot_api::state::SharedState> = Arc::new(SharedStateImpl::new());
    let clients: Arc<dyn ClientProvider> = Arc::new(StubClientProvider);

    let scheduler: Arc<dyn ScriptScheduler> = Arc::new(ScriptSchedulerImpl::new(
        Arc::new(|name| {
            tracing::info!("Scheduler triggered start for script: {}", name);
        }),
    ));

    let mut runtime = ScriptRuntime::new(
        config_store, game.clone(), events, messages, state, clients, scheduler,
    );

    // Load scripts from directory
    if args.scripts_dir.exists() {
        let mut loader = LocalScriptLoader::new(&args.scripts_dir);
        match unsafe { loader.load_scripts() } {
            Ok(scripts) => {
                let count = scripts.len();
                for script in scripts {
                    runtime.register(script);
                }
                tracing::info!("Registered {} script(s)", count);

                if args.auto_start {
                    let script_names: Vec<String> = runtime
                        .list_all()
                        .iter()
                        .map(|s| s.name.clone())
                        .collect();
                    for name in script_names {
                        if runtime.start(&name) {
                            tracing::info!("Auto-started script: {}", name);
                        }
                    }
                }
            }
            Err(e) => {
                tracing::warn!("Failed to load scripts: {}", e);
            }
        }
    } else {
        tracing::info!(
            "Scripts directory {:?} does not exist, no scripts loaded",
            args.scripts_dir
        );
    }

    if args.headless {
        // Headless mode - wait for Ctrl+C
        let all_scripts = runtime.list_all();
        if all_scripts.is_empty() {
            tracing::info!("No scripts loaded. Place .dll/.so/.dylib files in the scripts directory.");
        } else {
            for script in &all_scripts {
                tracing::info!(
                    "  {} v{} by {} [{}]",
                    script.name, script.version, script.author,
                    if script.running { "RUNNING" } else { "STOPPED" }
                );
            }
        }

        tracing::info!("Headless mode. Press Ctrl+C to stop...");
        tokio_rt.block_on(async {
            tokio::signal::ctrl_c().await.ok();
        });
    } else {
        // GUI mode - run ImGui window
        let app_ctx = AppContext::new(
            runtime,
            game.clone(),
            rpc_client.clone(),
            args.pipe.clone(),
            args.scripts_dir.clone(),
        );

        if let Err(e) = gui::run(app_ctx) {
            tracing::error!("GUI error: {}", e);
        }

        // Clean up after GUI closes - runtime was moved into app_ctx
        // and will be dropped, stopping scripts.

        if let Some(rpc) = &rpc_client {
            rpc.close();
            tracing::info!("RPC connection closed.");
        }

        tracing::info!("All scripts stopped. Goodbye!");
        return Ok(());
    }

    // Headless cleanup
    tracing::info!("Shutting down...");
    runtime.stop_all();
    tokio_rt.block_on(async {
        runtime.await_all_stopped(std::time::Duration::from_secs(5)).await;
    });

    if let Some(rpc) = rpc_client {
        rpc.close();
        tracing::info!("RPC connection closed.");
    }

    tracing::info!("All scripts stopped. Goodbye!");
    Ok(())
}
