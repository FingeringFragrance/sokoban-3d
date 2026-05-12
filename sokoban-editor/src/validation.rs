use bevy::prelude::*;
use crate::grid::{CellKind, GridData};

#[derive(Clone, PartialEq, Eq)]
pub struct ValidationResult {
    pub errors: Vec<String>,
    pub warnings: Vec<String>,
    pub solvable: Option<bool>,
}

pub fn validate_level(grid: &GridData) -> ValidationResult {
    let mut errors = Vec::new();
    let mut warnings = Vec::new();

    let mut box_count = 0u32;
    let mut target_count = 0u32;
    let mut player_count = 0u32;

    for x in 0..grid.width as i32 {
        for z in 0..grid.height as i32 {
            match grid.get(x, z) {
                CellKind::Box => box_count += 1,
                CellKind::Target => target_count += 1,
                CellKind::Player => player_count += 1,
                _ => {}
            }
        }
    }

    if player_count == 0 {
        errors.push("缺少玩家".into());
    }
    if player_count > 1 {
        errors.push("存在多个玩家".into());
    }
    if box_count == 0 {
        errors.push("缺少箱子".into());
    }
    if target_count == 0 {
        errors.push("缺少目标点".into());
    }
    if box_count != target_count {
        errors.push(format!("箱子({})与目标点({})数量不匹配", box_count, target_count));
    }

    let mut deadlocked = 0u32;
    let targets: Vec<(i32, i32)> = (0..grid.width as i32).flat_map(|x| {
        (0..grid.height as i32).filter_map(move |z| {
            if grid.get(x, z) == CellKind::Target { Some((x, z)) } else { None }
        })
    }).collect();

    for x in 0..grid.width as i32 {
        for z in 0..grid.height as i32 {
            if grid.get(x, z) == CellKind::Box
                && !targets.contains(&(x, z))
                && is_deadlock_corner(x, z, grid)
            {
                deadlocked += 1;
            }
        }
    }
    if deadlocked > 0 {
        warnings.push(format!("{}个箱子处于死角位置", deadlocked));
    }

    ValidationResult {
        errors,
        warnings,
        solvable: None,
    }
}

pub fn is_deadlock_corner(x: i32, z: i32, grid: &GridData) -> bool {
    let up = grid.get(x, z - 1);
    let down = grid.get(x, z + 1);
    let left = grid.get(x - 1, z);
    let right = grid.get(x + 1, z);

    let is_blocked = |c: CellKind| c == CellKind::Wall || c == CellKind::Decoration;

    let up_blocked = is_blocked(up) || z == 0;
    let down_blocked = is_blocked(down) || z == grid.height as i32 - 1;
    let left_blocked = is_blocked(left) || x == 0;
    let right_blocked = is_blocked(right) || x == grid.width as i32 - 1;

    (up_blocked && left_blocked) || (up_blocked && right_blocked)
        || (down_blocked && left_blocked) || (down_blocked && right_blocked)
}

pub fn try_solve(grid: &GridData) -> Option<bool> {
    use std::collections::{HashSet, VecDeque};

    let mut player_pos = None;
    let mut boxes = Vec::new();
    let mut targets = Vec::new();
    let mut keys = Vec::new();
    let mut gates = Vec::new();

    for x in 0..grid.width as i32 {
        for z in 0..grid.height as i32 {
            match grid.get(x, z) {
                CellKind::Player => player_pos = Some((x, z)),
                CellKind::Box => boxes.push((x, z)),
                CellKind::Target => targets.push((x, z)),
                CellKind::Key => keys.push((x, z)),
                CellKind::Gate => gates.push((x, z)),
                _ => {}
            }
        }
    }

    let player = player_pos?;
    if boxes.len() != targets.len() { return Some(false); }
    if boxes.is_empty() { return Some(false); }

    let is_wall = |x: i32, z: i32| -> bool {
        let c = grid.get(x, z);
        c == CellKind::Wall || c == CellKind::Decoration
    };

    let is_gate = |x: i32, z: i32| -> bool {
        gates.contains(&(x, z))
    };

    let mut boxes_sorted = boxes.clone();
    boxes_sorted.sort();
    let targets_set: HashSet<(i32, i32)> = targets.iter().copied().collect();
    let keys_set: HashSet<(i32, i32)> = keys.iter().copied().collect();

    let initial_state = (player, boxes_sorted, false);
    let mut visited: HashSet<((i32, i32), Vec<(i32, i32)>, bool)> = HashSet::new();
    let mut queue: VecDeque<(((i32, i32), Vec<(i32, i32)>, bool), u32)> = VecDeque::new();
    queue.push_back((initial_state.clone(), 0u32));
    visited.insert(initial_state);

    let max_steps = 2000u32;

    while let Some(((player_state, bxs, has_key), steps)) = queue.pop_front() {
        let (px, pz) = player_state;

        let all_on_target = bxs.iter().all(|b| targets_set.contains(b));
        if all_on_target {
            return Some(true);
        }
        if steps >= max_steps { continue; }

        let dirs = [(0, -1), (0, 1), (-1, 0), (1, 0)];
        for &(dx, dz) in &dirs {
            let nx = px + dx;
            let nz = pz + dz;
            if is_wall(nx, nz) { continue; }

            if is_gate(nx, nz) && !has_key { continue; }

            if let Some(bi) = bxs.iter().position(|&b| b == (nx, nz)) {
                let bx = nx + dx;
                let bz = nz + dz;
                if is_wall(bx, bz) { continue; }
                if is_gate(bx, bz) { continue; }
                if bxs.iter().any(|&b| b == (bx, bz)) { continue; }

                let mut new_boxes = bxs.clone();
                new_boxes[bi] = (bx, bz);
                new_boxes.sort();

                let mut dead = false;
                for &(bx, bz) in &new_boxes {
                    if !targets_set.contains(&(bx, bz)) && is_deadlock_corner(bx, bz, grid) {
                        dead = true;
                        break;
                    }
                }
                if dead { continue; }

                let new_state = ((nx, nz), new_boxes, has_key);
                if visited.insert(new_state.clone()) {
                    queue.push_back((new_state, steps + 1));
                }
            } else {
                let new_has_key = has_key || keys_set.contains(&(nx, nz));
                let new_state = ((nx, nz), bxs.clone(), new_has_key);
                if visited.insert(new_state.clone()) {
                    queue.push_back((new_state, steps + 1));
                }
            }
        }
    }

    None
}

#[derive(Resource, Default, Clone)]
pub struct ValidationState {
    pub result: Option<ValidationResult>,
    pub running: bool,
    pub pending_grid: Option<GridData>,
}

pub fn run_validation(
    keyboard: Res<ButtonInput<KeyCode>>,
    playtest: Res<crate::playtest::PlaytestState>,
    grid: Res<GridData>,
    mut state: ResMut<ValidationState>,
) {
    if playtest.active { return; }
    if keyboard.just_pressed(KeyCode::F5) {
        let result = validate_level(&grid);
        if result.errors.is_empty() {
            state.running = true;
            state.pending_grid = Some(GridData {
                cells: grid.cells.clone(),
                width: grid.width,
                height: grid.height,
                version: grid.version,
            });
        }
        state.result = Some(result);
    }
}

pub fn run_solver(
    mut state: ResMut<ValidationState>,
) {
    if !state.running { return; }
    if let Some(grid_clone) = state.pending_grid.take() {
        let solvable = try_solve(&grid_clone);
        if let Some(ref mut result) = state.result {
            result.solvable = solvable;
        }
        state.running = false;
    }
}