mod grid;
mod tool;
mod editor_camera;
mod lighting;
mod playtest;
mod level_meta;
mod scene;
mod level_pack;
mod validation;
mod ui;

use bevy::prelude::*;
use grid::GridData;
use tool::{EditorTool, UndoRedo};
use editor_camera::EditorCamera;
use playtest::PlaytestState;
use level_meta::{LevelMeta, EditState};
use level_pack::{LevelPack, CurrentLevel, SavePath, SaveToast, DirtyFlag};
use validation::ValidationState;

fn main() {
    App::new()
        .add_plugins(DefaultPlugins.set(WindowPlugin {
            primary_window: Some(Window {
                title: "Sokoban Editor".into(),
                resolution: (1400, 900).into(),
                ..default()
            }),
            ..default()
        }))
        .insert_resource(ClearColor(Color::srgb(0.08, 0.08, 0.12)))
        .insert_resource(EditorCamera::default())
        .insert_resource(GridData::default())
        .insert_resource(EditorTool::default())
        .insert_resource(UndoRedo::default())
        .insert_resource(PlaytestState::default())
        .insert_resource(LevelMeta::default())
        .insert_resource(EditState::default())
        .insert_resource(LevelPack::default())
        .insert_resource(CurrentLevel::default())
        .insert_resource(ValidationState::default())
        .insert_resource(SavePath::default())
        .insert_resource(SaveToast::default())
        .insert_resource(DirtyFlag::default())
        .add_systems(Startup, (
            ui::load_font,
            ui::load_decorations,
            lighting::setup_environment_light,
            scene::init_scene_materials,
            editor_camera::spawn_camera,
            editor_camera::spawn_ui_camera,
            lighting::spawn_lights,
            scene::spawn_axes,
        ))
        .add_systems(Update, (
            playtest::toggle_playtest,
            playtest::playtest_move,
            level_meta::text_input,
            editor_camera::control_camera,
            tool::editor_click,
            scene::editor_hover,
            tool::editor_keys,
            tool::editor_commands,
            level_meta::meta_shortcuts,
            level_pack::pack_commands,
            level_pack::level_navigation,
            validation::run_validation,
            validation::run_solver,
            scene::sync_world,
            scene::sync_grid_lines,
            scene::adjust_decoration_materials,
        ))
        .add_systems(Update, (
            ui::build_ui,
            playtest::playtest_hud,
            level_pack::level_card_click,
            level_pack::toast_tick,
        ))
        .run();
}