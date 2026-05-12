use bevy::prelude::*;
use serde::{Serialize, Deserialize};
use sokoban_core::level::LevelData;
use sokoban_core::grid::Grid;
use sokoban_core::types::*;
use crate::grid::{CellKind, GridData};
use crate::tool::UndoRedo;
use crate::level_pack::{LevelPack, DirtyFlag};

#[derive(Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum Difficulty {
    Easy,
    Medium,
    Hard,
}

impl Difficulty {
    pub fn name(&self) -> &str {
        match self {
            Difficulty::Easy => "简单",
            Difficulty::Medium => "中等",
            Difficulty::Hard => "困难",
        }
    }

    pub fn next(&self) -> Self {
        match self {
            Difficulty::Easy => Difficulty::Medium,
            Difficulty::Medium => Difficulty::Hard,
            Difficulty::Hard => Difficulty::Easy,
        }
    }
}

#[derive(Resource, Clone, Serialize, Deserialize)]
pub struct LevelMeta {
    pub name: String,
    pub author: String,
    pub difficulty: Difficulty,
    pub par_steps: u32,
    pub description: String,
}

impl Default for LevelMeta {
    fn default() -> Self {
        Self {
            name: "未命名关卡".into(),
            author: "匿名".into(),
            difficulty: Difficulty::Easy,
            par_steps: 10,
            description: String::new(),
        }
    }
}

#[derive(Clone, PartialEq, Eq)]
pub enum EditField {
    Name,
    Author,
    Description,
    PackName,
}

#[derive(Resource, Default)]
pub struct EditState {
    pub active: Option<EditField>,
    pub buffer: String,
}

pub fn text_input(
    keyboard: Res<ButtonInput<KeyCode>>,
    playtest: Res<crate::playtest::PlaytestState>,
    mut edit: ResMut<EditState>,
    mut meta: ResMut<LevelMeta>,
    mut pack: ResMut<LevelPack>,
    mut dirty: ResMut<DirtyFlag>,
) {
    if playtest.active { return; }

    if keyboard.just_pressed(KeyCode::Escape) {
        edit.active = None;
        edit.buffer.clear();
        return;
    }

    let Some(ref field) = edit.active else { return };

    if keyboard.just_pressed(KeyCode::Enter) {
        let s = edit.buffer.clone();
        match field {
            EditField::Name => meta.name = s,
            EditField::Author => meta.author = s,
            EditField::Description => meta.description = s,
            EditField::PackName => pack.name = s,
        }
        edit.active = None;
        edit.buffer.clear();
        dirty.0 = true;
        return;
    }

    if keyboard.just_pressed(KeyCode::Backspace) {
        edit.buffer.pop();
        return;
    }

    for &(code, ch) in key_to_char() {
        if keyboard.just_pressed(code) {
            edit.buffer.push(ch);
        }
    }
}

pub fn key_to_char() -> &'static [(KeyCode, char)] {
    use KeyCode::*;
    &[
        (KeyA, 'a'), (KeyB, 'b'), (KeyC, 'c'), (KeyD, 'd'), (KeyE, 'e'),
        (KeyF, 'f'), (KeyG, 'g'), (KeyH, 'h'), (KeyI, 'i'), (KeyJ, 'j'),
        (KeyK, 'k'), (KeyL, 'l'), (KeyM, 'm'), (KeyN, 'n'), (KeyO, 'o'),
        (KeyP, 'p'), (KeyQ, 'q'), (KeyR, 'r'), (KeyS, 's'), (KeyT, 't'),
        (KeyU, 'u'), (KeyV, 'v'), (KeyW, 'w'), (KeyX, 'x'), (KeyY, 'y'),
        (KeyZ, 'z'),
        (Digit0, '0'), (Digit1, '1'), (Digit2, '2'), (Digit3, '3'),
        (Digit4, '4'), (Digit5, '5'), (Digit6, '6'), (Digit7, '7'),
        (Digit8, '8'), (Digit9, '9'),
        (Space, ' '), (Period, '.'), (Comma, ','), (Minus, '-'),
        (Equal, '='), (Slash, '/'), (Backslash, '\\'),
        (BracketLeft, '['), (BracketRight, ']'),
        (Semicolon, ';'), (Quote, '\''),
    ]
}

pub fn meta_shortcuts(
    keyboard: Res<ButtonInput<KeyCode>>,
    playtest: Res<crate::playtest::PlaytestState>,
    mut meta: ResMut<LevelMeta>,
    mut edit: ResMut<EditState>,
    mut grid: ResMut<GridData>,
    mut undo: ResMut<UndoRedo>,
    mut dirty: ResMut<DirtyFlag>,
    pack: Res<LevelPack>,
) {
    if playtest.active { return; }
    let ctrl = keyboard.pressed(KeyCode::ControlLeft) || keyboard.pressed(KeyCode::ControlRight);

    if ctrl && keyboard.just_pressed(KeyCode::Digit1) {
        edit.active = Some(EditField::Name);
        edit.buffer = meta.name.clone();
    }
    if ctrl && keyboard.just_pressed(KeyCode::Digit2) {
        edit.active = Some(EditField::Author);
        edit.buffer = meta.author.clone();
    }
    if ctrl && keyboard.just_pressed(KeyCode::Digit3) {
        edit.active = Some(EditField::Description);
        edit.buffer = meta.description.clone();
    }
    if ctrl && keyboard.just_pressed(KeyCode::Digit4) {
        edit.active = Some(EditField::PackName);
        edit.buffer = pack.name.clone();
    }

    if keyboard.just_pressed(KeyCode::F2) && edit.active.is_none() {
        meta.difficulty = meta.difficulty.next();
        dirty.0 = true;
    }

    if keyboard.just_pressed(KeyCode::ArrowLeft) && edit.active.is_none() {
        if ctrl {
            let w = grid.width;
            let h = grid.height;
            if w > 4 {
                undo.push(&grid.cells);
                grid.resize(w - 1, h);
                dirty.0 = true;
            }
        } else {
            meta.par_steps = meta.par_steps.saturating_sub(1);
            dirty.0 = true;
        }
    }
    if keyboard.just_pressed(KeyCode::ArrowRight) && edit.active.is_none() {
        if ctrl {
            let w = grid.width;
            let h = grid.height;
            if w < 50 {
                undo.push(&grid.cells);
                grid.resize(w + 1, h);
                dirty.0 = true;
            }
        } else {
            meta.par_steps = meta.par_steps.saturating_add(1);
            dirty.0 = true;
        }
    }
    if ctrl && keyboard.just_pressed(KeyCode::ArrowUp) && edit.active.is_none() {
        let w = grid.width;
        let h = grid.height;
        if h < 50 {
            undo.push(&grid.cells);
            grid.resize(w, h + 1);
            dirty.0 = true;
        }
    }
    if ctrl && keyboard.just_pressed(KeyCode::ArrowDown) && edit.active.is_none() {
        let w = grid.width;
        let h = grid.height;
        if h > 4 {
            undo.push(&grid.cells);
            grid.resize(w, h - 1);
            dirty.0 = true;
        }
    }
}

pub fn export_current_level(grid: &GridData, meta: &LevelMeta) {
    let path = rfd::FileDialog::new()
        .add_filter("RON Level", &["ron"])
        .save_file();
    let Some(path) = path else { return };
    let path_str = path.to_string_lossy().to_string();
    let p = if !path_str.ends_with(".ron") { format!("{}.ron", path_str) } else { path_str };

    let mut core_grid = Grid::new(grid.width, grid.height);
    for x in 0..grid.width as i32 {
        for z in 0..grid.height as i32 {
            let cell = match grid.get(x, z) {
                CellKind::Wall => Cell::wall(),
                CellKind::Player => Cell { floor: FloorType::Normal, object: ObjectType::Player, color: None, facing: None, linked_id: None },
                CellKind::Box => Cell { floor: FloorType::Normal, object: ObjectType::Box, color: None, facing: None, linked_id: None },
                CellKind::Target => Cell::target(),
                CellKind::Key => Cell { floor: FloorType::Normal, object: ObjectType::Key(ItemColor::Red), color: None, facing: None, linked_id: None },
                CellKind::Gate => Cell { floor: FloorType::Normal, object: ObjectType::Gate(ItemColor::Red), color: None, facing: None, linked_id: None },
                CellKind::Decoration => Cell::empty(),
                CellKind::Empty => Cell::empty(),
            };
            core_grid.set(GridPos::new(x, z), cell);
        }
    }

    let core_meta = sokoban_core::level::LevelMeta {
        id: 0,
        name: meta.name.clone(),
        author: meta.author.clone(),
        difficulty: match meta.difficulty {
            Difficulty::Easy => 1,
            Difficulty::Medium => 2,
            Difficulty::Hard => 3,
        },
        par_steps: Some(meta.par_steps),
        tags: Vec::new(),
        description: meta.description.clone(),
    };

    let lv = LevelData {
        meta: core_meta,
        grid: Some(core_grid),
        ascii: None,
        scene_theme: "default".to_string(),
    };

    match lv.save_to_ron(&p) {
        Ok(()) => {
            println!("Exported level to {}", p);
        }
        Err(e) => {
            println!("Export error: {}", e);
        }
    }
}

pub fn load_level() -> Option<(u32, u32, Vec<Vec<CellKind>>, LevelMeta)> {
    let path = rfd::FileDialog::new()
        .add_filter("RON", &["ron"])
        .pick_file()?;
    let s = std::fs::read_to_string(&path).ok()?;
    ron::from_str(&s).ok()
}