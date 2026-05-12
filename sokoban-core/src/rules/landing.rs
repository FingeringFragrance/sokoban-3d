use crate::grid::*;
use crate::types::*;

use super::movement::PLAYER_ENTITY_ID;

const MAX_SLIDE_DEPTH: u32 = 50;

pub fn resolve_landing(
    grid: &mut GridState,
    entity_id: u64,
    pos: GridPos3D,
    direction: Direction,
    scene: &SceneTheme,
    result: &mut MoveResult,
    depth: u32,
) {
    if depth > MAX_SLIDE_DEPTH {
        return;
    }

    let is_player = entity_id == PLAYER_ENTITY_ID;
    let floor = grid.floor_at(pos.pos);
    let obj = grid.object_at(pos.pos);

    // --- Mirror redirect ---
    if let ObjectType::Mirror(mirror_dir) = obj {
        // 镜子改变推力方向 90 度（以镜子朝向为基准）
        let new_dir = mirror_deflect(mirror_dir);
        let target = pos.shift(new_dir);
        if can_move_to(grid, target.pos, entity_id) {
            let target_floor = grid.floor_at(target.pos);
            result.steps.push(MoveStep {
                entity_id,
                from: pos,
                to: target,
                step_type: MoveStepType::Push,
                floor_type: target_floor,
            });
            if is_player {
                grid.move_player(target);
            } else {
                grid.move_box(entity_id, target);
            }
            resolve_landing(grid, entity_id, target, new_dir, scene, result, depth + 1);
        }
        return;
    }

    // --- Spikes destroys boxes ---
    if obj == ObjectType::Spikes && !is_player {
        result.steps.push(MoveStep {
            entity_id,
            from: pos,
            to: pos,
            step_type: MoveStepType::Destroy,
            floor_type: floor,
        });
        grid.remove_box(entity_id);
        result.destroyed_entities.push(entity_id);
        return;
    }

    // --- Magnet pull (box lands adjacent to magnet, gets pulled in) ---
    if !is_player {
        magnet_pull(grid, entity_id, pos, result);
    }

    // --- Forced movement: Ice, Conveyor, Portal, Spring ---
    if let Some((target, step_type, move_dir)) =
        get_forced_movement(grid, entity_id, pos, direction, floor, obj, scene)
    {
        let target_floor = grid.floor_at(target.pos);
        result.steps.push(MoveStep {
            entity_id,
            from: pos,
            to: target,
            step_type,
            floor_type: target_floor,
        });
        if is_player {
            grid.move_player(target);
        } else {
            grid.move_box(entity_id, target);
        }
        resolve_landing(grid, entity_id, target, move_dir, scene, result, depth + 1);
        return;
    }

    // --- Hazard floors ---
    match floor {
        FloorType::Water | FloorType::Pit => {
            if !is_player {
                result.steps.push(MoveStep {
                    entity_id,
                    from: pos,
                    to: pos,
                    step_type: MoveStepType::Fall,
                    floor_type: floor,
                });
                grid.remove_box(entity_id);
                result.destroyed_entities.push(entity_id);
                return;
            } else {
                result.player_died = true;
                return;
            }
        }
        FloorType::Glass => {
            if !is_player {
                grid.set_floor(pos.pos, FloorType::Pit);
            }
        }
        FloorType::Mud => {
            // 泥地减速：额外消耗一步
            grid.advance_step();
        }
        _ => {}
    }

    resolve_items(grid, entity_id, pos, result);
    resolve_triggers(grid, pos, result);

    // 跨层移动检查
    resolve_cross_floor(grid, entity_id, pos, result);

    let _ = scene;
}

/// 检查并执行跨层移动
fn resolve_cross_floor(
    grid: &mut GridState,
    entity_id: u64,
    pos: GridPos3D,
    result: &mut MoveResult,
) {
    let current_floor = pos.floor;
    let target = grid.connections.iter().find_map(|conn| {
        if conn.from_floor == current_floor && conn.from_pos == pos.pos {
            Some(GridPos3D::new(conn.to_pos.x, conn.to_pos.z, conn.to_floor))
        } else {
            None
        }
    });

    if let Some(target) = target {
        result.steps.push(MoveStep {
            entity_id,
            from: pos,
            to: target,
            step_type: MoveStepType::Teleport,
            floor_type: FloorType::Normal,
        });
        if entity_id == PLAYER_ENTITY_ID {
            grid.move_player(target);
            grid.floor_layer = target.floor;
        } else {
            grid.move_box(entity_id, target);
        }
    }
}

/// 镜子偏转方向：顺时针 90 度
fn mirror_deflect(mirror_facing: Direction) -> Direction {
    match mirror_facing {
        Direction::Up => Direction::Right,
        Direction::Right => Direction::Down,
        Direction::Down => Direction::Left,
        Direction::Left => Direction::Up,
    }
}

/// 磁铁吸引：箱子停在磁铁旁时被吸到磁铁位置上并销毁
fn magnet_pull(grid: &mut GridState, entity_id: u64, pos: GridPos3D, result: &mut MoveResult) {
    for dir in Direction::all() {
        let adj = pos.pos.shift(dir);
        if grid.object_at(adj) == ObjectType::Magnet {
            let magnet_pos = GridPos3D::new(adj.x, adj.z, pos.floor);
            result.steps.push(MoveStep {
                entity_id,
                from: pos,
                to: magnet_pos,
                step_type: MoveStepType::Destroy,
                floor_type: grid.floor_at(adj),
            });
            grid.remove_box(entity_id);
            result.destroyed_entities.push(entity_id);
            return;
        }
    }
}

fn get_forced_movement(
    grid: &GridState,
    entity_id: u64,
    pos: GridPos3D,
    direction: Direction,
    floor: FloorType,
    obj: ObjectType,
    scene: &SceneTheme,
) -> Option<(GridPos3D, MoveStepType, Direction)> {
    // Spring bounce
    if obj == ObjectType::Spring && entity_id != PLAYER_ENTITY_ID {
        let bounce_steps = (1.0 * scene.environment_rules.gravity_multiplier).max(1.0) as u32;
        let mut target = pos;
        for _ in 0..bounce_steps {
            let next = target.shift(direction);
            if can_move_to(grid, next.pos, entity_id) && grid.find_box_at(next.pos).is_none() {
                target = next;
            } else {
                break;
            }
        }
        if target != pos {
            return Some((target, MoveStepType::SpringBounce, direction));
        }
    }

    let box_type = if entity_id != PLAYER_ENTITY_ID {
        grid.box_positions
            .iter()
            .find(|b| b.entity_id == entity_id)
            .map(|b| b.box_type)
    } else {
        None
    };

    match floor {
        FloorType::Ice => {
            if let Some(ObjectType::HeavyBox) = box_type {
                return None;
            }
            // 摩擦系数影响滑行：低摩擦 = 高概率继续滑行
            let friction = scene.environment_rules.friction;
            if friction < 1.0 {
                let target = pos.shift(direction);
                if can_move_to(grid, target.pos, entity_id) && grid.find_box_at(target.pos).is_none() {
                    return Some((target, MoveStepType::Slide, direction));
                }
            }
        }
        FloorType::Conveyor(conv_dir) => {
            if let Some(bt) = box_type {
                if bt == ObjectType::IceBox || bt == ObjectType::HeavyBox {
                    return None;
                }
            }
            let target = pos.shift(conv_dir);
            if can_move_to(grid, target.pos, entity_id) && grid.find_box_at(target.pos).is_none() {
                return Some((target, MoveStepType::Conveyor, conv_dir));
            }
        }
        FloorType::Portal(id) => {
            if let Some(dest) = find_matching_portal(grid, id, pos.pos) {
                let dest_3d = GridPos3D::new(dest.x, dest.z, pos.floor);
                return Some((dest_3d, MoveStepType::Teleport, direction));
            }
        }
        _ => {}
    }

    None
}

fn resolve_items(
    grid: &mut GridState,
    entity_id: u64,
    pos: GridPos3D,
    result: &mut MoveResult,
) {
    let obj = grid.object_at(pos.pos);

    match obj {
        ObjectType::Key(color) => {
            if entity_id == PLAYER_ENTITY_ID {
                grid.collect_key(color);
                grid.remove_object(pos.pos);
                result.collected_keys.push(color);
                result.steps.push(MoveStep {
                    entity_id,
                    from: pos,
                    to: pos,
                    step_type: MoveStepType::Collect,
                    floor_type: grid.floor_at(pos.pos),
                });
            }
        }
        _ => {}
    }
}

fn resolve_triggers(grid: &mut GridState, pos: GridPos3D, result: &mut MoveResult) {
    let obj = grid.object_at(pos.pos);
    if let ObjectType::Switch(id) = obj {
        grid.toggle_switch(id);
        result.triggered_switches.push(id);
    }

    if let FloorType::PressurePlate = grid.floor_at(pos.pos) {
        if let Some(cell) = grid.get(pos.pos) {
            if let Some(id) = cell.linked_id {
                grid.toggle_switch(id);
                result.triggered_switches.push(id);
            }
        }
    }
}

fn can_move_to(grid: &GridState, pos: GridPos, _self_id: u64) -> bool {
    if !grid.in_bounds(pos) {
        return false;
    }
    if !grid.floor_at(pos).is_passable() {
        return false;
    }
    !is_object_blocking(grid, pos)
}

fn is_object_blocking(grid: &GridState, pos: GridPos) -> bool {
    match grid.object_at(pos) {
        ObjectType::None
        | ObjectType::Player
        | ObjectType::Key(_)
        | ObjectType::Switch(_)
        | ObjectType::Spring
        | ObjectType::Spikes
        | ObjectType::Magnet
        | ObjectType::Mirror(_) => false,
        ObjectType::Pillar(id) => !grid.is_switch_active(id),
        _ => true,
    }
}

fn find_matching_portal(grid: &GridState, portal_id: u8, current_pos: GridPos) -> Option<GridPos> {
    for z in 0..grid.height as i32 {
        for x in 0..grid.width as i32 {
            let pos = GridPos::new(x, z);
            if pos == current_pos {
                continue;
            }
            if let FloorType::Portal(pid) = grid.floor_at(pos) {
                if pid == portal_id {
                    return Some(pos);
                }
            }
        }
    }
    None
}