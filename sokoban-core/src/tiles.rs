use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::fmt;

use crate::types::*;

// ============================================================
//  Tile ID — extensible string-based identifier
// ============================================================

#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct TileId(pub String);

impl TileId {
    pub fn new(s: &str) -> Self { Self(s.to_string()) }
    pub fn as_str(&self) -> &str { &self.0 }
}

impl fmt::Display for TileId {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result { write!(f, "{}", self.0) }
}

// ============================================================
//  Tile category
// ============================================================

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum TileCategory {
    Floor,      // walkable surface (Normal, Ice, Water, Target, etc.)
    Wall,       // blocks movement (Wall, CrackedWall, Rock)
    Object,     // pushable/interactable (Box, Bomb, Spring)
    Item,       // collectible (Key)
    Entity,     // dynamic (Player)
    Door,       // Gate
    Switch,     // Switch, PressurePlate
    Portal,     // teleport
    Decoration, // visual only
}

// ============================================================
//  Tile definition — everything about a tile type
// ============================================================

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TileModel {
    /// Path to Blender .glb file (relative to assets/models/), None = procedural
    pub glb_path: Option<String>,
    /// Fallback color for procedural rendering
    pub color: [f32; 4],
    /// Height in world units (for 3D block size)
    pub height: f32,
    /// Whether this tile has animations
    pub animated: bool,
    /// Animation clip names
    pub animations: Vec<String>,
    /// Whether this tile can contain other items (like a chest)
    pub is_container: bool,
    /// Number of container slots
    pub container_slots: u32,
}

impl Default for TileModel {
    fn default() -> Self {
        Self { glb_path: None, color: [0.8, 0.8, 0.8, 1.0], height: 1.0,
               animated: false, animations: vec![], is_container: false, container_slots: 0 }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TilePhysics {
    pub solid: bool,
    pub pushable: bool,
    pub push_weight: u32,     // 0 = can't push, 1 = normal, 2+ = heavy
    pub friction: f32,        // 0 = ice, 1 = normal
    pub destructible: bool,
    pub falls: bool,          // falls into pits/water
}

impl Default for TilePhysics {
    fn default() -> Self {
        Self { solid: false, pushable: false, push_weight: 0, friction: 1.0, destructible: false, falls: false }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TileDef {
    pub id: TileId,
    pub display_name: String,
    pub category: TileCategory,
    pub model: TileModel,
    pub physics: TilePhysics,
    /// Maps to old FloorType (if this is a floor tile)
    pub floor_type: Option<FloorType>,
    /// Maps to old ObjectType (if this is an object tile)
    pub object_type: Option<ObjectType>,
    /// Extensible key-value properties
    pub props: HashMap<String, String>,
}

impl TileDef {
    pub fn new(id: &str, name: &str, category: TileCategory) -> Self {
        Self { id: TileId::new(id), display_name: name.to_string(), category,
               model: TileModel::default(), physics: TilePhysics::default(),
               floor_type: None, object_type: None, props: HashMap::new() }
    }
    pub fn with_floor(mut self, f: FloorType) -> Self { self.floor_type = Some(f); self }
    pub fn with_object(mut self, o: ObjectType) -> Self { self.object_type = Some(o); self }
    pub fn with_model(mut self, m: TileModel) -> Self { self.model = m; self }
    pub fn with_physics(mut self, p: TilePhysics) -> Self { self.physics = p; self }
}

// ============================================================
//  Tile registry — all known tile types
// ============================================================

#[derive(Debug, Clone)]
pub struct TileRegistry {
    tiles: HashMap<TileId, TileDef>,
    by_category: HashMap<TileCategory, Vec<TileId>>,
    /// Map old FloorType → TileId
    floor_map: HashMap<FloorType, TileId>,
    /// Map old ObjectType → TileId (approximate, uses first match)
    object_map: HashMap<String, TileId>,
}

impl TileRegistry {
    pub fn new() -> Self {
        Self { tiles: HashMap::new(), by_category: HashMap::new(),
               floor_map: HashMap::new(), object_map: HashMap::new() }
    }

    pub fn register(&mut self, def: TileDef) {
        if let Some(ft) = def.floor_type { self.floor_map.insert(ft, def.id.clone()); }
        if let Some(ot) = def.object_type { self.object_map.insert(format!("{:?}", ot), def.id.clone()); }
        self.by_category.entry(def.category).or_default().push(def.id.clone());
        self.tiles.insert(def.id.clone(), def);
    }

    pub fn get(&self, id: &TileId) -> Option<&TileDef> { self.tiles.get(id) }
    pub fn by_category(&self, cat: TileCategory) -> &[TileId] {
        self.by_category.get(&cat).map_or(&[], |v| v.as_slice())
    }
    pub fn all(&self) -> impl Iterator<Item = &TileDef> { self.tiles.values() }
    pub fn floor_to_tile(&self, f: FloorType) -> TileId {
        self.floor_map.get(&f).cloned().unwrap_or_else(|| TileId::new("floor.normal"))
    }
    pub fn object_to_tile(&self, o: ObjectType) -> Option<TileId> {
        self.object_map.get(&format!("{:?}", o)).cloned()
    }
    pub fn len(&self) -> usize { self.tiles.len() }
}

/// Build the default tile registry with all standard tiles
pub fn default_tile_registry() -> TileRegistry {
    let mut r = TileRegistry::new();

    // === Floor tiles ===
    let floor_physics = TilePhysics { friction: 1.0, ..Default::default() };
    let ice_physics = TilePhysics { friction: 0.05, ..Default::default() };

    r.register(TileDef::new("floor.normal", "Normal", TileCategory::Floor)
        .with_floor(FloorType::Normal)
        .with_model(TileModel { color: [0.88, 0.83, 0.73, 1.0], height: 0.15, ..Default::default() })
        .with_physics(floor_physics.clone()));
    r.register(TileDef::new("floor.target", "Target", TileCategory::Floor)
        .with_floor(FloorType::Target)
        .with_model(TileModel { color: [0.4, 0.78, 0.5, 1.0], height: 0.15, ..Default::default() })
        .with_physics(floor_physics.clone()));
    r.register(TileDef::new("floor.ice", "Ice", TileCategory::Floor)
        .with_floor(FloorType::Ice)
        .with_model(TileModel { color: [0.7, 0.88, 0.95, 1.0], height: 0.15, ..Default::default() })
        .with_physics(ice_physics));
    r.register(TileDef::new("floor.water", "Water", TileCategory::Floor)
        .with_floor(FloorType::Water)
        .with_model(TileModel { color: [0.2, 0.4, 0.8, 0.7], height: 0.05, ..Default::default() })
        .with_physics(TilePhysics { falls: true, ..Default::default() }));
    r.register(TileDef::new("floor.pit", "Pit", TileCategory::Floor)
        .with_floor(FloorType::Pit)
        .with_model(TileModel { color: [0.05, 0.05, 0.08, 1.0], height: 0.05, ..Default::default() })
        .with_physics(TilePhysics { falls: true, ..Default::default() }));
    r.register(TileDef::new("floor.mud", "Mud", TileCategory::Floor)
        .with_floor(FloorType::Mud)
        .with_model(TileModel { color: [0.5, 0.4, 0.25, 1.0], height: 0.15, ..Default::default() })
        .with_physics(TilePhysics { friction: 0.3, ..Default::default() }));
    r.register(TileDef::new("floor.glass", "Glass", TileCategory::Floor)
        .with_floor(FloorType::Glass)
        .with_model(TileModel { color: [0.85, 0.9, 0.95, 0.5], height: 0.1, ..Default::default() })
        .with_physics(TilePhysics { destructible: true, ..Default::default() }));
    r.register(TileDef::new("floor.plate", "Pressure Plate", TileCategory::Switch)
        .with_floor(FloorType::PressurePlate)
        .with_model(TileModel { color: [0.75, 0.72, 0.65, 1.0], height: 0.05, ..Default::default() })
        .with_physics(floor_physics.clone()));
    r.register(TileDef::new("floor.conveyor", "Conveyor", TileCategory::Floor)
        .with_floor(FloorType::Conveyor(Direction::Up))
        .with_model(TileModel { color: [0.5, 0.5, 0.45, 1.0], height: 0.15, animated: true, ..Default::default() })
        .with_physics(floor_physics.clone()));
    r.register(TileDef::new("floor.portal", "Portal", TileCategory::Portal)
        .with_floor(FloorType::Portal(0))
        .with_model(TileModel { color: [0.5, 0.3, 0.7, 1.0], height: 0.1, animated: true, ..Default::default() })
        .with_physics(TilePhysics { friction: 0.0, ..Default::default() }));
    r.register(TileDef::new("floor.ramp", "Ramp", TileCategory::Floor)
        .with_floor(FloorType::Ramp(Direction::Up))
        .with_model(TileModel { color: [0.8, 0.75, 0.65, 1.0], height: 0.5, ..Default::default() })
        .with_physics(floor_physics));

    // === Wall tiles ===
    let wall_physics = TilePhysics { solid: true, push_weight: 0, ..Default::default() };
    r.register(TileDef::new("wall.stone", "Wall", TileCategory::Wall)
        .with_object(ObjectType::Wall)
        .with_model(TileModel { color: [0.35, 0.35, 0.42, 1.0], height: 3.0, ..Default::default() })
        .with_physics(wall_physics.clone()));
    r.register(TileDef::new("wall.cracked", "Cracked Wall", TileCategory::Wall)
        .with_object(ObjectType::CrackedWall)
        .with_model(TileModel { color: [0.5, 0.35, 0.3, 1.0], height: 3.0, ..Default::default() })
        .with_physics(TilePhysics { solid: true, destructible: true, ..Default::default() }));
    r.register(TileDef::new("wall.rock", "Rock", TileCategory::Wall)
        .with_object(ObjectType::Rock)
        .with_model(TileModel { color: [0.45, 0.4, 0.35, 1.0], height: 1.6, ..Default::default() })
        .with_physics(TilePhysics { solid: true, push_weight: 3, ..Default::default() }));

    // === Object tiles ===
    let box_physics = TilePhysics { solid: false, pushable: true, push_weight: 1, friction: 1.0, falls: true, ..Default::default() };
    r.register(TileDef::new("obj.box", "Box", TileCategory::Object)
        .with_object(ObjectType::Box)
        .with_model(TileModel { color: [0.82, 0.52, 0.2, 1.0], height: 1.5, ..Default::default() })
        .with_physics(box_physics.clone()));
    r.register(TileDef::new("obj.heavy_box", "Heavy Box", TileCategory::Object)
        .with_object(ObjectType::HeavyBox)
        .with_model(TileModel { color: [0.4, 0.4, 0.45, 1.0], height: 1.7, ..Default::default() })
        .with_physics(TilePhysics { pushable: true, push_weight: 2, falls: true, ..Default::default() }));
    r.register(TileDef::new("obj.fragile_box", "Fragile Box", TileCategory::Object)
        .with_object(ObjectType::FragileBox)
        .with_model(TileModel { color: [0.9, 0.6, 0.6, 1.0], height: 1.4, ..Default::default() })
        .with_physics(TilePhysics { pushable: true, push_weight: 1, destructible: true, falls: true, ..Default::default() }));
    r.register(TileDef::new("obj.ice_box", "Ice Box", TileCategory::Object)
        .with_object(ObjectType::IceBox)
        .with_model(TileModel { color: [0.7, 0.88, 0.95, 1.0], height: 1.5, ..Default::default() })
        .with_physics(TilePhysics { pushable: true, push_weight: 1, friction: 0.0, falls: true, ..Default::default() }));
    r.register(TileDef::new("obj.bomb", "Bomb", TileCategory::Object)
        .with_object(ObjectType::Bomb)
        .with_model(TileModel { color: [0.9, 0.3, 0.2, 1.0], height: 1.3, ..Default::default() })
        .with_physics(TilePhysics { pushable: true, push_weight: 1, destructible: true, ..Default::default() }));
    r.register(TileDef::new("obj.spring", "Spring", TileCategory::Object)
        .with_object(ObjectType::Spring)
        .with_model(TileModel { color: [0.3, 0.8, 0.4, 1.0], height: 0.5, animated: true, ..Default::default() })
        .with_physics(TilePhysics { pushable: false, ..Default::default() }));
    r.register(TileDef::new("obj.spikes", "Spikes", TileCategory::Object)
        .with_object(ObjectType::Spikes)
        .with_model(TileModel { color: [0.7, 0.2, 0.1, 1.0], height: 0.3, ..Default::default() })
        .with_physics(TilePhysics { destructible: true, ..Default::default() }));
    r.register(TileDef::new("obj.mirror", "Mirror", TileCategory::Object)
        .with_object(ObjectType::Mirror(Direction::Up))
        .with_model(TileModel { color: [0.8, 0.85, 0.9, 1.0], height: 2.0, ..Default::default() })
        .with_physics(TilePhysics { solid: false, ..Default::default() }));
    r.register(TileDef::new("obj.magnet", "Magnet", TileCategory::Object)
        .with_object(ObjectType::Magnet)
        .with_model(TileModel { color: [0.5, 0.5, 0.6, 1.0], height: 1.2, ..Default::default() })
        .with_physics(TilePhysics { solid: false, ..Default::default() }));

    // === Items ===
    r.register(TileDef::new("item.key", "Key", TileCategory::Item)
        .with_object(ObjectType::Key(ItemColor::Red))
        .with_model(TileModel { color: [0.9, 0.2, 0.2, 1.0], height: 0.5, animated: true, ..Default::default() })
        .with_physics(TilePhysics::default()));
    r.register(TileDef::new("item.gate", "Gate", TileCategory::Door)
        .with_object(ObjectType::Gate(ItemColor::Red))
        .with_model(TileModel { color: [0.55, 0.3, 0.3, 1.0], height: 3.0, animated: true, ..Default::default() })
        .with_physics(TilePhysics { solid: true, ..Default::default() }));

    // === Entities ===
    r.register(TileDef::new("entity.player", "Player", TileCategory::Entity)
        .with_object(ObjectType::Player)
        .with_model(TileModel { color: [0.3, 0.55, 0.85, 1.0], height: 1.8, animated: true, animations: vec!["walk".into(), "push".into()], ..Default::default() })
        .with_physics(TilePhysics::default()));

    // === Switch ===
    r.register(TileDef::new("switch.lever", "Switch", TileCategory::Switch)
        .with_object(ObjectType::Switch(0))
        .with_model(TileModel { color: [0.9, 0.8, 0.2, 1.0], height: 0.1, animated: true, ..Default::default() })
        .with_physics(TilePhysics::default()));
    r.register(TileDef::new("switch.pillar", "Pillar", TileCategory::Wall)
        .with_object(ObjectType::Pillar(0))
        .with_model(TileModel { color: [0.6, 0.55, 0.45, 1.0], height: 3.0, animated: true, ..Default::default() })
        .with_physics(TilePhysics { solid: true, ..Default::default() }));

    r
}
