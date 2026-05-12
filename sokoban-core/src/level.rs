use std::collections::HashMap;
use std::fs;
use std::path::Path;

use serde::{Deserialize, Serialize};

use crate::grid::Grid;
use crate::types::*;

// ========== 关卡元数据 ==========

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LevelMeta {
    pub id: u32,
    pub name: String,
    pub author: String,
    pub difficulty: u8,
    pub par_steps: Option<u32>,
    pub tags: Vec<String>,
    pub description: String,
}

impl Default for LevelMeta {
    fn default() -> Self {
        Self {
            id: 0,
            name: "Untitled".to_string(),
            author: "Unknown".to_string(),
            difficulty: 1,
            par_steps: None,
            tags: Vec::new(),
            description: String::new(),
        }
    }
}

// ========== 经典单层关卡 ==========

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LevelData {
    pub meta: LevelMeta,
    pub scene_theme: String,
    #[serde(default)]
    pub grid: Option<Grid>,
    #[serde(default)]
    pub ascii: Option<Vec<String>>,
}

impl LevelData {
    /// 获取网格数据，优先从 ascii 字段解析
    pub fn get_grid(&self) -> Grid {
        if let Some(ref ascii) = self.ascii {
            let rows: Vec<&str> = ascii.iter().map(|s| s.as_str()).collect();
            Grid::from_ascii(&rows)
        } else if let Some(ref grid) = self.grid {
            grid.clone()
        } else {
            Grid::new(1, 1)
        }
    }

    pub fn load_from_ron(path: &str) -> Result<Self, LoadError> {
        let content = std::fs::read_to_string(path).map_err(|e| LoadError::Io(e.to_string()))?;
        ron::from_str(&content).map_err(|e| LoadError::Parse(e.to_string()))
    }

    // [新增] 保存关卡为 ron 文件
    pub fn save_to_ron(&self, path: &str) -> Result<(), SaveError> {
        let pretty = ron::Options::default()
            .with_default_extension(ron::extensions::Extensions::UNWRAP_NEWTYPES)
            .to_string_pretty(self, ron::ser::PrettyConfig::default())
            .map_err(|e| SaveError::Serialize(e.to_string()))?;

        if let Some(parent) = Path::new(path).parent() {
            fs::create_dir_all(parent).map_err(|e| SaveError::Io(e.to_string()))?;
        }
        fs::write(path, pretty).map_err(|e| SaveError::Io(e.to_string()))
    }

    /// 快速创建一个简单关卡（测试用）
    pub fn simple(width: u32, height: u32, cells: Vec<Vec<Cell>>) -> Self {
        Self {
            meta: LevelMeta::default(),
            grid: Some(Grid {
                width,
                height,
                cells,
            }),
            ascii: None,
            scene_theme: "default".to_string(),
        }
    }
}

// ========== 多层关卡 ==========

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MultiFloorLevel {
    pub meta: LevelMeta,
    pub floors: Vec<FloorLayer>,
    pub connections: Vec<FloorConnection>,
    pub scene_theme: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FloorLayer {
    pub level: u8,
    pub grid: Grid,
    pub elevation: f32,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FloorConnection {
    pub connection_type: ConnectionType,
    pub from_floor: u8,
    pub from_pos: GridPos,
    pub to_floor: u8,
    pub to_pos: GridPos,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum ConnectionType {
    Stairs(Direction),
    Ladder,
    Elevator(u8),
    Hole,
    Ramp(Direction),
    Portal,
}

impl MultiFloorLevel {
    pub fn load_from_ron(path: &str) -> Result<Self, LoadError> {
        let content = fs::read_to_string(path).map_err(|e| LoadError::Io(e.to_string()))?;
        ron::from_str(&content).map_err(|e| LoadError::Parse(e.to_string()))
    }

    pub fn save_to_ron(&self, path: &str) -> Result<(), SaveError> {
        let pretty = ron::Options::default()
            .with_default_extension(ron::extensions::Extensions::UNWRAP_NEWTYPES)
            .to_string_pretty(self, ron::ser::PrettyConfig::default())
            .map_err(|e| SaveError::Serialize(e.to_string()))?;

        if let Some(parent) = Path::new(path).parent() {
            fs::create_dir_all(parent).map_err(|e| SaveError::Io(e.to_string()))?;
        }
        fs::write(path, pretty).map_err(|e| SaveError::Io(e.to_string()))
    }

    pub fn floor_count(&self) -> u8 {
        self.floors.len() as u8
    }

    pub fn get_floor(&self, level: u8) -> Option<&FloorLayer> {
        self.floors.iter().find(|f| f.level == level)
    }
}

// ========== 地牢 ==========

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DungeonData {
    pub id: u32,
    pub name: String,
    pub theme: String,
    pub rooms: HashMap<String, RoomSlot>,
    pub start_room: String,
    pub boss_room: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RoomSlot {
    pub room_id: String,
    pub room_type: RoomType,
    pub connections: Vec<RoomConnection>,
    pub floor_level: u8,
    pub reward: Option<RewardType>,
    pub grid: Grid,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RoomConnection {
    pub direction: Direction,
    pub target_room: String,
    pub is_locked: bool,
    pub lock_color: Option<ItemColor>,
}

impl DungeonData {
    pub fn load_from_ron(path: &str) -> Result<Self, LoadError> {
        let content = fs::read_to_string(path).map_err(|e| LoadError::Io(e.to_string()))?;
        ron::from_str(&content).map_err(|e| LoadError::Parse(e.to_string()))
    }

    pub fn save_to_ron(&self, path: &str) -> Result<(), SaveError> {
        let pretty = ron::Options::default()
            .with_default_extension(ron::extensions::Extensions::UNWRAP_NEWTYPES)
            .to_string_pretty(self, ron::ser::PrettyConfig::default())
            .map_err(|e| SaveError::Serialize(e.to_string()))?;

        if let Some(parent) = Path::new(path).parent() {
            fs::create_dir_all(parent).map_err(|e| SaveError::Io(e.to_string()))?;
        }
        fs::write(path, pretty).map_err(|e| SaveError::Io(e.to_string()))
    }

    pub fn room_count(&self) -> usize {
        self.rooms.len()
    }

    pub fn get_room(&self, id: &str) -> Option<&RoomSlot> {
        self.rooms.get(id)
    }

    pub fn get_room_mut(&mut self, id: &str) -> Option<&mut RoomSlot> {
        self.rooms.get_mut(id)
    }
}

// ========== 错误类型 ==========

#[derive(Debug, Clone)]
pub enum LoadError {
    Io(String),
    Parse(String),
}

impl std::fmt::Display for LoadError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            LoadError::Io(msg) => write!(f, "IO error: {}", msg),
            LoadError::Parse(msg) => write!(f, "Parse error: {}", msg),
        }
    }
}

impl std::error::Error for LoadError {}

#[derive(Debug, Clone)]
pub enum SaveError {
    Io(String),
    Serialize(String),
}

impl std::fmt::Display for SaveError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            SaveError::Io(msg) => write!(f, "IO error: {}", msg),
            SaveError::Serialize(msg) => write!(f, "Serialize error: {}", msg),
        }
    }
}

impl std::error::Error for SaveError {}

// ========== 验证 ==========

/// 验证经典关卡
pub fn validate_level(level: &LevelData) -> ValidationResult {
    let mut issues = Vec::new();
    let grid = level.get_grid();

    let box_count = grid.count_boxes();
    let target_count = grid.count_targets();
    let player_spawns = count_player_spawns(&grid);

    if box_count == 0 {
        issues.push(ValidationIssue::NoBoxes);
    }
    if target_count == 0 {
        issues.push(ValidationIssue::NoTargets);
    }
    if box_count != target_count && box_count > 0 && target_count > 0 {
        issues.push(ValidationIssue::BoxTargetMismatch {
            boxes: box_count,
            targets: target_count,
        });
    }
    if player_spawns == 0 {
        issues.push(ValidationIssue::NoPlayerSpawn);
    }
    if player_spawns > 1 {
        issues.push(ValidationIssue::MultiplePlayerSpawns {
            count: player_spawns,
        });
    }
    if !is_grid_connected(&grid) {
        issues.push(ValidationIssue::NotConnected);
    }

    // 钥匙-门配对检查
    let mut gate_colors = std::collections::HashSet::new();
    let mut key_colors = std::collections::HashSet::new();
    for row in &grid.cells {
        for cell in row {
            match cell.object {
                ObjectType::Gate(c) => { gate_colors.insert(c); }
                ObjectType::Key(c) => { key_colors.insert(c); }
                _ => {}
            }
        }
    }
    for color in gate_colors {
        if !key_colors.contains(&color) {
            issues.push(ValidationIssue::MissingKey { color });
        }
    }

    ValidationResult {
        is_valid: issues.is_empty(),
        issues,
    }
}

/// 验证多层关卡
pub fn validate_multifloor(level: &MultiFloorLevel) -> ValidationResult {
    let mut issues = Vec::new();

    if level.floors.is_empty() {
        issues.push(ValidationIssue::NoFloors);
        return ValidationResult {
            is_valid: false,
            issues,
        };
    }

    let mut total_boxes = 0u32;
    let mut total_targets = 0u32;
    let mut total_players = 0u32;

    for floor in &level.floors {
        total_boxes += floor.grid.count_boxes();
        total_targets += floor.grid.count_targets();
        total_players += count_player_spawns(&floor.grid);

        if !is_grid_connected(&floor.grid) {
            issues.push(ValidationIssue::NotConnected);
        }
    }

    if total_boxes == 0 {
        issues.push(ValidationIssue::NoBoxes);
    }
    if total_targets == 0 {
        issues.push(ValidationIssue::NoTargets);
    }
    if total_boxes != total_targets && total_boxes > 0 && total_targets > 0 {
        issues.push(ValidationIssue::BoxTargetMismatch {
            boxes: total_boxes,
            targets: total_targets,
        });
    }
    if total_players == 0 {
        issues.push(ValidationIssue::NoPlayerSpawn);
    }
    if total_players > 1 {
        issues.push(ValidationIssue::MultiplePlayerSpawns {
            count: total_players,
        });
    }

    // 验证层间连接的有效性
    for conn in &level.connections {
        let from_valid = level
            .get_floor(conn.from_floor)
            .and_then(|f| f.grid.get(conn.from_pos))
            .map_or(false, |c| c.floor.is_passable());
        let to_valid = level
            .get_floor(conn.to_floor)
            .and_then(|f| f.grid.get(conn.to_pos))
            .map_or(false, |c| c.floor.is_passable());

        if !from_valid || !to_valid {
            issues.push(ValidationIssue::NotConnected);
        }
    }

    ValidationResult {
        is_valid: issues.is_empty(),
        issues,
    }
}

/// 验证地牢
pub fn validate_dungeon(dungeon: &DungeonData) -> ValidationResult {
    let mut issues = Vec::new();

    if dungeon.rooms.is_empty() {
        issues.push(ValidationIssue::NoRooms);
        return ValidationResult {
            is_valid: false,
            issues,
        };
    }

    // 检查起始房间和 Boss 房间是否存在
    if !dungeon.rooms.contains_key(&dungeon.start_room) {
        issues.push(ValidationIssue::NoPlayerSpawn);
    }
    if !dungeon.rooms.contains_key(&dungeon.boss_room) {
        issues.push(ValidationIssue::NoTargets);
    }

    // 验证每个房间
    for (_room_id, room) in &dungeon.rooms {
        let box_count = room.grid.count_boxes();
        let target_count = room.grid.count_targets();

        // 只有谜题类房间需要箱子和目标点
        if room.room_type == RoomType::Puzzle || room.room_type == RoomType::Challenge || room.room_type == RoomType::Boss {
            if box_count == 0 && room.room_type != RoomType::Boss {
                // Boss 房可以没有箱子（特殊通关条件）
            }
            if box_count != target_count && box_count > 0 && target_count > 0 {
                issues.push(ValidationIssue::BoxTargetMismatch {
                    boxes: box_count,
                    targets: target_count,
                });
            }
        }

        // 验证连接的房间是否存在
        for conn in &room.connections {
            if !dungeon.rooms.contains_key(&conn.target_room) {
                issues.push(ValidationIssue::NotConnected);
            }
        }
    }

    // 检查从起始房间能否到达 Boss 房间（BFS，跳过锁门）
    if !is_dungeon_reachable(dungeon) {
        issues.push(ValidationIssue::NotConnected);
    }

    ValidationResult {
        is_valid: issues.is_empty(),
        issues,
    }
}

// ========== 验证辅助函数 ==========

/// 统计玩家出生点数量
fn count_player_spawns(grid: &Grid) -> u32 {
    let mut count = 0;
    for row in &grid.cells {
        for cell in row {
            if cell.object == ObjectType::Player {
                count += 1;
            }
        }
    }
    count
}

/// 检查网格是否全部连通（BFS）
pub(crate) fn is_grid_connected(grid: &Grid) -> bool {
    // 收集所有可通行格子
    let mut passable = Vec::new();
    for z in 0..grid.height as i32 {
        for x in 0..grid.width as i32 {
            let pos = GridPos::new(x, z);
            if let Some(cell) = grid.get(pos) {
                if cell.floor.is_passable() {
                    passable.push(pos);
                }
            }
        }
    }

    if passable.is_empty() {
        return true;
    }

    // BFS
    let start = passable[0];
    let mut visited = std::collections::HashSet::new();
    let mut queue = std::collections::VecDeque::new();
    visited.insert(start);
    queue.push_back(start);

    while let Some(current) = queue.pop_front() {
        for dir in Direction::all() {
            let next = current.shift(dir);
            if !visited.contains(&next) && passable.contains(&next) {
                visited.insert(next);
                queue.push_back(next);
            }
        }
    }

    visited.len() == passable.len()
}

/// 检查地牢中从起始房间能否到达 Boss 房间（跳过锁门）
fn is_dungeon_reachable(dungeon: &DungeonData) -> bool {
    let mut visited = std::collections::HashSet::new();
    let mut queue = std::collections::VecDeque::new();

    visited.insert(dungeon.start_room.clone());
    queue.push_back(dungeon.start_room.clone());

    while let Some(current) = queue.pop_front() {
        if current == dungeon.boss_room {
            return true;
        }

        if let Some(room) = dungeon.rooms.get(&current) {
            for conn in &room.connections {
                // 跳过锁定的连接（无钥匙无法通过）
                if conn.is_locked {
                    continue;
                }
                if !visited.contains(&conn.target_room) {
                    visited.insert(conn.target_room.clone());
                    queue.push_back(conn.target_room.clone());
                }
            }
        }
    }

    false
}
