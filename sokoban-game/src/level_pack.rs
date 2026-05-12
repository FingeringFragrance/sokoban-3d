use serde::{Deserialize, Serialize};

#[derive(Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum Difficulty {
    Easy,
    Medium,
    Hard,
}

impl std::fmt::Display for Difficulty {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Easy => write!(f, "Easy"),
            Self::Medium => write!(f, "Medium"),
            Self::Hard => write!(f, "Hard"),
        }
    }
}

#[derive(Clone, Serialize, Deserialize)]
pub struct PackLevelMeta {
    pub name: String,
    pub difficulty: Difficulty,
    pub par_steps: u32,
    pub author: String,
    pub description: String,
}

#[derive(Clone, Serialize, Deserialize)]
pub struct PackLevelEntry {
    pub width: u32,
    pub height: u32,
    pub cells: Vec<Vec<u8>>,
    pub meta: PackLevelMeta,
}

#[derive(Clone, Serialize, Deserialize)]
pub struct LevelPack {
    pub name: String,
    pub levels: Vec<PackLevelEntry>,
}

pub fn load_pack(path: &str) -> Option<LevelPack> {
    let s = std::fs::read_to_string(path).ok()?;
    ron::from_str(&s).ok()
}

pub fn scan_packs() -> Vec<String> {
    let mut paths = Vec::new();
    let dirs = ["assets/levels/packs"];
    for dir in &dirs {
        if let Ok(entries) = std::fs::read_dir(dir) {
            for entry in entries.flatten() {
                let path = entry.path();
                if path.extension().map_or(false, |e| e == "sok") {
                    paths.push(path.to_string_lossy().to_string());
                }
            }
        }
    }
    paths.sort();
    paths.dedup();
    paths
}
