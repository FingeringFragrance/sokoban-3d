use bevy::prelude::*;
use serde::{Serialize, Deserialize};
use crate::grid::{CellKind, GridData};
use crate::level_meta::LevelMeta;
use crate::tool::UndoRedo;

pub fn cell_kind_to_u8(k: CellKind) -> u8 {
    match k {
        CellKind::Empty => 0,
        CellKind::Wall => 1,
        CellKind::Player => 2,
        CellKind::Box => 3,
        CellKind::Target => 4,
        CellKind::Key => 5,
        CellKind::Gate => 6,
        CellKind::Decoration => 7,
    }
}

pub fn u8_to_cell_kind(v: u8) -> CellKind {
    match v {
        1 => CellKind::Wall,
        2 => CellKind::Player,
        3 => CellKind::Box,
        4 => CellKind::Target,
        5 => CellKind::Key,
        6 => CellKind::Gate,
        7 => CellKind::Decoration,
        _ => CellKind::Empty,
    }
}

#[derive(Clone, Serialize, Deserialize)]
pub struct LevelEntry {
    pub width: u32,
    pub height: u32,
    pub cells: Vec<Vec<u8>>,
    pub meta: LevelMeta,
}

impl LevelEntry {
    pub fn from_grid_and_meta(grid: &GridData, meta: &LevelMeta) -> Self {
        let cells: Vec<Vec<u8>> = grid.cells.iter()
            .map(|col| col.iter().map(|&c| cell_kind_to_u8(c)).collect())
            .collect();
        Self {
            width: grid.width,
            height: grid.height,
            cells,
            meta: meta.clone(),
        }
    }
}

#[derive(Resource, Clone, Serialize, Deserialize)]
pub struct LevelPack {
    pub name: String,
    pub levels: Vec<LevelEntry>,
}

impl Default for LevelPack {
    fn default() -> Self {
        let entry = LevelEntry::from_grid_and_meta(&GridData::default(), &LevelMeta::default());
        Self {
            name: "未命名关卡包".into(),
            levels: vec![entry],
        }
    }
}

#[derive(Resource, Default)]
pub struct CurrentLevel(pub usize);

#[derive(Resource, Default)]
pub struct SavePath(pub Option<std::path::PathBuf>);

#[derive(Resource, Default)]
pub struct SaveToast {
    pub message: String,
    pub timer: f32,
}

#[derive(Resource, Default)]
pub struct DirtyFlag(pub bool);

pub fn toast_tick(time: Res<Time>, mut toast: ResMut<SaveToast>) {
    if toast.timer > 0.0 {
        toast.timer -= time.delta_secs();
        if toast.timer <= 0.0 {
            toast.timer = 0.0;
            toast.message.clear();
        }
    }
}

pub fn save_current_to_pack(pack: &mut LevelPack, grid: &GridData, meta: &LevelMeta, current: usize) {
    if current < pack.levels.len() {
        pack.levels[current] = LevelEntry::from_grid_and_meta(grid, meta);
    }
}

pub fn load_entry_into_grid(entry: &LevelEntry, grid: &mut GridData, meta: &mut LevelMeta) {
    let cells: Vec<Vec<CellKind>> = entry.cells.iter()
        .map(|col| col.iter().map(|&c| u8_to_cell_kind(c)).collect())
        .collect();
    grid.cells = cells;
    grid.width = entry.width;
    grid.height = entry.height;
    grid.version += 1;
    *meta = entry.meta.clone();
}

pub fn save_pack(pack: &LevelPack, path: &SavePath) {
    let s = ron::ser::to_string_pretty(pack, ron::ser::PrettyConfig::default()).unwrap();
    let p = path.0.as_deref().unwrap_or(std::path::Path::new("pack.sok"));
    std::fs::write(p, s).ok();
}

pub fn load_pack() -> Option<(LevelPack, std::path::PathBuf)> {
    let path = rfd::FileDialog::new()
        .add_filter("SOK", &["sok"])
        .pick_file()?;
    let s = std::fs::read_to_string(&path).ok()?;
    let pack: LevelPack = ron::from_str(&s).ok()?;
    Some((pack, path))
}

pub fn switch_level(
    idx: usize,
    pack: &LevelPack,
    grid: &mut GridData,
    meta: &mut LevelMeta,
    current: &mut CurrentLevel,
) {
    if idx < pack.levels.len() {
        load_entry_into_grid(&pack.levels[idx], grid, meta);
        current.0 = idx;
    }
}

pub fn pack_commands(
    keyboard: Res<ButtonInput<KeyCode>>,
    playtest: Res<crate::playtest::PlaytestState>,
    mut pack: ResMut<LevelPack>,
    mut grid: ResMut<GridData>,
    mut meta: ResMut<LevelMeta>,
    current: Res<CurrentLevel>,
    mut save_path: ResMut<SavePath>,
    mut toast: ResMut<SaveToast>,
    mut commands: Commands,
    mut dirty: ResMut<DirtyFlag>,
) {
    if playtest.active { return; }
    let ctrl = keyboard.pressed(KeyCode::ControlLeft) || keyboard.pressed(KeyCode::ControlRight);
    let shift = keyboard.pressed(KeyCode::ShiftLeft) || keyboard.pressed(KeyCode::ShiftRight);

    if ctrl && shift && keyboard.just_pressed(KeyCode::KeyS) {
        save_current_to_pack(&mut pack, &grid, &meta, current.0);
        save_pack(&pack, &save_path);
        dirty.0 = false;
        toast.message = "关卡包已保存".into();
        toast.timer = 1.5;
    }
    if ctrl && shift && keyboard.just_pressed(KeyCode::KeyO) {
        save_current_to_pack(&mut pack, &grid, &meta, current.0);
        save_pack(&pack, &save_path);
        if let Some((loaded, path)) = load_pack() {
            save_path.0 = Some(path);
            commands.insert_resource(loaded.clone());
            commands.insert_resource(CurrentLevel(0));
            load_entry_into_grid(&loaded.levels[0], &mut grid, &mut meta);
            dirty.0 = false;
            toast.message = "关卡包已加载".into();
            toast.timer = 1.5;
        }
    }
    if ctrl && shift && keyboard.just_pressed(KeyCode::KeyN) {
        save_current_to_pack(&mut pack, &grid, &meta, current.0);
        save_pack(&pack, &save_path);
        commands.insert_resource(LevelPack::default());
        commands.insert_resource(CurrentLevel(0));
        *grid = GridData::default();
        *meta = LevelMeta::default();
        save_path.0 = None;
        dirty.0 = false;
        toast.message = "新建关卡包".into();
        toast.timer = 1.5;
    }
}

pub fn level_navigation(
    keyboard: Res<ButtonInput<KeyCode>>,
    playtest: Res<crate::playtest::PlaytestState>,
    edit: Res<crate::level_meta::EditState>,
    mut pack: ResMut<LevelPack>,
    mut grid: ResMut<GridData>,
    mut meta: ResMut<LevelMeta>,
    mut current: ResMut<CurrentLevel>,
    mut undo: ResMut<UndoRedo>,
    mut toast: ResMut<SaveToast>,
    mut dirty: ResMut<DirtyFlag>,
) {
    if playtest.active { return; }
    if edit.active.is_some() { return; }
    let ctrl = keyboard.pressed(KeyCode::ControlLeft) || keyboard.pressed(KeyCode::ControlRight);
    let shift = keyboard.pressed(KeyCode::ShiftLeft) || keyboard.pressed(KeyCode::ShiftRight);

    if keyboard.just_pressed(KeyCode::PageUp) {
        if current.0 > 0 {
            save_current_to_pack(&mut pack, &grid, &meta, current.0);
            switch_level(current.0 - 1, &pack, &mut grid, &mut meta, &mut current);
            undo.undo_stack.clear();
            undo.redo_stack.clear();
            dirty.0 = false;
        }
    }
    if keyboard.just_pressed(KeyCode::PageDown) {
        if current.0 + 1 < pack.levels.len() {
            save_current_to_pack(&mut pack, &grid, &meta, current.0);
            switch_level(current.0 + 1, &pack, &mut grid, &mut meta, &mut current);
            undo.undo_stack.clear();
            undo.redo_stack.clear();
            dirty.0 = false;
        }
    }
    if keyboard.just_pressed(KeyCode::Insert) {
        save_current_to_pack(&mut pack, &grid, &meta, current.0);
        let new_entry = LevelEntry::from_grid_and_meta(&GridData::default(), &LevelMeta::default());
        pack.levels.insert(current.0 + 1, new_entry);
        switch_level(current.0 + 1, &pack, &mut grid, &mut meta, &mut current);
        undo.undo_stack.clear();
        undo.redo_stack.clear();
        dirty.0 = false;
        toast.message = format!("已添加第{}关", current.0 + 1);
        toast.timer = 1.5;
    }
    if keyboard.just_pressed(KeyCode::Delete) {
        if pack.levels.len() > 1 {
            save_current_to_pack(&mut pack, &grid, &meta, current.0);
            pack.levels.remove(current.0);
            let idx = if current.0 >= pack.levels.len() { pack.levels.len() - 1 } else { current.0 };
            switch_level(idx, &pack, &mut grid, &mut meta, &mut current);
            undo.undo_stack.clear();
            undo.redo_stack.clear();
            dirty.0 = false;
            toast.message = "关卡已删除".into();
            toast.timer = 1.5;
        }
    }
    if ctrl && keyboard.just_pressed(KeyCode::KeyD) {
        save_current_to_pack(&mut pack, &grid, &meta, current.0);
        let clone = pack.levels[current.0].clone();
        pack.levels.insert(current.0 + 1, clone);
        switch_level(current.0 + 1, &pack, &mut grid, &mut meta, &mut current);
        undo.undo_stack.clear();
        undo.redo_stack.clear();
        dirty.0 = false;
        toast.message = "关卡已复制".into();
        toast.timer = 1.5;
    }
    if ctrl && shift && keyboard.just_pressed(KeyCode::ArrowUp) && edit.active.is_none() {
        if current.0 > 0 {
            save_current_to_pack(&mut pack, &grid, &meta, current.0);
            pack.levels.swap(current.0, current.0 - 1);
            switch_level(current.0 - 1, &pack, &mut grid, &mut meta, &mut current);
            undo.undo_stack.clear();
            undo.redo_stack.clear();
            dirty.0 = false;
            toast.message = "关卡已上移".into();
            toast.timer = 1.0;
        }
    }
    if ctrl && shift && keyboard.just_pressed(KeyCode::ArrowDown) && edit.active.is_none() {
        if current.0 + 1 < pack.levels.len() {
            save_current_to_pack(&mut pack, &grid, &meta, current.0);
            pack.levels.swap(current.0, current.0 + 1);
            switch_level(current.0 + 1, &pack, &mut grid, &mut meta, &mut current);
            undo.undo_stack.clear();
            undo.redo_stack.clear();
            dirty.0 = false;
            toast.message = "关卡已下移".into();
            toast.timer = 1.0;
        }
    }
}

#[derive(Component)]
pub struct LevelCard(pub usize);

pub fn level_card_click(
    playtest: Res<crate::playtest::PlaytestState>,
    edit: Res<crate::level_meta::EditState>,
    mut pack: ResMut<LevelPack>,
    mut grid: ResMut<GridData>,
    mut meta: ResMut<LevelMeta>,
    mut current: ResMut<CurrentLevel>,
    mut undo: ResMut<UndoRedo>,
    mut dirty: ResMut<DirtyFlag>,
    cards: Query<(&Interaction, &LevelCard), Changed<Interaction>>,
) {
    if playtest.active { return; }
    if edit.active.is_some() { return; }

    for (&interaction, card) in &cards {
        if interaction == Interaction::Pressed {
            let idx = card.0;
            if idx != current.0 && idx < pack.levels.len() {
                save_current_to_pack(&mut pack, &grid, &meta, current.0);
                switch_level(idx, &pack, &mut grid, &mut meta, &mut current);
                undo.undo_stack.clear();
                undo.redo_stack.clear();
                dirty.0 = false;
            }
        }
    }
}