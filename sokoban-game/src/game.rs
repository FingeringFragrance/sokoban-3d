use bevy::prelude::*;
use sokoban_core::grid::*;
use sokoban_core::history::*;
use sokoban_core::replay::*;
use sokoban_core::rules::*;
use sokoban_core::solver::*;
use sokoban_core::types::*;

use crate::dungeon::*;
use crate::dungeon_items::handle_dungeon_items;
use crate::easing;
use crate::effects::PendingEffects;
use crate::multifloor::{MultiFloorRun, FLOOR_HEIGHT};
use crate::scene_themes::build_scene_theme;
use crate::states::AppState;
use crate::stats::SessionStats;
use crate::tutorial::TutorialState;
use crate::particles::AmbientParticle;
use crate::camera::GameCamera;
use crate::effects::ParticleEffect;
use crate::spawner::GridObject;

pub const CELL_SIZE: f32 = 2.0;
const MOVE_SPEED: f32 = 16.0;

#[derive(Component)]
pub struct SokobanEntity(pub u64);

/// 失败抖动状态
#[derive(Resource)]
pub struct ShakeState {
    pub timer: f32,
    pub duration: f32,
    pub intensity: f32,
}

impl Default for ShakeState {
    fn default() -> Self {
        Self {
            timer: 0.0,
            duration: 0.22,
            intensity: 0.28,
        }
    }
}

#[derive(Resource)]
pub struct RestartConfirmState {
    pub pending: bool,
    pub timer: f32,
}

impl Default for RestartConfirmState {
    fn default() -> Self {
        Self {
            pending: false,
            timer: 0.0,
        }
    }
}

#[derive(Resource, Default)]
pub struct LevelPackState {
    pub pack: Option<crate::level_pack::LevelPack>,
    pub pack_path: String,
}

#[derive(Resource)]
pub struct GameState {
    pub grid: GridState,
    pub history: MoveHistory,
    pub initial_snapshot: GridSnapshot,
    pub cell_size: f32,
    pub current_level_index: usize,
    pub level_paths: Vec<String>,
    pub level_complete: bool,
    pub replay: ReplayData,
    pub replay_string: String,
    pub hint_direction: Option<Direction>,
    pub hint_timer: f32,
    pub hint_step: u32,
    pub player_facing: Direction,
    pub is_dungeon_mode: bool,
    pub dungeon_room_name: String,
    pub dungeon_current: usize,
    pub dungeon_total: usize,
    pub par_steps: Option<u32>,
    pub stars_earned: u8,
    pub is_daily: bool,
    pub daily_seed: u64,
    pub is_multifloor: bool,
    pub current_floor: u8,
    pub floor_count: u8,
    pub max_slide_streak: u32,
    pub scene_theme: String,
    pub level_name: String,
    pub deadlock_detected: bool,
    pub is_resuming: bool,
}

pub fn calculate_stars(steps: u32, par_steps: Option<u32>) -> u8 {
    match par_steps {
        Some(par) if par > 0 && steps <= par => 3,
        Some(par) if par > 0 && steps <= par * 2 => 2,
        _ => 1,
    }
}

pub fn today_daily_seed() -> u64 {
    let now = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .unwrap_or_default()
        .as_secs();
    now / 86400
}

pub fn player_input(
    keyboard: Res<ButtonInput<KeyCode>>,
    mut game_state: ResMut<GameState>,
    mut next_state: ResMut<NextState<AppState>>,
    mut dungeon_manager: ResMut<DungeonManager>,
    mut session: ResMut<SessionStats>,
    tutorial_state: Res<TutorialState>,
    mut shake_state: ResMut<ShakeState>,
    mut pending: ResMut<PendingEffects>,
    mut restart_confirm: ResMut<RestartConfirmState>,
) {
    if tutorial_state.active {
        return;
    }

    if restart_confirm.pending {
        let just_pressed: Vec<KeyCode> = keyboard.get_just_pressed().copied().collect();
        let has_other_key = just_pressed.iter().any(|k| *k != KeyCode::KeyR);
        if has_other_key {
            restart_confirm.pending = false;
        }
    }

    if game_state.level_complete {
        return;
    }

    if handle_dungeon_items(&keyboard, &mut game_state, &mut dungeon_manager, &mut pending, &mut shake_state) {
        return;
    }

    // ---- Movement ----
    let dir = if keyboard.just_pressed(KeyCode::KeyW)
        || keyboard.just_pressed(KeyCode::ArrowUp)
    {
        Some(Direction::Up)
    } else if keyboard.just_pressed(KeyCode::KeyS)
        || keyboard.just_pressed(KeyCode::ArrowDown)
    {
        Some(Direction::Down)
    } else if keyboard.just_pressed(KeyCode::KeyA)
        || keyboard.just_pressed(KeyCode::ArrowLeft)
    {
        Some(Direction::Left)
    } else if keyboard.just_pressed(KeyCode::KeyD)
        || keyboard.just_pressed(KeyCode::ArrowRight)
    {
        Some(Direction::Right)
    } else if keyboard.just_pressed(KeyCode::KeyZ) {
        if let Some(snapshot) = game_state.history.pop() {
            game_state.grid.restore(&snapshot);
            game_state.replay.pop_move();
            game_state.hint_direction = None;
            session.undos_this_session += 1;
        }
        return;
    } else if keyboard.just_pressed(KeyCode::KeyY)
        && keyboard.any_pressed([KeyCode::ControlLeft, KeyCode::ControlRight])
    {
        if let Some(snapshot) = game_state.history.redo() {
            game_state.grid.restore(&snapshot);
            if let Some(dir) = game_state.history.peek_direction() {
                game_state.replay.record_move(dir);
            }
            game_state.hint_direction = None;
            session.redos_this_session += 1;
        }
        return;
    } else if keyboard.just_pressed(KeyCode::KeyR) {
        if restart_confirm.pending {
            next_state.set(AppState::Loading);
            return;
        } else {
            restart_confirm.pending = true;
            restart_confirm.timer = 2.0;
            return;
        }
    } else {
        None
    };

    if let Some(direction) = dir {
        let snapshot = game_state.grid.snapshot();
        let scene = build_scene_theme(&game_state.scene_theme);
        let intent = MoveIntent { direction };
        let result = resolve_move(&mut game_state.grid, intent, &scene);

        if result.success {
            if result.player_died {
                let snapshot = game_state.initial_snapshot.clone();
                game_state.grid.restore(&snapshot);
                game_state.history = MoveHistory::new();
                game_state.replay = ReplayData::new(
                    game_state.replay.level_id,
                    String::new(),
                );
                game_state.hint_direction = None;
                game_state.deadlock_detected = false;
                session.last_step_count = 0;
                session.moves_this_session = 0;
                shake_state.timer = shake_state.duration;
                return;
            }

            game_state.history.push(snapshot, direction);
            game_state.replay.record_move(direction);
            game_state.player_facing = direction;
            game_state.hint_direction = None;

            let mut streak = 0u32;
            for step in &result.steps {
                if step.step_type == MoveStepType::Slide {
                    streak += 1;
                    if streak > game_state.max_slide_streak {
                        game_state.max_slide_streak = streak;
                    }
                } else {
                    streak = 0;
                }
                if step.step_type == MoveStepType::Push {
                    session.pushes_this_session += 1;
                }
            }

            // ---- 爆炸 / 传送 特效触发 ----
            for step in &result.steps {
                match step.step_type {
                    MoveStepType::Fall | MoveStepType::Destroy => {
                        let world = step.to.to_world(game_state.cell_size, 0.0);
                        pending.explosions.push(Vec3::new(world[0], world[1], world[2]));
                        shake_state.timer = shake_state.duration.max(shake_state.timer);
                    }
                    MoveStepType::Teleport => {
                        pending.portal_flash = true;
                    }
                    _ => {}
                }
            }

            // ---- 场景周期性机制（风、岩浆等）----
            let step = game_state.grid.current_step;
            let scene_for_mechanics = build_scene_theme(&game_state.scene_theme);
            let mechanic_events =
                tick_scene_mechanics(&mut game_state.grid, &scene_for_mechanics, step);
            if !mechanic_events.is_empty() {
                shake_state.timer =
                    (shake_state.duration * 0.5).max(shake_state.timer);
            }

            game_state.deadlock_detected = detect_deadlock(&game_state.grid).is_some();

            if game_state.grid.all_boxes_on_targets() {
                game_state.level_complete = true;
                game_state.stars_earned =
                    calculate_stars(game_state.grid.current_step, game_state.par_steps);
                let encoded = game_state.replay.encode();
                game_state.replay_string = encoded.clone();
                println!(
                    "Level complete! Steps: {} Stars: {} Replay: {}",
                    game_state.grid.current_step, game_state.stars_earned, encoded
                );
                next_state.set(AppState::LevelComplete);
            }
        } else {
            // 移动失败 → 抖动
            shake_state.timer = shake_state.duration;
        }
    }
}

/// 通关画面按键处理
pub fn level_complete_input(
    keyboard: Res<ButtonInput<KeyCode>>,
    mut game_state: ResMut<GameState>,
    mut next_state: ResMut<NextState<AppState>>,
) {
    if keyboard.just_pressed(KeyCode::KeyN) {
        let next = game_state.current_level_index + 1;
        if next < game_state.level_paths.len() {
            game_state.current_level_index = next;
            next_state.set(AppState::Loading);
        } else {
            next_state.set(AppState::Menu);
        }
    } else if keyboard.just_pressed(KeyCode::KeyR) {
        next_state.set(AppState::Loading);
    } else if keyboard.just_pressed(KeyCode::Escape) {
        next_state.set(AppState::Menu);
    }
}

/// 同步逻辑位置到 3D Transform（含抖动）
pub fn sync_positions(
    game_state: Option<Res<GameState>>,
    multi_floor: Option<Res<MultiFloorRun>>,
    shake_state: Res<ShakeState>,
    time: Res<Time>,
    mut query: Query<
        (&SokobanEntity, &mut Transform),
        (
            Without<AmbientParticle>,
            Without<GameCamera>,
            Without<GridObject>,
            Without<ParticleEffect>,
        ),
    >,
) {
    let Some(ref gs) = game_state else {
        return;
    };
    let dt = time.delta_secs();

    let floor_height = if gs.is_multifloor {
        multi_floor
            .as_ref()
            .filter(|mf| mf.active)
            .map(|_| FLOOR_HEIGHT)
            .unwrap_or(0.0)
    } else {
        0.0
    };

    let (shake_x, shake_z) = easing::shake_offset(
        shake_state.timer,
        shake_state.duration,
        shake_state.intensity,
        time.elapsed_secs(),
    );

    for (sok_entity, mut transform) in &mut query {
        let target = if sok_entity.0 == PLAYER_ENTITY_ID {
            gs.grid.player_pos.to_world(gs.cell_size, floor_height)
        } else if let Some(b) = gs
            .grid
            .box_positions
            .iter()
            .find(|b| b.entity_id == sok_entity.0)
        {
            b.pos.to_world(gs.cell_size, floor_height)
        } else {
            continue;
        };

        transform.translation.x =
            easing::exp_decay(transform.translation.x, target[0], MOVE_SPEED, dt);
        transform.translation.z =
            easing::exp_decay(transform.translation.z, target[2], MOVE_SPEED, dt);

        if sok_entity.0 == PLAYER_ENTITY_ID {
            transform.translation.y =
                easing::exp_decay(transform.translation.y, target[1], MOVE_SPEED, dt);

            transform.translation.x += shake_x;
            transform.translation.z += shake_z;

            use std::f32::consts::{FRAC_PI_2, PI};
            let target_rot = match gs.player_facing {
                Direction::Up => Quat::from_rotation_y(0.0),
                Direction::Down => Quat::from_rotation_y(PI),
                Direction::Left => Quat::from_rotation_y(FRAC_PI_2),
                Direction::Right => Quat::from_rotation_y(-FRAC_PI_2),
            };
            let t = (MOVE_SPEED * dt).min(1.0);
            transform.rotation = transform.rotation.slerp(target_rot, t);
        } else {
            let on_target = gs
                .grid
                .box_positions
                .iter()
                .find(|b| b.entity_id == sok_entity.0)
                .map(|b| gs.grid.is_target(b.pos.pos))
                .unwrap_or(false);

            if on_target {
                let t = time.elapsed_secs() * 3.0;
                transform.translation.y = target[1] + t.sin() * 0.15;
            } else {
                transform.translation.y =
                    easing::exp_decay(transform.translation.y, target[1], MOVE_SPEED, dt);
            }
        }
    }
}

pub fn hint_system(
    keyboard: Res<ButtonInput<KeyCode>>,
    mut game_state: Option<ResMut<GameState>>,
    time: Res<Time>,
    tutorial_state: Res<TutorialState>,
) {
    if tutorial_state.active {
        return;
    }

    let Some(ref mut gs) = game_state else {
        return;
    };

    if gs.level_complete {
        return;
    }

    if gs.hint_direction.is_some() {
        gs.hint_timer -= time.delta_secs();
        if gs.hint_timer <= 0.0 {
            gs.hint_direction = None;
        }
    }

    if gs.hint_direction.is_some() && gs.grid.current_step != gs.hint_step {
        gs.hint_direction = None;
    }

    if keyboard.just_pressed(KeyCode::KeyH) && !gs.level_complete {
        let config = SolverConfig {
            max_states: 50_000,
            timeout_ms: 1_000,
        };
        let result = solve(&gs.grid, &config);
        if let Some(solution) = result.solution {
            if let Some(&first_dir) = solution.first() {
                gs.hint_direction = Some(first_dir);
                gs.hint_timer = 3.0;
                gs.hint_step = gs.grid.current_step;
            }
        }
    }
}

pub fn pause_toggle(
    keyboard: Res<ButtonInput<KeyCode>>,
    state: Res<State<AppState>>,
    mut next_state: ResMut<NextState<AppState>>,
    tutorial_state: Res<TutorialState>,
) {
    if keyboard.just_pressed(KeyCode::Escape) {
        if tutorial_state.active {
            return;
        }
        match state.get() {
            AppState::Playing => next_state.set(AppState::Paused),
            AppState::Paused => next_state.set(AppState::Playing),
            _ => {}
        }
    }
}

pub fn tick_restart_confirm(
    time: Res<Time>,
    mut restart_confirm: ResMut<RestartConfirmState>,
) {
    if restart_confirm.pending {
        restart_confirm.timer -= time.delta_secs();
        if restart_confirm.timer <= 0.0 {
            restart_confirm.pending = false;
        }
    }
}
