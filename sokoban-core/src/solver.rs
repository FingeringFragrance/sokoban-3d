use std::collections::{HashMap, HashSet, VecDeque};
use std::time::Instant;

use crate::grid::*;
use crate::rules::*;
use crate::types::*;

// ============================================================
//  公开 API
// ============================================================

/// 求解器配置
#[derive(Debug, Clone)]
pub struct SolverConfig {
    pub max_states: usize,
    pub timeout_ms: u64,
}

impl Default for SolverConfig {
    fn default() -> Self {
        Self {
            max_states: 500_000,
            timeout_ms: 5_000,
        }
    }
}

/// 求解结果
#[derive(Debug, Clone)]
pub struct SolverResult {
    pub solution: Option<Vec<Direction>>,
    pub states_explored: usize,
    pub time_ms: u64,
}

/// 求解一个关卡
pub fn solve(initial_grid: &GridState, config: &SolverConfig) -> SolverResult {
    let start_time = Instant::now();
    let scene = SceneTheme::default();

    let targets = initial_grid.target_positions();
    if targets.is_empty() {
        return SolverResult {
            solution: None,
            states_explored: 0,
            time_ms: 0,
        };
    }

    let initial_node = SolverNode::from_grid(initial_grid, initial_grid);

    let mut visited: HashSet<SolverNode> = HashSet::new();
    visited.insert(initial_node.clone());

    // came_from[node] = (parent_node, direction)
    let mut came_from: HashMap<SolverNode, (SolverNode, Direction)> = HashMap::new();

    let mut queue = VecDeque::new();
    queue.push_back(initial_node);

    while let Some(node) = queue.pop_front() {
        // 超时检查
        if start_time.elapsed().as_millis() as u64 > config.timeout_ms {
            return SolverResult {
                solution: None,
                states_explored: visited.len(),
                time_ms: start_time.elapsed().as_millis() as u64,
            };
        }

        // 状态数检查
        if visited.len() > config.max_states {
            return SolverResult {
                solution: None,
                states_explored: visited.len(),
                time_ms: start_time.elapsed().as_millis() as u64,
            };
        }

        // 胜利检查
        if node.is_solved(&targets) {
            let path = reconstruct_path(&came_from, &node);
            return SolverResult {
                solution: Some(path),
                states_explored: visited.len(),
                time_ms: start_time.elapsed().as_millis() as u64,
            };
        }

        // 快速死锁预检查
        if has_deadlock(&node, &targets, initial_grid) {
            continue;
        }

        // 克隆网格并恢复到当前节点状态（每个节点只克隆 + 恢复一次）
        let mut base_grid = initial_grid.clone();
        restore_to_node(&mut base_grid, &node);

        // 尝试四个方向
        for dir in Direction::all() {
            let mut test_grid = base_grid.clone();
            let intent = MoveIntent { direction: dir };
            let result = resolve_move(&mut test_grid, intent, &scene);

            if !result.success {
                continue;
            }

            let next_node = SolverNode::from_grid(&test_grid, initial_grid);

            if !visited.contains(&next_node) {
                visited.insert(next_node.clone());
                came_from.insert(next_node.clone(), (node.clone(), dir));
                queue.push_back(next_node);
            }
        }
    }

    SolverResult {
        solution: None,
        states_explored: visited.len(),
        time_ms: start_time.elapsed().as_millis() as u64,
    }
}

/// 快速检查：当前状态是否已经无解
pub fn is_solvable(initial_grid: &GridState) -> bool {
    let config = SolverConfig {
        max_states: 100_000,
        timeout_ms: 2_000,
    };
    solve(initial_grid, &config).solution.is_some()
}

/// 找到最短解法的步数
pub fn optimal_steps(initial_grid: &GridState) -> Option<u32> {
    let config = SolverConfig::default();
    solve(initial_grid, &config)
        .solution
        .map(|s| s.len() as u32)
}

// ============================================================
//  内部数据结构
// ============================================================

/// 求解器节点：完整记录相对于初始状态的所有关键差异
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
struct SolverNode {
    player: GridPos,
    /// 箱子位置及类型，按 (z, x) 排序保证 Hash/Eq 一致性
    /// HeavyBox 也在此列，但不参与胜利判定和死锁检测
    boxes: Vec<(GridPos, ObjectType)>,
    /// 已收集的钥匙，按颜色索引排序
    collected_keys: Vec<ItemColor>,
    /// 已移除的物体位置（开门、拾取钥匙、破坏墙壁等），按 (z, x) 排序
    removed_objects: Vec<GridPos>,
    /// 已激活的开关 ID，升序排列
    active_switches: Vec<u8>,
    /// 地板变更记录（如玻璃碎裂变坑），按 (z, x) 排序
    floor_changes: Vec<(GridPos, FloorType)>,
}

impl SolverNode {
    /// 从当前网格状态提取求解器节点
    ///
    /// HeavyBox 始终在 `box_positions` 中（由 `GridState::from_grid` 处理），
    /// 通过 `is_pushable()` 自动归入。
    fn from_grid(current: &GridState, initial: &GridState) -> Self {
        let mut boxes: Vec<(GridPos, ObjectType)> = current
            .box_positions
            .iter()
            .map(|b| (b.pos.pos, b.box_type))
            .collect();
        boxes.sort_by_key(|(pos, _)| (pos.z, pos.x));

        let mut collected_keys = current.collected_keys.clone();
        collected_keys.sort_by_key(|k| k.index());

        let mut removed_objects = Vec::new();
        for z in 0..current.height as i32 {
            for x in 0..current.width as i32 {
                let pos = GridPos::new(x, z);
                let cur_obj = current.object_at(pos);
                let init_obj = initial.object_at(pos);
                if init_obj != ObjectType::None
                    && init_obj != ObjectType::Wall
                    && init_obj != ObjectType::CrackedWall
                    && init_obj != ObjectType::Rock
                    && cur_obj == ObjectType::None
                {
                    removed_objects.push(pos);
                }
            }
        }
        removed_objects.sort_by_key(|pos| (pos.z, pos.x));

        // ---- 开关 ----
        let mut active_switches: Vec<u8> = current.active_switches.iter().copied().collect();
        active_switches.sort();

        // ---- 地板变更 ----
        let mut floor_changes = Vec::new();
        for z in 0..current.height as i32 {
            for x in 0..current.width as i32 {
                let pos = GridPos::new(x, z);
                if let Some(cell) = current.get(pos) {
                    // 记录非默认地板（Glass 变 Pit 等）
                    // 由于我们没有 initial 对比，改为只记录 Pit 类型
                    // （正常关卡不会在初始时有 Pit，Pit 通常是 Glass 碎裂后的结果）
                    if cell.floor == FloorType::Pit {
                        floor_changes.push((pos, FloorType::Pit));
                    }
                }
            }
        }
        floor_changes.sort_by_key(|(pos, _)| (pos.z, pos.x));

        Self {
            player: current.player_pos.pos,
            boxes,
            collected_keys,
            removed_objects,
            active_switches,
            floor_changes,
        }
    }

    /// 胜利条件：所有可移动箱子都在目标点上（HeavyBox 不参与）
    fn is_solved(&self, targets: &[GridPos]) -> bool {
        let solvable: Vec<_> = self
            .boxes
            .iter()
            .filter(|(_, bt)| *bt != ObjectType::HeavyBox)
            .collect();
        if solvable.is_empty() {
            return false;
        }
        solvable.iter().all(|(pos, _)| targets.contains(pos))
    }
}

// ============================================================
//  内部辅助函数
// ============================================================

/// 从 came_from 回溯路径
fn reconstruct_path(
    came_from: &HashMap<SolverNode, (SolverNode, Direction)>,
    goal: &SolverNode,
) -> Vec<Direction> {
    let mut path = Vec::new();
    let mut current = goal;

    while let Some((parent, dir)) = came_from.get(current) {
        path.push(*dir);
        current = parent;
    }

    path.reverse();
    path
}

/// 把网格恢复到指定求解器节点的状态
///
/// `grid` 应当是 `initial_grid` 的 clone，本函数在其基础上应用所有差异。
/// HeavyBox 始终保留在 `box_positions` 中。
fn restore_to_node(grid: &mut GridState, node: &SolverNode) {
    for &(pos, floor) in &node.floor_changes {
        grid.set_floor(pos, floor);
    }

    for &pos in &node.removed_objects {
        grid.remove_object(pos);
    }

    grid.active_switches.clear();
    for &id in &node.active_switches {
        grid.active_switches.insert(id);
    }

    // 3. 恢复已收集的钥匙
    grid.collected_keys = node.collected_keys.clone();

    // 4. 重新放置所有箱子（包括 HeavyBox，全部在 box_positions 中）
    grid.box_positions.clear();
    for (i, &(pos, box_type)) in node.boxes.iter().enumerate() {
        grid.box_positions.push(TrackedBox {
            entity_id: (i + 1) as u64,
            pos: GridPos3D::new(pos.x, pos.z, 0),
            box_type,
        });
    }

    // 5. 设置玩家位置
    grid.player_pos = GridPos3D::new(node.player.x, node.player.z, 0);
}

/// 快速死锁检查（仅角落死锁，保证无误判；跳过 HeavyBox）
fn has_deadlock(node: &SolverNode, targets: &[GridPos], grid: &GridState) -> bool {
    for &(box_pos, box_type) in &node.boxes {
        // HeavyBox 不可移动，跳过死锁检查
        if box_type == ObjectType::HeavyBox {
            continue;
        }
        if targets.contains(&box_pos) {
            continue;
        }

        let left_blocked = is_blocked(grid, node, box_pos.shift(Direction::Left));
        let right_blocked = is_blocked(grid, node, box_pos.shift(Direction::Right));
        let up_blocked = is_blocked(grid, node, box_pos.shift(Direction::Up));
        let down_blocked = is_blocked(grid, node, box_pos.shift(Direction::Down));

        // 角落死锁：两个相邻方向都被阻挡
        if (left_blocked && up_blocked)
            || (left_blocked && down_blocked)
            || (right_blocked && up_blocked)
            || (right_blocked && down_blocked)
        {
            return true;
        }
    }
    false
}

/// 检查某个位置是否阻挡通行（用于死锁检测）
///
/// 综合考虑：边界、地板可通行性（含变更）、物体阻挡（含已移除）、其他箱子。
fn is_blocked(grid: &GridState, node: &SolverNode, pos: GridPos) -> bool {
    // 边界检查
    if !grid.in_bounds(pos) {
        return true;
    }

    // 地板检查（考虑地板变更，如玻璃变坑）
    let floor = node
        .floor_changes
        .iter()
        .find(|&&(p, _)| p == pos)
        .map(|&(_, f)| f)
        .unwrap_or_else(|| grid.floor_at(pos));
    if !floor.is_passable() {
        return true;
    }

    // 物体阻挡检查（跳过已移除的物体）
    if !node.removed_objects.contains(&pos) {
        match grid.object_at(pos) {
            // 不阻挡的物体
            ObjectType::None
            | ObjectType::Player
            | ObjectType::Key(_)
            | ObjectType::Switch(_)
            | ObjectType::Spring
            | ObjectType::Spikes
            | ObjectType::Magnet
            | ObjectType::Mirror(_) => {}
            // 石柱：取决于对应开关是否激活
            ObjectType::Pillar(id) => {
                if !node.active_switches.contains(&id) {
                    return true;
                }
            }
            // Wall, CrackedWall, Rock, Gate, Bomb 等均阻挡
            _ => {
                return true;
            }
        }
    }

    // 其他箱子阻挡（包括 HeavyBox）
    if node.boxes.iter().any(|&(bpos, _)| bpos == pos) {
        return true;
    }

    false
}
