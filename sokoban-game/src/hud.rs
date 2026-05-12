use bevy::prelude::*;

use crate::assets::FontAssets;
use crate::audio::GameVolume;
use crate::dungeon::DungeonManager;
use crate::effects::PortalFlashOverlay;
use crate::game::{GameState, RestartConfirmState};
use crate::locale::Locale;
use crate::multifloor::MultiFloorRun;
use crate::save::{save_progress, ProgressData};
use crate::states::AppState;
use crate::tutorial::{
    current_tutorial, TutorialDescText, TutorialOverlay, TutorialProgressText,
    TutorialState, TutorialTitleText,
};
use sokoban_core::types::Direction;

fn hud_font(font_size: f32, fonts: &Option<Res<FontAssets>>) -> TextFont {
    let font = fonts.as_ref().map(|f| f.default_font.clone()).unwrap_or_default();
    TextFont { font, font_size, ..default() }
}

// ---- HUD components ----

#[derive(Component)]
pub struct HudRoot;

#[derive(Component)]
pub struct StepText;

#[derive(Component)]
pub struct StatusText;

#[derive(Component)]
pub struct LevelNameText;

#[derive(Component)]
pub struct HintText;

#[derive(Component)]
pub struct VolumeText;

#[derive(Component)]
pub struct ProgressText;

#[derive(Component)]
pub struct LanguageText;

#[derive(Component)]
pub struct ColorblindText;

// ---- Dungeon HUD components ----

#[derive(Component)]
pub struct DungeonHudRoot;

#[derive(Component)]
pub struct ItemSlot(pub usize);

#[derive(Component)]
pub struct MinimapContainer;

#[derive(Component)]
pub struct MinimapRoomText;

// ---- Pause components ----

#[derive(Component)]
pub struct PauseOverlay;

#[derive(Component)]
pub(crate) struct PauseButton(pub(crate) PauseAction);

#[derive(Clone, Copy)]
pub(crate) enum PauseAction {
    Resume,
    Restart,
    Settings,
    Menu,
    SaveAndExit,
    VolumeUp,
    VolumeDown,
    ResetProgress,
    ToggleLanguage,
    ToggleColorblind,
}

// ---- Setup HUD ----

pub fn setup_hud(commands: &mut Commands, fonts: Option<Res<FontAssets>>) {
    commands
        .spawn((
            Node {
                width: Val::Percent(100.0),
                height: Val::Percent(100.0),
                flex_direction: FlexDirection::Column,
                justify_content: JustifyContent::SpaceBetween,
                ..default()
            },
            HudRoot,
        ))
        .with_children(|parent| {
            // Top info bar
            parent
                .spawn(Node {
                    width: Val::Percent(100.0),
                    padding: UiRect::all(Val::Px(16.0)),
                    justify_content: JustifyContent::SpaceBetween,
                    ..default()
                })
                .with_children(|bar| {
                    bar.spawn((
                        Text::new("Level: -"),
                        hud_font(28.0, &fonts),
                        TextColor(Color::WHITE),
                        LevelNameText,
                    ));
                    bar.spawn((
                        Text::new("Steps: 0"),
                        hud_font(28.0, &fonts),
                        TextColor(Color::WHITE),
                        StepText,
                    ));
                });

            // Controls hint
            parent
                .spawn(Node {
                    width: Val::Percent(100.0),
                    padding: UiRect::new(
                        Val::Px(16.0),
                        Val::Px(16.0),
                        Val::Px(0.0),
                        Val::Px(0.0),
                    ),
                    justify_content: JustifyContent::Center,
                    ..default()
                })
                .with_children(|bar| {
                    bar.spawn((
                        Text::new(
                            "WASD: Move  Z: Undo  Ctrl+Y: Redo  R: Restart  H: Hint  N: Next  ESC: Pause",
                        ),
                        hud_font(18.0, &fonts),
                        TextColor(Color::srgba(1.0, 1.0, 1.0, 0.5)),
                    ));
                });

            // Hint / Replay text
            parent
                .spawn(Node {
                    width: Val::Percent(100.0),
                    padding: UiRect::new(
                        Val::Px(16.0),
                        Val::Px(16.0),
                        Val::Px(0.0),
                        Val::Px(0.0),
                    ),
                    justify_content: JustifyContent::Center,
                    ..default()
                })
                .with_children(|bar| {
                    bar.spawn((
                        Text::new(""),
                        hud_font(24.0, &fonts),
                        TextColor(Color::srgb(0.9, 0.85, 0.3)),
                        HintText,
                    ));
                });

            // Status bar (bottom)
            parent
                .spawn(Node {
                    width: Val::Percent(100.0),
                    padding: UiRect::all(Val::Px(16.0)),
                    justify_content: JustifyContent::Center,
                    align_items: AlignItems::Center,
                    ..default()
                })
                .with_children(|bar| {
                    bar.spawn((
                        Text::new(""),
                        hud_font(36.0, &fonts),
                        TextColor(Color::srgb(0.3, 0.85, 0.5)),
                        StatusText,
                    ));
                });

            // Dungeon HUD: item bar + minimap
            parent
                .spawn((
                    Node {
                        width: Val::Percent(100.0),
                        padding: UiRect::new(
                            Val::Px(16.0),
                            Val::Px(16.0),
                            Val::Px(0.0),
                            Val::Px(16.0),
                        ),
                        justify_content: JustifyContent::SpaceBetween,
                        align_items: AlignItems::FlexEnd,
                        ..default()
                    },
                    DungeonHudRoot,
                ))
                .with_children(|bar| {
                    // Item bar (left side)
                    bar.spawn(Node {
                        flex_direction: FlexDirection::Row,
                        column_gap: Val::Px(6.0),
                        align_items: AlignItems::Center,
                        ..default()
                    })
                    .with_children(|row| {
                        for i in 0..5 {
                            row.spawn((
                                Node {
                                    width: Val::Px(48.0),
                                    height: Val::Px(48.0),
                                    flex_direction: FlexDirection::Column,
                                    align_items: AlignItems::Center,
                                    justify_content: JustifyContent::Center,
                                    border: UiRect::all(Val::Px(2.0)),
                                    ..default()
                                },
                                BackgroundColor(Color::srgba(0.15, 0.15, 0.18, 0.4)),
                                BorderColor::all(Color::srgba(1.0, 1.0, 1.0, 0.1)),
                                ItemSlot(i),
                            ));
                        }
                    });

                    // Minimap (right side)
                    bar.spawn((
                        Node {
                            flex_direction: FlexDirection::Column,
                            row_gap: Val::Px(2.0),
                            padding: UiRect::all(Val::Px(8.0)),
                            ..default()
                        },
                        BackgroundColor(Color::srgba(0.0, 0.0, 0.0, 0.5)),
                        MinimapContainer,
                    ));
                });
        });

    // Tutorial overlay
    spawn_tutorial_overlay(commands, &fonts);

    // 传送闪屏覆盖层（初始隐藏）
    commands.spawn((
        Node {
            width: Val::Percent(100.0),
            height: Val::Percent(100.0),
            position_type: PositionType::Absolute,
            ..default()
        },
        BackgroundColor(Color::srgba(1.0, 1.0, 1.0, 0.0)),
        ZIndex(100),
        Visibility::Hidden,
        PortalFlashOverlay,
        HudRoot,
    ));
}

fn spawn_tutorial_overlay(commands: &mut Commands, fonts: &Option<Res<FontAssets>>) {
    commands
        .spawn((
            Node {
                width: Val::Percent(100.0),
                height: Val::Percent(100.0),
                position_type: PositionType::Absolute,
                flex_direction: FlexDirection::Column,
                align_items: AlignItems::Center,
                justify_content: JustifyContent::Center,
                ..default()
            },
            BackgroundColor(Color::srgba(0.0, 0.0, 0.0, 0.78)),
            Visibility::Hidden,
            TutorialOverlay,
        ))
        .with_children(|root| {
            root.spawn(Node {
                width: Val::Px(480.0),
                padding: UiRect::all(Val::Px(32.0)),
                flex_direction: FlexDirection::Column,
                align_items: AlignItems::Center,
                row_gap: Val::Px(16.0),
                ..default()
            })
            .with_children(|panel| {
                panel.spawn((
                    Text::new("1/1"),
                    hud_font(16.0, &fonts),
                    TextColor(Color::srgba(1.0, 1.0, 1.0, 0.4)),
                    TutorialProgressText,
                ));
                panel.spawn((
                    Text::new("Title"),
                    hud_font(36.0, &fonts),
                    TextColor(Color::srgb(0.9, 0.85, 0.3)),
                    TutorialTitleText,
                ));
                panel.spawn((
                    Text::new("Description"),
                    hud_font(20.0, &fonts),
                    TextColor(Color::WHITE),
                    TutorialDescText,
                ));
                panel.spawn((
                    Text::new("Press Space or Enter"),
                    hud_font(16.0, &fonts),
                    TextColor(Color::srgba(1.0, 1.0, 1.0, 0.4)),
                    Node {
                        margin: UiRect::top(Val::Px(8.0)),
                        ..default()
                    },
                ));
            });
        });
}

// ---- Update systems ----

#[allow(clippy::type_complexity)]
pub fn update_hud(
    time: Res<Time>,
    game_state: Option<Res<GameState>>,
    locale: Option<Res<Locale>>,
    restart_confirm: Option<Res<RestartConfirmState>>,
    mut step_query: Query<
        &mut Text,
        (
            With<StepText>,
            Without<StatusText>,
            Without<LevelNameText>,
            Without<HintText>,
        ),
    >,
    mut status_query: Query<
        &mut Text,
        (
            With<StatusText>,
            Without<StepText>,
            Without<LevelNameText>,
            Without<HintText>,
        ),
    >,
    mut name_query: Query<
        &mut Text,
        (
            With<LevelNameText>,
            Without<StepText>,
            Without<StatusText>,
            Without<HintText>,
        ),
    >,
    mut hint_query: Query<
        &mut Text,
        (
            With<HintText>,
            Without<StepText>,
            Without<StatusText>,
            Without<LevelNameText>,
        ),
    >,
    mut status_color_query: Query<&mut TextColor, With<StatusText>>,
) {
    let Some(ref gs) = game_state else {
        return;
    };
    let lang = locale.as_ref().map(|l| l.lang.as_str()).unwrap_or("en");

    for mut text in &mut step_query {
        **text = format!(
            "{}: {}",
            translate("hud.steps", lang),
            gs.grid.current_step
        );
    }

    for mut text in &mut name_query {
        if gs.is_dungeon_mode {
            **text = format!(
                "{}: {} ({}/{})",
                translate("hud.room", lang),
                gs.dungeon_room_name,
                gs.dungeon_current + 1,
                gs.dungeon_total
            );
        } else if gs.is_daily {
            **text = format!("Daily Challenge (seed: {})", gs.daily_seed);
        } else if gs.is_multifloor {
            **text = format!(
                "Floor {}/{} \u{2014} {}",
                gs.current_floor + 1,
                gs.floor_count,
                gs.level_name
            );
        } else {
            **text = format!(
                "{} {}/{} \u{2014} {}",
                translate("hud.level", lang),
                gs.current_level_index + 1,
                gs.level_paths.len(),
                gs.level_name
            );
        }
    }

    // Hint / Replay text
    for mut text in &mut hint_query {
        if let Some(ref rc) = restart_confirm {
            if rc.pending {
                **text = format!(
                    "Press R again to restart ({:.0}s)",
                    rc.timer.ceil()
                );
                continue;
            }
        }
        if gs.level_complete && !gs.replay_string.is_empty() {
            **text = format!("Replay: {}", gs.replay_string);
        } else if let Some(dir) = gs.hint_direction {
            let arrow = match dir {
                Direction::Up => "\u{2191}",
                Direction::Down => "\u{2193}",
                Direction::Left => "\u{2190}",
                Direction::Right => "\u{2192}",
            };
            **text = format!("Hint: {}", arrow);
        } else {
            **text = String::new();
        }
    }

    for mut text in &mut status_query {
        if gs.level_complete {
            let stars_display = format!(
                "{}{}",
                "\u{2605}".repeat(gs.stars_earned as usize),
                "\u{2606}".repeat(3 - gs.stars_earned as usize),
            );
            if gs.is_dungeon_mode {
                **text = format!(
                    "{} {} ({} {})\n{}",
                    translate("hud.room_complete", lang),
                    stars_display,
                    gs.grid.current_step,
                    translate("hud.steps", lang).to_lowercase(),
                    translate("hud.press_next_room", lang),
                );
            } else if gs.is_daily {
                **text = format!(
                    "{} {} ({} {})\nR: Retry | ESC: Menu",
                    translate("hud.complete", lang),
                    stars_display,
                    gs.grid.current_step,
                    translate("hud.steps", lang).to_lowercase(),
                );
            } else {
                let has_next = gs.current_level_index + 1 < gs.level_paths.len();
                let next_hint = if has_next { "N: Next | " } else { "" };
                **text = format!(
                    "{} {} ({} {})\n{}R: Retry | ESC: Menu",
                    translate("hud.complete", lang),
                    stars_display,
                    gs.grid.current_step,
                    translate("hud.steps", lang).to_lowercase(),
                    next_hint,
                );
            }
            for mut color in &mut status_color_query {
                let alpha = (time.elapsed_secs() * 3.0).sin() * 0.3 + 0.7;
                color.0 = Color::srgba(0.3, 0.85, 0.5, alpha);
            }
        } else {
            for mut color in &mut status_color_query {
                color.0 = Color::srgb(0.3, 0.85, 0.5);
            }
            let done = gs.grid.boxes_on_targets();
            let total = gs.grid.box_count();
            if total > 0 {
                **text = format!(
                    "{}: {} / {} {}",
                    translate("hud.boxes", lang),
                    done,
                    total,
                    translate("hud.on_target", lang)
                );
            } else {
                **text = String::new();
            }

            if gs.deadlock_detected {
                let warning = match lang {
                    "zh" => "\n⚠ 死局！按 Z 撤销或 R 重置",
                    _ => "\n⚠ Deadlock! Press Z to undo or R to restart",
                };
                **text = format!("{}{}", text.as_str(), warning);
                for mut color in &mut status_color_query {
                    color.0 = Color::srgb(0.95, 0.3, 0.2);
                }
            }
        }
    }
}

/// System: update tutorial overlay visibility and content
pub fn update_tutorial_overlay(
    _fonts: Option<Res<FontAssets>>,
    tutorial_state: Res<TutorialState>,
    locale: Option<Res<Locale>>,
    mut overlay_query: Query<&mut Visibility, With<TutorialOverlay>>,
    mut title_query: Query<
        &mut Text,
        (
            With<TutorialTitleText>,
            Without<TutorialDescText>,
            Without<TutorialProgressText>,
        ),
    >,
    mut desc_query: Query<
        &mut Text,
        (
            With<TutorialDescText>,
            Without<TutorialTitleText>,
            Without<TutorialProgressText>,
        ),
    >,
    mut progress_query: Query<
        &mut Text,
        (
            With<TutorialProgressText>,
            Without<TutorialTitleText>,
            Without<TutorialDescText>,
        ),
    >,
) {
    let Ok(mut vis) = overlay_query.single_mut() else {
        return;
    };
    let lang = locale.as_ref().map(|l| l.lang.as_str()).unwrap_or("en");

    if tutorial_state.active {
        *vis = Visibility::Visible;
        if let Some(entry) = current_tutorial(&tutorial_state) {
            let title = if lang == "zh" {
                entry.title_zh
            } else {
                entry.title_en
            };
            let desc = if lang == "zh" {
                entry.desc_zh
            } else {
                entry.desc_en
            };
            let prog = format!(
                "{}/{}",
                tutorial_state.current_index + 1,
                tutorial_state.entries.len()
            );

            for mut text in &mut title_query {
                **text = title.to_string();
            }
            for mut text in &mut desc_query {
                **text = desc.to_string();
            }
            for mut text in &mut progress_query {
                **text = prog.clone();
            }
        }
    } else {
        *vis = Visibility::Hidden;
    }
}

/// System: update dungeon item bar
pub fn update_dungeon_items(
    fonts: Option<Res<FontAssets>>,
    dungeon_manager: Res<DungeonManager>,
    mut slot_query: Query<(Entity, &ItemSlot, &mut BackgroundColor, &mut BorderColor)>,
    mut commands: Commands,
) {
    for (slot_entity, slot, mut bg, mut border) in &mut slot_query {
        commands.entity(slot_entity).despawn_related::<Children>();

        if let Some(item) = dungeon_manager.inventory.get(slot.0) {
            bg.0 = Color::srgba(0.3, 0.3, 0.35, 0.8);
            *border = BorderColor::all(Color::srgb(0.9, 0.85, 0.3));

            let key_text = item.key_hint();
            let name_text = item.label();
            commands.entity(slot_entity).with_children(|parent| {
                parent.spawn((
                    Text::new(key_text),
                    hud_font(14.0, &fonts),
                    TextColor(Color::srgb(0.9, 0.85, 0.3)),
                ));
                parent.spawn((
                    Text::new(name_text),
                    hud_font(10.0, &fonts),
                    TextColor(Color::srgba(1.0, 1.0, 1.0, 0.7)),
                ));
            });
        } else {
            bg.0 = Color::srgba(0.15, 0.15, 0.18, 0.4);
            *border = BorderColor::all(Color::srgba(1.0, 1.0, 1.0, 0.1));
        }
    }
}

/// System: update minimap display
pub fn update_minimap(
    fonts: Option<Res<FontAssets>>,
    dungeon_manager: Res<DungeonManager>,
    minimap_query: Query<Entity, With<MinimapContainer>>,
    mut commands: Commands,
) {
    if !dungeon_manager.active {
        return;
    }

    let data = dungeon_manager.minimap_data();

    for container_entity in &minimap_query {
        commands.entity(container_entity).despawn_related::<Children>();

        for (_i, name, _rt, explored, completed, is_current) in &data {
            let text_color = if *is_current {
                Color::srgb(0.9, 0.85, 0.3)
            } else if *completed {
                Color::srgb(0.3, 0.85, 0.5)
            } else if *explored {
                Color::srgba(1.0, 1.0, 1.0, 0.6)
            } else {
                Color::srgba(1.0, 1.0, 1.0, 0.2)
            };

            let prefix = if *is_current {
                "> "
            } else if *completed {
                "v "
            } else if *explored {
                "  "
            } else {
                "? "
            };

            let label = format!("{}{}", prefix, name);
            commands.entity(container_entity).with_children(|parent| {
                parent.spawn((
                    Text::new(label),
                    hud_font(13.0, &fonts),
                    TextColor(text_color),
                    MinimapRoomText,
                ));
            });
        }
    }
}

// ---- Pause menu ----

pub fn setup_pause(
    fonts: Option<Res<FontAssets>>,
    mut commands: Commands,
    volume: Res<GameVolume>,
    progress: Res<ProgressData>,
    settings: Option<Res<crate::save::SettingsData>>,
    locale: Option<Res<Locale>>,
) {
    let vol_label = volume.label();
    let completed_count = progress.completed.len();
    let lang = locale.as_ref().map(|l| l.lang.as_str()).unwrap_or("en");
    let lang_label = locale
        .as_ref()
        .map(|l| l.lang_label())
        .unwrap_or("English");
    let cb_label = if settings.as_ref().map_or(false, |s| s.colorblind_mode) {
        "ON"
    } else {
        "OFF"
    };

    commands
        .spawn((
            Node {
                width: Val::Percent(100.0),
                height: Val::Percent(100.0),
                flex_direction: FlexDirection::Column,
                align_items: AlignItems::Center,
                justify_content: JustifyContent::Center,
                row_gap: Val::Px(16.0),
                ..default()
            },
            BackgroundColor(Color::srgba(0.0, 0.0, 0.0, 0.7)),
            PauseOverlay,
        ))
        .with_children(|root| {
            root.spawn((
                Text::new(translate("pause.title", lang)),
                hud_font(64.0, &fonts),
                TextColor(Color::WHITE),
                Node {
                    margin: UiRect::bottom(Val::Px(10.0)),
                    ..default()
                },
            ));

            let basic_buttons = [
                (translate("pause.resume", lang), PauseAction::Resume),
                (translate("pause.restart", lang), PauseAction::Restart),
                (translate("pause.save_exit", lang), PauseAction::SaveAndExit),
                (translate("pause.settings", lang), PauseAction::Settings),
                (translate("pause.menu", lang), PauseAction::Menu),
            ];

            for (label, action) in basic_buttons {
                root.spawn((
                    Button,
                    Node {
                        width: Val::Px(220.0),
                        height: Val::Px(46.0),
                        align_items: AlignItems::Center,
                        justify_content: JustifyContent::Center,
                        ..default()
                    },
                    BackgroundColor(Color::srgb(0.2, 0.2, 0.25)),
                    PauseButton(action),
                ))
                .with_children(|btn| {
                    btn.spawn((
                        Text::new(label),
                        hud_font(22.0, &fonts),
                        TextColor(Color::WHITE),
                    ));
                });
            }

            // Separator
            root.spawn((
                Node {
                    width: Val::Px(220.0),
                    height: Val::Px(1.0),
                    margin: UiRect::vertical(Val::Px(8.0)),
                    ..default()
                },
                BackgroundColor(Color::srgba(1.0, 1.0, 1.0, 0.15)),
            ));

            // Volume
            root.spawn(Node {
                flex_direction: FlexDirection::Row,
                align_items: AlignItems::Center,
                justify_content: JustifyContent::Center,
                column_gap: Val::Px(12.0),
                ..default()
            })
            .with_children(|row| {
                row.spawn((
                    Text::new(translate("pause.volume", lang)),
                    hud_font(20.0, &fonts),
                    TextColor(Color::srgba(1.0, 1.0, 1.0, 0.7)),
                    Node {
                        width: Val::Px(70.0),
                        ..default()
                    },
                ));
                row.spawn((
                    Button,
                    Node {
                        width: Val::Px(40.0),
                        height: Val::Px(40.0),
                        align_items: AlignItems::Center,
                        justify_content: JustifyContent::Center,
                        ..default()
                    },
                    BackgroundColor(Color::srgb(0.25, 0.25, 0.3)),
                    PauseButton(PauseAction::VolumeDown),
                ))
                .with_children(|btn| {
                    btn.spawn((
                        Text::new("-"),
                        hud_font(24.0, &fonts),
                        TextColor(Color::WHITE),
                    ));
                });
                row.spawn((
                    Text::new(vol_label.clone()),
                    hud_font(20.0, &fonts),
                    TextColor(Color::srgb(0.9, 0.85, 0.3)),
                    Node {
                        width: Val::Px(50.0),
                        justify_content: JustifyContent::Center,
                        ..default()
                    },
                    VolumeText,
                ));
                row.spawn((
                    Button,
                    Node {
                        width: Val::Px(40.0),
                        height: Val::Px(40.0),
                        align_items: AlignItems::Center,
                        justify_content: JustifyContent::Center,
                        ..default()
                    },
                    BackgroundColor(Color::srgb(0.25, 0.25, 0.3)),
                    PauseButton(PauseAction::VolumeUp),
                ))
                .with_children(|btn| {
                    btn.spawn((
                        Text::new("+"),
                        hud_font(24.0, &fonts),
                        TextColor(Color::WHITE),
                    ));
                });
            });

            // Language + Colorblind
            root.spawn(Node {
                flex_direction: FlexDirection::Row,
                align_items: AlignItems::Center,
                justify_content: JustifyContent::Center,
                column_gap: Val::Px(20.0),
                margin: UiRect::top(Val::Px(4.0)),
                ..default()
            })
            .with_children(|row| {
                row.spawn(Node {
                    flex_direction: FlexDirection::Row,
                    align_items: AlignItems::Center,
                    column_gap: Val::Px(8.0),
                    ..default()
                })
                .with_children(|group| {
                    group.spawn((
                        Text::new(translate("pause.language", lang)),
                        hud_font(18.0, &fonts),
                        TextColor(Color::srgba(1.0, 1.0, 1.0, 0.7)),
                    ));
                    group
                        .spawn((
                            Button,
                            Node {
                                width: Val::Px(90.0),
                                height: Val::Px(34.0),
                                align_items: AlignItems::Center,
                                justify_content: JustifyContent::Center,
                                ..default()
                            },
                            BackgroundColor(Color::srgb(0.25, 0.25, 0.3)),
                            PauseButton(PauseAction::ToggleLanguage),
                        ))
                        .with_children(|btn| {
                            btn.spawn((
                                Text::new(lang_label),
                                hud_font(16.0, &fonts),
                                TextColor(Color::srgb(0.9, 0.85, 0.3)),
                                LanguageText,
                            ));
                        });
                });

                row.spawn(Node {
                    flex_direction: FlexDirection::Row,
                    align_items: AlignItems::Center,
                    column_gap: Val::Px(8.0),
                    ..default()
                })
                .with_children(|group| {
                    group.spawn((
                        Text::new(translate("pause.colorblind", lang)),
                        hud_font(18.0, &fonts),
                        TextColor(Color::srgba(1.0, 1.0, 1.0, 0.7)),
                    ));
                    group
                        .spawn((
                            Button,
                            Node {
                                width: Val::Px(60.0),
                                height: Val::Px(34.0),
                                align_items: AlignItems::Center,
                                justify_content: JustifyContent::Center,
                                ..default()
                            },
                            BackgroundColor(Color::srgb(0.25, 0.25, 0.3)),
                            PauseButton(PauseAction::ToggleColorblind),
                        ))
                        .with_children(|btn| {
                            btn.spawn((
                                Text::new(cb_label),
                                hud_font(16.0, &fonts),
                                TextColor(Color::srgb(0.9, 0.85, 0.3)),
                                ColorblindText,
                            ));
                        });
                });
            });

            // Separator
            root.spawn((
                Node {
                    width: Val::Px(220.0),
                    height: Val::Px(1.0),
                    margin: UiRect::vertical(Val::Px(8.0)),
                    ..default()
                },
                BackgroundColor(Color::srgba(1.0, 1.0, 1.0, 0.15)),
            ));

            // Progress + Reset
            root.spawn(Node {
                flex_direction: FlexDirection::Row,
                align_items: AlignItems::Center,
                justify_content: JustifyContent::Center,
                column_gap: Val::Px(12.0),
                margin: UiRect::top(Val::Px(4.0)),
                ..default()
            })
            .with_children(|row| {
                row.spawn((
                    Text::new(format!(
                        "{}: {}",
                        translate("pause.completed", lang),
                        completed_count
                    )),
                    hud_font(18.0, &fonts),
                    TextColor(Color::srgba(1.0, 1.0, 1.0, 0.5)),
                    ProgressText,
                ));
                row.spawn((
                    Button,
                    Node {
                        width: Val::Px(100.0),
                        height: Val::Px(32.0),
                        align_items: AlignItems::Center,
                        justify_content: JustifyContent::Center,
                        ..default()
                    },
                    BackgroundColor(Color::srgb(0.45, 0.2, 0.2)),
                    PauseButton(PauseAction::ResetProgress),
                ))
                .with_children(|btn| {
                    btn.spawn((
                        Text::new(translate("pause.reset", lang)),
                        hud_font(16.0, &fonts),
                        TextColor(Color::srgb(1.0, 0.6, 0.6)),
                    ));
                });
            });

            // Separator
            root.spawn((
                Node {
                    width: Val::Px(220.0),
                    height: Val::Px(1.0),
                    margin: UiRect::vertical(Val::Px(8.0)),
                    ..default()
                },
                BackgroundColor(Color::srgba(1.0, 1.0, 1.0, 0.15)),
            ));

            // Controls
            let bindings = [
                ("WASD / Arrows", "Move"),
                ("Z", "Undo"),
                ("R", "Restart"),
                ("H", "Hint"),
                ("N", "Next Level"),
                ("P", "Prev Level"),
                ("1-9", "Jump to Level"),
                ("ESC", "Pause / Resume"),
            ];

            root.spawn(Node {
                flex_direction: FlexDirection::Column,
                align_items: AlignItems::Center,
                row_gap: Val::Px(2.0),
                ..default()
            })
            .with_children(|col| {
                col.spawn((
                    Text::new(translate("pause.controls", lang)),
                    hud_font(16.0, &fonts),
                    TextColor(Color::srgba(1.0, 1.0, 1.0, 0.4)),
                    Node {
                        margin: UiRect::bottom(Val::Px(4.0)),
                        ..default()
                    },
                ));
                for (key, desc) in bindings {
                    col.spawn(Node {
                        flex_direction: FlexDirection::Row,
                        column_gap: Val::Px(8.0),
                        ..default()
                    })
                    .with_children(|row| {
                        row.spawn((
                            Text::new(key),
                            hud_font(14.0, &fonts),
                            TextColor(Color::srgb(0.9, 0.85, 0.3)),
                            Node {
                                width: Val::Px(120.0),
                                justify_content: JustifyContent::FlexEnd,
                                ..default()
                            },
                        ));
                        row.spawn((
                            Text::new(desc),
                            hud_font(14.0, &fonts),
                            TextColor(Color::srgba(1.0, 1.0, 1.0, 0.5)),
                        ));
                    });
                }
            });
        });
}

pub fn teardown_pause(mut commands: Commands, query: Query<Entity, With<PauseOverlay>>) {
    for entity in &query {
        commands.entity(entity).despawn();
    }
}

#[allow(clippy::type_complexity)]
pub fn pause_button_interaction(
    mut interaction_query: Query<
        (&Interaction, &PauseButton, &mut BackgroundColor),
        Changed<Interaction>,
    >,
    mut next_state: ResMut<NextState<AppState>>,
    mut volume: ResMut<GameVolume>,
    mut progress: ResMut<ProgressData>,
    mut settings: Option<ResMut<crate::save::SettingsData>>,
    mut locale: ResMut<Locale>,
    game_state: Option<Res<GameState>>,
    _dungeon_manager: Option<Res<DungeonManager>>,
    _multi_floor: Option<Res<MultiFloorRun>>,
    mut volume_text_query: Query<
        &mut Text,
        (
            With<VolumeText>,
            Without<ProgressText>,
            Without<LanguageText>,
            Without<ColorblindText>,
        ),
    >,
    mut progress_text_query: Query<
        &mut Text,
        (
            With<ProgressText>,
            Without<VolumeText>,
            Without<LanguageText>,
            Without<ColorblindText>,
        ),
    >,
    mut language_text_query: Query<
        &mut Text,
        (
            With<LanguageText>,
            Without<VolumeText>,
            Without<ProgressText>,
            Without<ColorblindText>,
        ),
    >,
    mut colorblind_text_query: Query<
        &mut Text,
        (
            With<ColorblindText>,
            Without<VolumeText>,
            Without<ProgressText>,
            Without<LanguageText>,
        ),
    >,
) {
    for (interaction, button, mut bg) in &mut interaction_query {
        match *interaction {
            Interaction::Pressed => match button.0 {
                PauseAction::Resume => next_state.set(AppState::Playing),
                PauseAction::Restart => next_state.set(AppState::Loading),
                PauseAction::Settings => next_state.set(AppState::Settings),
                PauseAction::Menu => next_state.set(AppState::Menu),
                PauseAction::SaveAndExit => {
                    if let Some(gs) = game_state.as_ref() {
                        let save_data = crate::save::MidLevelSave {
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
                        save_data.save();
                    }
                    next_state.set(AppState::Menu);
                }
                PauseAction::VolumeUp => {
                    volume.increase();
                    for mut text in &mut volume_text_query {
                        **text = volume.label();
                    }
                }
                PauseAction::VolumeDown => {
                    volume.decrease();
                    for mut text in &mut volume_text_query {
                        **text = volume.label();
                    }
                }
                PauseAction::ResetProgress => {
                    progress.completed.clear();
                    progress.stars.clear();
                    progress.seen_tutorials.clear();
                    save_progress(&progress);
                    for mut text in &mut progress_text_query {
                        **text = format!(
                            "{}: 0",
                            translate("pause.completed", &locale.lang)
                        );
                    }
                }
                PauseAction::ToggleLanguage => {
                    locale.lang = if locale.lang == "en" {
                        "zh".into()
                    } else {
                        "en".into()
                    };
                    if let Some(ref mut s) = settings {
                        s.language = locale.lang.clone();
                        s.save();
                    }
                    for mut text in &mut language_text_query {
                        **text = locale.lang_label().to_string();
                    }
                }
                PauseAction::ToggleColorblind => {
                    if let Some(ref mut s) = settings {
                        s.colorblind_mode = !s.colorblind_mode;
                        s.save();
                    }
                    for mut text in &mut colorblind_text_query {
                        let cb = settings.as_ref().map_or(false, |s| s.colorblind_mode);
                        **text = if cb {
                            "ON".into()
                        } else {
                            "OFF".into()
                        };
                    }
                }
            },
            Interaction::Hovered => {
                bg.0 = Color::srgb(0.3, 0.3, 0.4);
            }
            Interaction::None => match button.0 {
                PauseAction::ResetProgress => {
                    bg.0 = Color::srgb(0.45, 0.2, 0.2);
                }
                PauseAction::VolumeUp
                | PauseAction::VolumeDown
                | PauseAction::ToggleLanguage
                | PauseAction::ToggleColorblind => {
                    bg.0 = Color::srgb(0.25, 0.25, 0.3);
                }
                _ => {
                    bg.0 = Color::srgb(0.2, 0.2, 0.25);
                }
            },
        }
    }
}

// ---- Helper ----

fn translate<'a>(key: &'a str, lang: &str) -> &'a str {
    match lang {
        "zh" => Locale::t_zh(key),
        _ => Locale::t_en(key),
    }
}
