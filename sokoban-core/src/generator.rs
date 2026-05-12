use rand::rngs::StdRng;
use rand::Rng;
use std::collections::{HashMap, HashSet};

use crate::grid::*;
use crate::level::{is_grid_connected, DungeonData, LevelData, LevelMeta, RoomConnection, RoomSlot};
use crate::solver::*;
use crate::types::*;

// ============================================================
//  公开 API — 经典关卡生成
// ============================================================

#[derive(Debug, Clone)]
pub struct GenParams {
    pub min_width: u32,
    pub max_width: u32,
    pub min_height: u32,
    pub max_height: u32,
    pub min_boxes: u32,
    pub max_boxes: u32,
    pub target_difficulty: u8,
    pub scene_theme: String,
    pub wall_density: f32,
    pub special_floor_density: f32,
    pub available_items: Vec<ObjectType>,
    pub max_retries: u32,
    pub solver_config: SolverConfig,
}

impl Default for GenParams {
    fn default() -> Self {
        Self {
            min_width: 6,
            max_width: 12,
            min_height: 6,
            max_height: 12,
            min_boxes: 1,
            max_boxes: 4,
            target_difficulty: 2,
            scene_theme: "default".to_string(),
            wall_density: 0.15,
            special_floor_density: 0.05,
            available_items: vec![],
            max_retries: 50,
            solver_config: SolverConfig {
                max_states: 200_000,
                timeout_ms: 3_000,
            },
        }
    }
}

#[derive(Debug)]
pub struct GenResult {
    pub grid: Grid,
    pub box_count: u32,
    pub optimal_steps: Option<u32>,
    pub difficulty: Option<DifficultyReport>,
    pub attempts: u32,
}

/// 生成一个可解的经典关卡
pub fn generate(params: &GenParams, rng: &mut StdRng) -> Option<GenResult> {
    for attempt in 1..=params.max_retries {
        let grid = generate_candidate(params, rng);

        let validation = crate::level::validate_level(&LevelData {
            meta: LevelMeta::default(),
            grid: Some(grid.clone()),
            ascii: None,
            scene_theme: params.scene_theme.clone(),
        });

        if !validation.is_valid {
            continue;
        }

        let grid_state = GridState::from_grid(&grid, 0);
        let solver_result = solve(&grid_state, &params.solver_config);

        if let Some(solution) = solver_result.solution {
            let steps = solution.len() as u32;
            let box_count = grid.count_boxes();
            let difficulty = evaluate_difficulty(&grid, steps);

            return Some(GenResult {
                grid,
                box_count,
                optimal_steps: Some(steps),
                difficulty: Some(difficulty),
                attempts: attempt,
            });
        }
    }

    None
}

// ============================================================
//  8 步生成流水线
// ============================================================

fn generate_candidate(params: &GenParams, rng: &mut StdRng) -> Grid {
    let width = rng.gen_range(params.min_width..=params.max_width);
    let height = rng.gen_range(params.min_height..=params.max_height);
    let box_count = rng.gen_range(params.min_boxes..=params.max_boxes);

    // 第 1 步：生成房间轮廓
    let mut grid = generate_outline(width, height, rng);

    // 第 2 步：生成内部地形（cellular automata 让特殊地板连成区域）
    generate_terrain(&mut grid, params.special_floor_density, rng);

    // 第 3 步：放置障碍物（随机加连通性检查）
    place_walls(&mut grid, params.wall_density, rng);

    // 第 4 步：放置功能物品（钥匙门配对、开关石柱配对、传送门成对）
    place_functional_items(&mut grid, &params.available_items, rng);

    // 第 5 步：放置箱子和目标点
    place_boxes_and_targets(&mut grid, box_count, rng);

    // 第 6 步：放置玩家
    place_player(&mut grid, rng);

    grid
}

// ============================================================
//  第一步：房间轮廓
// ============================================================

fn generate_outline(width: u32, height: u32, rng: &mut StdRng) -> Grid {
    let shape = rng.gen_range(0u32..3);
    let mut grid = Grid::new(width, height);

    let w = width as i32;
    let h = height as i32;
    let cx = w / 2;
    let cz = h / 2;
    let rx = (w - 2) as f32 / 2.0;
    let rz = (h - 2) as f32 / 2.0;

    for z in 0..h {
        for x in 0..w {
            let pos = GridPos::new(x, z);

            let inside = match shape {
                0 => {
                    // 矩形
                    x > 0 && z > 0 && x < w - 1 && z < h - 1
                }
                1 => {
                    // 圆形/椭圆
                    let dx = (x - cx) as f32 / rx;
                    let dz = (z - cz) as f32 / rz;
                    dx * dx + dz * dz < 1.0
                }
                _ => {
                    // 不规则（矩形 + 随机凸出/凹陷）
                    if x <= 0 || z <= 0 || x >= w - 1 || z >= h - 1 {
                        false
                    } else {
                        let corner_cut = rng.gen_bool(0.15);
                        if corner_cut
                            && ((x <= 1 && z <= 1)
                                || (x >= w - 2 && z <= 1)
                                || (x <= 1 && z >= h - 2)
                                || (x >= w - 2 && z >= h - 2))
                        {
                            false
                        } else {
                            true
                        }
                    }
                }
            };

            if inside {
                grid.set(pos, Cell::empty());
            } else {
                grid.set(pos, Cell::wall());
            }
        }
    }

    // 确保连通且最小面积
    let passable = get_passable_positions(&grid);
    let min_area = ((width - 2) * (height - 2) * 3 / 4) as usize;
    if !is_grid_connected(&grid) || passable.len() < min_area {
        // 回退到简单矩形
        let mut rect = Grid::new(width, height);
        for z in 0..h {
            for x in 0..w {
                let pos = GridPos::new(x, z);
                if x > 0 && z > 0 && x < w - 1 && z < h - 1 {
                    rect.set(pos, Cell::empty());
                } else {
                    rect.set(pos, Cell::wall());
                }
            }
        }
        return rect;
    }

    grid
}

// ============================================================
//  第二步：内部地形（cellular automata）
// ============================================================

fn generate_terrain(grid: &mut Grid, density: f32, rng: &mut StdRng) {
    if density <= 0.0 {
        return;
    }

    let passable = get_passable_positions(grid);
    let seed_count = (passable.len() as f32 * density * 0.5) as u32;

    let floor_options = [FloorType::Ice, FloorType::Mud, FloorType::Glass];

    // 随机撒种子
    let mut seeds = Vec::new();
    for _ in 0..seed_count {
        let idx = rng.gen_range(0..passable.len());
        let pos = passable[idx];
        let floor = floor_options[rng.gen_range(0..floor_options.len())];
        seeds.push((pos, floor));
    }

    // Cellular automata 扩展（3 轮）
    for _round in 0..3 {
        let mut new_seeds = Vec::new();
        for &(seed_pos, seed_floor) in &seeds {
            // 邻居有 40% 概率也变成同类型
            for dir in Direction::all() {
                let neighbor = seed_pos.shift(dir);
                if grid.in_bounds(neighbor)
                    && grid.get(neighbor).map_or(false, |c| {
                        c.floor == FloorType::Normal && c.object == ObjectType::None
                    })
                    && rng.gen_bool(0.4)
                {
                    if let Some(cell) = grid.get_mut(neighbor) {
                        cell.floor = seed_floor;
                    }
                    new_seeds.push((neighbor, seed_floor));
                }
            }
        }
        seeds.extend(new_seeds);
    }

    // 最终写入（限制不超过 passable 的 20%）
    let mut changed = 0u32;
    let max_changed = (passable.len() as f32 * 0.2) as u32;
    for &(pos, floor) in &seeds {
        if changed >= max_changed {
            break;
        }
        if let Some(cell) = grid.get_mut(pos) {
            if cell.floor == FloorType::Normal {
                cell.floor = floor;
                changed += 1;
            }
        }
    }
}

// ============================================================
//  第三步：放置墙壁（连通性检查）
// ============================================================

fn place_walls(grid: &mut Grid, density: f32, rng: &mut StdRng) {
    let passable = get_passable_positions(grid);
    let wall_count = (passable.len() as f32 * density) as u32;

    let mut placed = 0;
    let mut attempts = 0;
    let max_attempts = wall_count * 10;

    while placed < wall_count && attempts < max_attempts {
        attempts += 1;

        let idx = rng.gen_range(0..passable.len());
        let pos = passable[idx];

        // 不在玩家出生区域放置
        if pos.x <= 2 && pos.z <= 2 {
            continue;
        }

        let old_cell = grid.get(pos).cloned().unwrap_or_default();
        if old_cell.floor != FloorType::Normal || old_cell.object != ObjectType::None {
            continue;
        }

        grid.set(pos, Cell::wall());

        if is_grid_connected(grid) {
            placed += 1;
        } else {
            grid.set(pos, old_cell);
        }
    }
}

// ============================================================
//  第四步：放置功能物品
// ============================================================

fn place_functional_items(grid: &mut Grid, available: &[ObjectType], rng: &mut StdRng) {
    let passable = get_empty_passable_positions(grid);
    if passable.len() < 6 {
        return;
    }

    let mut used = HashSet::new();

    // 从可用物品中选择
    let has_keys = available.iter().any(|o| matches!(o, ObjectType::Key(_)));
    let has_switches = available.iter().any(|o| matches!(o, ObjectType::Switch(_)));
    let has_portals = available.iter().any(|o| matches!(o, ObjectType::Bomb));

    // 钥匙-门配对
    if has_keys || available.is_empty() {
        let pair_count = rng.gen_range(0u32..=2).min(passable.len() as u32 / 6);
        for i in 0..pair_count {
            let color = ItemColor::from_index(i as usize);
            let pool: Vec<GridPos> = passable.iter().filter(|p| !used.contains(*p)).copied().collect();
            if pool.len() < 2 {
                break;
            }

            // 钥匙放在较远位置
            let key_idx = rng.gen_range(0..pool.len());
            let key_pos = pool[key_idx];
            used.insert(key_pos);

            let remaining: Vec<GridPos> = pool.iter().filter(|p| **p != key_pos && !used.contains(*p)).copied().collect();
            if remaining.is_empty() {
                break;
            }

            // 门放在离钥匙最远的位置
            let gate_pos = remaining
                .iter()
                .max_by_key(|&&p| manhattan_distance(p, key_pos))
                .copied()
                .unwrap_or(remaining[0]);
            used.insert(gate_pos);

            grid.set(
                key_pos,
                Cell {
                    floor: FloorType::Normal,
                    object: ObjectType::Key(color),
                    color: Some(color),
                    facing: None,
                    linked_id: None,
                },
            );
            grid.set(
                gate_pos,
                Cell {
                    floor: FloorType::Normal,
                    object: ObjectType::Gate(color),
                    color: Some(color),
                    facing: None,
                    linked_id: None,
                },
            );
        }
    }

    // 开关-石柱配对
    if has_switches || available.is_empty() {
        let pair_count = rng.gen_range(0u32..=1);
        let pool: Vec<GridPos> = passable.iter().filter(|p| !used.contains(*p)).copied().collect();

        if pool.len() >= 2 && pair_count > 0 {
            let id = rng.gen_range(1u8..255);

            let switch_idx = rng.gen_range(0..pool.len());
            let switch_pos = pool[switch_idx];
            used.insert(switch_pos);

            let remaining: Vec<GridPos> = pool.iter().filter(|p| **p != switch_pos && !used.contains(*p)).copied().collect();
            if !remaining.is_empty() {
                let pillar_pos = remaining[rng.gen_range(0..remaining.len())];
                used.insert(pillar_pos);

                grid.set(
                    switch_pos,
                    Cell {
                        floor: FloorType::Normal,
                        object: ObjectType::Switch(id),
                        color: None,
                        facing: None,
                        linked_id: Some(id),
                    },
                );
                grid.set(
                    pillar_pos,
                    Cell {
                        floor: FloorType::Normal,
                        object: ObjectType::Pillar(id),
                        color: None,
                        facing: None,
                        linked_id: Some(id),
                    },
                );
            }
        }
    }

    // 传送门成对
    if has_portals || available.is_empty() {
        let pool: Vec<GridPos> = passable.iter().filter(|p| !used.contains(*p)).copied().collect();
        if pool.len() >= 2 && rng.gen_bool(0.3) {
            let id = rng.gen_range(0u8..16);
            let a_idx = rng.gen_range(0..pool.len());
            let a_pos = pool[a_idx];
            used.insert(a_pos);

            let mut b_pos = a_pos;
            let remaining: Vec<GridPos> = pool.iter().filter(|p| !used.contains(*p)).copied().collect();
            if !remaining.is_empty() {
                b_pos = remaining
                    .iter()
                    .max_by_key(|&&p| manhattan_distance(p, a_pos))
                    .copied()
                    .unwrap_or(remaining[0]);
                used.insert(b_pos);
            }

            grid.set(
                a_pos,
                Cell {
                    floor: FloorType::Portal(id),
                    object: ObjectType::None,
                    color: None,
                    facing: None,
                    linked_id: Some(id),
                },
            );
            grid.set(
                b_pos,
                Cell {
                    floor: FloorType::Portal(id),
                    object: ObjectType::None,
                    color: None,
                    facing: None,
                    linked_id: Some(id),
                },
            );
        }
    }
}

// ============================================================
//  第五步：放置箱子和目标点
// ============================================================

fn place_boxes_and_targets(grid: &mut Grid, box_count: u32, rng: &mut StdRng) {
    let passable = get_empty_passable_positions(grid);

    if passable.len() < (box_count * 2) as usize {
        return;
    }

    let mut used = HashSet::new();
    let mut targets = Vec::new();
    let mut boxes = Vec::new();

    // 目标点优先放在难以到达的位置（边缘、角落、死胡同）
    let hard_positions: Vec<GridPos> = passable
        .iter()
        .filter(|p| {
            let x_edge = p.x <= 1 || p.x >= (grid.width as i32 - 2);
            let z_edge = p.z <= 1 || p.z >= (grid.height as i32 - 2);
            let is_dead_end = count_empty_neighbors(grid, **p) <= 2;
            x_edge || z_edge || is_dead_end
        })
        .copied()
        .collect();

    let target_pool = if hard_positions.len() >= box_count as usize {
        &hard_positions
    } else {
        &passable
    };

    // 目标点互相远离
    for _ in 0..box_count {
        if let Some(pos) = pick_farthest_from_set(target_pool, &targets, &used, rng) {
            used.insert(pos);
            targets.push(pos);
        }
    }

    // 箱子放在离目标点远的位置（容易到达）
    let easy_positions: Vec<GridPos> = passable
        .iter()
        .filter(|p| !used.contains(*p))
        .filter(|p| count_empty_neighbors(grid, **p) >= 3)
        .copied()
        .collect();

    let box_pool = if easy_positions.len() >= box_count as usize {
        &easy_positions
    } else {
        &passable
    };

    for _ in 0..box_count {
        if let Some(pos) = pick_farthest_from_set(box_pool, &targets, &used, rng) {
            used.insert(pos);
            boxes.push(pos);
        }
    }

    // 写入网格
    for pos in targets {
        grid.set(
            pos,
            Cell {
                floor: FloorType::Target,
                object: ObjectType::None,
                color: None,
                facing: None,
                linked_id: None,
            },
        );
    }

    for &pos in &boxes {
        let box_type = if rng.gen_bool(0.15) {
            ObjectType::HeavyBox
        } else if rng.gen_bool(0.1) {
            ObjectType::FragileBox
        } else {
            ObjectType::Box
        };

        grid.set(
            pos,
            Cell {
                floor: FloorType::Normal,
                object: box_type,
                color: None,
                facing: None,
                linked_id: None,
            },
        );
    }
}

// ============================================================
//  第六步：放置玩家
// ============================================================

fn place_player(grid: &mut Grid, rng: &mut StdRng) {
    let passable: Vec<GridPos> = get_empty_passable_positions(grid)
        .into_iter()
        .filter(|&pos| {
            grid.get(pos)
                .map_or(true, |c| c.floor != FloorType::Target)
        })
        .collect();

    // 收集箱子位置
    let box_positions: Vec<GridPos> = grid_cells_iter(grid)
        .filter(|(_, c)| c.object.is_box())
        .map(|(p, _)| p)
        .collect();

    // 优先放在箱子附近（1-2 格距离）
    let mut near_box: Vec<GridPos> = Vec::new();
    for &box_pos in &box_positions {
        for dist in 1..=2 {
            let candidates = positions_at_distance(box_pos, dist);
            for c in candidates {
                if passable.contains(&c) {
                    near_box.push(c);
                }
            }
        }
    }

    if near_box.is_empty() {
        near_box = passable.clone();
    }

    let pos = near_box[rng.gen_range(0..near_box.len())];

    grid.set(
        pos,
        Cell {
            floor: FloorType::Normal,
            object: ObjectType::Player,
            color: None,
            facing: None,
            linked_id: None,
        },
    );
}

// ============================================================
//  第八步（改）：难度评估
// ============================================================

pub fn evaluate_difficulty(grid: &Grid, optimal_steps: u32) -> DifficultyReport {
    let box_count = grid.count_boxes();
    let _target_count = grid.count_targets();
    let item_count = count_items(grid);
    let branch_factor = compute_branching_factor(grid);
    let has_dead_ends = has_potential_dead_ends(grid);

    // 综合性评分
    let mut score = 0.0;
    score += box_count as f32 * 10.0;
    score += optimal_steps as f32 * 2.0;
    score += item_count as f32 * 5.0;
    if has_dead_ends {
        score += 15.0;
    }
    score += (1.0 - branch_factor.min(1.0)) * 30.0;

    let star_rating = if score < 20.0 {
        1
    } else if score < 40.0 {
        2
    } else if score < 65.0 {
        3
    } else if score < 90.0 {
        4
    } else {
        5
    };

    DifficultyReport {
        score,
        star_rating,
        optimal_steps: Some(optimal_steps),
        box_count,
        item_count,
        has_dead_ends,
        branching_factor: branch_factor,
    }
}

// ============================================================
//  地牢生成
// ============================================================

#[derive(Debug, Clone)]
pub struct DungeonGenParams {
    pub min_rooms: u32,
    pub max_rooms: u32,
    pub theme: String,
    pub room_gen_params: GenParams,
}

impl Default for DungeonGenParams {
    fn default() -> Self {
        Self {
            min_rooms: 7,
            max_rooms: 12,
            theme: "default".to_string(),
            room_gen_params: GenParams::default(),
        }
    }
}

/// 生成一个完整地牢
pub fn generate_dungeon(params: &DungeonGenParams, rng: &mut StdRng) -> Option<DungeonData> {
    let room_count = rng.gen_range(params.min_rooms..=params.max_rooms);

    // 1. 生成房间拓扑（BFS 扩展保证连通）
    let topology = generate_room_topology(room_count, rng);
    if topology.is_empty() {
        return None;
    }

    // 2. 分配房间类型（前期简单，后期复杂）
    let room_types = assign_room_types(&topology, rng);

    // 3. 为每个房间生成内部地图
    let mut rooms = HashMap::new();
    for (coord, room_type) in &room_types {
        let difficulty = match room_type {
            RoomType::Boss => 4u8,
            RoomType::Challenge => 3,
            RoomType::Puzzle => rng.gen_range(1u8..=3),
            _ => 1,
        };

        let mut room_params = params.room_gen_params.clone();
        room_params.min_boxes = rng.gen_range(1u32..=3);
        room_params.max_boxes = room_params.min_boxes + 2;
        room_params.target_difficulty = difficulty;

        // 多次尝试生成可解房间
        let mut room_grid = None;
        for _ in 0..20 {
            if let Some(result) = generate(&room_params, rng) {
                room_grid = Some(result.grid);
                break;
            }
        }

        let grid = room_grid.unwrap_or_else(|| {
            let mut g = Grid::new(6, 6);
            for z in 0..6i32 {
                for x in 0..6i32 {
                    let pos = GridPos::new(x, z);
                    if x == 0 || z == 0 || x == 5 || z == 5 {
                        g.set(pos, Cell::wall());
                    }
                }
            }
            g.set(GridPos::new(3, 3), Cell {
                floor: FloorType::Target,
                object: ObjectType::Player,
                color: None, facing: None, linked_id: None,
            });
            g
        });

        let connections = topology
            .get(coord)
            .map(|conns| {
                conns
                    .iter()
                    .map(|(dir, target)| RoomConnection {
                        direction: *dir,
                        target_room: format!("{}_{}", target.0, target.1),
                        is_locked: false,
                        lock_color: None,
                    })
                    .collect()
            })
            .unwrap_or_default();

        let reward = match room_type {
            RoomType::Treasure => Some(RewardType::Item(DungeonItem::BombPickup)),
            RoomType::Shop => Some(RewardType::ExtraUndo),
            RoomType::Boss => Some(RewardType::UnlockKey("master_key".to_string())),
            RoomType::Challenge => Some(RewardType::HintToken),
            RoomType::Rest => Some(RewardType::ExtraUndo),
            _ => None,
        };

        let room_id = format!("{}_{}", coord.0, coord.1);
        rooms.insert(
            room_id.clone(),
            RoomSlot {
                room_id: room_id.clone(),
                room_type: *room_type,
                connections,
                floor_level: 0,
                reward,
                grid,
            },
        );
    }

    // 4. 随机锁一些门并放置对应的钥匙
    add_locked_doors(&mut rooms, rng);

    // 5. 确定起始房间和 Boss 房间
    let start_room = format!("{}_{}", 0, 0);
    let boss_room = rooms
        .iter()
        .find(|(_, r)| r.room_type == RoomType::Boss)
        .map(|(id, _)| id.clone())
        .unwrap_or_else(|| {
            rooms.keys().last().cloned().unwrap_or_else(|| "0_0".to_string())
        });

    // 6. 验证地牢可达性
    if !is_dungeon_solvable(&rooms, &start_room, &boss_room) {
        return None;
    }

    Some(DungeonData {
        id: rng.gen_range(1u32..999999),
        name: format!("Generated Dungeon {}", rng.gen_range(1u32..999)),
        theme: params.theme.clone(),
        rooms,
        start_room,
        boss_room,
    })
}

// ============================================================
//  地牢生成辅助
// ============================================================

type RoomCoord = (i32, i32);
type Topology = HashMap<RoomCoord, Vec<(Direction, RoomCoord)>>;

fn generate_room_topology(room_count: u32, rng: &mut StdRng) -> Topology {
    let mut topology: Topology = HashMap::new();
    let mut visited: HashSet<RoomCoord> = HashSet::new();
    let mut frontier: Vec<RoomCoord> = Vec::new();

    // 起始房间
    let start = (0i32, 0i32);
    visited.insert(start);
    frontier.push(start);
    topology.insert(start, Vec::new());

    let directions = Direction::all();

    while topology.len() < room_count as usize {
        if frontier.is_empty() {
            break;
        }

        // 随机选择一个边界房间扩展
        let idx = rng.gen_range(0..frontier.len());
        let current = frontier[idx];

        // 尝试向随机方向扩展
        let mut expanded = false;
        for dir in directions.iter().cycle().skip(rng.gen_range(0..4)).take(4) {
            let next = match dir {
                Direction::Up => (current.0, current.1 - 1),
                Direction::Down => (current.0, current.1 + 1),
                Direction::Left => (current.0 - 1, current.1),
                Direction::Right => (current.0 + 1, current.1),
            };

            if !visited.contains(&next) && topology.len() < room_count as usize {
                visited.insert(next);
                frontier.push(next);
                topology.insert(next, Vec::new());
                topology.get_mut(&current).unwrap().push((*dir, next));
                topology.get_mut(&next).unwrap().push((dir.opposite(), current));
                expanded = true;
                break;
            }
        }

        if !expanded {
            frontier.remove(idx);
        }
    }

    // 添加一些额外连接（20% 概率）
    let all_rooms: Vec<RoomCoord> = topology.keys().copied().collect();
    for i in 0..all_rooms.len() {
        if rng.gen_bool(0.2) && i + 1 < all_rooms.len() {
            let a = all_rooms[i];
            let b = all_rooms[i + 1];
            let dir = direction_between(a, b);
            if !topology.get(&a).map_or(false, |conns| conns.iter().any(|(_, t)| *t == b)) {
                topology.get_mut(&a).unwrap().push((dir, b));
                topology.get_mut(&b).unwrap().push((dir.opposite(), a));
            }
        }
    }

    topology
}

fn assign_room_types(topology: &Topology, rng: &mut StdRng) -> HashMap<RoomCoord, RoomType> {
    let mut types = HashMap::new();

    // 按距起始房间的距离排序
    let start = (0i32, 0i32);
    let mut rooms_with_dist: Vec<(RoomCoord, i32)> = topology
        .keys()
        .map(|&c| {
            let dist = (c.0 - start.0).abs() + (c.1 - start.1).abs();
            (c, dist)
        })
        .collect();
    rooms_with_dist.sort_by_key(|(_, d)| *d);

    let total = rooms_with_dist.len();
    for (i, (coord, _dist)) in rooms_with_dist.iter().enumerate() {
        let room_type = if i == 0 {
            RoomType::Rest // 起始房间
        } else if i == total - 1 {
            RoomType::Boss // 最后一间是 Boss
        } else if rng.gen_bool(0.2) {
            RoomType::Treasure
        } else if rng.gen_bool(0.1) {
            RoomType::Shop
        } else if rng.gen_bool(0.15) {
            RoomType::Challenge
        } else if rng.gen_bool(0.1) {
            RoomType::Rest
        } else {
            RoomType::Puzzle
        };
        types.insert(*coord, room_type);
    }

    types
}

fn add_locked_doors(rooms: &mut HashMap<String, RoomSlot>, rng: &mut StdRng) {
    let room_ids: Vec<String> = rooms.keys().cloned().collect();
    if room_ids.len() < 3 {
        return;
    }

    // 随机选一个非 Boss 房间的门上锁
    let lock_count = rng.gen_range(0u32..=2).min(room_ids.len() as u32 / 2);
    for i in 0..lock_count {
        let color = ItemColor::from_index(i as usize);

        // 找一个有连接的房间
        let idx = rng.gen_range(0..room_ids.len());
        let room_id = &room_ids[idx];

        let prev_room_id = {
            let Some(room) = rooms.get_mut(room_id) else { continue };
            if room.room_type == RoomType::Boss || room.connections.is_empty() {
                continue;
            }
            let conn_idx = rng.gen_range(0..room.connections.len());
            room.connections[conn_idx].is_locked = true;
            room.connections[conn_idx].lock_color = Some(color);
            room.connections[conn_idx].target_room.clone()
        };

        if let Some(prev_room) = rooms.get_mut(&prev_room_id) {
            let key_pos = GridPos::new(1, 1);
            if prev_room.grid.get(key_pos).map_or(false, |c| c.floor.is_passable()) {
                prev_room.grid.set(
                    key_pos,
                    Cell {
                        floor: FloorType::Normal,
                        object: ObjectType::Key(color),
                        color: Some(color),
                        facing: None,
                        linked_id: None,
                    },
                );
            }
        }
    }
}

fn is_dungeon_solvable(
    rooms: &HashMap<String, RoomSlot>,
    start: &str,
    boss: &str,
) -> bool {
    let mut visited = HashSet::new();
    let mut queue = std::collections::VecDeque::new();
    visited.insert(start.to_string());
    queue.push_back(start.to_string());

    while let Some(current) = queue.pop_front() {
        if current == boss {
            return true;
        }
        if let Some(room) = rooms.get(&current) {
            for conn in &room.connections {
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

fn direction_between(a: RoomCoord, b: RoomCoord) -> Direction {
    if b.0 > a.0 {
        Direction::Right
    } else if b.0 < a.0 {
        Direction::Left
    } else if b.1 > a.1 {
        Direction::Down
    } else {
        Direction::Up
    }
}

// ============================================================
//  通用辅助函数
// ============================================================

fn get_passable_positions(grid: &Grid) -> Vec<GridPos> {
    let mut result = Vec::new();
    for z in 0..grid.height as i32 {
        for x in 0..grid.width as i32 {
            let pos = GridPos::new(x, z);
            if let Some(cell) = grid.get(pos) {
                if cell.floor.is_passable() {
                    result.push(pos);
                }
            }
        }
    }
    result
}

fn get_empty_passable_positions(grid: &Grid) -> Vec<GridPos> {
    let mut result = Vec::new();
    for z in 0..grid.height as i32 {
        for x in 0..grid.width as i32 {
            let pos = GridPos::new(x, z);
            if let Some(cell) = grid.get(pos) {
                if cell.floor.is_passable() && cell.object == ObjectType::None {
                    result.push(pos);
                }
            }
        }
    }
    result
}

fn grid_cells_iter(grid: &Grid) -> impl Iterator<Item = (GridPos, &Cell)> {
    let h = grid.height;
    let w = grid.width;
    (0..h as i32).flat_map(move |z| {
        (0..w as i32).filter_map(move |x| {
            let pos = GridPos::new(x, z);
            grid.get(pos).map(|cell| (pos, cell))
        })
    })
}

fn count_empty_neighbors(grid: &Grid, pos: GridPos) -> u32 {
    let mut count = 0;
    for dir in Direction::all() {
        let n = pos.shift(dir);
        if grid.in_bounds(n)
            && grid
                .get(n)
                .map_or(false, |c| c.floor.is_passable() && c.object == ObjectType::None)
        {
            count += 1;
        }
    }
    count
}

fn count_items(grid: &Grid) -> u32 {
    let mut count = 0;
    for row in &grid.cells {
        for cell in row {
            match cell.object {
                ObjectType::Key(_)
                | ObjectType::Gate(_)
                | ObjectType::Switch(_)
                | ObjectType::Pillar(_)
                | ObjectType::Bomb
                | ObjectType::Spring
                | ObjectType::Mirror(_)
                | ObjectType::Magnet
                | ObjectType::Spikes => count += 1,
                _ => {}
            }
        }
    }
    count
}

fn has_potential_dead_ends(grid: &Grid) -> bool {
    for z in 1..grid.height as i32 - 1 {
        for x in 1..grid.width as i32 - 1 {
            let pos = GridPos::new(x, z);
            if grid.get(pos).map_or(false, |c| c.floor.is_passable()) {
                if count_empty_neighbors(grid, pos) <= 1 {
                    return true;
                }
            }
        }
    }
    false
}

fn compute_branching_factor(grid: &Grid) -> f32 {
    let passable = get_passable_positions(grid);
    if passable.is_empty() {
        return 0.0;
    }

    let total_branches: u32 = passable
        .iter()
        .map(|pos| {
            let mut branches = 0;
            for dir in Direction::all() {
                let n = pos.shift(dir);
                if grid.in_bounds(n)
                    && grid
                        .get(n)
                        .map_or(false, |c| c.floor.is_passable())
                {
                    branches += 1;
                }
            }
            branches
        })
        .sum();

    total_branches as f32 / passable.len() as f32
}

fn positions_at_distance(center: GridPos, dist: i32) -> Vec<GridPos> {
    let mut positions = Vec::new();
    for dx in -dist..=dist {
        for dz in -dist..=dist {
            if dx.abs() + dz.abs() == dist {
                positions.push(GridPos::new(center.x + dx, center.z + dz));
            }
        }
    }
    positions
}

fn pick_farthest_from_set(
    pool: &[GridPos],
    reference: &[GridPos],
    used: &HashSet<GridPos>,
    rng: &mut StdRng,
) -> Option<GridPos> {
    let available: Vec<GridPos> = pool.iter().filter(|p| !used.contains(*p)).copied().collect();
    if available.is_empty() {
        return None;
    }

    if reference.is_empty() {
        return Some(available[rng.gen_range(0..available.len())]);
    }

    let mut scored: Vec<(GridPos, f32)> = available
        .iter()
        .map(|&pos| {
            let min_dist = reference
                .iter()
                .map(|t| manhattan_distance(pos, *t))
                .min()
                .unwrap_or(i32::MAX) as f32;
            (pos, min_dist)
        })
        .collect();

    scored.sort_by(|a, b| b.1.partial_cmp(&a.1).unwrap());

    let top_count = (scored.len() / 3).max(1);
    let idx = rng.gen_range(0..top_count);
    Some(scored[idx].0)
}

fn manhattan_distance(a: GridPos, b: GridPos) -> i32 {
    (a.x - b.x).abs() + (a.z - b.z).abs()
}