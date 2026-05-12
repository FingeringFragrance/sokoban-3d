use serde::{Deserialize, Serialize};
use std::collections::HashMap;

use crate::grid::*;
use crate::types::*;

// ============================================================
//  Map connection
// ============================================================

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum ConnectionKind {
    Portal,    // step on to teleport
    Stairs,    // walk up/down stairs
    Door,      // requires key to open
    Warp,      // instant teleport
    Hole,      // fall through to lower map
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MapLink {
    pub from_pos: GridPos,
    pub to_map: String,
    pub to_pos: GridPos,
    pub kind: ConnectionKind,
    pub locked: bool,
    pub lock_color: Option<ItemColor>,
}

// ============================================================
//  Game map — one named level
// ============================================================

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GameMap {
    pub id: String,
    pub name: String,
    pub grid: Grid,
    pub theme: String,
    pub links: Vec<MapLink>,
    pub width: u32,
    pub height: u32,
}

impl GameMap {
    pub fn new(id: &str, name: &str, w: u32, h: u32) -> Self {
        Self { id: id.to_string(), name: name.to_string(),
            grid: Grid::new(w, h), theme: "default".to_string(),
            links: vec![], width: w, height: h }
    }
}

// ============================================================
//  Map collection — multiple connected maps
// ============================================================

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MapCollection {
    pub maps: HashMap<String, GameMap>,
    pub order: Vec<String>,      // ordered list of map IDs
    pub start_map: String,
    pub name: String,
    pub author: String,
}

impl MapCollection {
    pub fn new(name: &str) -> Self {
        Self { maps: HashMap::new(), order: vec![], start_map: String::new(),
               name: name.to_string(), author: String::new() }
    }

    pub fn add_map(&mut self, map: GameMap) {
        if self.start_map.is_empty() { self.start_map = map.id.clone(); }
        self.order.push(map.id.clone());
        self.maps.insert(map.id.clone(), map);
    }

    pub fn get(&self, id: &str) -> Option<&GameMap> { self.maps.get(id) }
    pub fn get_mut(&mut self, id: &str) -> Option<&mut GameMap> { self.maps.get_mut(id) }
    pub fn start(&self) -> Option<&GameMap> { self.maps.get(&self.start_map) }
    pub fn map_count(&self) -> usize { self.maps.len() }
    pub fn map_order(&self) -> &[String] { &self.order }

    pub fn remove_map(&mut self, id: &str) {
        self.maps.remove(id);
        self.order.retain(|i| i != id);
        if self.start_map == id {
            self.start_map = self.order.first().cloned().unwrap_or_default();
        }
    }

    /// Check if all maps are reachable from the start map
    pub fn validate_connectivity(&self) -> bool {
        if self.maps.is_empty() { return false; }
        let mut visited = HashMap::new();
        for id in self.maps.keys() { visited.insert(id.clone(), false); }
        let mut stack = vec![self.start_map.clone()];
        while let Some(current) = stack.pop() {
            if *visited.get(&current).unwrap_or(&false) { continue; }
            visited.insert(current.clone(), true);
            if let Some(map) = self.maps.get(&current) {
                for link in &map.links {
                    if !*visited.get(&link.to_map).unwrap_or(&true) {
                        stack.push(link.to_map.clone());
                    }
                }
            }
        }
        visited.values().all(|v| *v)
    }

    /// Get all maps connected to a given map
    pub fn connected_maps(&self, id: &str) -> Vec<String> {
        let mut result = vec![];
        if let Some(map) = self.maps.get(id) {
            for link in &map.links {
                if !result.contains(&link.to_map) { result.push(link.to_map.clone()); }
            }
        }
        result
    }

    /// Save to .ron file
    pub fn save_to_file(&self, path: &str) -> Result<(), String> {
        let pretty = ron::ser::to_string_pretty(self, ron::ser::PrettyConfig::default())
            .map_err(|e| e.to_string())?;
        std::fs::write(path, pretty).map_err(|e| e.to_string())
    }

    /// Load from .ron file
    pub fn load_from_file(path: &str) -> Result<Self, String> {
        let content = std::fs::read_to_string(path).map_err(|e| e.to_string())?;
        ron::from_str(&content).map_err(|e| e.to_string())
    }
}
