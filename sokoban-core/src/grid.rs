use std::collections::HashSet;

use serde::{Deserialize, Serialize};

use crate::types::*;

/// 运行时网格状态
/// 游戏过程中每一帧的世界状态都由它表示
#[derive(Debug, Clone)]
pub struct GridState {
    pub width: u32,
    pub height: u32,
    pub cells: Vec<Vec<Cell>>,
    pub player_pos: GridPos3D,
    pub box_positions: Vec<TrackedBox>,
    pub collected_keys: Vec<ItemColor>,
    pub active_switches: HashSet<u8>,
    pub current_step: u32,
    pub connections: Vec<FloorLink>,
    pub floor_layer: u8,
}

/// 被追踪的箱子
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct TrackedBox {
    pub entity_id: u64,
    pub pos: GridPos3D,
    pub box_type: ObjectType,
}

/// 撤销用的快照
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GridSnapshot {
    pub cells: Vec<Vec<Cell>>,
    pub player_pos: GridPos3D,
    pub box_positions: Vec<(u64, GridPos3D, ObjectType)>,
    pub collected_keys: Vec<ItemColor>,
    pub active_switches: Vec<u8>,
    pub step: u32,
}

impl GridState {

    pub fn empty() -> Self {
        Self::new(1, 1)
    }

    pub fn new(width: u32, height: u32) -> Self {
        Self {
            width,
            height,
            cells: vec![vec![Cell::empty(); width as usize]; height as usize],
            player_pos: GridPos3D::new(0, 0, 0),
            box_positions: Vec::new(),
            collected_keys: Vec::new(),
            active_switches: std::collections::HashSet::new(),
            current_step: 0,
            connections: Vec::new(),
            floor_layer: 0,
        }
    }
    // ========== 构造 ==========

    /// 从关卡网格数据初始化运行时状态
    pub fn from_grid(grid: &Grid, floor: u8) -> Self {
        let mut boxes = Vec::new();
        let mut player_pos = GridPos3D::new(1, 1, floor);
        let mut next_id: u64 = 1;
        let mut cells = Vec::new();

        for (z, row) in grid.cells.iter().enumerate() {
            let mut new_row = Vec::new();
            for (x, cell) in row.iter().enumerate() {
                let mut new_cell = cell.clone();

                match cell.object {
                    ObjectType::Player => {
                        player_pos = GridPos3D::new(x as i32, z as i32, floor);
                        new_cell.object = ObjectType::None;
                    }
                    obj if obj.is_pushable() => {
                        boxes.push(TrackedBox {
                            entity_id: next_id,
                            pos: GridPos3D::new(x as i32, z as i32, floor),
                            box_type: obj,
                        });
                        next_id += 1;
                        new_cell.object = ObjectType::None;
                    }
                    _ => {}
                }

                new_row.push(new_cell);
            }
            cells.push(new_row);
        }

        Self {
            width: grid.width,
            height: grid.height,
            cells,
            player_pos,
            box_positions: boxes,
            collected_keys: Vec::new(),
            active_switches: HashSet::new(),
            current_step: 0,
            connections: Vec::new(),
            floor_layer: floor,
        }
    }

    // ========== 查询 ==========

    /// 检查坐标是否在网格范围内
    pub fn in_bounds(&self, pos: GridPos) -> bool {
        pos.x >= 0
            && pos.z >= 0
            && (pos.x as u32) < self.width
            && (pos.z as u32) < self.height
    }

    /// 获取指定位置的格子引用
    pub fn get(&self, pos: GridPos) -> Option<&Cell> {
        if !self.in_bounds(pos) {
            return None;
        }
        self.cells
            .get(pos.z as usize)
            .and_then(|row| row.get(pos.x as usize))
    }

    /// 获取指定位置的格子可变引用
    pub fn get_mut(&mut self, pos: GridPos) -> Option<&mut Cell> {
        if !self.in_bounds(pos) {
            return None;
        }
        self.cells
            .get_mut(pos.z as usize)
            .and_then(|row| row.get_mut(pos.x as usize))
    }

    /// 指定位置是否可通行
    pub fn is_passable(&self, pos: GridPos) -> bool {
        match self.get(pos) {
            Some(cell) => cell.is_passable() && self.find_box_at(pos).is_none(),
            None => false,
        }
    }

    /// 指定位置是否是墙
    pub fn is_wall(&self, pos: GridPos) -> bool {
        matches!(
            self.get(pos).map(|c| c.object),
            Some(ObjectType::Wall | ObjectType::CrackedWall | ObjectType::Rock)
        )
    }

    /// 指定位置是否有门
    pub fn is_gate(&self, pos: GridPos) -> Option<ItemColor> {
        match self.get(pos).map(|c| c.object) {
            Some(ObjectType::Gate(color)) => Some(color),
            _ => None,
        }
    }

    /// 指定位置是否是目标点
    pub fn is_target(&self, pos: GridPos) -> bool {
        self.get(pos).map_or(false, |c| c.floor.is_target())
    }

    /// 查找指定位置的箱子
    pub fn find_box_at(&self, pos: GridPos) -> Option<&TrackedBox> {
        self.box_positions
            .iter()
            .find(|b| b.pos.pos == pos)
    }

    /// 查找指定位置的箱子（可变）
    pub fn find_box_at_mut(&mut self, pos: GridPos) -> Option<&mut TrackedBox> {
        self.box_positions
            .iter_mut()
            .find(|b| b.pos.pos == pos)
    }

    /// 指定位置是否有可推动的物体
    pub fn has_pushable_at(&self, pos: GridPos) -> bool {
        self.find_box_at(pos).is_some()
    }

    /// 获取所有箱子位置（排序后用于求解器）
    pub fn sorted_box_positions(&self) -> Vec<GridPos> {
        let mut positions: Vec<GridPos> = self
            .box_positions
            .iter()
            .map(|b| b.pos.pos)
            .collect();
        positions.sort_by(|a, b| {
            a.z.cmp(&b.z).then(a.x.cmp(&b.x))
        });
        positions
    }

    /// 获取所有目标点位置
    pub fn target_positions(&self) -> Vec<GridPos> {
        let mut targets = Vec::new();
        for z in 0..self.height {
            for x in 0..self.width {
                let pos = GridPos::new(x as i32, z as i32);
                if self.is_target(pos) {
                    targets.push(pos);
                }
            }
        }
        targets
    }

    /// 胜利条件检查：所有可推动箱子（不含重型箱）是否都在目标点上
    pub fn all_boxes_on_targets(&self) -> bool {
        let moveable: Vec<_> = self.box_positions.iter()
            .filter(|b| b.box_type != ObjectType::HeavyBox)
            .collect();
        if moveable.is_empty() {
            return true;
        }
        moveable.iter().all(|b| self.is_target(b.pos.pos))
    }

    /// 统计已在目标点上的箱子数（不含重型箱）
    pub fn boxes_on_targets(&self) -> u32 {
        self.box_positions
            .iter()
            .filter(|b| b.box_type != ObjectType::HeavyBox && self.is_target(b.pos.pos))
            .count() as u32
    }

    /// 统计需要推到目标点的箱子总数（不含重型箱）
    pub fn box_count(&self) -> u32 {
        self.box_positions
            .iter()
            .filter(|b| b.box_type != ObjectType::HeavyBox)
            .count() as u32
    }

    /// 统计目标点总数
    pub fn target_count(&self) -> u32 {
        self.target_positions().len() as u32
    }

    /// 是否拥有指定颜色的钥匙
    pub fn has_key(&self, color: ItemColor) -> bool {
        self.collected_keys.contains(&color)
    }

    /// 指定位置的地板类型
    pub fn floor_at(&self, pos: GridPos) -> FloorType {
        self.get(pos).map_or(FloorType::Empty, |c| c.floor)
    }

    /// 指定位置的物体类型
    pub fn object_at(&self, pos: GridPos) -> ObjectType {
        self.get(pos).map_or(ObjectType::None, |c| c.object)
    }

    // ========== 修改 ==========

    /// 移动箱子到新位置
    pub fn move_box(&mut self, entity_id: u64, to: GridPos3D) {
        if let Some(b) = self.box_positions.iter_mut().find(|b| b.entity_id == entity_id) {
            b.pos = to;
        }
    }

    /// 移动玩家到新位置
    pub fn move_player(&mut self, to: GridPos3D) {
        self.player_pos = to;
    }

    /// 移除指定位置的物体（开门、拾取钥匙等）
    pub fn remove_object(&mut self, pos: GridPos) {
        if let Some(cell) = self.get_mut(pos) {
            cell.object = ObjectType::None;
        }
    }

    /// 设置指定位置的物体
    pub fn set_object(&mut self, pos: GridPos, obj: ObjectType) {
        if let Some(cell) = self.get_mut(pos) {
            cell.object = obj;
        }
    }

    /// 设置指定位置的地板
    pub fn set_floor(&mut self, pos: GridPos, floor: FloorType) {
        if let Some(cell) = self.get_mut(pos) {
            cell.floor = floor;
        }
    }

    /// 收集钥匙
    pub fn collect_key(&mut self, color: ItemColor) {
        if !self.collected_keys.contains(&color) {
            self.collected_keys.push(color);
        }
    }

    /// 移除箱子
    pub fn remove_box(&mut self, entity_id: u64) {
        self.box_positions.retain(|b| b.entity_id != entity_id);
    }

    /// 切换开关状态
    pub fn toggle_switch(&mut self, id: u8) {
        if self.active_switches.contains(&id) {
            self.active_switches.remove(&id);
        } else {
            self.active_switches.insert(id);
        }
    }

    /// 检查开关是否激活
    pub fn is_switch_active(&self, id: u8) -> bool {
        self.active_switches.contains(&id)
    }

    /// 增加步数
    pub fn advance_step(&mut self) {
        self.current_step += 1;
    }

    // ========== 快照（撤销用）==========

    /// 生成当前状态的快照
    pub fn snapshot(&self) -> GridSnapshot {
        GridSnapshot {
            cells: self.cells.clone(),
            player_pos: self.player_pos,
            box_positions: self
                .box_positions
                .iter()
                .map(|b| (b.entity_id, b.pos, b.box_type))
                .collect(),
            collected_keys: self.collected_keys.clone(),
            active_switches: self.active_switches.iter().copied().collect(),
            step: self.current_step,
        }
    }

    /// 从快照恢复状态
    pub fn restore(&mut self, snapshot: &GridSnapshot) {
        self.cells = snapshot.cells.clone();
        self.player_pos = snapshot.player_pos;
        self.box_positions = snapshot
            .box_positions
            .iter()
            .map(|(id, pos, box_type)| TrackedBox {
                entity_id: *id,
                pos: *pos,
                box_type: *box_type,
            })
            .collect();
        self.collected_keys = snapshot.collected_keys.clone();
        self.active_switches = snapshot.active_switches.iter().copied().collect();
        self.current_step = snapshot.step;
    }

    // ========== 辅助 ==========

    /// 打印网格的 ASCII 表示（调试用）
    pub fn to_ascii(&self) -> String {
        let mut result = String::new();
        for z in 0..self.height as i32 {
            for x in 0..self.width as i32 {
                let pos = GridPos::new(x, z);
                let ch = if self.player_pos.pos == pos {
                    '@'
                } else if let Some(b) = self.find_box_at(pos) {
                    let on_target = self.is_target(pos);
                    match b.box_type {
                        ObjectType::Box => if on_target { '*' } else { '$' },
                        ObjectType::HeavyBox => if on_target { 'ẖ' } else { 'H' },
                        ObjectType::FragileBox => if on_target { 'ḟ' } else { 'F' },
                        ObjectType::IceBox => if on_target { 'ḯ' } else { 'i' },
                        _ => if on_target { '*' } else { '$' },
                    }
                } else {
                    match self.get(pos) {
                        Some(cell) => match cell.floor {
                            FloorType::Empty => {
                                if cell.object == ObjectType::Wall {
                                    '#'
                                } else {
                                    ' '
                                }
                            }
                            FloorType::Normal => '.',
                            FloorType::Ice => '~',
                            FloorType::Water => 'W',
                            FloorType::Pit => 'O',
                            FloorType::Target => {
                                if cell.object != ObjectType::None {
                                    '.'
                                } else {
                                    'x'
                                }
                            }
                            FloorType::PressurePlate => '_',
                            FloorType::Portal(_) => 'P',
                            FloorType::Mud => 'M',
                            FloorType::Glass => 'G',
                            FloorType::Conveyor(_) => 'C',
                            FloorType::Ramp(_) => 'R',
                        },
                        None => '?',
                    }
                };
                result.push(ch);
                result.push(' ');
            }
            result.push('\n');
        }
        result
    }
}

/// 关卡网格数据（静态，用于存储和加载）
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Grid {
    pub width: u32,
    pub height: u32,
    pub cells: Vec<Vec<Cell>>,
}

impl Grid {
    pub fn new(width: u32, height: u32) -> Self {
        Self {
            width,
            height,
            cells: vec![vec![Cell::empty(); width as usize]; height as usize],
        }
    }

    pub fn get(&self, pos: GridPos) -> Option<&Cell> {
        if pos.x < 0 || pos.z < 0 {
            return None;
        }
        self.cells
            .get(pos.z as usize)
            .and_then(|row| row.get(pos.x as usize))
    }

    pub fn get_mut(&mut self, pos: GridPos) -> Option<&mut Cell> {
        if pos.x < 0 || pos.z < 0 {
            return None;
        }
        self.cells
            .get_mut(pos.z as usize)
            .and_then(|row| row.get_mut(pos.x as usize))
    }

    pub fn set(&mut self, pos: GridPos, cell: Cell) {
        if let Some(c) = self.get_mut(pos) {
            *c = cell;
        }
    }

    pub fn in_bounds(&self, pos: GridPos) -> bool {
        pos.x >= 0
            && pos.z >= 0
            && (pos.x as u32) < self.width
            && (pos.z as u32) < self.height
    }

    pub fn resize(&mut self, new_width: u32, new_height: u32) {
        let mut new_cells = vec![vec![Cell::empty(); new_width as usize]; new_height as usize];
        for z in 0..(self.height as usize).min(new_height as usize) {
            for x in 0..(self.width as usize).min(new_width as usize) {
                new_cells[z][x] = self.cells[z][x].clone();
            }
        }
        self.cells = new_cells;
        self.width = new_width;
        self.height = new_height;
    }

    /// 统计指定类型的物体数量
    pub fn count_object(&self, obj: ObjectType) -> u32 {
        let mut count = 0;
        for row in &self.cells {
            for cell in row {
                if cell.object == obj {
                    count += 1;
                }
            }
        }
        count
    }

    /// 统计所有箱子数量
    pub fn count_boxes(&self) -> u32 {
        let mut count = 0;
        for row in &self.cells {
            for cell in row {
                if cell.object.is_box() {
                    count += 1;
                }
            }
        }
        count
    }

    /// 统计所有目标点数量
    pub fn count_targets(&self) -> u32 {
        let mut count = 0;
        for row in &self.cells {
            for cell in row {
                if cell.floor.is_target() {
                    count += 1;
                }
            }
        }
        count
    }

    /// 查找玩家出生点
    pub fn find_player_spawn(&self) -> Option<GridPos> {
        for z in 0..self.height as i32 {
            for x in 0..self.width as i32 {
                let pos = GridPos::new(x, z);
                if self.get(pos).map_or(false, |c| c.object == ObjectType::Player) {
                    return Some(pos);
                }
            }
        }
        None
    }

    /// 打印 ASCII 表示（调试用）
    pub fn to_ascii(&self) -> String {
        let mut result = String::new();
        for z in 0..self.height as i32 {
            for x in 0..self.width as i32 {
                let pos = GridPos::new(x, z);
                let ch = match self.get(pos) {
                    Some(cell) => {
                        match cell.object {
                            ObjectType::Player => '@',
                            ObjectType::Wall => '#',
                            ObjectType::Box => {
                                if cell.floor.is_target() { '*' } else { '$' }
                            }
                            ObjectType::HeavyBox => {
                                if cell.floor.is_target() { 'ẖ' } else { 'H' }
                            }
                            ObjectType::FragileBox => {
                                if cell.floor.is_target() { 'ḟ' } else { 'F' }
                            }
                            ObjectType::IceBox => {
                                if cell.floor.is_target() { 'ḯ' } else { 'i' }
                            }
                            ObjectType::Key(_) => 'k',
                            ObjectType::Gate(_) => 'D',
                            ObjectType::Bomb => 'B',
                            ObjectType::Spring => 'S',
                            ObjectType::Rock => 'r',
                            _ => match cell.floor {
                                FloorType::Empty => ' ',
                                FloorType::Normal => '.',
                                FloorType::Ice => '~',
                                FloorType::Water => 'W',
                                FloorType::Pit => 'O',
                                FloorType::Target => 'x',
                                FloorType::PressurePlate => '_',
                                FloorType::Portal(_) => 'P',
                                FloorType::Mud => 'M',
                                FloorType::Glass => 'G',
                                FloorType::Conveyor(_) => 'C',
                                FloorType::Ramp(_) => 'R',
                            },
                        }
                    }
                    None => '?',
                };
                result.push(ch);
                result.push(' ');
            }
            result.push('\n');
        }
        result
    }
}

impl Grid {
    /// 用字符串数组快速构建网格
    /// 每个字符代表一个格子，空格忽略
    /// # = 墙  . = 空地  @ = 玩家  $ = 箱子  x = 目标点
    /// ~ = 冰面  k = 钥匙(红)  D = 门(红)  B = 炸弹  S = 弹簧
    /// H = 重型箱子  F = 脆弱箱子  i = 冰箱  r = 岩石  c = 裂墙
    /// W = 水面  O = 深坑  M = 泥地  G = 玻璃
    pub fn from_ascii(rows: &[&str]) -> Self {
        let h = rows.len();
        let w = rows.iter().map(|r| r.chars().filter(|c| !c.is_whitespace()).count()).max().unwrap_or(0);
        let mut cells = vec![vec![Cell::empty(); w]; h];

        for (z, row) in rows.iter().enumerate() {
            let chars: Vec<char> = row.chars().filter(|c| !c.is_whitespace()).collect();
            for (x, ch) in chars.iter().enumerate() {
                if x >= w || z >= h { continue; }
                match ch {
                    '#' => cells[z][x] = Cell::wall(),
                    '.' => cells[z][x] = Cell::empty(),
                    '@' => cells[z][x] = Cell {
                        floor: FloorType::Normal,
                        object: ObjectType::Player,
                        color: None, facing: None, linked_id: None,
                    },
                    '$' => cells[z][x] = Cell {
                        floor: FloorType::Normal,
                        object: ObjectType::Box,
                        color: None, facing: None, linked_id: None,
                    },
                    'x' => cells[z][x] = Cell::target(),
                    '~' => cells[z][x] = Cell {
                        floor: FloorType::Ice,
                        object: ObjectType::None,
                        color: None, facing: None, linked_id: None,
                    },
                    'k' => cells[z][x] = Cell {
                        floor: FloorType::Normal,
                        object: ObjectType::Key(ItemColor::Red),
                        color: None, facing: None, linked_id: None,
                    },
                    'D' => cells[z][x] = Cell {
                        floor: FloorType::Normal,
                        object: ObjectType::Gate(ItemColor::Red),
                        color: None, facing: None, linked_id: None,
                    },
                    'H' => cells[z][x] = Cell {
                        floor: FloorType::Normal,
                        object: ObjectType::HeavyBox,
                        color: None, facing: None, linked_id: None,
                    },
                    'F' => cells[z][x] = Cell {
                        floor: FloorType::Normal,
                        object: ObjectType::FragileBox,
                        color: None, facing: None, linked_id: None,
                    },
                    'i' => cells[z][x] = Cell {
                        floor: FloorType::Normal,
                        object: ObjectType::IceBox,
                        color: None, facing: None, linked_id: None,
                    },
                    'B' => cells[z][x] = Cell {
                        floor: FloorType::Normal,
                        object: ObjectType::Bomb,
                        color: None, facing: None, linked_id: None,
                    },
                    'S' => cells[z][x] = Cell {
                        floor: FloorType::Normal,
                        object: ObjectType::Spring,
                        color: None, facing: None, linked_id: None,
                    },
                    'r' => cells[z][x] = Cell {
                        floor: FloorType::Normal,
                        object: ObjectType::Rock,
                        color: None, facing: None, linked_id: None,
                    },
                    'c' => cells[z][x] = Cell {
                        floor: FloorType::Normal,
                        object: ObjectType::CrackedWall,
                        color: None, facing: None, linked_id: None,
                    },
                    'W' => cells[z][x] = Cell {
                        floor: FloorType::Water,
                        object: ObjectType::None,
                        color: None, facing: None, linked_id: None,
                    },
                    'O' => cells[z][x] = Cell {
                        floor: FloorType::Pit,
                        object: ObjectType::None,
                        color: None, facing: None, linked_id: None,
                    },
                    'M' => cells[z][x] = Cell {
                        floor: FloorType::Mud,
                        object: ObjectType::None,
                        color: None, facing: None, linked_id: None,
                    },
                    'G' => cells[z][x] = Cell {
                        floor: FloorType::Glass,
                        object: ObjectType::None,
                        color: None, facing: None, linked_id: None,
                    },
                    'C' => cells[z][x] = Cell {
                        floor: FloorType::Conveyor(Direction::Right),
                        object: ObjectType::None,
                        color: None, facing: None, linked_id: None,
                    },
                    'P' => cells[z][x] = Cell {
                        floor: FloorType::Portal(0),
                        object: ObjectType::None,
                        color: None, facing: None, linked_id: None,
                    },
                    _ => {}
                }
            }
        }

        Self { width: w as u32, height: h as u32, cells }
    }
}
