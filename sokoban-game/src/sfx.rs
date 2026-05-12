use bevy::prelude::*;

use sokoban_core::grid::*;
use sokoban_core::types::*;

use crate::audio::{play_sound, GameAudio, GameVolume};
use crate::game::GameState;
use crate::save::{save_progress, ProgressData};
use crate::stats::SessionStats;

#[derive(Resource)]
pub struct SfxState {
    pub prev_step: u32,
    pub prev_boxes_on_target: u32,
    pub prev_level_complete: bool,
    pub prev_box_count: u32,
    pub prev_keys_count: u32,
    pub prev_gate_count: u32,
}

impl Default for SfxState {
    fn default() -> Self {
        Self {
            prev_step: 0,
            prev_boxes_on_target: 0,
            prev_level_complete: false,
            prev_box_count: 0,
            prev_keys_count: 0,
            prev_gate_count: 0,
        }
    }
}

pub fn count_gates(grid: &GridState) -> u32 {
    let mut count = 0;
    for z in 0..grid.height as i32 {
        for x in 0..grid.width as i32 {
            if matches!(grid.object_at(GridPos::new(x, z)), ObjectType::Gate(_)) {
                count += 1;
            }
        }
    }
    count
}

pub fn detect_events(
    mut commands: Commands,
    game_state: Option<Res<GameState>>,
    mut sfx: ResMut<SfxState>,
    mut progress: Option<ResMut<ProgressData>>,
    mut session: ResMut<SessionStats>,
    audio: Option<Res<GameAudio>>,
    volume: Option<Res<GameVolume>>,
) {
    let Some(ref gs) = game_state else {
        return;
    };

    let vol = volume.as_ref().map(|v| v.0).unwrap_or(0.7);
    let current_step = gs.grid.current_step;
    let boxes_on_target = gs.grid.boxes_on_targets();
    let box_count = gs.grid.box_positions.len() as u32;

    // Track box destruction (box count decreased)
    if box_count < sfx.prev_box_count {
        let destroyed = sfx.prev_box_count - box_count;
        session.boxes_destroyed_this_session += destroyed;
        if let Some(ref audio) = audio {
            play_sound(&mut commands, &audio.push_sound, vol);
        }
    }

    if current_step != sfx.prev_step {
        sfx.prev_step = current_step;

        let gate_count = count_gates(&gs.grid);
        let door_opened = gate_count < sfx.prev_gate_count;
        sfx.prev_gate_count = gate_count;

        let keys_count = gs.grid.collected_keys.len() as u32;
        let key_collected = keys_count > sfx.prev_keys_count;
        sfx.prev_keys_count = keys_count;

        if let Some(ref audio) = audio {
            if boxes_on_target > sfx.prev_boxes_on_target {
                play_sound(&mut commands, &audio.target_sound, vol);
            } else if door_opened {
                play_sound(&mut commands, &audio.door_sound, vol);
            } else if key_collected {
                play_sound(&mut commands, &audio.collect_sound, vol);
            } else {
                play_sound(&mut commands, &audio.move_sound, vol);
            }
        }

        // Key collection stats
        if key_collected {
            session.keys_this_session += 1;
        }
    }

    // Detect box landing on target (from spring/conveyor etc.)
    if boxes_on_target > sfx.prev_boxes_on_target && current_step == sfx.prev_step {
        if let Some(ref audio) = audio {
            play_sound(&mut commands, &audio.target_sound, vol);
        }
    }

    // Detect level complete (once)
    if gs.level_complete && !sfx.prev_level_complete {
        if let Some(ref audio) = audio {
            play_sound(&mut commands, &audio.complete_sound, vol);
        }

        crate::save::MidLevelSave::delete();

        if !gs.is_dungeon_mode {
            if let Some(ref mut p) = progress {
                p.complete_level(gs.current_level_index, gs.grid.current_step, gs.stars_earned);
                save_progress(p);
            }
        }
    }

    sfx.prev_boxes_on_target = boxes_on_target;
    sfx.prev_level_complete = gs.level_complete;
    sfx.prev_box_count = box_count;
}
