use bevy::prelude::{Res, Resource};
use serde::{Deserialize, Serialize};
use std::collections::{HashMap, HashSet};
use std::fs;
use std::path::PathBuf;

use crate::stats::GameStats;

// ============================================================
//  Save data directory (cross-platform)
// ============================================================

fn save_dir() -> PathBuf {
    dirs::data_dir()
        .unwrap_or_else(|| PathBuf::from("."))
        .join("sokoban-3d")
}

fn ensure_save_dir() {
    let dir = save_dir();
    let _ = fs::create_dir_all(&dir);
}

// ============================================================
//  progress.ron — 游戏进度
// ============================================================

#[derive(Resource, Debug, Clone, Serialize, Deserialize)]
pub struct ProgressData {
    #[serde(default)]
    pub completed: HashSet<usize>,
    #[serde(default)]
    pub stars: HashMap<usize, u8>,
    #[serde(default)]
    pub best_steps: HashMap<usize, u32>,
    #[serde(default)]
    pub completion_times: HashMap<usize, f64>,
    #[serde(default)]
    pub unlocked_levels: HashSet<usize>,
    #[serde(default)]
    pub dungeon_progress: HashMap<String, u32>,
    #[serde(default)]
    pub seen_tutorials: HashSet<String>,
    #[serde(default)]
    pub unlocked_achievements: HashSet<String>,
}

impl Default for ProgressData {
    fn default() -> Self {
        let mut unlocked = HashSet::new();
        unlocked.insert(0);
        Self {
            completed: HashSet::new(),
            stars: HashMap::new(),
            best_steps: HashMap::new(),
            completion_times: HashMap::new(),
            unlocked_levels: unlocked,
            dungeon_progress: HashMap::new(),
            seen_tutorials: HashSet::new(),
            unlocked_achievements: HashSet::new(),
        }
    }
}

impl ProgressData {
    pub fn load_or_default() -> Self {
        ensure_save_dir();
        let path = save_dir().join("progress.ron");
        if path.exists() {
            if let Ok(content) = fs::read_to_string(&path) {
                if let Ok(data) = ron::from_str(&content) {
                    return data;
                }
            }
        }
        Self::default()
    }

    pub fn save(&self) {
        ensure_save_dir();
        let path = save_dir().join("progress.ron");
        match ron::ser::to_string_pretty(self, ron::ser::PrettyConfig::default()) {
            Ok(content) => {
                if let Err(e) = fs::write(&path, content) {
                    println!("Failed to save progress: {}", e);
                }
            }
            Err(e) => println!("Failed to serialize progress: {}", e),
        }
    }

    pub fn complete_level(&mut self, index: usize, steps: u32, stars_earned: u8) {
        self.completed.insert(index);
        let prev_stars = self.stars.entry(index).or_insert(0);
        *prev_stars = (*prev_stars).max(stars_earned);
        let prev_steps = self.best_steps.entry(index).or_insert(u32::MAX);
        *prev_steps = (*prev_steps).min(steps);
        self.unlocked_levels.insert(index + 1);
    }
}

// ============================================================
//  settings.ron — 游戏设置
// ============================================================

#[derive(Resource, Debug, Clone, Serialize, Deserialize)]
pub struct SettingsData {
    #[serde(default = "default_volume")]
    pub music_volume: f32,
    #[serde(default = "default_volume")]
    pub sound_volume: f32,
    #[serde(default)]
    pub camera_mode: CameraMode,
    #[serde(default = "default_language")]
    pub language: String,
    #[serde(default)]
    pub colorblind_mode: bool,
    #[serde(default)]
    pub high_contrast: bool,
    #[serde(default = "default_font_size")]
    pub font_size: f32,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, Default)]
pub enum CameraMode {
    #[default]
    FreeOrbit,
    Fixed,
    FollowPlayer,
    FocusLayer,
}

fn default_volume() -> f32 { 0.7 }
fn default_language() -> String { "en".to_string() }
fn default_font_size() -> f32 { 16.0 }

impl Default for SettingsData {
    fn default() -> Self {
        Self {
            music_volume: 0.7,
            sound_volume: 0.7,
            camera_mode: CameraMode::FreeOrbit,
            language: "en".to_string(),
            colorblind_mode: false,
            high_contrast: false,
            font_size: 16.0,
        }
    }
}

impl SettingsData {
    pub fn load_or_default() -> Self {
        ensure_save_dir();
        let path = save_dir().join("settings.ron");
        if path.exists() {
            if let Ok(content) = fs::read_to_string(&path) {
                if let Ok(data) = ron::from_str(&content) {
                    return data;
                }
            }
        }
        Self::default()
    }

    pub fn save(&self) {
        ensure_save_dir();
        let path = save_dir().join("settings.ron");
        match ron::ser::to_string_pretty(self, ron::ser::PrettyConfig::default()) {
            Ok(content) => {
                if let Err(e) = fs::write(&path, content) {
                    println!("Failed to save settings: {}", e);
                }
            }
            Err(e) => println!("Failed to serialize settings: {}", e),
        }
    }
}

// ============================================================
//  stats.ron — 全局统计
// ============================================================

impl GameStats {
    pub fn load_or_default() -> Self {
        ensure_save_dir();
        let path = save_dir().join("stats.ron");
        if path.exists() {
            if let Ok(content) = fs::read_to_string(&path) {
                if let Ok(data) = ron::from_str(&content) {
                    return data;
                }
            }
        }
        Self::default()
    }

    pub fn save_to_file(&self) {
        ensure_save_dir();
        let path = save_dir().join("stats.ron");
        match ron::ser::to_string_pretty(self, ron::ser::PrettyConfig::default()) {
            Ok(content) => {
                if let Err(e) = fs::write(&path, content) {
                    println!("Failed to save stats: {}", e);
                }
            }
            Err(e) => println!("Failed to serialize stats: {}", e),
        }
    }
}

// ============================================================
//  便捷统一的加载/保存
// ============================================================

#[allow(dead_code)]
pub fn load_all() -> (ProgressData, SettingsData, GameStats) {
    (ProgressData::load_or_default(), SettingsData::load_or_default(), GameStats::load_or_default())
}

#[allow(dead_code)]
pub fn save_all(progress: &ProgressData, settings: &SettingsData, stats: &GameStats) {
    progress.save();
    settings.save();
    stats.save_to_file();
}

#[allow(dead_code)]
pub fn load_progress() -> ProgressData {
    ProgressData::load_or_default()
}

pub fn save_progress(data: &ProgressData) {
    data.save();
}

// ============================================================
//  mid_level_save.ron — 中段存档
// ============================================================

use sokoban_core::grid::GridSnapshot;
use sokoban_core::history::MoveHistory;
use sokoban_core::replay::ReplayData;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MidLevelSave {
    pub current_level_index: usize,
    pub level_paths: Vec<String>,
    pub grid_snapshot: GridSnapshot,
    pub initial_snapshot: GridSnapshot,
    pub history: MoveHistory,
    pub replay: ReplayData,
    pub is_daily: bool,
    pub daily_seed: u64,
    pub is_dungeon_mode: bool,
    pub is_multifloor: bool,
    pub current_floor: u8,
    pub floor_count: u8,
    pub scene_theme: String,
    pub level_name: String,
    pub par_steps: Option<u32>,
    pub dungeon_current: usize,
    pub dungeon_total: usize,
    pub dungeon_room_name: String,
}

impl MidLevelSave {
    pub fn save(&self) {
        ensure_save_dir();
        let path = save_dir().join("mid_level_save.ron");
        match ron::ser::to_string_pretty(self, ron::ser::PrettyConfig::default()) {
            Ok(content) => {
                if let Err(e) = fs::write(&path, content) {
                    println!("Failed to save mid-level: {}", e);
                }
            }
            Err(e) => println!("Failed to serialize mid-level: {}", e),
        }
    }

    pub fn load() -> Option<Self> {
        ensure_save_dir();
        let path = save_dir().join("mid_level_save.ron");
        if path.exists() {
            if let Ok(content) = fs::read_to_string(&path) {
                if let Ok(data) = ron::from_str(&content) {
                    return Some(data);
                }
            }
        }
        None
    }

    pub fn exists() -> bool {
        save_dir().join("mid_level_save.ron").exists()
    }

    pub fn delete() {
        let path = save_dir().join("mid_level_save.ron");
        let _ = fs::remove_file(&path);
    }
}

pub fn save_mid_level(game_state: Option<Res<crate::game::GameState>>) {
    if let Some(gs) = game_state {
        if !gs.level_complete {
            let save = MidLevelSave {
                current_level_index: gs.current_level_index,
                level_paths: gs.level_paths.clone(),
                grid_snapshot: gs.grid.snapshot(),
                initial_snapshot: gs.initial_snapshot.clone(),
                history: gs.history.clone(),
                replay: gs.replay.clone(),
                is_daily: gs.is_daily,
                daily_seed: gs.daily_seed,
                is_dungeon_mode: gs.is_dungeon_mode,
                is_multifloor: gs.is_multifloor,
                current_floor: gs.current_floor,
                floor_count: gs.floor_count,
                scene_theme: gs.scene_theme.clone(),
                level_name: gs.level_name.clone(),
                par_steps: gs.par_steps,
                dungeon_current: gs.dungeon_current,
                dungeon_total: gs.dungeon_total,
                dungeon_room_name: gs.dungeon_room_name.clone(),
            };
            save.save();
        }
    }
}