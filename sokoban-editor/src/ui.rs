use bevy::prelude::*;
use bevy::window::PrimaryWindow;
use crate::grid::{CellKind, GridData};
use crate::tool::{EditorTool, ToolMode};
use crate::level_meta::{LevelMeta, Difficulty, EditField, EditState};
use crate::level_pack::{LevelPack, LevelEntry, CurrentLevel, SaveToast, DirtyFlag, LevelCard};
use crate::validation::{ValidationResult, ValidationState};
use crate::editor_camera::{EditorCam, mouse_to_grid};
use crate::playtest::PlaytestState;

#[derive(Resource)]
pub struct UiFont(pub Handle<Font>);

pub fn load_font(mut commands: Commands, server: Res<AssetServer>) {
    let font = server.load("fonts/MiSans-Regular.ttf");
    commands.insert_resource(UiFont(font));
}

#[derive(Resource)]
pub struct DecorationAssets {
    pub scene: Handle<Scene>,
}

pub fn load_decorations(mut commands: Commands, server: Res<AssetServer>) {
    let scene: Handle<Scene> = server.load("models/girl_pearl_earring.glb#Scene0");
    commands.insert_resource(DecorationAssets { scene });
}

#[derive(Component)]
pub struct UiRoot;

#[derive(Clone, PartialEq, Eq)]
pub(crate) struct UiState {
    mode: ToolMode,
    selected: CellKind,
    grid_pos: Option<(i32, i32)>,
    name: String,
    difficulty: Difficulty,
    par_steps: u32,
    edit_active: Option<EditField>,
    edit_buffer: String,
    pack_name: String,
    level_count: usize,
    current_level: usize,
    validation: Option<ValidationResult>,
    validation_running: bool,
    dirty: bool,
}

impl Default for UiState {
    fn default() -> Self {
        Self {
            mode: ToolMode::Brush,
            selected: CellKind::Wall,
            grid_pos: None,
            name: String::new(),
            difficulty: Difficulty::Easy,
            par_steps: 0,
            edit_active: None,
            edit_buffer: String::new(),
            pack_name: String::new(),
            level_count: 0,
            current_level: 0,
            validation: None,
            validation_running: false,
            dirty: false,
        }
    }
}

pub fn build_ui(
    mut commands: Commands,
    tool: Res<EditorTool>,
    grid: Res<GridData>,
    meta: Res<LevelMeta>,
    edit: Res<EditState>,
    pack: Res<LevelPack>,
    current: Res<CurrentLevel>,
    validation: Res<ValidationState>,
    font: Res<UiFont>,
    playtest: Res<PlaytestState>,
    toast: Res<SaveToast>,
    dirty: Res<DirtyFlag>,
    windows: Query<&Window, With<PrimaryWindow>>,
    cameras: Query<(&Camera, &GlobalTransform), With<EditorCam>>,
    existing: Query<Entity, With<UiRoot>>,
    mut last_state: Local<UiState>,
) {
    if playtest.active {
        if !existing.is_empty() {
            for e in &existing { commands.entity(e).despawn(); }
        }
        return;
    }
    let Ok(w) = windows.single() else { return };
    let cursor = w.cursor_position();
    let Ok((cam, cam_t)) = cameras.single() else { return };

    let grid_pos = cursor.and_then(|c| mouse_to_grid(c, w, cam, cam_t, grid.width, grid.height));
    let current_state = UiState {
        mode: tool.mode,
        selected: tool.selected,
        grid_pos,
        name: meta.name.clone(),
        difficulty: meta.difficulty,
        par_steps: meta.par_steps,
        edit_active: edit.active.clone(),
        edit_buffer: edit.buffer.clone(),
        pack_name: pack.name.clone(),
        level_count: pack.levels.len(),
        current_level: current.0,
        validation: validation.result.clone(),
        validation_running: validation.running,
        dirty: dirty.0,
    };

    if *last_state == current_state && !existing.is_empty() {
        return;
    }
    *last_state = current_state;

    for e in &existing {
        commands.entity(e).despawn();
    }

    let panel_bg = Color::srgba(0.05, 0.05, 0.08, 0.92);
    let card_bg = Color::srgba(0.08, 0.08, 0.13, 0.95);
    let card_active_bg = Color::srgba(0.12, 0.25, 0.15, 0.95);

    commands.spawn((
        Node {
            position_type: PositionType::Absolute,
            top: Val::Px(8.0),
            left: Val::Px(8.0),
            flex_direction: FlexDirection::Column,
            row_gap: Val::Px(4.0),
            padding: UiRect::all(Val::Px(8.0)),
            max_height: Val::Percent(95.0),
            ..default()
        },
        BackgroundColor(panel_bg),
        UiRoot,
    ))
    .with_children(|parent| {
        let edit_pack = edit.active == Some(EditField::PackName);
        let pack_display = if edit_pack {
            format!("关卡包: |{}_ ({}关)", edit.buffer, pack.levels.len())
        } else {
            format!("关卡包: {}{} ({}关)", pack.name, if dirty.0 { " *" } else { "" }, pack.levels.len())
        };
        parent.spawn((
            Text::new(pack_display),
            TextFont { font: font.0.clone(), font_size: 14.0, ..default() },
            TextColor(if edit_pack {
                Color::srgb(0.3, 1.0, 0.3)
            } else if dirty.0 {
                Color::srgb(1.0, 0.5, 0.2)
            } else {
                Color::srgb(0.9, 0.7, 0.3)
            }),
        ));

        parent.spawn((
            Text::new("点击卡片切换关卡 | PgUp/PgDn"),
            TextFont { font: font.0.clone(), font_size: 9.0, ..default() },
            TextColor(Color::srgba(0.5, 0.5, 0.5, 0.6)),
        ));

        parent.spawn(Node {
            flex_direction: FlexDirection::Column,
            row_gap: Val::Px(3.0),
            overflow: Overflow::scroll_y(),
            max_height: Val::Percent(85.0),
            ..default()
        })
        .with_children(|list| {
            for (i, entry) in pack.levels.iter().enumerate() {
                let is_current = i == current.0;
                let bg = if is_current { card_active_bg } else { card_bg };

                list.spawn((
                    Button,
                    Node {
                        flex_direction: FlexDirection::Row,
                        column_gap: Val::Px(8.0),
                        align_items: AlignItems::Center,
                        padding: UiRect::all(Val::Px(6.0)),
                        border: UiRect::all(Val::Px(1.0)),
                        min_width: Val::Px(220.0),
                        ..default()
                    },
                    BackgroundColor(bg),
                    LevelCard(i),
                ))
                .with_children(|card| {
                    build_minimap(card, entry, 48.0);

                    card.spawn(Node {
                        flex_direction: FlexDirection::Column,
                        row_gap: Val::Px(2.0),
                        ..default()
                    })
                    .with_children(|info| {
                        let name_color = if is_current {
                            Color::srgb(0.3, 1.0, 0.3)
                        } else {
                            Color::srgba(0.9, 0.9, 0.9, 0.9)
                        };
                        info.spawn((
                            Text::new(format!("#{} {}", i + 1, entry.meta.name)),
                            TextFont { font: font.0.clone(), font_size: 12.0, ..default() },
                            TextColor(name_color),
                        ));

                        let diff_color = match entry.meta.difficulty {
                            Difficulty::Easy => Color::srgb(0.3, 0.8, 0.3),
                            Difficulty::Medium => Color::srgb(0.9, 0.7, 0.2),
                            Difficulty::Hard => Color::srgb(0.9, 0.3, 0.3),
                        };
                        info.spawn((
                            Text::new(format!("{}  {}x{}  标准{}步", entry.meta.difficulty.name(), entry.width, entry.height, entry.meta.par_steps)),
                            TextFont { font: font.0.clone(), font_size: 10.0, ..default() },
                            TextColor(diff_color),
                        ));
                    });
                });
            }
        });

        parent.spawn(Node {
            flex_direction: FlexDirection::Column,
            row_gap: Val::Px(2.0),
            ..default()
        })
        .with_children(|hint| {
            hint.spawn((
                Text::new("Ctrl+S:保存包 Ctrl+O:导入 N:新建"),
                TextFont { font: font.0.clone(), font_size: 9.0, ..default() },
                TextColor(Color::srgba(0.5, 0.5, 0.5, 0.6)),
            ));
            hint.spawn((
                Text::new("Ins:添加 Del:删除 Ctrl+D:复制"),
                TextFont { font: font.0.clone(), font_size: 9.0, ..default() },
                TextColor(Color::srgba(0.5, 0.5, 0.5, 0.6)),
            ));
            hint.spawn((
                Text::new("Ctrl+Shift+↑↓:移动关卡"),
                TextFont { font: font.0.clone(), font_size: 9.0, ..default() },
                TextColor(Color::srgba(0.5, 0.5, 0.5, 0.6)),
            ));
        });
    });

    let mode_name = match tool.mode {
        ToolMode::Brush => "笔刷",
        ToolMode::Eraser => "橡皮擦",
    };
    let block_name = match tool.selected {
        CellKind::Wall => "墙",
        CellKind::Player => "玩家",
        CellKind::Box => "箱子",
        CellKind::Target => "目标",
        CellKind::Key => "钥匙",
        CellKind::Gate => "门",
        CellKind::Decoration => "装饰",
        CellKind::Empty => "空",
    };

    commands.spawn((
        Node {
            position_type: PositionType::Absolute,
            top: Val::Px(8.0),
            right: Val::Px(8.0),
            flex_direction: FlexDirection::Column,
            align_items: AlignItems::FlexEnd,
            row_gap: Val::Px(3.0),
            padding: UiRect::all(Val::Px(8.0)),
            ..default()
        },
        BackgroundColor(panel_bg),
        UiRoot,
    ))
    .with_children(|parent| {
        parent.spawn((
            Text::new(format!("工具: {} | 方块: {}", mode_name, block_name)),
            TextFont { font: font.0.clone(), font_size: 14.0, ..default() },
            TextColor(Color::srgb(0.9, 0.9, 0.9)),
        ));
        parent.spawn((
            Text::new(format!("网格: {}x{} (Ctrl+←→↑↓调整)", grid.width, grid.height)),
            TextFont { font: font.0.clone(), font_size: 12.0, ..default() },
            TextColor(Color::srgba(0.6, 0.6, 0.6, 0.8)),
        ));
        parent.spawn((
            Text::new("B:笔刷 E:橡皮擦 | 1-7:选方块"),
            TextFont { font: font.0.clone(), font_size: 10.0, ..default() },
            TextColor(Color::srgba(0.5, 0.5, 0.5, 0.6)),
        ));

        if let Some((gx, gz)) = grid_pos {
            let cell = grid.get(gx, gz);
            let cell_name = match cell {
                CellKind::Wall => "墙",
                CellKind::Player => "玩家",
                CellKind::Box => "箱子",
                CellKind::Target => "目标",
                CellKind::Key => "钥匙",
                CellKind::Gate => "门",
                CellKind::Decoration => "装饰",
                CellKind::Empty => "空",
            };
            parent.spawn((
                Text::new(format!("光标: ({}, {}) : {}", gx, gz, cell_name)),
                TextFont { font: font.0.clone(), font_size: 12.0, ..default() },
                TextColor(Color::srgba(0.8, 0.8, 0.8, 0.8)),
            ));
        }
    });

    commands.spawn((
        Node {
            position_type: PositionType::Absolute,
            top: Val::Px(80.0),
            right: Val::Px(8.0),
            flex_direction: FlexDirection::Column,
            row_gap: Val::Px(5.0),
            padding: UiRect::all(Val::Px(8.0)),
            max_width: Val::Px(220.0),
            ..default()
        },
        BackgroundColor(panel_bg),
        UiRoot,
    ))
    .with_children(|parent| {
        parent.spawn((
            Text::new("关卡属性"),
            TextFont { font: font.0.clone(), font_size: 14.0, ..default() },
            TextColor(Color::srgb(0.9, 0.7, 0.3)),
        ));

        let edit_name = edit.active == Some(EditField::Name);
        parent.spawn((
            Text::new(format!("名称: {}{}", meta.name, if edit_name { format!("|{}_", edit.buffer) } else { String::new() })),
            TextFont { font: font.0.clone(), font_size: 12.0, ..default() },
            TextColor(if edit_name { Color::srgb(0.3, 1.0, 0.3) } else { Color::srgba(0.8, 0.8, 0.8, 0.9) }),
        ));

        parent.spawn((
            Text::new(format!("难度: {} (F2切换)", meta.difficulty.name())),
            TextFont { font: font.0.clone(), font_size: 12.0, ..default() },
            TextColor(Color::srgba(0.8, 0.8, 0.8, 0.9)),
        ));

        parent.spawn((
            Text::new(format!("标准步数: {} (← →调整)", meta.par_steps)),
            TextFont { font: font.0.clone(), font_size: 12.0, ..default() },
            TextColor(Color::srgba(0.8, 0.8, 0.8, 0.9)),
        ));

        let edit_author = edit.active == Some(EditField::Author);
        parent.spawn((
            Text::new(format!("作者: {}{}", meta.author, if edit_author { format!("|{}_", edit.buffer) } else { String::new() })),
            TextFont { font: font.0.clone(), font_size: 12.0, ..default() },
            TextColor(if edit_author { Color::srgb(0.3, 1.0, 0.3) } else { Color::srgba(0.8, 0.8, 0.8, 0.9) }),
        ));

        let edit_desc = edit.active == Some(EditField::Description);
        parent.spawn((
            Text::new(format!("描述: {}{}", meta.description, if edit_desc { format!("|{}_", edit.buffer) } else { String::new() })),
            TextFont { font: font.0.clone(), font_size: 11.0, ..default() },
            TextColor(if edit_desc { Color::srgb(0.3, 1.0, 0.3) } else { Color::srgba(0.7, 0.7, 0.7, 0.8) }),
        ));

        parent.spawn((
            Text::new("Ctrl+1:名称 Ctrl+2:作者 Ctrl+3:描述 Ctrl+4:包名"),
            TextFont { font: font.0.clone(), font_size: 9.0, ..default() },
            TextColor(Color::srgba(0.5, 0.5, 0.5, 0.6)),
        ));
    });

    if let Some(ref result) = validation.result {
        commands.spawn((
            Node {
                position_type: PositionType::Absolute,
                top: Val::Px(280.0),
                right: Val::Px(8.0),
                flex_direction: FlexDirection::Column,
                row_gap: Val::Px(3.0),
                padding: UiRect::all(Val::Px(8.0)),
                max_width: Val::Px(240.0),
                ..default()
            },
            BackgroundColor(panel_bg),
            UiRoot,
        ))
        .with_children(|parent| {
            parent.spawn((
                Text::new("关卡验证 (F5)"),
                TextFont { font: font.0.clone(), font_size: 14.0, ..default() },
                TextColor(Color::srgb(0.9, 0.9, 0.3)),
            ));

            if result.errors.is_empty() && result.warnings.is_empty() && result.solvable == Some(true) {
                parent.spawn((
                    Text::new("✓ 关卡有效且可解"),
                    TextFont { font: font.0.clone(), font_size: 13.0, ..default() },
                    TextColor(Color::srgb(0.3, 1.0, 0.3)),
                ));
            }

            for e in &result.errors {
                parent.spawn((
                    Text::new(format!("✗ {}", e)),
                    TextFont { font: font.0.clone(), font_size: 11.0, ..default() },
                    TextColor(Color::srgb(1.0, 0.3, 0.3)),
                ));
            }
            for w in &result.warnings {
                parent.spawn((
                    Text::new(format!("⚠ {}", w)),
                    TextFont { font: font.0.clone(), font_size: 11.0, ..default() },
                    TextColor(Color::srgb(1.0, 0.8, 0.2)),
                ));
            }

            match result.solvable {
                Some(true) => {
                    parent.spawn((
                        Text::new("可解性: ✓ 可解"),
                        TextFont { font: font.0.clone(), font_size: 12.0, ..default() },
                        TextColor(Color::srgb(0.3, 1.0, 0.3)),
                    ));
                }
                Some(false) => {
                    parent.spawn((
                        Text::new("可解性: ✗ 不可解"),
                        TextFont { font: font.0.clone(), font_size: 12.0, ..default() },
                        TextColor(Color::srgb(1.0, 0.3, 0.3)),
                    ));
                }
                None => {
                    parent.spawn((
                        Text::new("可解性: ? 未验证(先修复错误)"),
                        TextFont { font: font.0.clone(), font_size: 11.0, ..default() },
                        TextColor(Color::srgba(0.6, 0.6, 0.6, 0.8)),
                    ));
                }
            }
        });
    }

    if validation.running {
        commands.spawn((
            Node {
                position_type: PositionType::Absolute,
                bottom: Val::Px(40.0),
                left: Val::Px(12.0),
                ..default()
            },
            UiRoot,
        ))
        .with_children(|parent| {
            parent.spawn((
                Text::new("正在求解..."),
                TextFont { font: font.0.clone(), font_size: 14.0, ..default() },
                TextColor(Color::srgb(1.0, 0.8, 0.2)),
            ));
        });
    }

    commands.spawn((
        Node {
            position_type: PositionType::Absolute,
            bottom: Val::Px(6.0),
            left: Val::Px(0.0),
            right: Val::Px(0.0),
            justify_content: JustifyContent::Center,
            ..default()
        },
        UiRoot,
    ))
    .with_children(|parent| {
        parent.spawn((
            Text::new("左键:放置 | B:笔刷 E:橡皮擦 | 1-7:方块 | Ctrl+Z/Y:撤销/重做 | Ctrl+S:保存包 Ctrl+O:导入 Ctrl+N:新建 | 右键拖拽:旋转 | 滚轮:缩放 | WASD:平移 | F:重置视角 | Tab:试玩 | F5:验证"),
            TextFont { font: font.0.clone(), font_size: 11.0, ..default() },
            TextColor(Color::srgba(0.5, 0.5, 0.5, 0.7)),
        ));
    });

    if toast.timer > 0.0 {
        commands.spawn((
            Node {
                position_type: PositionType::Absolute,
                bottom: Val::Px(30.0),
                left: Val::Px(0.0),
                right: Val::Px(0.0),
                justify_content: JustifyContent::Center,
                ..default()
            },
            UiRoot,
        ))
        .with_children(|parent| {
            parent.spawn((
                Text::new(&toast.message),
                TextFont { font: font.0.clone(), font_size: 14.0, ..default() },
                TextColor(Color::srgb(0.3, 1.0, 0.3)),
            ));
        });
    }
}

pub fn build_minimap(parent: &mut bevy::ecs::hierarchy::ChildSpawnerCommands, entry: &LevelEntry, size: f32) {
    let w = entry.width;
    let h = entry.height;
    let max_dim = w.max(h);
    let step = if max_dim > 20 { (max_dim as f32 / 20.0).ceil() as u32 } else { 1 };
    let dw = w / step;
    let dh = h / step;
    let cell_px = (size / dw.max(dh) as f32).max(2.0);

    parent.spawn(Node {
        flex_direction: FlexDirection::Column,
        min_width: Val::Px(size + 4.0),
        min_height: Val::Px(size + 4.0),
        align_items: AlignItems::Center,
        justify_content: JustifyContent::Center,
        ..default()
    })
    .with_children(|map| {
        map.spawn(Node {
            flex_direction: FlexDirection::Column,
            ..default()
        })
        .with_children(|grid| {
            for z in (0..h).step_by(step as usize) {
                grid.spawn(Node {
                    flex_direction: FlexDirection::Row,
                    ..default()
                })
                .with_children(|row| {
                    for x in (0..w).step_by(step as usize) {
                        let cell = entry.cells[x as usize][z as usize];
                        let color = match cell {
                            1 => Color::srgb(0.35, 0.35, 0.42),
                            2 => Color::srgb(0.3, 0.55, 0.85),
                            3 => Color::srgb(0.82, 0.52, 0.2),
                            4 => Color::srgb(0.4, 0.78, 0.5),
                            5 => Color::srgb(0.9, 0.2, 0.2),
                            6 => Color::srgb(0.85, 0.3, 0.3),
                            7 => Color::srgb(0.7, 0.4, 0.9),
                            _ => Color::srgba(0.15, 0.15, 0.18, 0.5),
                        };
                        row.spawn((
                            Node {
                                min_width: Val::Px(cell_px),
                                min_height: Val::Px(cell_px),
                                ..default()
                            },
                            BackgroundColor(color),
                        ));
                    }
                });
            }
        });
    });
}