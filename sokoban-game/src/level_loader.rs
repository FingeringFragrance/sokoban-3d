use bevy::prelude::*;
use rand::rngs::StdRng;
use rand::SeedableRng;

use sokoban_core::generator::{generate, GenParams};
use sokoban_core::grid::*;
use sokoban_core::types::*;
use sokoban_core::history::MoveHistory;
use sokoban_core::level::LevelData;
use sokoban_core::replay::ReplayData;
use sokoban_core::solver::*;
use crate::assets::{AssetCatalog, FontAssets};
use crate::camera::setup_camera;
use crate::dungeon::DungeonManager;
use crate::environment::setup_environment;
use crate::game::GameState;
use crate::hud::setup_hud;
use crate::multifloor::MultiFloorRun;
use crate::particles::AmbientParticleSpawned;
use crate::save::ProgressData;
use crate::sfx::{count_gates, SfxState};
use crate::spawner::{spawn_grid, SceneEntity};
use crate::states::AppState;
use crate::stats::SessionStats;
use crate::tutorial::TutorialState;

pub fn load_level(
    mut commands: Commands,
    mut game_state: ResMut<GameState>,
    mut sfx: ResMut<SfxState>,
    dungeon_manager: ResMut<DungeonManager>,
    multi_floor: ResMut<MultiFloorRun>,
    mut effect_state: ResMut<crate::effects::EffectState>,
    mut tutorial_state: ResMut<TutorialState>,
    mut session: ResMut<SessionStats>,
    progress: Res<ProgressData>,
    cleanup: Query<
        Entity,
        Or<(
            With<SceneEntity>,
            With<crate::hud::HudRoot>,
            With<crate::camera::GameCamera>,
        )>,
    >,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
    mut next_state: ResMut<NextState<AppState>>,
    catalog: Option<Res<AssetCatalog>>,
    fonts: Option<Res<FontAssets>>,
    pack_state: Option<Res<crate::game::LevelPackState>>,
) {
    for entity in &cleanup {
        commands.entity(entity).despawn();
    }

    let is_resuming = game_state.is_resuming;
    game_state.is_resuming = false;

    let catalog_ref = catalog.as_ref().map(|c| c.as_ref());
    let grid_state;

    if is_resuming {
        let saved_step = game_state.grid.current_step;
        let saved_keys = game_state.grid.collected_keys.clone();
        let saved_switches = game_state.grid.active_switches.clone();

        let grid = Grid {
            width: game_state.grid.width,
            height: game_state.grid.height,
            cells: game_state.grid.cells.clone(),
        };
        let mut grid_state = spawn_grid(
            &mut commands, &mut meshes, &mut materials,
            &grid, catalog_ref, &game_state.scene_theme,
        );
        grid_state.current_step = saved_step;
        grid_state.collected_keys = saved_keys;
        grid_state.active_switches = saved_switches;
        game_state.grid = grid_state;
        game_state.initial_snapshot = game_state.grid.snapshot();
        game_state.level_complete = false;
        game_state.hint_direction = None;
        game_state.deadlock_detected = false;
        session.last_step_count = game_state.grid.current_step;
        session.moves_this_session = 0;
        session.last_hint_step = 0;
        session.completion_flushed = false;

        *sfx = SfxState::default();
        sfx.prev_gate_count = count_gates(&game_state.grid);
        effect_state.prev_level_complete = false;

        commands.insert_resource(AmbientParticleSpawned(false));

        setup_camera(&mut commands, game_state.grid.width, game_state.grid.height, game_state.cell_size);
        setup_hud(&mut commands, fonts);
        let settings = crate::save::SettingsData::load_or_default();
        setup_environment(
            &mut commands,
            &game_state.scene_theme,
            settings.colorblind_mode,
            settings.high_contrast,
        );

        crate::save::MidLevelSave::delete();
        next_state.set(AppState::Playing);
        return;
    }

    if multi_floor.active {
        grid_state = load_multifloor(
            &mut commands, &mut meshes, &mut materials,
            &mut game_state, &multi_floor, &progress,
            &mut tutorial_state, catalog_ref, &mut next_state,
        );
        if grid_state.is_none() {
            return;
        }
    } else if dungeon_manager.active {
        grid_state = load_dungeon(
            &mut commands, &mut meshes, &mut materials,
            &mut game_state, &dungeon_manager, &progress,
            &mut tutorial_state, catalog_ref, &mut next_state,
        );
        if grid_state.is_none() {
            return;
        }
    } else if game_state.is_daily {
        grid_state = load_daily(
            &mut commands, &mut meshes, &mut materials,
            &mut game_state, &progress, &mut tutorial_state,
            catalog_ref, &mut next_state,
        );
        if grid_state.is_none() {
            return;
        }
    } else {
        grid_state = load_classic(
            &mut commands, &mut meshes, &mut materials,
            &mut game_state, &progress, &mut tutorial_state,
            catalog_ref, &mut next_state,
            pack_state.as_deref(),
        );
        if grid_state.is_none() {
            return;
        }
    }

    let grid_state = grid_state.unwrap();
    game_state.grid = grid_state;
    game_state.initial_snapshot = game_state.grid.snapshot();
    game_state.history = MoveHistory::new();
    game_state.replay = ReplayData::new(game_state.current_level_index as u32, String::new());
    game_state.level_complete = false;
    game_state.hint_direction = None;
    game_state.deadlock_detected = false;
    session.last_step_count = 0;
    session.moves_this_session = 0;
    session.last_hint_step = 0;
    session.completion_flushed = false;

    *sfx = SfxState::default();
    sfx.prev_gate_count = count_gates(&game_state.grid);
    effect_state.prev_level_complete = false;

    commands.insert_resource(AmbientParticleSpawned(false));

    setup_camera(&mut commands, game_state.grid.width, game_state.grid.height, game_state.cell_size);
    setup_hud(&mut commands, fonts);
    let settings = crate::save::SettingsData::load_or_default();
    setup_environment(
        &mut commands,
        &game_state.scene_theme,
        settings.colorblind_mode,
        settings.high_contrast,
    );

    next_state.set(AppState::Playing);
}

fn load_multifloor(
    commands: &mut Commands,
    meshes: &mut ResMut<Assets<Mesh>>,
    materials: &mut ResMut<Assets<StandardMaterial>>,
    game_state: &mut GameState,
    multi_floor: &MultiFloorRun,
    progress: &ProgressData,
    tutorial_state: &mut TutorialState,
    catalog: Option<&AssetCatalog>,
    _next_state: &mut NextState<AppState>,
) -> Option<GridState> {
    let level = multi_floor.level.as_ref()?;
    let floor = level.get_floor(multi_floor.current_floor)?;
    let grid_state = spawn_grid(commands, meshes, materials, &floor.grid, catalog, &game_state.scene_theme);
    game_state.is_multifloor = true;
    game_state.floor_count = level.floor_count();
    game_state.current_floor = multi_floor.current_floor;
    game_state.scene_theme = level.scene_theme.clone();
    game_state.level_name = level.meta.name.clone();
    Some(grid_state)
}

fn load_dungeon(
    commands: &mut Commands,
    meshes: &mut ResMut<Assets<Mesh>>,
    materials: &mut ResMut<Assets<StandardMaterial>>,
    game_state: &mut GameState,
    dungeon_manager: &DungeonManager,
    progress: &ProgressData,
    tutorial_state: &mut TutorialState,
    catalog: Option<&AssetCatalog>,
    _next_state: &mut NextState<AppState>,
) -> Option<GridState> {
    let grid = dungeon_manager.current_room_grid()?;
    let grid_state = spawn_grid(commands, meshes, materials, grid, catalog, &game_state.scene_theme);
    game_state.is_dungeon_mode = true;
    game_state.dungeon_room_name = dungeon_manager.current_room_name();
    game_state.dungeon_current = dungeon_manager.current_index;
    game_state.dungeon_total = dungeon_manager.room_count();
    game_state.scene_theme = "default".to_string();
    game_state.level_name = dungeon_manager.current_room_name();
    Some(grid_state)
}

fn load_daily(
    commands: &mut Commands,
    meshes: &mut ResMut<Assets<Mesh>>,
    materials: &mut ResMut<Assets<StandardMaterial>>,
    game_state: &mut GameState,
    progress: &ProgressData,
    tutorial_state: &mut TutorialState,
    catalog: Option<&AssetCatalog>,
    _next_state: &mut NextState<AppState>,
) -> Option<GridState> {
    let seed = game_state.daily_seed;
    let params = GenParams {
        min_width: 7, max_width: 10,
        min_height: 7, max_height: 10,
        min_boxes: 2, max_boxes: 4,
        wall_density: 0.12, max_retries: 200,
        scene_theme: "default".to_string(),
        target_difficulty: 2,
        special_floor_density: 0.0,
        available_items: vec![],
        solver_config: SolverConfig { max_states: 500_000, timeout_ms: 10_000 },
    };
    let mut rng = StdRng::seed_from_u64(seed);
    let result = generate(&params, &mut rng)?;

    let grid = result.grid;
    let par = result.optimal_steps;
    let grid_clone = grid.clone();
    let grid_state = spawn_grid(commands, meshes, materials, &grid, catalog, "default");

    game_state.par_steps = par;
    game_state.is_multifloor = false;
    game_state.is_dungeon_mode = false;
    game_state.scene_theme = "default".to_string();
    game_state.level_name = format!("Daily #{}", game_state.daily_seed);

    *tutorial_state = TutorialState::default();
    if let Some(diff) = result.difficulty {
        game_state.par_steps = diff.optimal_steps;
    }

    Some(grid_state)
}

fn load_classic(
    commands: &mut Commands,
    meshes: &mut ResMut<Assets<Mesh>>,
    materials: &mut ResMut<Assets<StandardMaterial>>,
    game_state: &mut GameState,
    progress: &ProgressData,
    tutorial_state: &mut TutorialState,
    catalog: Option<&AssetCatalog>,
    _next_state: &mut NextState<AppState>,
    pack_state: Option<&crate::game::LevelPackState>,
) -> Option<GridState> {
    let path = game_state.level_paths.get(game_state.current_level_index)?;

    if path.starts_with("pack://") {
        let pack = pack_state?.pack.as_ref()?;
        let idx_str = path.strip_prefix("pack://")?.split('/').nth(1)?;
        let idx: usize = idx_str.parse().ok()?;
        let entry = pack.levels.get(idx)?;

        let mut grid = Grid::new(entry.width, entry.height);
        for z in 0..entry.height as i32 {
            for x in 0..entry.width as i32 {
                let cell = match entry.cells[z as usize][x as usize] {
                    1 => Cell::wall(),
                    2 => Cell { floor: FloorType::Normal, object: ObjectType::Player, color: None, facing: None, linked_id: None },
                    3 => Cell { floor: FloorType::Normal, object: ObjectType::Box, color: None, facing: None, linked_id: None },
                    4 => Cell::target(),
                    5 => Cell { floor: FloorType::Normal, object: ObjectType::Key(ItemColor::Red), color: None, facing: None, linked_id: None },
                    6 => Cell { floor: FloorType::Normal, object: ObjectType::Gate(ItemColor::Red), color: None, facing: None, linked_id: None },
                    _ => Cell::empty(),
                };
                grid.set(GridPos::new(x, z), cell);
            }
        }
        let grid_state = spawn_grid(commands, meshes, materials, &grid, catalog, "default");
        game_state.par_steps = Some(entry.meta.par_steps);
        game_state.is_multifloor = false;
        game_state.is_dungeon_mode = false;
        game_state.scene_theme = "default".to_string();
        game_state.level_name = entry.meta.name.clone();
        *tutorial_state = TutorialState::default();
        check_tutorials(&grid, tutorial_state, progress);
        return Some(grid_state);
    }

    let level = LevelData::load_from_ron(path).ok()?;
    let grid = level.get_grid();
    let par = level.meta.par_steps;
    let grid_state = spawn_grid(commands, meshes, materials, &grid, catalog, &level.scene_theme);

    game_state.par_steps = par;
    game_state.is_multifloor = false;
    game_state.is_dungeon_mode = false;
    game_state.scene_theme = level.scene_theme.clone();
    game_state.level_name = level.meta.name.clone();

    *tutorial_state = TutorialState::default();
    check_tutorials(&grid, tutorial_state, progress);

    Some(grid_state)
}

fn check_tutorials(grid: &Grid, tutorial_state: &mut TutorialState, progress: &ProgressData) {
    let items = crate::tutorial::scan_level_items(grid);
    for item in &items {
        if !progress.seen_tutorials.contains(item.as_str()) {
            if let Some(entry) = crate::tutorial::TUTORIALS.iter().find(|e| e.id == item.as_str()) {
                tutorial_state.entries.push(entry);
            }
        }
    }
    if !tutorial_state.entries.is_empty() {
        tutorial_state.active = true;
        tutorial_state.current_index = 0;
    }
}
