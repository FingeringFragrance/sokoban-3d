use bevy::prelude::*;
use serde::{Deserialize, Serialize};

use crate::achievements::AchievementState;
use crate::game::GameState;
use crate::save::ProgressData;

#[derive(Resource, Debug, Clone, Serialize, Deserialize)]
pub struct GameStats {
    pub total_moves: u32,
    pub total_pushes: u32,
    pub total_undos: u32,
    pub total_redos: u32,
    pub total_hints: u32,
    pub levels_completed: u32,
    pub keys_collected: u32,
    pub boxes_destroyed: u32,
    pub portals_used: u32,
    pub total_play_time_secs: f64,
    pub daily_completed: u32,
    pub dungeon_rooms_cleared: u32,
}

impl Default for GameStats {
    fn default() -> Self {
        Self {
            total_moves: 0,
            total_pushes: 0,
            total_undos: 0,
            total_redos: 0,
            total_hints: 0,
            levels_completed: 0,
            keys_collected: 0,
            boxes_destroyed: 0,
            portals_used: 0,
            total_play_time_secs: 0.0,
            daily_completed: 0,
            dungeon_rooms_cleared: 0,
        }
    }
}

#[derive(Resource, Default)]
pub struct SessionStats {
    pub moves_this_session: u32,
    pub pushes_this_session: u32,
    pub undos_this_session: u32,
    pub redos_this_session: u32,
    pub hints_this_session: u32,
    pub keys_this_session: u32,
    pub boxes_destroyed_this_session: u32,
    pub portals_this_session: u32,
    pub play_time_this_session: f64,
    pub last_step_count: u32,
    pub last_hint_step: u32,
    pub completion_flushed: bool,
}

pub fn track_stats(
    time: Res<Time>,
    game_state: Option<Res<GameState>>,
    mut session: ResMut<SessionStats>,
    mut stats: ResMut<GameStats>,
) {
    let Some(ref gs) = game_state else { return; };

    session.play_time_this_session += time.delta_secs_f64();

    if gs.grid.current_step > session.last_step_count {
        let delta = gs.grid.current_step - session.last_step_count;
        session.moves_this_session += delta;
        session.last_step_count = gs.grid.current_step;
    }

    if gs.hint_step != session.last_hint_step && gs.hint_direction.is_some() {
        session.hints_this_session += 1;
        session.last_hint_step = gs.hint_step;
    }

    if session.moves_this_session >= 100 {
        flush_session(&mut session, &mut stats);
    }
}

pub fn flush_on_completion(
    game_state: Option<Res<GameState>>,
    mut session: ResMut<SessionStats>,
    mut stats: ResMut<GameStats>,
    mut progress: ResMut<ProgressData>,
    ach_state: Res<AchievementState>,
) {
    let Some(ref gs) = game_state else { return; };

    if gs.level_complete && !session.completion_flushed {
        session.completion_flushed = true;

        if gs.is_daily {
            stats.daily_completed += 1;
        } else if gs.is_dungeon_mode {
            stats.dungeon_rooms_cleared += 1;
        } else {
            stats.levels_completed += 1;
        }

        flush_session(&mut session, &mut stats);

        progress.unlocked_achievements = ach_state.unlocked.clone();
        progress.save();
        stats.save_to_file();
    }
}

fn flush_session(session: &mut SessionStats, stats: &mut GameStats) {
    stats.total_moves += session.moves_this_session;
    stats.total_pushes += session.pushes_this_session;
    stats.total_undos += session.undos_this_session;
    stats.total_redos += session.redos_this_session;
    stats.total_hints += session.hints_this_session;
    stats.keys_collected += session.keys_this_session;
    stats.boxes_destroyed += session.boxes_destroyed_this_session;
    stats.portals_used += session.portals_this_session;
    stats.total_play_time_secs += session.play_time_this_session;

    let last_step = session.last_step_count;
    let last_hint = session.last_hint_step;
    *session = SessionStats::default();
    session.last_step_count = last_step;
    session.last_hint_step = last_hint;
}

#[allow(dead_code)]
pub fn format_play_time(secs: f64) -> String {
    let total = secs as u64;
    let h = total / 3600;
    let m = (total % 3600) / 60;
    let s = total % 60;
    if h > 0 {
        format!("{}h {}m {}s", h, m, s)
    } else if m > 0 {
        format!("{}m {}s", m, s)
    } else {
        format!("{}s", s)
    }
}
