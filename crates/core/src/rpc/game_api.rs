use bot_api::game::*;
use crate::rpc::client::RpcClient;
use crate::rpc::codec::MapHelper;
use std::collections::HashMap;
use std::sync::Arc;

/// Convert serde_json::Value to rmpv::Value for RPC transport.
fn json_to_rmpv(value: &serde_json::Value) -> rmpv::Value {
    match value {
        serde_json::Value::Null => rmpv::Value::Nil,
        serde_json::Value::Bool(b) => rmpv::Value::Boolean(*b),
        serde_json::Value::Number(n) => {
            if let Some(i) = n.as_i64() {
                rmpv::Value::Integer(i.into())
            } else if let Some(f) = n.as_f64() {
                rmpv::Value::F64(f)
            } else {
                rmpv::Value::Nil
            }
        }
        serde_json::Value::String(s) => rmpv::Value::String(s.clone().into()),
        serde_json::Value::Array(arr) => {
            rmpv::Value::Array(arr.iter().map(json_to_rmpv).collect())
        }
        serde_json::Value::Object(map) => {
            rmpv::Value::Map(
                map.iter()
                    .map(|(k, v)| (rmpv::Value::String(k.clone().into()), json_to_rmpv(v)))
                    .collect(),
            )
        }
    }
}

/// Convert a serde_json filter map to rmpv params map.
fn json_filter_to_rmpv(filter: &HashMap<String, serde_json::Value>) -> HashMap<String, rmpv::Value> {
    filter.iter().map(|(k, v)| (k.clone(), json_to_rmpv(v))).collect()
}

/// Convert rmpv::Value back to serde_json::Value.
fn rmpv_to_json(value: &rmpv::Value) -> serde_json::Value {
    match value {
        rmpv::Value::Nil => serde_json::Value::Null,
        rmpv::Value::Boolean(b) => serde_json::Value::Bool(*b),
        rmpv::Value::Integer(i) => {
            if let Some(n) = i.as_i64() {
                serde_json::Value::Number(n.into())
            } else if let Some(n) = i.as_u64() {
                serde_json::Value::Number(n.into())
            } else {
                serde_json::Value::Null
            }
        }
        rmpv::Value::F32(f) => {
            serde_json::Number::from_f64(*f as f64)
                .map(serde_json::Value::Number)
                .unwrap_or(serde_json::Value::Null)
        }
        rmpv::Value::F64(f) => {
            serde_json::Number::from_f64(*f)
                .map(serde_json::Value::Number)
                .unwrap_or(serde_json::Value::Null)
        }
        rmpv::Value::String(s) => {
            serde_json::Value::String(s.as_str().unwrap_or_default().to_string())
        }
        rmpv::Value::Binary(b) => {
            serde_json::Value::Array(b.iter().map(|&byte| serde_json::Value::Number(byte.into())).collect())
        }
        rmpv::Value::Array(arr) => {
            serde_json::Value::Array(arr.iter().map(rmpv_to_json).collect())
        }
        rmpv::Value::Map(pairs) => {
            let mut map = serde_json::Map::new();
            for (k, v) in pairs {
                let key = match k {
                    rmpv::Value::String(s) => s.as_str().unwrap_or_default().to_string(),
                    _ => k.to_string(),
                };
                map.insert(key, rmpv_to_json(v));
            }
            serde_json::Value::Object(map)
        }
        rmpv::Value::Ext(_, _) => serde_json::Value::Null,
    }
}

/// GameApi implementation that delegates all calls to RPC over named pipes.
/// Equivalent to Java's GameAPIImpl.
pub struct RpcGameApi {
    rpc: Arc<RpcClient>,
}

impl RpcGameApi {
    pub fn new(rpc: Arc<RpcClient>) -> Self {
        Self { rpc }
    }

    /// Helper: call with no params, return raw value.
    fn call_raw(&self, method: &str) -> Result<rmpv::Value, GameApiError> {
        self.rpc
            .call_sync_raw(method, HashMap::new())
            .map_err(|e| GameApiError::RpcError(e.to_string()))
    }

    /// Helper: call with params, return raw value.
    fn call_raw_with(&self, method: &str, params: HashMap<String, rmpv::Value>) -> Result<rmpv::Value, GameApiError> {
        self.rpc
            .call_sync_raw(method, params)
            .map_err(|e| GameApiError::RpcError(e.to_string()))
    }

    /// Helper: call with no params, return map.
    fn call_map(&self, method: &str) -> Result<HashMap<String, rmpv::Value>, GameApiError> {
        self.rpc
            .call_sync(method, HashMap::new())
            .map_err(|e| GameApiError::RpcError(e.to_string()))
    }

    /// Helper: call with params, return map.
    fn call_map_with(&self, method: &str, params: HashMap<String, rmpv::Value>) -> Result<HashMap<String, rmpv::Value>, GameApiError> {
        self.rpc
            .call_sync(method, params)
            .map_err(|e| GameApiError::RpcError(e.to_string()))
    }

    /// Helper: call with no params, return list.
    fn call_list(&self, method: &str) -> Result<Vec<rmpv::Value>, GameApiError> {
        self.rpc
            .call_sync_list(method, HashMap::new())
            .map_err(|e| GameApiError::RpcError(e.to_string()))
    }

    /// Helper: call with params, return list.
    fn call_list_with(&self, method: &str, params: HashMap<String, rmpv::Value>) -> Result<Vec<rmpv::Value>, GameApiError> {
        self.rpc
            .call_sync_list(method, params)
            .map_err(|e| GameApiError::RpcError(e.to_string()))
    }

    fn params_1<K: Into<String>>(k: K, v: rmpv::Value) -> HashMap<String, rmpv::Value> {
        let mut m = HashMap::new();
        m.insert(k.into(), v);
        m
    }

    fn params_2<K1: Into<String>, K2: Into<String>>(k1: K1, v1: rmpv::Value, k2: K2, v2: rmpv::Value) -> HashMap<String, rmpv::Value> {
        let mut m = HashMap::new();
        m.insert(k1.into(), v1);
        m.insert(k2.into(), v2);
        m
    }

    fn params_3<K1: Into<String>, K2: Into<String>, K3: Into<String>>(k1: K1, v1: rmpv::Value, k2: K2, v2: rmpv::Value, k3: K3, v3: rmpv::Value) -> HashMap<String, rmpv::Value> {
        let mut m = HashMap::new();
        m.insert(k1.into(), v1);
        m.insert(k2.into(), v2);
        m.insert(k3.into(), v3);
        m
    }

    fn int(v: i64) -> rmpv::Value { rmpv::Value::Integer(v.into()) }
    fn str_val(v: &str) -> rmpv::Value { rmpv::Value::String(v.into()) }
    fn bool_val(v: bool) -> rmpv::Value { rmpv::Value::Boolean(v) }
    fn float_val(v: f64) -> rmpv::Value { rmpv::Value::F64(v) }

    /// Parse an EntityInfo from a response map.
    fn parse_entity(map: &HashMap<String, rmpv::Value>) -> EntityInfo {
        EntityInfo {
            handle: MapHelper::get_int(map, "handle"),
            name: MapHelper::get_string(map, "name"),
            position: WorldPosition {
                x: MapHelper::get_int(map, "tile_x") as i32,
                y: MapHelper::get_int(map, "tile_y") as i32,
                z: MapHelper::get_int(map, "plane") as i32,
            },
            entity_type: MapHelper::get_string(map, "type"),
            type_id: MapHelper::get_int(map, "type_id"),
            animation_id: MapHelper::get_int(map, "animation") as i32,
            health: MapHelper::get_int(map, "health") as i32,
            max_health: MapHelper::get_int(map, "max_health") as i32,
            overhead_text: MapHelper::get_string(map, "overhead_text"),
            moving: MapHelper::get_bool(map, "moving"),
            in_combat: MapHelper::get_bool(map, "in_combat"),
        }
    }

    fn parse_entity_list(&self, list: Vec<rmpv::Value>) -> Vec<EntityInfo> {
        list.into_iter()
            .filter_map(|v| {
                if let rmpv::Value::Map(pairs) = v {
                    let map = pairs_to_map(pairs);
                    Some(Self::parse_entity(&map))
                } else {
                    None
                }
            })
            .collect()
    }

    fn parse_inventory_item(map: &HashMap<String, rmpv::Value>) -> InventoryItem {
        InventoryItem {
            item_id: MapHelper::get_int(map, "item_id"),
            name: MapHelper::get_string(map, "name"),
            quantity: MapHelper::get_int(map, "quantity") as i32,
            slot: MapHelper::get_int(map, "slot") as i32,
            inventory_id: MapHelper::get_int(map, "inventory_id") as i32,
        }
    }

    fn parse_player_stat(map: &HashMap<String, rmpv::Value>) -> PlayerStat {
        PlayerStat {
            skill_id: MapHelper::get_int(map, "skill_id") as i32,
            name: MapHelper::get_string(map, "name"),
            level: MapHelper::get_int(map, "level") as i32,
            boosted_level: MapHelper::get_int(map, "boosted_level") as i32,
            experience: MapHelper::get_int(map, "experience"),
        }
    }

    fn parse_component(map: &HashMap<String, rmpv::Value>) -> ComponentInfo {
        ComponentInfo {
            interface_id: MapHelper::get_int(map, "interface_id") as i32,
            component_id: MapHelper::get_int(map, "component_id") as i32,
            sub_component_id: MapHelper::get_int(map, "sub_component_id") as i32,
            text: MapHelper::get_string(map, "text"),
            item_id: MapHelper::get_int(map, "item_id"),
            item_quantity: MapHelper::get_int(map, "item_quantity") as i32,
            sprite_id: MapHelper::get_int(map, "sprite_id") as i32,
            component_type: MapHelper::get_int(map, "type") as i32,
            visible: MapHelper::get_bool(map, "visible"),
            position: (
                MapHelper::get_int(map, "x") as i32,
                MapHelper::get_int(map, "y") as i32,
                MapHelper::get_int(map, "width") as i32,
                MapHelper::get_int(map, "height") as i32,
            ),
            options: MapHelper::get_string_list(map, "options"),
        }
    }

    fn parse_ground_item(map: &HashMap<String, rmpv::Value>) -> GroundItemInfo {
        GroundItemInfo {
            handle: MapHelper::get_int(map, "handle"),
            item_id: MapHelper::get_int(map, "item_id"),
            name: MapHelper::get_string(map, "name"),
            quantity: MapHelper::get_int(map, "quantity") as i32,
            position: WorldPosition {
                x: MapHelper::get_int(map, "tile_x") as i32,
                y: MapHelper::get_int(map, "tile_y") as i32,
                z: MapHelper::get_int(map, "plane") as i32,
            },
        }
    }
}

fn pairs_to_map(pairs: Vec<(rmpv::Value, rmpv::Value)>) -> HashMap<String, rmpv::Value> {
    let mut map = HashMap::with_capacity(pairs.len());
    for (k, v) in pairs {
        let key = match k {
            rmpv::Value::String(s) => s.into_str().unwrap_or_default().to_string(),
            _ => k.to_string(),
        };
        map.insert(key, v);
    }
    map
}

fn value_to_map_list(list: Vec<rmpv::Value>) -> Vec<HashMap<String, rmpv::Value>> {
    list.into_iter()
        .filter_map(|v| {
            if let rmpv::Value::Map(pairs) = v {
                Some(pairs_to_map(pairs))
            } else {
                None
            }
        })
        .collect()
}

impl GameApi for RpcGameApi {
    // ===== System Methods =====

    fn ping(&self) -> Result<bool, GameApiError> {
        self.call_raw("rpc.ping")?;
        Ok(true)
    }

    fn list_methods(&self) -> Result<Vec<String>, GameApiError> {
        let list = self.call_list("rpc.list_methods")?;
        Ok(list.into_iter().filter_map(|v| v.as_str().map(|s| s.to_string())).collect())
    }

    fn subscribe(&self, event: &str) -> Result<bool, GameApiError> {
        let r = self.call_raw_with("rpc.subscribe", Self::params_1("event", Self::str_val(event)))?;
        Ok(r.as_bool().unwrap_or(true))
    }

    fn unsubscribe(&self, event: &str) -> Result<bool, GameApiError> {
        let r = self.call_raw_with("rpc.unsubscribe", Self::params_1("event", Self::str_val(event)))?;
        Ok(r.as_bool().unwrap_or(true))
    }

    fn get_client_count(&self) -> Result<i32, GameApiError> {
        let r = self.call_raw("rpc.client_count")?;
        Ok(r.as_i64().unwrap_or(0) as i32)
    }

    fn list_events(&self) -> Result<Vec<String>, GameApiError> {
        let list = self.call_list("rpc.list_events")?;
        Ok(list.into_iter().filter_map(|v| v.as_str().map(|s| s.to_string())).collect())
    }

    fn get_subscriptions(&self) -> Result<Vec<String>, GameApiError> {
        let list = self.call_list("rpc.get_subscriptions")?;
        Ok(list.into_iter().filter_map(|v| v.as_str().map(|s| s.to_string())).collect())
    }

    // ===== Action Queue Methods =====

    fn queue_action(&self, action: &GameAction) -> Result<bool, GameApiError> {
        let mut params = HashMap::new();
        params.insert("action_id".into(), Self::int(action.action_id as i64));
        params.insert("param1".into(), Self::int(action.param1));
        params.insert("param2".into(), Self::int(action.param2));
        params.insert("param3".into(), Self::int(action.param3));
        let r = self.call_raw_with("queue_action", params)?;
        Ok(r.as_bool().unwrap_or(true))
    }

    fn queue_actions(&self, actions: &[GameAction]) -> Result<bool, GameApiError> {
        let list: Vec<rmpv::Value> = actions.iter().map(|a| {
            rmpv::Value::Map(vec![
                (rmpv::Value::String("action_id".into()), Self::int(a.action_id as i64)),
                (rmpv::Value::String("param1".into()), Self::int(a.param1)),
                (rmpv::Value::String("param2".into()), Self::int(a.param2)),
                (rmpv::Value::String("param3".into()), Self::int(a.param3)),
            ])
        }).collect();
        let r = self.call_raw_with("queue_actions", Self::params_1("actions", rmpv::Value::Array(list)))?;
        Ok(r.as_bool().unwrap_or(true))
    }

    fn get_action_queue_size(&self) -> Result<i32, GameApiError> {
        let r = self.call_raw("get_action_queue_size")?;
        Ok(r.as_i64().unwrap_or(0) as i32)
    }

    fn clear_action_queue(&self) -> Result<(), GameApiError> {
        self.call_raw("clear_action_queue")?;
        Ok(())
    }

    fn get_action_history(&self, max_results: i32, action_id_filter: Option<i32>) -> Result<Vec<GameAction>, GameApiError> {
        let mut params = Self::params_1("max_results", Self::int(max_results as i64));
        if let Some(id) = action_id_filter {
            params.insert("action_id".into(), Self::int(id as i64));
        }
        let list = self.call_list_with("get_action_history", params)?;
        Ok(value_to_map_list(list).into_iter().map(|m| {
            GameAction {
                action_id: MapHelper::get_int(&m, "action_id") as i32,
                param1: MapHelper::get_int(&m, "param1"),
                param2: MapHelper::get_int(&m, "param2"),
                param3: MapHelper::get_int(&m, "param3"),
            }
        }).collect())
    }

    fn get_last_action_time(&self) -> Result<i64, GameApiError> {
        let r = self.call_raw("get_last_action_time")?;
        Ok(r.as_i64().unwrap_or(0))
    }

    fn set_behavior_mod(&self, mod_id: &str, value: f64) -> Result<(), GameApiError> {
        self.call_raw_with("set_behavior_mod", Self::params_2("mod_id", Self::str_val(mod_id), "value", Self::float_val(value)))?;
        Ok(())
    }

    fn clear_behavior_mod(&self, mod_id: &str) -> Result<(), GameApiError> {
        self.call_raw_with("clear_behavior_mod", Self::params_1("mod_id", Self::str_val(mod_id)))?;
        Ok(())
    }

    fn get_behavior_mod(&self, mod_id: &str) -> Result<f64, GameApiError> {
        let r = self.call_raw_with("get_behavior_mod", Self::params_1("mod_id", Self::str_val(mod_id)))?;
        Ok(r.as_f64().unwrap_or(0.0))
    }

    fn are_actions_blocked(&self) -> Result<bool, GameApiError> {
        let r = self.call_raw("are_actions_blocked")?;
        Ok(r.as_bool().unwrap_or(false))
    }

    fn set_actions_blocked(&self, blocked: bool) -> Result<(), GameApiError> {
        self.call_raw_with("set_actions_blocked", Self::params_1("blocked", Self::bool_val(blocked)))?;
        Ok(())
    }

    // ===== Entity Query Methods =====

    fn query_entities(&self, filter: &HashMap<String, serde_json::Value>) -> Result<Vec<EntityInfo>, GameApiError> {
        let list = self.call_list_with("query_entities", json_filter_to_rmpv(filter))?;
        Ok(self.parse_entity_list(list))
    }

    fn get_entity_info(&self, handle: i64) -> Result<EntityInfo, GameApiError> {
        let map = self.call_map_with("get_entity_info", Self::params_1("handle", Self::int(handle)))?;
        Ok(Self::parse_entity(&map))
    }

    fn get_entity_name(&self, handle: i64) -> Result<String, GameApiError> {
        let r = self.call_raw_with("get_entity_name", Self::params_1("handle", Self::int(handle)))?;
        Ok(r.as_str().unwrap_or("").to_string())
    }

    fn get_entity_health(&self, handle: i64) -> Result<(i32, i32), GameApiError> {
        let map = self.call_map_with("get_entity_health", Self::params_1("handle", Self::int(handle)))?;
        Ok((MapHelper::get_int(&map, "current") as i32, MapHelper::get_int(&map, "max") as i32))
    }

    fn get_entity_position(&self, handle: i64) -> Result<WorldPosition, GameApiError> {
        let map = self.call_map_with("get_entity_position", Self::params_1("handle", Self::int(handle)))?;
        Ok(WorldPosition {
            x: MapHelper::get_int(&map, "tile_x") as i32,
            y: MapHelper::get_int(&map, "tile_y") as i32,
            z: MapHelper::get_int(&map, "plane") as i32,
        })
    }

    fn is_entity_valid(&self, handle: i64) -> Result<bool, GameApiError> {
        let r = self.call_raw_with("is_entity_valid", Self::params_1("handle", Self::int(handle)))?;
        Ok(r.as_bool().unwrap_or(false))
    }

    fn get_entity_hitmarks(&self, handle: i64) -> Result<Vec<Hitmark>, GameApiError> {
        let list = self.call_list_with("get_entity_hitmarks", Self::params_1("handle", Self::int(handle)))?;
        Ok(value_to_map_list(list).into_iter().map(|m| Hitmark {
            damage: MapHelper::get_int(&m, "damage") as i32,
            hitmark_type: MapHelper::get_int(&m, "type") as i32,
            cycle: MapHelper::get_int(&m, "cycle") as i32,
        }).collect())
    }

    fn get_entity_animation(&self, handle: i64) -> Result<i32, GameApiError> {
        let r = self.call_raw_with("get_entity_animation", Self::params_1("handle", Self::int(handle)))?;
        Ok(r.as_i64().unwrap_or(-1) as i32)
    }

    fn get_entity_overhead_text(&self, handle: i64) -> Result<String, GameApiError> {
        let r = self.call_raw_with("get_entity_overhead_text", Self::params_1("handle", Self::int(handle)))?;
        Ok(r.as_str().unwrap_or("").to_string())
    }

    fn get_animation_length(&self, animation_id: i32) -> Result<i32, GameApiError> {
        let r = self.call_raw_with("get_animation_length", Self::params_1("animation_id", Self::int(animation_id as i64)))?;
        Ok(r.as_i64().unwrap_or(0) as i32)
    }

    // ===== Ground Items =====

    fn query_ground_items(&self, filter: &HashMap<String, serde_json::Value>) -> Result<Vec<GroundItemInfo>, GameApiError> {
        let list = self.call_list_with("query_ground_items", json_filter_to_rmpv(filter))?;
        Ok(value_to_map_list(list).into_iter().map(|m| Self::parse_ground_item(&m)).collect())
    }

    fn get_obj_stack_items(&self, handle: i64) -> Result<Vec<GroundItemInfo>, GameApiError> {
        let list = self.call_list_with("get_obj_stack_items", Self::params_1("handle", Self::int(handle)))?;
        Ok(value_to_map_list(list).into_iter().map(|m| Self::parse_ground_item(&m)).collect())
    }

    fn query_obj_stacks(&self, filter: &HashMap<String, serde_json::Value>) -> Result<Vec<GroundItemInfo>, GameApiError> {
        let list = self.call_list_with("query_obj_stacks", json_filter_to_rmpv(filter))?;
        Ok(value_to_map_list(list).into_iter().map(|m| Self::parse_ground_item(&m)).collect())
    }

    // ===== Projectiles & Effects =====

    fn query_projectiles(&self, projectile_id: Option<i32>, plane: Option<i32>, max_results: i32) -> Result<Vec<ProjectileInfo>, GameApiError> {
        let mut params = Self::params_1("max_results", Self::int(max_results as i64));
        if let Some(id) = projectile_id { params.insert("projectile_id".into(), Self::int(id as i64)); }
        if let Some(p) = plane { params.insert("plane".into(), Self::int(p as i64)); }
        let list = self.call_list_with("query_projectiles", params)?;
        Ok(value_to_map_list(list).into_iter().map(|m| ProjectileInfo {
            id: MapHelper::get_int(&m, "id") as i32,
            start_position: WorldPosition {
                x: MapHelper::get_int(&m, "start_x") as i32,
                y: MapHelper::get_int(&m, "start_y") as i32,
                z: MapHelper::get_int(&m, "plane") as i32,
            },
            end_position: WorldPosition {
                x: MapHelper::get_int(&m, "end_x") as i32,
                y: MapHelper::get_int(&m, "end_y") as i32,
                z: MapHelper::get_int(&m, "plane") as i32,
            },
            target_handle: MapHelper::get_int(&m, "target_handle"),
            remaining_cycles: MapHelper::get_int(&m, "remaining_cycles") as i32,
        }).collect())
    }

    fn query_spot_anims(&self, anim_id: Option<i32>, plane: Option<i32>, max_results: i32) -> Result<Vec<HashMap<String, serde_json::Value>>, GameApiError> {
        let mut params = Self::params_1("max_results", Self::int(max_results as i64));
        if let Some(id) = anim_id { params.insert("anim_id".into(), Self::int(id as i64)); }
        if let Some(p) = plane { params.insert("plane".into(), Self::int(p as i64)); }
        let list = self.call_list_with("query_spot_anims", params)?;
        Ok(value_to_map_list(list).into_iter().map(|m| {
            m.into_iter().map(|(k, v)| (k, rmpv_to_json(&v))).collect()
        }).collect())
    }

    fn query_hint_arrows(&self, max_results: i32) -> Result<Vec<HashMap<String, serde_json::Value>>, GameApiError> {
        let list = self.call_list_with("query_hint_arrows", Self::params_1("max_results", Self::int(max_results as i64)))?;
        Ok(value_to_map_list(list).into_iter().map(|m| {
            m.into_iter().map(|(k, v)| (k, rmpv_to_json(&v))).collect()
        }).collect())
    }

    // ===== World & Navigation =====

    fn query_worlds(&self, include_activity: bool) -> Result<Vec<WorldInfo>, GameApiError> {
        let list = self.call_list_with("query_worlds", Self::params_1("include_activity", Self::bool_val(include_activity)))?;
        Ok(value_to_map_list(list).into_iter().map(|m| WorldInfo {
            id: MapHelper::get_int(&m, "id") as i32,
            members: MapHelper::get_bool(&m, "members"),
            population: MapHelper::get_int(&m, "population") as i32,
            location: MapHelper::get_string(&m, "location"),
            activity: MapHelper::get_string(&m, "activity"),
        }).collect())
    }

    fn get_current_world(&self) -> Result<WorldInfo, GameApiError> {
        let map = self.call_map("get_current_world")?;
        Ok(WorldInfo {
            id: MapHelper::get_int(&map, "id") as i32,
            members: MapHelper::get_bool(&map, "members"),
            population: MapHelper::get_int(&map, "population") as i32,
            location: MapHelper::get_string(&map, "location"),
            activity: MapHelper::get_string(&map, "activity"),
        })
    }

    fn compute_name_hash(&self, name: &str) -> Result<i64, GameApiError> {
        let r = self.call_raw_with("compute_name_hash", Self::params_1("name", Self::str_val(name)))?;
        Ok(r.as_i64().unwrap_or(0))
    }

    fn update_query_context(&self) -> Result<(), GameApiError> {
        self.call_raw("update_query_context")?;
        Ok(())
    }

    fn invalidate_query_context(&self) -> Result<(), GameApiError> {
        self.call_raw("invalidate_query_context")?;
        Ok(())
    }

    // ===== UI Component Methods =====

    fn query_components(&self, filter: &HashMap<String, serde_json::Value>) -> Result<Vec<ComponentInfo>, GameApiError> {
        let list = self.call_list_with("query_components", json_filter_to_rmpv(filter))?;
        Ok(value_to_map_list(list).into_iter().map(|m| Self::parse_component(&m)).collect())
    }

    fn is_component_valid(&self, interface_id: i32, component_id: i32, sub_component_id: i32) -> Result<bool, GameApiError> {
        let r = self.call_raw_with("is_component_valid", Self::params_3(
            "interface_id", Self::int(interface_id as i64),
            "component_id", Self::int(component_id as i64),
            "sub_component_id", Self::int(sub_component_id as i64),
        ))?;
        Ok(r.as_bool().unwrap_or(false))
    }

    fn get_component_text(&self, interface_id: i32, component_id: i32) -> Result<String, GameApiError> {
        let r = self.call_raw_with("get_component_text", Self::params_2(
            "interface_id", Self::int(interface_id as i64),
            "component_id", Self::int(component_id as i64),
        ))?;
        Ok(r.as_str().unwrap_or("").to_string())
    }

    fn get_component_item(&self, interface_id: i32, component_id: i32, sub_component_id: i32) -> Result<(i64, i32), GameApiError> {
        let map = self.call_map_with("get_component_item", Self::params_3(
            "interface_id", Self::int(interface_id as i64),
            "component_id", Self::int(component_id as i64),
            "sub_component_id", Self::int(sub_component_id as i64),
        ))?;
        Ok((MapHelper::get_int(&map, "item_id"), MapHelper::get_int(&map, "quantity") as i32))
    }

    fn get_component_position(&self, interface_id: i32, component_id: i32) -> Result<(i32, i32, i32, i32), GameApiError> {
        let map = self.call_map_with("get_component_position", Self::params_2(
            "interface_id", Self::int(interface_id as i64),
            "component_id", Self::int(component_id as i64),
        ))?;
        Ok((
            MapHelper::get_int(&map, "x") as i32,
            MapHelper::get_int(&map, "y") as i32,
            MapHelper::get_int(&map, "width") as i32,
            MapHelper::get_int(&map, "height") as i32,
        ))
    }

    fn get_component_options(&self, interface_id: i32, component_id: i32) -> Result<Vec<String>, GameApiError> {
        let map = self.call_map_with("get_component_options", Self::params_2(
            "interface_id", Self::int(interface_id as i64),
            "component_id", Self::int(component_id as i64),
        ))?;
        Ok(MapHelper::get_string_list(&map, "options"))
    }

    fn get_component_sprite_id(&self, interface_id: i32, component_id: i32) -> Result<i32, GameApiError> {
        let r = self.call_raw_with("get_component_sprite_id", Self::params_2(
            "interface_id", Self::int(interface_id as i64),
            "component_id", Self::int(component_id as i64),
        ))?;
        Ok(r.as_i64().unwrap_or(-1) as i32)
    }

    fn get_component_type(&self, interface_id: i32, component_id: i32) -> Result<i32, GameApiError> {
        let r = self.call_raw_with("get_component_type", Self::params_2(
            "interface_id", Self::int(interface_id as i64),
            "component_id", Self::int(component_id as i64),
        ))?;
        Ok(r.as_i64().unwrap_or(0) as i32)
    }

    fn get_component_children(&self, interface_id: i32, component_id: i32) -> Result<Vec<ComponentInfo>, GameApiError> {
        let list = self.call_list_with("get_component_children", Self::params_2(
            "interface_id", Self::int(interface_id as i64),
            "component_id", Self::int(component_id as i64),
        ))?;
        Ok(value_to_map_list(list).into_iter().map(|m| Self::parse_component(&m)).collect())
    }

    fn get_component_by_hash(&self, interface_id: i32, component_id: i32, sub_component_id: i32) -> Result<ComponentInfo, GameApiError> {
        let map = self.call_map_with("get_component_by_hash", Self::params_3(
            "interface_id", Self::int(interface_id as i64),
            "component_id", Self::int(component_id as i64),
            "sub_component_id", Self::int(sub_component_id as i64),
        ))?;
        Ok(Self::parse_component(&map))
    }

    fn get_open_interfaces(&self) -> Result<Vec<i32>, GameApiError> {
        let list = self.call_list("get_open_interfaces")?;
        Ok(list.into_iter().filter_map(|v| v.as_i64().map(|i| i as i32)).collect())
    }

    fn is_interface_open(&self, interface_id: i32) -> Result<bool, GameApiError> {
        let r = self.call_raw_with("is_interface_open", Self::params_1("interface_id", Self::int(interface_id as i64)))?;
        Ok(r.as_bool().unwrap_or(false))
    }

    // ===== Game Variables =====

    fn get_varp(&self, var_id: i32) -> Result<i32, GameApiError> {
        let r = self.call_raw_with("get_varp", Self::params_1("var_id", Self::int(var_id as i64)))?;
        Ok(r.as_i64().unwrap_or(0) as i32)
    }

    fn get_varbit(&self, varbit_id: i32) -> Result<i32, GameApiError> {
        let r = self.call_raw_with("get_varbit", Self::params_1("varbit_id", Self::int(varbit_id as i64)))?;
        Ok(r.as_i64().unwrap_or(0) as i32)
    }

    fn get_varc_int(&self, varc_id: i32) -> Result<i32, GameApiError> {
        let r = self.call_raw_with("get_varc_int", Self::params_1("varc_id", Self::int(varc_id as i64)))?;
        Ok(r.as_i64().unwrap_or(0) as i32)
    }

    fn get_varc_string(&self, varc_id: i32) -> Result<String, GameApiError> {
        let r = self.call_raw_with("get_varc_string", Self::params_1("varc_id", Self::int(varc_id as i64)))?;
        Ok(r.as_str().unwrap_or("").to_string())
    }

    fn query_varbits(&self, varbit_ids: &[i32]) -> Result<HashMap<i32, i32>, GameApiError> {
        let ids: Vec<rmpv::Value> = varbit_ids.iter().map(|&id| Self::int(id as i64)).collect();
        let map = self.call_map_with("query_varbits", Self::params_1("varbit_ids", rmpv::Value::Array(ids)))?;
        let mut result = HashMap::new();
        for (k, v) in map {
            if let Ok(id) = k.parse::<i32>() {
                result.insert(id, v.as_i64().unwrap_or(0) as i32);
            }
        }
        Ok(result)
    }

    // ===== Script Execution =====

    fn get_script_handle(&self, script_id: i32) -> Result<i64, GameApiError> {
        let r = self.call_raw_with("get_script_handle", Self::params_1("script_id", Self::int(script_id as i64)))?;
        Ok(r.as_i64().unwrap_or(0))
    }

    fn execute_script(&self, handle: i64, int_args: &[i32], string_args: &[String], returns: &[String]) -> Result<HashMap<String, serde_json::Value>, GameApiError> {
        let mut params = HashMap::new();
        params.insert("handle".into(), Self::int(handle));
        params.insert("int_args".into(), rmpv::Value::Array(int_args.iter().map(|&i| Self::int(i as i64)).collect()));
        params.insert("string_args".into(), rmpv::Value::Array(string_args.iter().map(|s| Self::str_val(s)).collect()));
        params.insert("returns".into(), rmpv::Value::Array(returns.iter().map(|s| Self::str_val(s)).collect()));
        let map = self.call_map_with("execute_script", params)?;
        Ok(map.into_iter().map(|(k, v)| (k, rmpv_to_json(&v))).collect())
    }

    fn destroy_script_handle(&self, handle: i64) -> Result<(), GameApiError> {
        self.call_raw_with("destroy_script_handle", Self::params_1("handle", Self::int(handle)))?;
        Ok(())
    }

    fn fire_key_trigger(&self, interface_id: i32, component_id: i32, input: i32) -> Result<(), GameApiError> {
        self.call_raw_with("fire_key_trigger", Self::params_3(
            "interface_id", Self::int(interface_id as i64),
            "component_id", Self::int(component_id as i64),
            "input", Self::int(input as i64),
        ))?;
        Ok(())
    }

    // ===== Game State =====

    fn get_local_player(&self) -> Result<EntityInfo, GameApiError> {
        let map = self.call_map("get_local_player")?;
        Ok(Self::parse_entity(&map))
    }

    fn get_account_info(&self) -> Result<AccountInfo, GameApiError> {
        let map = self.call_map("get_account_info")?;
        Ok(AccountInfo {
            display_name: MapHelper::get_string(&map, "display_name"),
            login_state: MapHelper::get_int(&map, "login_state") as i32,
            world: MapHelper::get_int(&map, "world") as i32,
            members: MapHelper::get_bool(&map, "members"),
        })
    }

    fn get_game_cycle(&self) -> Result<i64, GameApiError> {
        let r = self.call_raw("get_game_cycle")?;
        Ok(r.as_i64().unwrap_or(0))
    }

    fn get_login_state(&self) -> Result<i32, GameApiError> {
        let r = self.call_raw("get_login_state")?;
        Ok(r.as_i64().unwrap_or(0) as i32)
    }

    fn get_mini_menu(&self) -> Result<Vec<MiniMenuEntry>, GameApiError> {
        let list = self.call_list("get_mini_menu")?;
        Ok(value_to_map_list(list).into_iter().map(|m| MiniMenuEntry {
            option: MapHelper::get_string(&m, "option"),
            target: MapHelper::get_string(&m, "target"),
            action_id: MapHelper::get_int(&m, "action_id") as i32,
            param1: MapHelper::get_int(&m, "param1"),
            param2: MapHelper::get_int(&m, "param2"),
        }).collect())
    }

    fn get_grand_exchange_offers(&self) -> Result<Vec<GrandExchangeOffer>, GameApiError> {
        let list = self.call_list("get_grand_exchange_offers")?;
        Ok(value_to_map_list(list).into_iter().map(|m| GrandExchangeOffer {
            slot: MapHelper::get_int(&m, "slot") as i32,
            item_id: MapHelper::get_int(&m, "item_id"),
            quantity: MapHelper::get_int(&m, "quantity") as i32,
            price: MapHelper::get_int(&m, "price") as i32,
            transferred: MapHelper::get_int(&m, "transferred") as i32,
            spent: MapHelper::get_int(&m, "spent"),
            state: MapHelper::get_int(&m, "state") as i32,
        }).collect())
    }

    fn get_world_to_screen(&self, tile_x: i32, tile_y: i32) -> Result<ScreenPosition, GameApiError> {
        let map = self.call_map_with("get_world_to_screen", Self::params_2(
            "tile_x", Self::int(tile_x as i64),
            "tile_y", Self::int(tile_y as i64),
        ))?;
        Ok(ScreenPosition {
            x: MapHelper::get_int(&map, "x") as i32,
            y: MapHelper::get_int(&map, "y") as i32,
        })
    }

    fn batch_world_to_screen(&self, positions: &[(i32, i32)]) -> Result<Vec<ScreenPosition>, GameApiError> {
        let pos_list: Vec<rmpv::Value> = positions.iter().map(|&(x, y)| {
            rmpv::Value::Array(vec![Self::int(x as i64), Self::int(y as i64)])
        }).collect();
        let list = self.call_list_with("batch_world_to_screen", Self::params_1("positions", rmpv::Value::Array(pos_list)))?;
        Ok(value_to_map_list(list).into_iter().map(|m| ScreenPosition {
            x: MapHelper::get_int(&m, "x") as i32,
            y: MapHelper::get_int(&m, "y") as i32,
        }).collect())
    }

    fn get_viewport_info(&self) -> Result<ViewportInfo, GameApiError> {
        let map = self.call_map("get_viewport_info")?;
        Ok(ViewportInfo {
            camera_x: MapHelper::get_int(&map, "camera_x") as i32,
            camera_y: MapHelper::get_int(&map, "camera_y") as i32,
            camera_z: MapHelper::get_int(&map, "camera_z") as i32,
            camera_yaw: MapHelper::get_int(&map, "camera_yaw") as i32,
            camera_pitch: MapHelper::get_int(&map, "camera_pitch") as i32,
        })
    }

    fn get_entity_screen_positions(&self, handles: &[i64]) -> Result<Vec<ScreenPosition>, GameApiError> {
        let ids: Vec<rmpv::Value> = handles.iter().map(|&h| Self::int(h)).collect();
        let list = self.call_list_with("get_entity_screen_positions", Self::params_1("handles", rmpv::Value::Array(ids)))?;
        Ok(value_to_map_list(list).into_iter().map(|m| ScreenPosition {
            x: MapHelper::get_int(&m, "x") as i32,
            y: MapHelper::get_int(&m, "y") as i32,
        }).collect())
    }

    fn get_game_window_rect(&self) -> Result<GameRect, GameApiError> {
        let map = self.call_map("get_game_window_rect")?;
        Ok(GameRect {
            x: MapHelper::get_int(&map, "x") as i32,
            y: MapHelper::get_int(&map, "y") as i32,
            width: MapHelper::get_int(&map, "width") as i32,
            height: MapHelper::get_int(&map, "height") as i32,
        })
    }

    fn set_world(&self, world_id: i32) -> Result<(), GameApiError> {
        self.call_raw_with("set_world", Self::params_1("world_id", Self::int(world_id as i64)))?;
        Ok(())
    }

    fn change_login_state(&self) -> Result<(), GameApiError> {
        self.call_raw("change_login_state")?;
        Ok(())
    }

    fn login_to_lobby(&self) -> Result<(), GameApiError> {
        self.call_raw("login_to_lobby")?;
        Ok(())
    }

    // ===== Cache =====

    fn get_cache_file(&self, index_id: i32, archive_id: i32, file_id: i32) -> Result<Vec<u8>, GameApiError> {
        let map = self.call_map_with("get_cache_file", Self::params_3(
            "index_id", Self::int(index_id as i64),
            "archive_id", Self::int(archive_id as i64),
            "file_id", Self::int(file_id as i64),
        ))?;
        Ok(MapHelper::get_bytes(&map, "data"))
    }

    fn get_cache_file_count(&self, index_id: i32, archive_id: i32, shift: i32) -> Result<i32, GameApiError> {
        let r = self.call_raw_with("get_cache_file_count", Self::params_3(
            "index_id", Self::int(index_id as i64),
            "archive_id", Self::int(archive_id as i64),
            "shift", Self::int(shift as i64),
        ))?;
        Ok(r.as_i64().unwrap_or(0) as i32)
    }

    fn get_navigation_archive(&self) -> Result<Vec<u8>, GameApiError> {
        let map = self.call_map("get_navigation_archive")?;
        Ok(MapHelper::get_bytes(&map, "data"))
    }

    // ===== Breaks & Scheduling =====

    fn schedule_break(&self) -> Result<(), GameApiError> {
        self.call_raw("schedule_break")?;
        Ok(())
    }

    fn interrupt_break(&self) -> Result<(), GameApiError> {
        self.call_raw("interrupt_break")?;
        Ok(())
    }

    // ===== Auto Login =====

    fn get_auto_login(&self) -> Result<bool, GameApiError> {
        let r = self.call_raw("get_auto_login")?;
        Ok(r.as_bool().unwrap_or(false))
    }

    fn set_auto_login(&self, enabled: bool) -> Result<(), GameApiError> {
        self.call_raw_with("set_auto_login", Self::params_1("enabled", Self::bool_val(enabled)))?;
        Ok(())
    }

    // ===== Screenshots =====

    fn take_screenshot(&self) -> Result<Vec<u8>, GameApiError> {
        let map = self.call_map("take_screenshot")?;
        Ok(MapHelper::get_bytes(&map, "data"))
    }

    // ===== Streaming =====

    fn start_stream(&self, frame_skip: u32, quality: u32, width: u32, height: u32) -> Result<StreamInfo, GameApiError> {
        let mut params = HashMap::new();
        params.insert("frame_skip".into(), Self::int(frame_skip as i64));
        params.insert("quality".into(), Self::int(quality as i64));
        params.insert("width".into(), Self::int(width as i64));
        params.insert("height".into(), Self::int(height as i64));
        let map = self.call_map_with("start_stream", params)?;
        Ok(StreamInfo {
            pipe_name: MapHelper::get_string(&map, "pipe_name"),
            width: MapHelper::get_int(&map, "width") as u32,
            height: MapHelper::get_int(&map, "height") as u32,
            quality: MapHelper::get_int(&map, "quality") as u32,
            frame_skip: MapHelper::get_int(&map, "frame_skip") as u32,
        })
    }

    fn stop_stream(&self) -> Result<(), GameApiError> {
        self.call_raw("stop_stream")?;
        Ok(())
    }

    // ===== Humanization =====

    fn get_humanization_enabled(&self) -> Result<bool, GameApiError> {
        let r = self.call_raw("get_humanization_enabled")?;
        Ok(r.as_bool().unwrap_or(false))
    }

    fn set_humanization_enabled(&self, enabled: bool) -> Result<(), GameApiError> {
        self.call_raw_with("set_humanization_enabled", Self::params_1("enabled", Self::bool_val(enabled)))?;
        Ok(())
    }

    fn get_personality(&self) -> Result<Personality, GameApiError> {
        let map = self.call_map("get_personality")?;
        Ok(Personality {
            reaction_speed: MapHelper::get_float(&map, "reaction_speed"),
            fatigue_level: MapHelper::get_float(&map, "fatigue_level"),
            attention_span: MapHelper::get_float(&map, "attention_span"),
            mouse_speed: MapHelper::get_float(&map, "mouse_speed"),
            click_accuracy: MapHelper::get_float(&map, "click_accuracy"),
        })
    }

    // ===== Inventory & Items =====

    fn query_inventories(&self) -> Result<Vec<HashMap<String, serde_json::Value>>, GameApiError> {
        let list = self.call_list("query_inventories")?;
        Ok(value_to_map_list(list).into_iter().map(|m| {
            m.into_iter().map(|(k, v)| (k, rmpv_to_json(&v))).collect()
        }).collect())
    }

    fn query_inventory_items(&self, filter: &HashMap<String, serde_json::Value>) -> Result<Vec<InventoryItem>, GameApiError> {
        let list = self.call_list_with("query_inventory_items", json_filter_to_rmpv(filter))?;
        Ok(value_to_map_list(list).into_iter().map(|m| Self::parse_inventory_item(&m)).collect())
    }

    fn get_inventory_item(&self, inventory_id: i32, slot: i32) -> Result<InventoryItem, GameApiError> {
        let map = self.call_map_with("get_inventory_item", Self::params_2(
            "inventory_id", Self::int(inventory_id as i64),
            "slot", Self::int(slot as i64),
        ))?;
        Ok(Self::parse_inventory_item(&map))
    }

    fn get_item_vars(&self, inventory_id: i32, slot: i32) -> Result<HashMap<i32, i32>, GameApiError> {
        let map = self.call_map_with("get_item_vars", Self::params_2(
            "inventory_id", Self::int(inventory_id as i64),
            "slot", Self::int(slot as i64),
        ))?;
        let mut result = HashMap::new();
        for (k, v) in map {
            if let Ok(id) = k.parse::<i32>() {
                result.insert(id, v.as_i64().unwrap_or(0) as i32);
            }
        }
        Ok(result)
    }

    fn get_item_var_value(&self, inventory_id: i32, slot: i32, var_id: i32) -> Result<i32, GameApiError> {
        let r = self.call_raw_with("get_item_var_value", Self::params_3(
            "inventory_id", Self::int(inventory_id as i64),
            "slot", Self::int(slot as i64),
            "var_id", Self::int(var_id as i64),
        ))?;
        Ok(r.as_i64().unwrap_or(0) as i32)
    }

    fn is_inventory_item_valid(&self, inventory_id: i32, slot: i32) -> Result<bool, GameApiError> {
        let r = self.call_raw_with("is_inventory_item_valid", Self::params_2(
            "inventory_id", Self::int(inventory_id as i64),
            "slot", Self::int(slot as i64),
        ))?;
        Ok(r.as_bool().unwrap_or(false))
    }

    // ===== Player Stats =====

    fn get_player_stats(&self) -> Result<Vec<PlayerStat>, GameApiError> {
        let list = self.call_list("get_player_stats")?;
        Ok(value_to_map_list(list).into_iter().map(|m| Self::parse_player_stat(&m)).collect())
    }

    fn get_player_stat(&self, skill_id: i32) -> Result<PlayerStat, GameApiError> {
        let map = self.call_map_with("get_player_stat", Self::params_1("skill_id", Self::int(skill_id as i64)))?;
        Ok(Self::parse_player_stat(&map))
    }

    fn get_player_stat_count(&self) -> Result<i32, GameApiError> {
        let r = self.call_raw("get_player_stat_count")?;
        Ok(r.as_i64().unwrap_or(0) as i32)
    }

    // ===== Chat =====

    fn query_chat_history(&self, message_type: Option<i32>, max_results: i32) -> Result<Vec<ChatMessage>, GameApiError> {
        let mut params = Self::params_1("max_results", Self::int(max_results as i64));
        if let Some(t) = message_type { params.insert("message_type".into(), Self::int(t as i64)); }
        let list = self.call_list_with("query_chat_history", params)?;
        Ok(value_to_map_list(list).into_iter().map(|m| ChatMessage {
            index: MapHelper::get_int(&m, "index") as i32,
            message_type: MapHelper::get_int(&m, "type") as i32,
            sender: MapHelper::get_string(&m, "sender"),
            text: MapHelper::get_string(&m, "text"),
        }).collect())
    }

    fn get_chat_message_text(&self, index: i32) -> Result<String, GameApiError> {
        let r = self.call_raw_with("get_chat_message_text", Self::params_1("index", Self::int(index as i64)))?;
        Ok(r.as_str().unwrap_or("").to_string())
    }

    fn get_chat_message_player(&self, index: i32) -> Result<String, GameApiError> {
        let r = self.call_raw_with("get_chat_message_player", Self::params_1("index", Self::int(index as i64)))?;
        Ok(r.as_str().unwrap_or("").to_string())
    }

    fn get_chat_message_type(&self, index: i32) -> Result<i32, GameApiError> {
        let r = self.call_raw_with("get_chat_message_type", Self::params_1("index", Self::int(index as i64)))?;
        Ok(r.as_i64().unwrap_or(0) as i32)
    }

    fn get_chat_history_size(&self) -> Result<i32, GameApiError> {
        let r = self.call_raw("get_chat_history_size")?;
        Ok(r.as_i64().unwrap_or(0) as i32)
    }

    // ===== Config Type Lookups =====

    fn get_item_type(&self, id: i32) -> Result<ConfigType, GameApiError> {
        self.get_config_type("get_item_type", id)
    }

    fn get_npc_type(&self, id: i32) -> Result<ConfigType, GameApiError> {
        self.get_config_type("get_npc_type", id)
    }

    fn get_location_type(&self, id: i32) -> Result<ConfigType, GameApiError> {
        self.get_config_type("get_location_type", id)
    }

    fn get_enum_type(&self, id: i32) -> Result<ConfigType, GameApiError> {
        self.get_config_type("get_enum_type", id)
    }

    fn get_struct_type(&self, id: i32) -> Result<ConfigType, GameApiError> {
        self.get_config_type("get_struct_type", id)
    }

    fn get_sequence_type(&self, id: i32) -> Result<ConfigType, GameApiError> {
        self.get_config_type("get_sequence_type", id)
    }

    fn get_quest_type(&self, id: i32) -> Result<ConfigType, GameApiError> {
        self.get_config_type("get_quest_type", id)
    }
}

impl RpcGameApi {
    fn get_config_type(&self, method: &str, id: i32) -> Result<ConfigType, GameApiError> {
        let map = self.call_map_with(method, Self::params_1("id", Self::int(id as i64)))?;
        Ok(ConfigType {
            id: MapHelper::get_int(&map, "id") as i32,
            name: MapHelper::get_string(&map, "name"),
            fields: HashMap::new(), // Simplified - full conversion would walk the map
        })
    }
}
