use bevy::prelude::*;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::fs;
use std::path::Path;

const MODELS_DIR: &str = concat!(env!("CARGO_MANIFEST_DIR"), "/assets/models");

/// meta.ron 文件格式（与 glb 文件同名，后缀为 .meta.ron）
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AssetMeta {
    pub model_path: String,
    pub display_name: String,
    pub display_name_key: String,
    pub category: String,
    pub object_type: String,
    pub scene_theme: String,
    pub model_height: f32,
    pub is_pushable: bool,
    #[serde(default)]
    pub animations: Vec<String>,
    #[serde(default = "default_scene_name")]
    pub scene_name: String,
}

fn default_scene_name() -> String {
    "Scene0".to_string()
}

#[derive(Debug, Clone)]
pub struct CatalogEntry {
    pub meta: AssetMeta,
    pub scene_handle: Handle<Scene>,
}

/// 素材目录资源：存储所有已加载的 glb 模型
/// Key 为 (scene_theme, object_type)，查找时先查场景专属再回退 common
#[derive(Resource, Default)]
pub struct AssetCatalog {
    pub entries: Vec<CatalogEntry>,
    lookup: HashMap<(String, String), usize>,
}

impl AssetCatalog {
    /// 查找模型：先查场景专属，再回退 common，找不到返回 None
    pub fn get(&self, scene_theme: &str, object_type: &str) -> Option<&CatalogEntry> {
        let key = (scene_theme.to_string(), object_type.to_string());
        if let Some(&idx) = self.lookup.get(&key) {
            return Some(&self.entries[idx]);
        }
        let common_key = ("common".to_string(), object_type.to_string());
        if let Some(&idx) = self.lookup.get(&common_key) {
            return Some(&self.entries[idx]);
        }
        None
    }

    pub fn has_models(&self) -> bool {
        !self.entries.is_empty()
    }
}

/// Startup 系统：扫描 assets/models/ 目录，加载所有 meta.ron + glb
pub fn load_asset_catalog(mut commands: Commands, asset_server: Res<AssetServer>) {
    let mut catalog = AssetCatalog::default();

    let models_path = Path::new(MODELS_DIR);
    if !models_path.exists() {
        println!("AssetCatalog: {} not found, using procedural meshes", MODELS_DIR);
        commands.insert_resource(catalog);
        return;
    }

    // 扫描每个场景子目录
    let Ok(scenes) = fs::read_dir(models_path) else {
        commands.insert_resource(catalog);
        return;
    };

    for scene_entry in scenes.flatten() {
        let scene_dir = scene_entry.path();
        if !scene_dir.is_dir() {
            continue;
        }

        let scene_name = scene_dir
            .file_name()
            .unwrap_or_default()
            .to_string_lossy()
            .to_string();

        // 查找所有 .meta.ron 文件
        let Ok(files) = fs::read_dir(&scene_dir) else {
            continue;
        };

        for file_entry in files.flatten() {
            let path = file_entry.path();
            let file_name = path
                .file_name()
                .unwrap_or_default()
                .to_string_lossy()
                .to_string();

            if !file_name.ends_with(".meta.ron") {
                continue;
            }

            let Ok(content) = fs::read_to_string(&path) else {
                continue;
            };

            let Ok(meta) = ron::from_str::<AssetMeta>(&content) else {
                println!("AssetCatalog: failed to parse {}", path.display());
                continue;
            };

            // glb 路径相对于 assets/ 目录
            let glb_full_path = scene_dir.join(&meta.model_path);
            if !glb_full_path.exists() {
                println!(
                    "AssetCatalog: skipping {} (glb file not found)",
                    glb_full_path.display()
                );
                continue;
            }

            let glb_path = format!("models/{}/{}", scene_name, meta.model_path);
            let scene_handle: Handle<Scene> =
                asset_server.load(format!("{}#{}", glb_path, meta.scene_name));

            let key = (meta.scene_theme.clone(), meta.object_type.clone());
            let idx = catalog.entries.len();
            catalog.entries.push(CatalogEntry {
                meta,
                scene_handle,
            });
            catalog.lookup.insert(key, idx);
        }
    }

    println!(
        "AssetCatalog: loaded {} entries from {}",
        catalog.entries.len(),
        MODELS_DIR
    );
    commands.insert_resource(catalog);
}

// ============================================================
//  Font assets — CJK-capable font
// ============================================================

#[derive(Resource, Clone)]
pub struct FontAssets {
    pub default_font: Handle<Font>,
}

pub fn load_fonts(mut commands: Commands, asset_server: Res<AssetServer>) {
    // Bevy 0.18 AssetServer: must use relative paths within assets/
    // File is at sokoban-game/assets/fonts/MiSans-Regular.ttf
    // AssetServer roots at sokoban-game/assets/, so use "fonts/MiSans-Regular.ttf"
    let font_path = "fonts/MiSans-Regular.ttf";
    let font_handle: Handle<Font> = asset_server.load(font_path);

    println!("Font loading: {} (handle exists: {})", font_path, font_handle != Handle::default());

    commands.insert_resource(FontAssets { default_font: font_handle });
}
