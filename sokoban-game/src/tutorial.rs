use bevy::prelude::*;
use std::collections::HashSet;

use sokoban_core::grid::Grid;
use sokoban_core::types::*;

use crate::save::ProgressData;

#[derive(Debug, Clone)]
pub struct TutorialEntry {
    pub id: &'static str,
    pub title_en: &'static str,
    pub title_zh: &'static str,
    pub desc_en: &'static str,
    pub desc_zh: &'static str,
}

pub const TUTORIALS: &[TutorialEntry] = &[
    TutorialEntry {
        id: "ice",
        title_en: "Ice Floor",
        title_zh: "冰面",
        desc_en: "Objects slide on ice in the push direction until hitting an obstacle.",
        desc_zh: "物体在冰面上沿推动方向滑行，直到碰到障碍物停下。",
    },
    TutorialEntry {
        id: "key",
        title_en: "Key",
        title_zh: "钥匙",
        desc_en: "Walk over a key to collect it. It opens a matching colored gate.",
        desc_zh: "走过钥匙自动拾取，可打开同色的门。",
    },
    TutorialEntry {
        id: "gate",
        title_en: "Gate",
        title_zh: "门",
        desc_en: "A gate blocks passage. Collect the matching colored key to open it.",
        desc_zh: "门会阻挡通行，需要拾取同色钥匙才能打开。",
    },
    TutorialEntry {
        id: "switch",
        title_en: "Switch",
        title_zh: "开关",
        desc_en: "Step on a switch to toggle the connected pillar up or down.",
        desc_zh: "踩下开关可以切换对应石柱的升降状态。",
    },
    TutorialEntry {
        id: "pillar",
        title_en: "Pillar",
        title_zh: "石柱",
        desc_en: "A raised pillar blocks passage. Activate the matching switch to lower it.",
        desc_zh: "升起的石柱会阻挡通行，激活对应开关可使其降下。",
    },
    TutorialEntry {
        id: "spring",
        title_en: "Spring",
        title_zh: "弹簧",
        desc_en: "A spring bounces a box one extra tile in the push direction.",
        desc_zh: "弹簧会将箱子沿推动方向额外弹射一格。",
    },
    TutorialEntry {
        id: "heavy_box",
        title_en: "Heavy Box",
        title_zh: "重型箱子",
        desc_en: "Heavy boxes cannot be pushed. They act as permanent obstacles.",
        desc_zh: "重型箱子无法被推动，等同于障碍物。",
    },
    TutorialEntry {
        id: "fragile_box",
        title_en: "Fragile Box",
        title_zh: "脆弱箱子",
        desc_en: "Push a fragile box into a cracked wall to destroy both and clear the path.",
        desc_zh: "将脆弱箱子推入裂墙，两者同时销毁，打通道路。",
    },
    TutorialEntry {
        id: "bomb",
        title_en: "Bomb",
        title_zh: "炸弹",
        desc_en: "Push a bomb into a cracked wall to destroy both.",
        desc_zh: "将炸弹推入裂墙，两者同时销毁。",
    },
    TutorialEntry {
        id: "water",
        title_en: "Water / Pit",
        title_zh: "水面 / 深坑",
        desc_en: "Boxes that fall into water or a pit are destroyed. Be careful!",
        desc_zh: "箱子掉入水面或深坑会被销毁，请小心！",
    },
    TutorialEntry {
        id: "conveyor",
        title_en: "Conveyor Belt",
        title_zh: "传送带",
        desc_en: "Conveyor belts automatically push objects in their direction each turn.",
        desc_zh: "传送带每回合自动沿方向推移物体。",
    },
    TutorialEntry {
        id: "portal",
        title_en: "Portal",
        title_zh: "传送门",
        desc_en: "Stepping on a portal teleports you to the matching portal.",
        desc_zh: "踩上传送门会传送到配对的另一个传送门。",
    },
    TutorialEntry {
        id: "glass",
        title_en: "Glass Floor",
        title_zh: "玻璃地板",
        desc_en: "Glass floors break when a box lands on them, becoming a pit.",
        desc_zh: "箱子推上玻璃地板后，玻璃碎裂变为深坑。",
    },
    TutorialEntry {
        id: "pressure_plate",
        title_en: "Pressure Plate",
        title_zh: "压力板",
        desc_en: "Stepping on a pressure plate activates the linked mechanism.",
        desc_zh: "踩上压力板会触发关联的机关。",
    },
    TutorialEntry {
        id: "cracked_wall",
        title_en: "Cracked Wall",
        title_zh: "裂墙",
        desc_en: "Cracked walls can be destroyed by pushing a fragile box or bomb into them.",
        desc_zh: "裂墙可以被脆弱箱子或炸弹破坏。",
    },
    TutorialEntry {
        id: "mud",
        title_en: "Mud",
        title_zh: "泥地",
        desc_en: "Mud slows movement. Objects stop immediately on mud.",
        desc_zh: "泥地会减速，物体到达泥地后立即停止滑行。",
    },
];

// ---- UI marker components (spawned in setup_hud) ----

#[derive(Component)]
pub struct TutorialOverlay;

#[derive(Component)]
pub struct TutorialTitleText;

#[derive(Component)]
pub struct TutorialDescText;

#[derive(Component)]
pub struct TutorialProgressText;

// ---- Resource ----

#[derive(Resource)]
pub struct TutorialState {
    pub entries: Vec<&'static TutorialEntry>,
    pub current_index: usize,
    pub active: bool,
}

impl Default for TutorialState {
    fn default() -> Self {
        Self {
            entries: Vec::new(),
            current_index: 0,
            active: false,
        }
    }
}

// ---- Logic ----

pub fn scan_level_items(grid: &Grid) -> HashSet<String> {
    let mut items = HashSet::new();
    for row in &grid.cells {
        for cell in row {
            match cell.object {
                ObjectType::Key(_) => { items.insert("key".into()); }
                ObjectType::Gate(_) => { items.insert("gate".into()); }
                ObjectType::Switch(_) => { items.insert("switch".into()); }
                ObjectType::Pillar(_) => { items.insert("pillar".into()); }
                ObjectType::Spring => { items.insert("spring".into()); }
                ObjectType::HeavyBox => { items.insert("heavy_box".into()); }
                ObjectType::FragileBox => { items.insert("fragile_box".into()); }
                ObjectType::Bomb => { items.insert("bomb".into()); }
                ObjectType::CrackedWall => { items.insert("cracked_wall".into()); }
                _ => {}
            }
            match cell.floor {
                FloorType::Ice => { items.insert("ice".into()); }
                FloorType::Water | FloorType::Pit => { items.insert("water".into()); }
                FloorType::Conveyor(_) => { items.insert("conveyor".into()); }
                FloorType::Portal(_) => { items.insert("portal".into()); }
                FloorType::Glass => { items.insert("glass".into()); }
                FloorType::PressurePlate => { items.insert("pressure_plate".into()); }
                FloorType::Mud => { items.insert("mud".into()); }
                _ => {}
            }
        }
    }
    items
}

#[allow(dead_code)]
pub fn setup_tutorials(
    tutorial_state: &mut TutorialState,
    progress: &ProgressData,
    grid: &Grid,
) {
    let items = scan_level_items(grid);
    let mut entries: Vec<&'static TutorialEntry> = items
        .iter()
        .filter(|id| !progress.seen_tutorials.contains(*id))
        .filter_map(|id| TUTORIALS.iter().find(|t| t.id == id.as_str()))
        .collect();

    // Keep a consistent order
    entries.sort_by_key(|e| e.id);

    if entries.is_empty() {
        tutorial_state.active = false;
        tutorial_state.entries.clear();
        tutorial_state.current_index = 0;
    } else {
        tutorial_state.entries = entries;
        tutorial_state.current_index = 0;
        tutorial_state.active = true;
    }
}

pub fn current_tutorial(state: &TutorialState) -> Option<&'static TutorialEntry> {
    if !state.active {
        return None;
    }
    state.entries.get(state.current_index).copied()
}

fn dismiss_current(state: &mut TutorialState, progress: &mut ProgressData) {
    if let Some(entry) = state.entries.get(state.current_index) {
        progress.seen_tutorials.insert(entry.id.to_string());
    }
    state.current_index += 1;
    if state.current_index >= state.entries.len() {
        state.active = false;
        state.entries.clear();
        state.current_index = 0;
    }
}

/// System: handle tutorial dismissal input (Space / Enter)
pub fn tutorial_input(
    keyboard: Res<ButtonInput<KeyCode>>,
    mut tutorial_state: ResMut<TutorialState>,
    mut progress: ResMut<ProgressData>,
) {
    if !tutorial_state.active {
        return;
    }
    if keyboard.just_pressed(KeyCode::Space) || keyboard.just_pressed(KeyCode::Enter) {
        dismiss_current(&mut tutorial_state, &mut progress);
    }
}
