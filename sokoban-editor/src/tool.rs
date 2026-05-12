use bevy::prelude::*;
use bevy::window::PrimaryWindow;
use crate::grid::{CellKind, GridData, find_player};
use crate::editor_camera::{EditorCam, mouse_to_grid};

#[derive(Resource)]
pub struct EditorTool {
    pub mode: ToolMode,
    pub selected: CellKind,
}

#[derive(Clone, Copy, PartialEq, Eq, Default)]
pub enum ToolMode {
    #[default]
    Brush,
    Eraser,
}

impl Default for EditorTool {
    fn default() -> Self {
        Self { mode: ToolMode::Brush, selected: CellKind::Wall }
    }
}

#[derive(Resource, Default)]
pub struct UndoRedo {
    pub undo_stack: Vec<Vec<Vec<CellKind>>>,
    pub redo_stack: Vec<Vec<Vec<CellKind>>>,
}

impl UndoRedo {
    pub fn push(&mut self, cells: &[Vec<CellKind>]) {
        self.undo_stack.push(cells.to_vec());
        self.redo_stack.clear();
    }

    pub fn undo(&mut self, grid: &mut GridData) {
        if let Some(prev) = self.undo_stack.pop() {
            self.redo_stack.push(grid.cells.clone());
            grid.cells = prev;
            grid.version += 1;
        }
    }

    pub fn redo(&mut self, grid: &mut GridData) {
        if let Some(next) = self.redo_stack.pop() {
            self.undo_stack.push(grid.cells.clone());
            grid.cells = next;
            grid.version += 1;
        }
    }
}

pub fn editor_keys(
    keyboard: Res<ButtonInput<KeyCode>>,
    playtest: Res<crate::playtest::PlaytestState>,
    mut tool: ResMut<EditorTool>,
) {
    if playtest.active { return; }
    if keyboard.just_pressed(KeyCode::KeyB) {
        tool.mode = ToolMode::Brush;
    }
    if keyboard.just_pressed(KeyCode::KeyE) {
        tool.mode = ToolMode::Eraser;
    }
    if keyboard.just_pressed(KeyCode::Digit1) {
        tool.selected = CellKind::Wall;
    }
    if keyboard.just_pressed(KeyCode::Digit2) {
        tool.selected = CellKind::Player;
    }
    if keyboard.just_pressed(KeyCode::Digit3) {
        tool.selected = CellKind::Box;
    }
    if keyboard.just_pressed(KeyCode::Digit4) {
        tool.selected = CellKind::Target;
    }
    if keyboard.just_pressed(KeyCode::Digit5) {
        tool.selected = CellKind::Key;
    }
    if keyboard.just_pressed(KeyCode::Digit6) {
        tool.selected = CellKind::Gate;
    }
    if keyboard.just_pressed(KeyCode::Digit7) {
        tool.selected = CellKind::Decoration;
    }
}

pub fn editor_click(
    mouse: Res<ButtonInput<MouseButton>>,
    playtest: Res<crate::playtest::PlaytestState>,
    windows: Query<&Window, With<PrimaryWindow>>,
    cameras: Query<(&Camera, &GlobalTransform), With<EditorCam>>,
    mut grid: ResMut<GridData>,
    tool: Res<EditorTool>,
    mut undo: ResMut<UndoRedo>,
    mut dirty: ResMut<crate::level_pack::DirtyFlag>,
) {
    if playtest.active { return; }
    if !mouse.just_pressed(MouseButton::Left) { return; }
    let Ok(w) = windows.single() else { return };
    let Some(cursor) = w.cursor_position() else { return };
    let Ok((cam, cam_t)) = cameras.single() else { return };

    let Some((gx, gz)) = mouse_to_grid(cursor, w, cam, cam_t, grid.width, grid.height) else { return };

    undo.push(&grid.cells);
    match tool.mode {
        ToolMode::Brush => {
            if tool.selected == CellKind::Player {
                let (opx, opz) = find_player(&grid.cells);
                if grid.get(opx, opz) == CellKind::Player {
                    grid.set(opx, opz, CellKind::Empty);
                }
            }
            grid.set(gx, gz, tool.selected);
        }
        ToolMode::Eraser => {
            grid.set(gx, gz, CellKind::Empty);
        }
    }
    dirty.0 = true;
}

pub fn editor_commands(
    keyboard: Res<ButtonInput<KeyCode>>,
    playtest: Res<crate::playtest::PlaytestState>,
    mut grid: ResMut<GridData>,
    mut meta: ResMut<crate::level_meta::LevelMeta>,
    mut undo: ResMut<UndoRedo>,
    mut pack: ResMut<crate::level_pack::LevelPack>,
    mut current: ResMut<crate::level_pack::CurrentLevel>,
    save_path: Res<crate::level_pack::SavePath>,
    mut toast: ResMut<crate::level_pack::SaveToast>,
    mut dirty: ResMut<crate::level_pack::DirtyFlag>,
) {
    if playtest.active { return; }
    let ctrl = keyboard.pressed(KeyCode::ControlLeft) || keyboard.pressed(KeyCode::ControlRight);

    if ctrl && keyboard.just_pressed(KeyCode::KeyZ) {
        undo.undo(&mut grid);
        dirty.0 = true;
    }
    if ctrl && keyboard.just_pressed(KeyCode::KeyY) {
        undo.redo(&mut grid);
        dirty.0 = true;
    }
    if ctrl && keyboard.just_pressed(KeyCode::KeyS) {
        crate::level_pack::save_current_to_pack(&mut pack, &grid, &meta, current.0);
        crate::level_pack::save_pack(&pack, &save_path);
        dirty.0 = false;
        toast.message = "关卡包已保存".into();
        toast.timer = 1.5;
    }
    if ctrl && keyboard.just_pressed(KeyCode::KeyO) {
        if let Some((w, h, cells, loaded_meta)) = crate::level_meta::load_level() {
            crate::level_pack::save_current_to_pack(&mut pack, &grid, &meta, current.0);
            let entry = crate::level_pack::LevelEntry {
                width: w,
                height: h,
                cells: cells.iter().map(|col| col.iter().map(|&c| crate::level_pack::cell_kind_to_u8(c)).collect()).collect(),
                meta: loaded_meta,
            };
            pack.levels.insert(current.0 + 1, entry);
            crate::level_pack::switch_level(current.0 + 1, &pack, &mut grid, &mut meta, &mut current);
            undo.undo_stack.clear();
            undo.redo_stack.clear();
            dirty.0 = false;
            toast.message = "已导入关卡".into();
            toast.timer = 1.5;
        }
    }
    if ctrl && keyboard.just_pressed(KeyCode::KeyN) {
        crate::level_pack::save_current_to_pack(&mut pack, &grid, &meta, current.0);
        undo.push(&grid.cells);
        *grid = GridData::default();
        *meta = crate::level_meta::LevelMeta::default();
        dirty.0 = true;
    }
    if ctrl && keyboard.just_pressed(KeyCode::KeyE) {
        crate::level_meta::export_current_level(&grid, &meta);
        toast.message = "关卡已导出".into();
        toast.timer = 1.5;
    }
}