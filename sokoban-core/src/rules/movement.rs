use crate::grid::*;
use crate::types::*;

use super::landing::resolve_landing;

pub const PLAYER_ENTITY_ID: u64 = 0;

pub fn resolve_move(
    grid: &mut GridState,
    intent: MoveIntent,
    scene: &SceneTheme,
) -> MoveResult {
    let mut result = MoveResult::success();
    let from = grid.player_pos;
    let to = from.shift(intent.direction);

    if !grid.in_bounds(to.pos) {
        return MoveResult::failure();
    }

    if !grid.floor_at(to.pos).is_passable() {
        return MoveResult::failure();
    }

    // Ramp 方向检查：只能沿斜坡方向通行
    if let FloorType::Ramp(ramp_dir) = grid.floor_at(to.pos) {
        if intent.direction != ramp_dir {
            return MoveResult::failure();
        }
    }

    let obj = grid.object_at(to.pos);
    match obj {
        ObjectType::Wall | ObjectType::CrackedWall | ObjectType::Rock => {
            return MoveResult::failure();
        }
        ObjectType::Gate(color) => {
            if grid.has_key(color) {
                grid.remove_object(to.pos);
                result.steps.push(MoveStep {
                    entity_id: PLAYER_ENTITY_ID,
                    from,
                    to,
                    step_type: MoveStepType::OpenDoor,
                    floor_type: grid.floor_at(to.pos),
                });
            } else {
                return MoveResult::failure();
            }
        }
        ObjectType::Pillar(id) => {
            if !grid.is_switch_active(id) {
                return MoveResult::failure();
            }
        }
        _ => {}
    }

    let box_info = grid
        .find_box_at(to.pos)
        .map(|b| (b.entity_id, b.box_type));
    if let Some((box_id, _)) = box_info {
        if !try_push_chain(grid, box_id, to, intent.direction, scene, &mut result) {
            return MoveResult::failure();
        }
    }

    let floor = grid.floor_at(to.pos);
    result.steps.push(MoveStep {
        entity_id: PLAYER_ENTITY_ID,
        from,
        to,
        step_type: MoveStepType::Walk,
        floor_type: floor,
    });
    grid.move_player(to);
    grid.advance_step();

    resolve_landing(grid, PLAYER_ENTITY_ID, to, intent.direction, scene, &mut result, 0);

    result
}

fn try_push_chain(
    grid: &mut GridState,
    box_id: u64,
    box_pos: GridPos3D,
    direction: Direction,
    scene: &SceneTheme,
    result: &mut MoveResult,
) -> bool {
    let target = box_pos.shift(direction);

    if !grid.in_bounds(target.pos) {
        return false;
    }

    if !grid.floor_at(target.pos).is_passable() {
        return false;
    }

    let pusher_type = grid
        .box_positions
        .iter()
        .find(|b| b.entity_id == box_id)
        .map(|b| b.box_type);

    if pusher_type == Some(ObjectType::HeavyBox) {
        return false;
    }

    // Ramp 方向检查
    if let FloorType::Ramp(ramp_dir) = grid.floor_at(target.pos) {
        if direction != ramp_dir {
            return false;
        }
    }

    let target_obj = grid.object_at(target.pos);
    match target_obj {
        ObjectType::Wall | ObjectType::Rock | ObjectType::Magnet => {
            return false;
        }
        ObjectType::CrackedWall => {
            if pusher_type == Some(ObjectType::FragileBox)
                || pusher_type == Some(ObjectType::Bomb)
            {
                grid.move_box(box_id, target);
                grid.remove_object(target.pos);
                grid.remove_box(box_id);
                result.destroyed_entities.push(box_id);
                result.steps.push(MoveStep {
                    entity_id: box_id,
                    from: box_pos,
                    to: target,
                    step_type: MoveStepType::Destroy,
                    floor_type: grid.floor_at(target.pos),
                });
                return true;
            }
            return false;
        }
        ObjectType::Gate(_) => {
            return false;
        }
        ObjectType::Pillar(id) => {
            if !grid.is_switch_active(id) {
                return false;
            }
        }
        _ => {}
    }

    let next_box = grid
        .find_box_at(target.pos)
        .map(|b| (b.entity_id, b.box_type));
    if let Some((next_id, next_type)) = next_box {
        if next_type == ObjectType::HeavyBox {
            return false;
        }
        if pusher_type == Some(ObjectType::Box) {
            return false;
        }
        if !try_push_chain(grid, next_id, target, direction, scene, result) {
            return false;
        }
    }

    let floor = grid.floor_at(target.pos);
    result.steps.push(MoveStep {
        entity_id: box_id,
        from: box_pos,
        to: target,
        step_type: MoveStepType::Push,
        floor_type: floor,
    });
    grid.move_box(box_id, target);

    resolve_landing(grid, box_id, target, direction, scene, result, 0);

    true
}
