mod achievements;
mod assets;
mod audio;
mod camera;
mod dungeon;
mod dungeon_items;
mod easing;
mod effects;
mod environment;
mod game;
mod hud;
mod level_loader;
mod level_pack;
mod locale;
mod menu;
mod multifloor;
mod particles;
mod save;
mod scene_themes;
mod sfx;
mod spawner;
mod states;
mod stats;
mod tutorial;

use bevy::prelude::*;
use achievements::*;
use assets::*;
use audio::*;
use camera::*;
use dungeon::*;
use effects::*;
use environment::*;
use game::*;
use hud::*;
use level_loader::*;
use locale::*;
use menu::*;
use multifloor::*;
use particles::*;
use sfx::*;
use sokoban_core::grid::*;
use sokoban_core::history::MoveHistory;
use sokoban_core::replay::ReplayData;
use sokoban_core::types::Direction;
use spawner::*;
use states::AppState;
use stats::*;
use tutorial::*;

#[derive(SystemSet, Debug, Clone, PartialEq, Eq, Hash)]
enum GameSystemSet {
    Input,
    Logic,
    Effects,
    Stats,
}

fn main() {
    App::new()
        .add_plugins(DefaultPlugins.set(WindowPlugin {
            primary_window: Some(Window {
                title: "Sokoban 3D".into(),
                resolution: (1280, 720).into(),
                ..default()
            }),
            ..default()
        }))
        .init_state::<AppState>()
        .insert_resource(MenuSpawned(false))
        .insert_resource(SfxState::default())
        .insert_resource(GameVolume::default())
        .insert_resource(GameState {
            grid: GridState::new(8, 8),
            history: MoveHistory::new(),
            initial_snapshot: GridState::new(8, 8).snapshot(),
            cell_size: CELL_SIZE,
            current_level_index: 0,
            level_paths: Vec::new(),
            level_complete: false,
            replay: ReplayData::new(0, String::new()),
            replay_string: String::new(),
            hint_direction: None,
            hint_timer: 0.0,
            hint_step: 0,
            player_facing: Direction::Down,
            is_dungeon_mode: false,
            dungeon_room_name: String::new(),
            dungeon_current: 0,
            dungeon_total: 0,
            par_steps: None,
            stars_earned: 0,
            is_daily: false,
            daily_seed: 0,
            is_multifloor: false,
            current_floor: 0,
            floor_count: 0,
            max_slide_streak: 0,
            scene_theme: "default".to_string(),
            level_name: String::new(),
            deadlock_detected: false,
            is_resuming: false,
        })
        .insert_resource(DungeonManager::default())
        .insert_resource(MultiFloorRun::default())
        .insert_resource(ShakeState::default())
        .insert_resource(RestartConfirmState::default())
        .insert_resource(PendingEffects::default())
        .insert_resource(PortalFlash::default())
        .insert_resource(save::ProgressData::load_or_default())
        .insert_resource(save::SettingsData::load_or_default())
        .insert_resource(Locale::new("en"))
        .insert_resource(AchievementState::default())
        .insert_resource(GameStats::load_or_default())
        .insert_resource(SessionStats::default())
        .insert_resource(TutorialState::default())
        .insert_resource(EffectState::default())
        .insert_resource(SceneEffectState::default())
        .insert_resource(AmbientParticleSpawned::default())
        .insert_resource(BgmState::default())
        .insert_resource(CameraConfig::default())
        .insert_resource(MenuPage::default())
        .insert_resource(LevelGridFocus { index: 0, columns: 4 })
        .insert_resource(LevelPackState::default())
        .configure_sets(
            Update,
            (
                GameSystemSet::Input,
                GameSystemSet::Logic.after(GameSystemSet::Input),
                GameSystemSet::Effects.after(GameSystemSet::Logic),
                GameSystemSet::Stats.after(GameSystemSet::Logic),
            ),
        )
        .add_systems(Startup, (setup, load_audio, load_bgm, load_asset_catalog, load_fonts))
        .add_systems(OnEnter(AppState::Menu), init_menu)
        .add_systems(OnEnter(AppState::Loading), reset_shake)
        .add_systems(OnEnter(AppState::Paused), setup_pause)
        .add_systems(OnExit(AppState::Paused), teardown_pause)
        .add_systems(OnExit(AppState::Settings), save_settings_on_exit)
        .add_systems(
            Update,
            (
                (
                    setup_menu.run_if(
                        in_state(AppState::Menu)
                            .or(in_state(AppState::ModeSelect))
                            .or(in_state(AppState::Settings)),
                    ),
                    menu_button_interaction.run_if(
                        in_state(AppState::Menu)
                            .or(in_state(AppState::ModeSelect))
                            .or(in_state(AppState::Settings)),
                    ),
                    despawn_menu.run_if(
                        not(in_state(AppState::Menu)
                            .or(in_state(AppState::ModeSelect))
                            .or(in_state(AppState::Settings))),
                    ),
                ),
                load_level.run_if(in_state(AppState::Loading)),
                (
                    sync_positions.run_if(in_state(AppState::Playing).or(in_state(AppState::Paused))),
                    update_hud.run_if(in_state(AppState::Playing).or(in_state(AppState::Paused)).or(in_state(AppState::LevelComplete))),
                    camera_input.run_if(in_state(AppState::Playing).or(in_state(AppState::Paused))),
                    auto_center_camera.run_if(in_state(AppState::Playing).or(in_state(AppState::Paused))),
                    update_camera.run_if(in_state(AppState::Playing).or(in_state(AppState::Paused))),
                    sync_grid_objects.run_if(in_state(AppState::Playing).or(in_state(AppState::Paused))),
                    sync_destroyed_boxes.run_if(in_state(AppState::Playing).or(in_state(AppState::Paused))),
                    level_switch.run_if(in_state(AppState::Playing).or(in_state(AppState::Paused))),
                    pause_toggle.run_if(in_state(AppState::Playing).or(in_state(AppState::Paused))),
                    tick_restart_confirm.run_if(in_state(AppState::Playing).or(in_state(AppState::Paused))),
                ),
                (
                    box_glow.run_if(in_state(AppState::Playing).or(in_state(AppState::Paused))),
                    update_dungeon_items.run_if(in_state(AppState::Playing).or(in_state(AppState::Paused))),
                    update_minimap.run_if(in_state(AppState::Playing).or(in_state(AppState::Paused))),
                    spawn_ambient_particles.run_if(in_state(AppState::Playing).or(in_state(AppState::Paused))),
                    update_ambient_particles.run_if(in_state(AppState::Playing).or(in_state(AppState::Paused))),
                    update_portal_flash.run_if(in_state(AppState::Playing).or(in_state(AppState::Paused))),
                    update_scene_effects.run_if(in_state(AppState::Playing).or(in_state(AppState::Paused))),
                    bgm_switch_system.run_if(in_state(AppState::Playing).or(in_state(AppState::Paused))),
                ),
                (
                    player_input.run_if(in_state(AppState::Playing)).in_set(GameSystemSet::Input),
                    tutorial_input.run_if(in_state(AppState::Playing)).in_set(GameSystemSet::Input),
                    process_pending_effects.run_if(in_state(AppState::Playing)).in_set(GameSystemSet::Logic),
                    hint_system.run_if(in_state(AppState::Playing)).in_set(GameSystemSet::Logic),
                    detect_events.run_if(in_state(AppState::Playing)).in_set(GameSystemSet::Effects),
                    trigger_effects.run_if(in_state(AppState::Playing)).in_set(GameSystemSet::Effects),
                    update_particles.run_if(in_state(AppState::Playing)).in_set(GameSystemSet::Effects),
                    update_tutorial_overlay.run_if(in_state(AppState::Playing)).in_set(GameSystemSet::Effects),
                    track_stats.run_if(in_state(AppState::Playing)).in_set(GameSystemSet::Stats),
                    check_achievements.run_if(in_state(AppState::Playing)).in_set(GameSystemSet::Stats),
                    flush_on_completion.run_if(in_state(AppState::Playing)).in_set(GameSystemSet::Stats),
                    show_achievement_popups.run_if(in_state(AppState::Playing)).in_set(GameSystemSet::Stats),
                    update_achievement_popups.run_if(in_state(AppState::Playing)).in_set(GameSystemSet::Stats),
                ),
                level_complete_input.run_if(in_state(AppState::LevelComplete)),
                pause_button_interaction.run_if(in_state(AppState::Paused)),
            ),
        )
        .run();
}

pub const LEVEL_PATHS: &[&str] = &[
    concat!(env!("CARGO_MANIFEST_DIR"), "/assets/levels/classic/001_tutorial.ron"),
    concat!(env!("CARGO_MANIFEST_DIR"), "/assets/levels/classic/002_two_boxes.ron"),
    concat!(env!("CARGO_MANIFEST_DIR"), "/assets/levels/classic/003_corridor.ron"),
    concat!(env!("CARGO_MANIFEST_DIR"), "/assets/levels/classic/004_maze.ron"),
    concat!(env!("CARGO_MANIFEST_DIR"), "/assets/levels/classic/005_warehouse.ron"),
    concat!(env!("CARGO_MANIFEST_DIR"), "/assets/levels/classic/006_switches.ron"),
    concat!(env!("CARGO_MANIFEST_DIR"), "/assets/levels/classic/007_key_gate.ron"),
    concat!(env!("CARGO_MANIFEST_DIR"), "/assets/levels/classic/008_heavy.ron"),
    concat!(env!("CARGO_MANIFEST_DIR"), "/assets/levels/classic/009_ice.ron"),
    concat!(env!("CARGO_MANIFEST_DIR"), "/assets/levels/classic/010_grand.ron"),
];

/// Scan custom levels directory for user-created levels (from the editor)
pub fn scan_custom_levels() -> Vec<String> {
    let custom_dir = concat!(env!("CARGO_MANIFEST_DIR"), "/assets/levels/custom");
    let mut paths = Vec::new();
    if let Ok(entries) = std::fs::read_dir(custom_dir) {
        for entry in entries.flatten() {
            let path = entry.path();
            if path.extension().map_or(false, |e| e == "ron") {
                paths.push(path.to_string_lossy().to_string());
            }
        }
        paths.sort();
    }
    paths
}

fn reset_shake(
    mut shake_state: Option<ResMut<ShakeState>>,
    mut particle_spawned: Option<ResMut<AmbientParticleSpawned>>,
) {
    if let Some(mut s) = shake_state {
        s.timer = 0.0;
    }
    if let Some(mut p) = particle_spawned {
        p.0 = false;
    }
}

fn setup(mut commands: Commands) {
    commands.spawn((Camera2d, Camera { order: 1, ..default() }));
    commands.insert_resource(ClearColor(Color::srgb(0.08, 0.08, 0.1)));

    let mut level_paths: Vec<String> = LEVEL_PATHS.iter().map(|s| s.to_string()).collect();
    level_paths.append(&mut scan_custom_levels());

    let pack_paths = level_pack::scan_packs();
    let mut pack_state = LevelPackState::default();
    if let Some(pack_path) = pack_paths.first() {
        if let Some(pack) = level_pack::load_pack(pack_path) {
            for (i, _entry) in pack.levels.iter().enumerate() {
                level_paths.push(format!("pack://{}/{}", pack_path, i));
            }
            pack_state.pack = Some(pack);
            pack_state.pack_path = pack_path.clone();
        }
    }

    let progress = save::ProgressData::load_or_default();
    let settings = save::SettingsData::load_or_default();
    let lang = settings.language.clone();
    let unlocked_achievements = progress.unlocked_achievements.clone();

    commands.insert_resource(GameState {
        grid: GridState::empty(),
        history: MoveHistory::new(),
        initial_snapshot: GridState::empty().snapshot(),
        cell_size: CELL_SIZE,
        current_level_index: 0,
        level_paths,
        level_complete: false,
        replay: ReplayData::new(0, String::new()),
        replay_string: String::new(),
        hint_direction: None,
        hint_timer: 0.0,
        hint_step: 0,
        player_facing: Direction::Down,
        is_dungeon_mode: false,
        dungeon_room_name: String::new(),
        dungeon_current: 0,
        dungeon_total: 0,
        par_steps: None,
        stars_earned: 0,
        is_daily: false,
        daily_seed: 0,
        is_multifloor: false,
        current_floor: 0,
        floor_count: 0,
        max_slide_streak: 0,
        scene_theme: "default".to_string(),
        level_name: String::new(),
        deadlock_detected: false,
        is_resuming: false,
    });

    commands.insert_resource(pack_state);
    commands.insert_resource(progress);
    commands.insert_resource(settings);
    commands.insert_resource(Locale::new(&lang));

    let mut ach_state = AchievementState::default();
    ach_state.unlocked = unlocked_achievements;
    commands.insert_resource(ach_state);
    commands.insert_resource(GameStats::load_or_default());
}

fn level_switch(
    keyboard: Res<ButtonInput<KeyCode>>,
    mut game_state: ResMut<GameState>,
    mut dungeon_manager: ResMut<DungeonManager>,
    mut multi_floor: ResMut<MultiFloorRun>,
    mut next_state: ResMut<NextState<AppState>>,
    tutorial_state: Res<TutorialState>,
) {
    if tutorial_state.active {
        return;
    }

    if multi_floor.active {
        let mut target_floor: Option<u8> = None;
        if keyboard.just_pressed(KeyCode::Digit1) && multi_floor.floor_count() > 0 {
            target_floor = Some(0);
        }
        if keyboard.just_pressed(KeyCode::Digit2) && multi_floor.floor_count() > 1 {
            target_floor = Some(1);
        }
        if keyboard.just_pressed(KeyCode::Digit3) && multi_floor.floor_count() > 2 {
            target_floor = Some(2);
        }

        if target_floor.is_none() {
            if let Some(ref level) = multi_floor.level {
                let player_pos = game_state.grid.player_pos.pos;
                let current = multi_floor.current_floor;
                for conn in &level.connections {
                    if conn.from_floor == current && conn.from_pos == player_pos {
                        target_floor = Some(conn.to_floor);
                        break;
                    }
                }
            }
        }

        if let Some(target) = target_floor {
            if target != multi_floor.current_floor {
                let old_floor = multi_floor.current_floor;
                multi_floor.saved_states.insert(old_floor, game_state.grid.clone());
                multi_floor.current_floor = target;
                next_state.set(AppState::Loading);
            }
        }
        return;
    }

    if game_state.is_daily {
        return;
    }

    if dungeon_manager.active {
        let auto_advance = game_state.level_complete
            && game_state.grid.player_pos.pos.x == game_state.grid.width as i32 - 1;

        if (keyboard.just_pressed(KeyCode::KeyN) && game_state.level_complete) || auto_advance {
            if dungeon_manager.advance() {
                game_state.dungeon_current = dungeon_manager.current_index;
                game_state.dungeon_room_name = dungeon_manager.current_room_name();
                next_state.set(AppState::Loading);
            } else {
                dungeon_manager.active = false;
                game_state.is_dungeon_mode = false;
                next_state.set(AppState::Menu);
            }
        }
        return;
    }

    let mut target_index: Option<usize> = None;
    if keyboard.just_pressed(KeyCode::Digit1) { target_index = Some(0); }
    if keyboard.just_pressed(KeyCode::Digit2) { target_index = Some(1); }
    if keyboard.just_pressed(KeyCode::Digit3) { target_index = Some(2); }
    if keyboard.just_pressed(KeyCode::Digit4) { target_index = Some(3); }
    if keyboard.just_pressed(KeyCode::Digit5) { target_index = Some(4); }
    if keyboard.just_pressed(KeyCode::Digit6) { target_index = Some(5); }
    if keyboard.just_pressed(KeyCode::Digit7) { target_index = Some(6); }
    if keyboard.just_pressed(KeyCode::Digit8) { target_index = Some(7); }
    if keyboard.just_pressed(KeyCode::Digit9) { target_index = Some(8); }

    if keyboard.just_pressed(KeyCode::KeyN) && game_state.level_complete {
        let next = game_state.current_level_index + 1;
        if next < game_state.level_paths.len() {
            target_index = Some(next);
        }
    }

    if keyboard.just_pressed(KeyCode::KeyP) {
        if game_state.current_level_index > 0 {
            target_index = Some(game_state.current_level_index - 1);
        }
    }

    if let Some(index) = target_index {
        if index < game_state.level_paths.len() {
            game_state.current_level_index = index;
            next_state.set(AppState::Loading);
        }
    }
}

// ============================================================
//  Menu init & settings persistence
// ============================================================

fn init_menu(
    mut locale: ResMut<Locale>,
    settings: Res<save::SettingsData>,
    mut menu_page: ResMut<MenuPage>,
    mut menu_spawned: ResMut<MenuSpawned>,
) {
    locale.lang = settings.language.clone();
    menu_page.0 = MenuPageType::Main;
    menu_spawned.0 = false;
}

fn save_settings_on_exit(
    settings: Option<Res<save::SettingsData>>,
) {
    if let Some(s) = settings {
        s.save();
    }
}
