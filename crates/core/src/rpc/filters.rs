use std::collections::HashMap;

/// Filter for querying game entities (NPCs, players, objects).
/// Equivalent to Java's EntityFilter.
#[derive(Debug, Clone, Default)]
pub struct EntityFilter {
    pub entity_type: Option<String>,
    pub type_id: Option<i64>,
    pub name_hash: Option<i64>,
    pub name_pattern: Option<String>,
    pub match_type: Option<String>,
    pub case_sensitive: Option<bool>,
    pub plane: Option<i32>,
    pub tile_x: Option<i32>,
    pub tile_y: Option<i32>,
    pub radius: Option<i32>,
    pub visible_only: Option<bool>,
    pub moving_only: Option<bool>,
    pub stationary_only: Option<bool>,
    pub in_combat: Option<bool>,
    pub not_in_combat: Option<bool>,
    pub sort_by_distance: Option<bool>,
    pub max_results: Option<i32>,
}

impl EntityFilter {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn entity_type(mut self, t: impl Into<String>) -> Self {
        self.entity_type = Some(t.into());
        self
    }

    pub fn type_id(mut self, id: i64) -> Self {
        self.type_id = Some(id);
        self
    }

    pub fn name_pattern(mut self, pattern: impl Into<String>) -> Self {
        self.name_pattern = Some(pattern.into());
        self
    }

    pub fn plane(mut self, plane: i32) -> Self {
        self.plane = Some(plane);
        self
    }

    pub fn at_tile(mut self, x: i32, y: i32) -> Self {
        self.tile_x = Some(x);
        self.tile_y = Some(y);
        self
    }

    pub fn radius(mut self, radius: i32) -> Self {
        self.radius = Some(radius);
        self
    }

    pub fn visible_only(mut self, v: bool) -> Self {
        self.visible_only = Some(v);
        self
    }

    pub fn sort_by_distance(mut self, v: bool) -> Self {
        self.sort_by_distance = Some(v);
        self
    }

    pub fn max_results(mut self, n: i32) -> Self {
        self.max_results = Some(n);
        self
    }

    /// Convert to RPC parameter map.
    pub fn to_params(&self) -> HashMap<String, rmpv::Value> {
        let mut params = HashMap::new();
        if let Some(ref v) = self.entity_type {
            params.insert("type".into(), rmpv::Value::String(v.clone().into()));
        }
        if let Some(v) = self.type_id {
            params.insert("type_id".into(), rmpv::Value::Integer(v.into()));
        }
        if let Some(v) = self.name_hash {
            params.insert("name_hash".into(), rmpv::Value::Integer(v.into()));
        }
        if let Some(ref v) = self.name_pattern {
            params.insert("name_pattern".into(), rmpv::Value::String(v.clone().into()));
        }
        if let Some(ref v) = self.match_type {
            params.insert("match_type".into(), rmpv::Value::String(v.clone().into()));
        }
        if let Some(v) = self.case_sensitive {
            params.insert("case_sensitive".into(), rmpv::Value::Boolean(v));
        }
        if let Some(v) = self.plane {
            params.insert("plane".into(), rmpv::Value::Integer((v as i64).into()));
        }
        if let Some(v) = self.tile_x {
            params.insert("tile_x".into(), rmpv::Value::Integer((v as i64).into()));
        }
        if let Some(v) = self.tile_y {
            params.insert("tile_y".into(), rmpv::Value::Integer((v as i64).into()));
        }
        if let Some(v) = self.radius {
            params.insert("radius".into(), rmpv::Value::Integer((v as i64).into()));
        }
        if let Some(v) = self.visible_only {
            params.insert("visible_only".into(), rmpv::Value::Boolean(v));
        }
        if let Some(v) = self.moving_only {
            params.insert("moving_only".into(), rmpv::Value::Boolean(v));
        }
        if let Some(v) = self.stationary_only {
            params.insert("stationary_only".into(), rmpv::Value::Boolean(v));
        }
        if let Some(v) = self.in_combat {
            params.insert("in_combat".into(), rmpv::Value::Boolean(v));
        }
        if let Some(v) = self.not_in_combat {
            params.insert("not_in_combat".into(), rmpv::Value::Boolean(v));
        }
        if let Some(v) = self.sort_by_distance {
            params.insert("sort_by_distance".into(), rmpv::Value::Boolean(v));
        }
        if let Some(v) = self.max_results {
            params.insert("max_results".into(), rmpv::Value::Integer((v as i64).into()));
        }
        params
    }
}

/// Filter for querying UI components/widgets.
/// Equivalent to Java's ComponentFilter.
#[derive(Debug, Clone, Default)]
pub struct ComponentFilter {
    pub interface_id: Option<i32>,
    pub component_id: Option<i32>,
    pub sub_component_id: Option<i32>,
    pub text_pattern: Option<String>,
    pub sprite_id: Option<i32>,
    pub item_id: Option<i32>,
    pub component_type: Option<i32>,
    pub visible_only: Option<bool>,
    pub max_results: Option<i32>,
}

impl ComponentFilter {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn interface_id(mut self, id: i32) -> Self {
        self.interface_id = Some(id);
        self
    }

    pub fn component_id(mut self, id: i32) -> Self {
        self.component_id = Some(id);
        self
    }

    pub fn text_pattern(mut self, pattern: impl Into<String>) -> Self {
        self.text_pattern = Some(pattern.into());
        self
    }

    pub fn visible_only(mut self, v: bool) -> Self {
        self.visible_only = Some(v);
        self
    }

    pub fn max_results(mut self, n: i32) -> Self {
        self.max_results = Some(n);
        self
    }

    pub fn to_params(&self) -> HashMap<String, rmpv::Value> {
        let mut params = HashMap::new();
        if let Some(v) = self.interface_id {
            params.insert("interface_id".into(), rmpv::Value::Integer((v as i64).into()));
        }
        if let Some(v) = self.component_id {
            params.insert("component_id".into(), rmpv::Value::Integer((v as i64).into()));
        }
        if let Some(v) = self.sub_component_id {
            params.insert("sub_component_id".into(), rmpv::Value::Integer((v as i64).into()));
        }
        if let Some(ref v) = self.text_pattern {
            params.insert("text_pattern".into(), rmpv::Value::String(v.clone().into()));
        }
        if let Some(v) = self.sprite_id {
            params.insert("sprite_id".into(), rmpv::Value::Integer((v as i64).into()));
        }
        if let Some(v) = self.item_id {
            params.insert("item_id".into(), rmpv::Value::Integer((v as i64).into()));
        }
        if let Some(v) = self.component_type {
            params.insert("component_type".into(), rmpv::Value::Integer((v as i64).into()));
        }
        if let Some(v) = self.visible_only {
            params.insert("visible_only".into(), rmpv::Value::Boolean(v));
        }
        if let Some(v) = self.max_results {
            params.insert("max_results".into(), rmpv::Value::Integer((v as i64).into()));
        }
        params
    }
}

/// Filter for querying inventory items.
/// Equivalent to Java's InventoryFilter.
#[derive(Debug, Clone, Default)]
pub struct InventoryFilter {
    pub inventory_id: Option<i32>,
    pub item_id: Option<i64>,
    pub slot: Option<i32>,
    pub name_pattern: Option<String>,
    pub max_results: Option<i32>,
}

impl InventoryFilter {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn inventory_id(mut self, id: i32) -> Self {
        self.inventory_id = Some(id);
        self
    }

    pub fn item_id(mut self, id: i64) -> Self {
        self.item_id = Some(id);
        self
    }

    pub fn slot(mut self, slot: i32) -> Self {
        self.slot = Some(slot);
        self
    }

    pub fn name_pattern(mut self, pattern: impl Into<String>) -> Self {
        self.name_pattern = Some(pattern.into());
        self
    }

    pub fn to_params(&self) -> HashMap<String, rmpv::Value> {
        let mut params = HashMap::new();
        if let Some(v) = self.inventory_id {
            params.insert("inventory_id".into(), rmpv::Value::Integer((v as i64).into()));
        }
        if let Some(v) = self.item_id {
            params.insert("item_id".into(), rmpv::Value::Integer(v.into()));
        }
        if let Some(v) = self.slot {
            params.insert("slot".into(), rmpv::Value::Integer((v as i64).into()));
        }
        if let Some(ref v) = self.name_pattern {
            params.insert("name_pattern".into(), rmpv::Value::String(v.clone().into()));
        }
        if let Some(v) = self.max_results {
            params.insert("max_results".into(), rmpv::Value::Integer((v as i64).into()));
        }
        params
    }
}
