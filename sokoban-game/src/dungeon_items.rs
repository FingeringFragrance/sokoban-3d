use bevy::prelude::*;
use sokoban_core::types::*;

use crate::dungeon::*;
use crate::effects::PendingEffects;
use crate::game::{GameState, ShakeState};

pub fn handle_dungeon_items(
    keyboard: &ButtonInput<KeyCode>,
    game_state: &mut GameState,
    dungeon_manager: &mut DungeonManager,
    pending: &mut PendingEffects,
    shake_state: &mut ShakeState,
) -> bool {
    if !game_state.is_dungeon_mode {
        return false;
    }

    let used_item = read_dungeon_item_input(keyboard);

    let Some(item) = used_item else {
        return false;
    };

    if !dungeon_manager.has_item(item) {
        return false;
    }

    match item {
        DungeonItemType::Bomb => {
            use_bomb(game_state, pending, shake_state);
        }
        DungeonItemType::Wing => {
            use_wing(game_state);
        }
        DungeonItemType::Glove => {
            use_glove(game_state);
        }
        DungeonItemType::Teleporter => {
            use_teleporter(game_state, dungeon_manager);
        }
        DungeonItemType::Shield => {
            dungeon_manager.shield_active = true;
        }
    }

    dungeon_manager.use_item(item);
    true
}

fn read_dungeon_item_input(keyboard: &ButtonInput<KeyCode>) -> Option<DungeonItemType> {
    if keyboard.just_pressed(KeyCode::Digit1) {
        Some(DungeonItemType::Bomb)
    } else if keyboard.just_pressed(KeyCode::Digit2) {
        Some(DungeonItemType::Wing)
    } else if keyboard.just_pressed(KeyCode::Digit3) {
        Some(DungeonItemType::Glove)
    } else if keyboard.just_pressed(KeyCode::Digit4) {
        Some(DungeonItemType::Teleporter)
    } else if keyboard.just_pressed(KeyCode::Digit5) {
        Some(DungeonItemType::Shield)
    } else {
        None
    }
}

fn use_bomb(
    game_state: &mut GameState,
    pending: &mut PendingEffects,
    shake_state: &mut ShakeState,
) {
    let pos = game_state.grid.player_pos.pos;
    let mut exploded = false;
    for dir in Direction::all() {
        let adj = pos.shift(dir);
        if game_state.grid.object_at(adj) == ObjectType::CrackedWall {
            game_state.grid.remove_object(adj);
            let world = GridPos3D::new(adj.x, adj.z, game_state.grid.player_pos.floor)
                .to_world(game_state.cell_size, 0.0);
            pending.explosions.push(Vec3::new(world[0], world[1], world[2]));
            exploded = true;
        }
    }
    if exploded {
        shake_state.timer = shake_state.duration;
    }
}

fn use_wing(game_state: &mut GameState) {
    let dir = game_state.player_facing;
    let mid = game_state.grid.player_pos.shift(dir);
    let skip = mid.shift(dir);
    if game_state.grid.in_bounds(mid.pos)
        && game_state.grid.is_passable(mid.pos)
        && game_state.grid.in_bounds(skip.pos)
        && game_state.grid.is_passable(skip.pos)
    {
        let snap = game_state.grid.snapshot();
        game_state.grid.move_player(skip);
        game_state.history.push(snap, dir);
    }
}

fn use_glove(game_state: &mut GameState) {
    let ppos = game_state.grid.player_pos.pos;
    for dir in Direction::all() {
        let adj = ppos.shift(dir);
        if game_state.grid.find_box_at(adj).is_some() {
            let behind = ppos.shift(dir.opposite());
            if game_state.grid.in_bounds(behind)
                && game_state.grid.is_passable(behind)
                && game_state.grid.find_box_at(behind).is_none()
            {
                if let Some(b) = game_state.grid.find_box_at(adj) {
                    let bid = b.entity_id;
                    let dest = GridPos3D::new(
                        behind.x,
                        behind.z,
                        game_state.grid.player_pos.floor,
                    );
                    game_state.grid.move_box(bid, dest);
                }
                break;
            }
        }
    }
}

fn use_teleporter(game_state: &mut GameState, dungeon_manager: &mut DungeonManager) {
    if !dungeon_manager.teleporter_placed {
        dungeon_manager.teleporter_pos = Some(game_state.grid.player_pos.pos);
        dungeon_manager.teleporter_placed = true;
    } else if let Some(tp) = dungeon_manager.teleporter_pos {
        let dest = GridPos3D::new(tp.x, tp.z, game_state.grid.player_pos.floor);
        if game_state.grid.in_bounds(dest.pos) && game_state.grid.is_passable(dest.pos) {
            game_state.grid.move_player(dest);
            dungeon_manager.teleporter_pos = None;
            dungeon_manager.teleporter_placed = false;
        }
    }
}
