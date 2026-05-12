use bevy::prelude::*;
use std::collections::HashSet;

use sokoban_core::level::LevelData;

use crate::dungeon::DungeonManager;
use crate::game::{GameState, today_daily_seed};
use crate::locale::{translate, Locale};
use crate::multifloor::MultiFloorRun;
use crate::assets::FontAssets;
use crate::save::{ProgressData, SettingsData};
use crate::states::AppState;

// ============================================================
//  Components & Resources
// ============================================================

#[derive(Component)] pub struct MenuRoot;
#[derive(Component)] pub struct LevelButton(pub usize);
#[derive(Component)] pub struct MenuButton(pub MenuAction);
#[derive(Component)] pub struct LevelCardGrid;
#[derive(Component)] pub struct LevelInfoPanel;
#[derive(Resource)] pub struct MenuSpawned(pub bool);
#[derive(Resource)] pub struct MenuPage(pub MenuPageType);
#[derive(Resource)]
pub struct LevelGridFocus {
    pub index: usize,
    pub columns: usize,
}

#[derive(Clone, Copy)]
pub enum MenuAction {
    Start, Continue, Settings, Exit, Back,
    Classic, Dungeon, Daily, MultiFloor,
    ToggleLang, ToggleColorblind, ToggleHighContrast,
    CycleCameraMode, CycleFontSize,
    VolMusicDown, VolMusicUp, VolSfxDown, VolSfxUp,
}

#[derive(Clone, Copy, PartialEq, Eq)]
pub enum MenuPageType { Main, ModeSelect, ClassicLevels, Settings }

impl Default for MenuPage { fn default() -> Self { Self(MenuPageType::Main) } }

// ============================================================
//  Visual theme
// ============================================================

const BG_DARK: Color = Color::srgb(0.04, 0.04, 0.08);
const BG_MID: Color = Color::srgb(0.07, 0.07, 0.12);
const ACCENT_GREEN: Color = Color::srgb(0.15, 0.62, 0.35);
const ACCENT_PURPLE: Color = Color::srgb(0.45, 0.30, 0.60);
const ACCENT_BLUE: Color = Color::srgb(0.20, 0.45, 0.65);
const ACCENT_GOLD: Color = Color::srgb(0.95, 0.72, 0.15);
#[allow(dead_code)]
const ACCENT_RED: Color = Color::srgb(0.75, 0.22, 0.25);
#[allow(dead_code)]
const ACCENT_CYAN: Color = Color::srgb(0.15, 0.55, 0.55);
const ACCENT_ORANGE: Color = Color::srgb(0.85, 0.50, 0.15);
const BTN_DEFAULT: Color = Color::srgb(0.15, 0.15, 0.22);
const BTN_HOVER: Color = Color::srgb(0.25, 0.25, 0.35);
#[allow(dead_code)]
const BTN_SELECTED: Color = Color::srgb(0.12, 0.45, 0.22);
const TEXT_PRIMARY: Color = Color::srgb(0.95, 0.95, 0.95);
const TEXT_SECONDARY: Color = Color::srgba(0.9, 0.9, 0.9, 0.55);
const TEXT_DIM: Color = Color::srgba(0.7, 0.7, 0.7, 0.25);
const DIVIDER: Color = Color::srgba(1.0, 1.0, 1.0, 0.06);

// Font sizes for readability
const FS_TITLE: f32 = 52.0;
const FS_HEADER: f32 = 34.0;
const FS_BTN_LARGE: f32 = 22.0;
const FS_BTN: f32 = 18.0;
const FS_BODY: f32 = 15.0;
const FS_SMALL: f32 = 13.0;
const FS_HINT: f32 = 11.0;

fn border(color: Color) -> BorderColor {
    BorderColor { top: color, right: color, bottom: color, left: color }
}
fn left_border(color: Color) -> BorderColor {
    BorderColor { top: Color::NONE, right: Color::NONE, bottom: Color::NONE, left: color }
}

// ============================================================
//  Widget helpers — return impl Bundle for clean composition
// ============================================================

fn title_text(text: &str, size: f32, font: &Handle<Font>) -> impl Bundle {
    (Text::new(text.to_string()), TextFont { font: font.clone(), font_size: size, ..default() }, TextColor(ACCENT_GOLD))
}

fn body_text(text: &str, font: &Handle<Font>) -> impl Bundle {
    (Text::new(text.to_string()), TextFont { font: font.clone(), font_size: FS_BODY, ..default() }, TextColor(TEXT_SECONDARY))
}

fn dim_text(text: &str, font: &Handle<Font>) -> impl Bundle {
    (Text::new(text.to_string()), TextFont { font: font.clone(), font_size: FS_HINT, ..default() }, TextColor(TEXT_DIM))
}

fn label_text(text: &str, font: &Handle<Font>) -> impl Bundle {
    (Text::new(text.to_string()), TextFont { font: font.clone(), font_size: FS_BODY, ..default() }, TextColor(Color::srgba(1.0, 1.0, 1.0, 0.7)))
}

fn section_header(text: &str, font: &Handle<Font>) -> impl Bundle {
    (Text::new(text.to_string()), TextFont { font: font.clone(), font_size: FS_BTN, ..default() }, TextColor(Color::srgba(1.0, 1.0, 1.0, 0.45)),
     Node { margin: UiRect::top(Val::Px(18.0)).with_bottom(Val::Px(6.0)), ..default() })
}

fn divider() -> impl Bundle {
    (Node { width: Val::Px(240.0), height: Val::Px(1.0), margin: UiRect::vertical(Val::Px(8.0)), ..default() },
     BackgroundColor(DIVIDER))
}

fn main_btn(label: &str, action: MenuAction, accent: Color, font: &Handle<Font>) -> impl Bundle {
    (Button, Node { width: Val::Px(280.0), height: Val::Px(54.0),
        border: UiRect::all(Val::Px(1.0)),
        align_items: AlignItems::Center, justify_content: JustifyContent::Center, ..default() },
     BackgroundColor(BTN_DEFAULT), border(accent.with_alpha(0.3)),
     MenuButton(action),
     children![(Text::new(label.to_string()), TextFont { font: font.clone(), font_size: FS_BTN_LARGE, ..default() }, TextColor(accent))])
}

fn secondary_btn(label: &str, action: MenuAction, font: &Handle<Font>) -> impl Bundle {
    (Button, Node { width: Val::Px(150.0), height: Val::Px(38.0),
        align_items: AlignItems::Center, justify_content: JustifyContent::Center, ..default() },
     BackgroundColor(Color::srgba(1.0, 1.0, 1.0, 0.06)),
     MenuButton(action),
     children![(Text::new(label.to_string()), TextFont { font: font.clone(), font_size: FS_BTN, ..default() }, TextColor(TEXT_SECONDARY))])
}

fn mode_card(title: &str, desc: &str, action: MenuAction, accent: Color, font: &Handle<Font>) -> impl Bundle {
    (Button, Node { width: Val::Px(400.0), height: Val::Px(76.0),
        flex_direction: FlexDirection::Column, align_items: AlignItems::Center,
        justify_content: JustifyContent::Center, row_gap: Val::Px(3.0),
        border: UiRect::left(Val::Px(3.0)), ..default() },
     BackgroundColor(BG_MID), left_border(accent),
     MenuButton(action),
     children![
         (Text::new(title.to_string()), TextFont { font: font.clone(), font_size: FS_BTN, ..default() }, TextColor(TEXT_PRIMARY)),
         (Text::new(desc.to_string()), TextFont { font: font.clone(), font_size: FS_SMALL, ..default() }, TextColor(TEXT_SECONDARY)),
     ])
}

fn level_card(i: usize, name: &str, is_done: bool, stars: u8, font: &Handle<Font>, is_focused: bool) -> impl Bundle {
    let accent = if is_focused {
        ACCENT_GOLD
    } else if is_done {
        Color::srgb(0.15, 0.55, 0.25)
    } else {
        Color::srgb(0.35, 0.35, 0.42)
    };
    let bg = if is_focused {
        Color::srgb(0.18, 0.16, 0.08)
    } else if is_done {
        Color::srgb(0.10, 0.22, 0.13)
    } else {
        BTN_DEFAULT
    };
    (Button, Node { width: Val::Px(150.0), height: Val::Px(90.0),
        flex_direction: FlexDirection::Column, align_items: AlignItems::Center,
        justify_content: JustifyContent::Center, row_gap: Val::Px(3.0),
        border: UiRect::all(Val::Px(if is_focused { 2.0 } else { 1.0 })), ..default() },
     BackgroundColor(bg),
     border(accent), LevelButton(i),
     children![
         (Text::new(format!("{}", i + 1)),
          TextFont { font: font.clone(), font_size: 28.0, ..default() },
          TextColor(if is_done { Color::srgb(0.2, 0.85, 0.45) } else { TEXT_PRIMARY })),
         (Text::new(name.to_string()), TextFont { font: font.clone(), font_size: FS_SMALL, ..default() }, TextColor(TEXT_SECONDARY)),
         (Text::new("\u{2605}".repeat(stars as usize) + &"\u{2606}".repeat(3usize.saturating_sub(stars as usize))),
          TextFont { font: font.clone(), font_size: 12.0, ..default() }, TextColor(ACCENT_GOLD)),
     ])
}

fn setting_toggle(label: &str, value: &str, action: MenuAction, font: &Handle<Font>) -> impl Bundle {
    (Node { flex_direction: FlexDirection::Row, align_items: AlignItems::Center, column_gap: Val::Px(10.0), ..default() },
     children![
         label_text(label, font),
         (Button, Node { min_width: Val::Px(160.0), height: Val::Px(32.0),
             align_items: AlignItems::Center, justify_content: JustifyContent::Center,
             padding: UiRect::horizontal(Val::Px(12.0)), ..default() },
          BackgroundColor(BTN_DEFAULT), MenuButton(action),
          children![(Text::new(value.to_string()), TextFont { font: font.clone(), font_size: FS_BODY, ..default() }, TextColor(ACCENT_GOLD))]),
     ])
}

fn setting_vol(label: &str, value: &str, dec: MenuAction, inc: MenuAction, font: &Handle<Font>) -> impl Bundle {
    (Node { flex_direction: FlexDirection::Row, align_items: AlignItems::Center, column_gap: Val::Px(10.0), ..default() },
     children![
         label_text(label, font),
         (Button, Node { width: Val::Px(34.0), height: Val::Px(28.0), align_items: AlignItems::Center, justify_content: JustifyContent::Center, ..default() },
          BackgroundColor(BTN_DEFAULT), MenuButton(dec),
          children![(Text::new("\u{2212}"), TextFont { font: font.clone(), font_size: 20.0, ..default() }, TextColor(TEXT_PRIMARY))]),
         (Text::new(value.to_string()), TextFont { font: font.clone(), font_size: FS_BODY, ..default() }, TextColor(TEXT_PRIMARY),
          Node { min_width: Val::Px(48.0), ..default() }),
         (Button, Node { width: Val::Px(34.0), height: Val::Px(28.0), align_items: AlignItems::Center, justify_content: JustifyContent::Center, ..default() },
          BackgroundColor(BTN_DEFAULT), MenuButton(inc),
          children![(Text::new("+"), TextFont { font: font.clone(), font_size: 20.0, ..default() }, TextColor(TEXT_PRIMARY))]),
     ])
}

// ============================================================
//  Screen builders
// ============================================================

fn build_main_menu(commands: &mut Commands, lang: &str, has_save: bool, font: &Handle<Font>) {
    commands.spawn((Node { width: Val::Percent(100.0), height: Val::Percent(100.0),
        flex_direction: FlexDirection::Column, align_items: AlignItems::Center,
        justify_content: JustifyContent::Center, row_gap: Val::Px(12.0), ..default() },
        BackgroundColor(BG_DARK), MenuRoot))
    .with_children(|root| {
        // Decorative top line
        root.spawn((Node { width: Val::Px(80.0), height: Val::Px(3.0), margin: UiRect::bottom(Val::Px(8.0)), ..default() },
            BackgroundColor(ACCENT_GOLD)));

        root.spawn(title_text(translate("menu.title", lang), FS_TITLE, font));
        root.spawn(body_text(translate("menu.subtitle", lang), font));
        root.spawn((Node { height: Val::Px(32.0), ..default() },)); // spacer

        root.spawn(main_btn(translate("menu.start", lang), MenuAction::Start, ACCENT_GREEN, font));
        if has_save {
            root.spawn(main_btn(translate("menu.continue", lang), MenuAction::Continue, ACCENT_BLUE, font));
        }
        root.spawn(main_btn(translate("menu.settings", lang), MenuAction::Settings, ACCENT_PURPLE, font));

        root.spawn((Node { height: Val::Px(8.0), ..default() },)); // spacer

        root.spawn(secondary_btn(translate("menu.exit", lang), MenuAction::Exit, font));
        root.spawn(dim_text("ESC: Quit  |  Click: Select", font));
    });
}

fn build_mode_select(commands: &mut Commands, lang: &str, font: &Handle<Font>) {
    commands.spawn((Node { width: Val::Percent(100.0), height: Val::Percent(100.0),
        flex_direction: FlexDirection::Column, align_items: AlignItems::Center,
        justify_content: JustifyContent::Center, row_gap: Val::Px(14.0), ..default() },
        BackgroundColor(BG_DARK), MenuRoot))
    .with_children(|root| {
        root.spawn(title_text(translate("menu.title", lang), FS_HEADER, font));
        root.spawn(body_text(translate("menu.subtitle", lang), font));
        root.spawn((Node { height: Val::Px(16.0), ..default() },));

        root.spawn(mode_card(translate("menu.mode.classic", lang), translate("menu.mode.classic.desc", lang), MenuAction::Classic, ACCENT_GREEN, font));
        root.spawn(mode_card(translate("menu.mode.dungeon", lang), translate("menu.mode.dungeon.desc", lang), MenuAction::Dungeon, ACCENT_PURPLE, font));
        root.spawn(mode_card(translate("menu.mode.daily", lang), translate("menu.mode.daily.desc", lang), MenuAction::Daily, ACCENT_ORANGE, font));
        root.spawn(mode_card(translate("menu.mode.multifloor", lang), translate("menu.mode.multifloor.desc", lang), MenuAction::MultiFloor, ACCENT_BLUE, font));

        root.spawn((Node { height: Val::Px(8.0), ..default() },));
        root.spawn(secondary_btn(translate("menu.back", lang), MenuAction::Back, font));
    });
}

fn build_classic_levels(
    commands: &mut Commands, lang: &str,
    level_paths: &[String], completed: &HashSet<usize>,
    stars_map: &std::collections::HashMap<usize, u8>,
    font: &Handle<Font>,
    pack: Option<&crate::level_pack::LevelPack>,
    focus: &mut ResMut<LevelGridFocus>,
) {
    let names: Vec<String> = level_paths.iter().enumerate().map(|(i, path)| {
        if path.starts_with("pack://") {
            if let Some(p) = pack {
                let idx_str = path.strip_prefix("pack://").unwrap().split('/').nth(1).unwrap_or("0");
                if let Ok(idx) = idx_str.parse::<usize>() {
                    if let Some(entry) = p.levels.get(idx) {
                        return format!("{} [{}]", entry.meta.name, entry.meta.difficulty);
                    }
                }
            }
            format!("Pack Level {}", i + 1)
        } else {
            LevelData::load_from_ron(path).map(|l| l.meta.name).unwrap_or_else(|_| format!("Level {}", i + 1))
        }
    }).collect();

    let card_width = 150.0;
    let card_gap = 10.0;
    let grid_padding = 16.0;
    let grid_max_width = 680.0;
    let columns = ((grid_max_width + card_gap) / (card_width + card_gap)) as usize;
    focus.columns = columns.max(1);
    if focus.index >= names.len() {
        focus.index = 0;
    }

    commands.spawn((Node { width: Val::Percent(100.0), height: Val::Percent(100.0),
        flex_direction: FlexDirection::Column, align_items: AlignItems::Center,
        justify_content: JustifyContent::Center, row_gap: Val::Px(14.0), ..default() },
        BackgroundColor(BG_DARK), MenuRoot))
    .with_children(|root| {
        if let Some(p) = pack {
            root.spawn(title_text(&format!("{} ({})", p.name, translate("menu.select_level", lang)), FS_HEADER, font));
        } else {
            root.spawn(title_text(translate("menu.select_level", lang), FS_HEADER, font));
        }
        let progress_text = format!("{} {} / {}", completed.len(), translate("complete.steps", lang), level_paths.len());
        root.spawn(body_text(&progress_text, font));

        root.spawn(Node {
            flex_direction: FlexDirection::Row,
            column_gap: Val::Px(20.0),
            align_items: AlignItems::FlexStart,
            ..default()
        })
        .with_children(|row| {
            row.spawn((
                Node {
                    flex_direction: FlexDirection::Column,
                    align_items: AlignItems::Center,
                    max_height: Val::Px(420.0),
                    padding: UiRect::all(Val::Px(grid_padding)),
                    ..default()
                },
                BackgroundColor(Color::srgba(0.0, 0.0, 0.0, 0.2)),
            ))
            .with_children(|scroll_area| {
                scroll_area.spawn((Node {
                    flex_direction: FlexDirection::Row,
                    flex_wrap: FlexWrap::Wrap,
                    justify_content: JustifyContent::Center,
                    column_gap: Val::Px(card_gap),
                    row_gap: Val::Px(card_gap),
                    max_width: Val::Px(grid_max_width),
                    ..default()
                }, LevelCardGrid))
                .with_children(|grid| {
                    for (i, name) in names.iter().enumerate() {
                        let is_focused = i == focus.index;
                        grid.spawn(level_card(i, name, completed.contains(&i), stars_map.get(&i).copied().unwrap_or(0), font, is_focused));
                    }
                });
            });

            row.spawn((
                Node {
                    width: Val::Px(220.0),
                    flex_direction: FlexDirection::Column,
                    row_gap: Val::Px(8.0),
                    padding: UiRect::all(Val::Px(16.0)),
                    ..default()
                },
                BackgroundColor(Color::srgba(0.0, 0.0, 0.0, 0.2)),
                LevelInfoPanel,
            ))
            .with_children(|panel| {
                let idx = focus.index;
                let name = names.get(idx).map(|s| s.as_str()).unwrap_or("-");
                let done = completed.contains(&idx);
                let stars = stars_map.get(&idx).copied().unwrap_or(0);

                panel.spawn(title_text(name, FS_BTN_LARGE, font));
                panel.spawn(divider());

                let status = if done {
                    format!("{} \u{2714}", translate("complete.steps", lang))
                } else {
                    translate("menu.not_completed", lang).to_string()
                };
                panel.spawn(body_text(&status, font));

                let stars_str = "\u{2605}".repeat(stars as usize) + &"\u{2606}".repeat(3usize.saturating_sub(stars as usize));
                panel.spawn((Text::new(stars_str),
                    TextFont { font: font.clone(), font_size: 24.0, ..default() },
                    TextColor(ACCENT_GOLD)));

                panel.spawn((Node { height: Val::Px(8.0), ..default() },));

                let nav_hint = translate("menu.grid_nav", lang);
                panel.spawn(dim_text(&nav_hint, font));
            });
        });

        root.spawn(secondary_btn(translate("menu.back", lang), MenuAction::Back, font));
    });
}

fn build_settings(commands: &mut Commands, lang: &str, s: &SettingsData, font: &Handle<Font>) {
    let vol_m = (s.music_volume * 100.0) as u32;
    let vol_s = (s.sound_volume * 100.0) as u32;
    let cam = match s.camera_mode {
        crate::save::CameraMode::FreeOrbit => translate("settings.camera_free", lang),
        crate::save::CameraMode::Fixed => translate("settings.camera_fixed", lang),
        crate::save::CameraMode::FollowPlayer => translate("settings.camera_follow", lang),
        crate::save::CameraMode::FocusLayer => "Focus Layer",
    };
    let cb = if s.colorblind_mode { "ON" } else { "OFF" };
    let hc = if s.high_contrast { "ON" } else { "OFF" };
    let fs = if s.font_size <= 13.0 { translate("settings.font_small", lang) }
        else if s.font_size >= 20.0 { translate("settings.font_large", lang) }
        else { translate("settings.font_normal", lang) };

    commands.spawn((Node { width: Val::Percent(100.0), height: Val::Percent(100.0),
        flex_direction: FlexDirection::Column, align_items: AlignItems::Center,
        row_gap: Val::Px(8.0), padding: UiRect::all(Val::Px(24.0)), ..default() },
        BackgroundColor(BG_DARK), MenuRoot))
    .with_children(|root| {
        root.spawn(title_text(translate("settings.title", lang), FS_HEADER, font));
        root.spawn(divider());

        root.spawn(section_header(translate("settings.audio", lang), font));
        root.spawn(setting_vol(translate("settings.music_vol", lang), &format!("{}%", vol_m), MenuAction::VolMusicDown, MenuAction::VolMusicUp, font));
        root.spawn(setting_vol(translate("settings.sfx_vol", lang), &format!("{}%", vol_s), MenuAction::VolSfxDown, MenuAction::VolSfxUp, font));

        root.spawn(section_header(translate("settings.controls", lang), font));
        root.spawn(dim_text(translate("settings.key_move", lang), font));
        root.spawn(dim_text(translate("settings.key_pause", lang), font));

        root.spawn(section_header(translate("settings.camera", lang), font));
        root.spawn(setting_toggle(translate("settings.camera_mode", lang), cam, MenuAction::CycleCameraMode, font));

        root.spawn(section_header(translate("settings.language", lang), font));
        root.spawn(setting_toggle("Language", s.language.as_str(), MenuAction::ToggleLang, font));

        root.spawn(section_header(translate("settings.accessibility", lang), font));
        root.spawn(setting_toggle(translate("settings.colorblind", lang), cb, MenuAction::ToggleColorblind, font));
        root.spawn(setting_toggle(translate("settings.high_contrast", lang), hc, MenuAction::ToggleHighContrast, font));
        root.spawn(setting_toggle(translate("settings.font_size", lang), fs, MenuAction::CycleFontSize, font));

        root.spawn((Node { height: Val::Px(8.0), ..default() },)); // spacer
        root.spawn(secondary_btn(translate("menu.back", lang), MenuAction::Back, font));
    });
}

// ============================================================
//  Menu spawn system
// ============================================================

pub fn setup_menu(
    mut commands: Commands,
    menu_spawned: Option<ResMut<MenuSpawned>>,
    menu_page: Option<Res<MenuPage>>,
    game_state: Option<Res<GameState>>,
    progress: Option<Res<ProgressData>>,
    settings: Option<Res<SettingsData>>,
    locale: Option<Res<Locale>>,
    fonts: Option<Res<FontAssets>>,
    pack_state: Option<Res<crate::game::LevelPackState>>,
    mut focus: Option<ResMut<LevelGridFocus>>,
    existing: Query<Entity, With<MenuRoot>>,
) {
    let Some(mut spawned) = menu_spawned else { return };
    if spawned.0 { return; }

    for entity in &existing {
        commands.entity(entity).despawn();
    }
    spawned.0 = true;

    let lang = locale.as_ref().map(|l| l.lang.as_str()).unwrap_or("en");
    let current = menu_page.as_ref().map_or(MenuPageType::Main, |mp| mp.0);
    let font = fonts.as_ref().map(|f| f.default_font.clone()).unwrap_or_default();

    match current {
        MenuPageType::Main => {
            let has_save = progress.as_ref().map_or(false, |p| !p.completed.is_empty())
                || crate::save::MidLevelSave::exists();
            build_main_menu(&mut commands, lang, has_save, &font);
        }
        MenuPageType::ModeSelect => build_mode_select(&mut commands, lang, &font),
        MenuPageType::ClassicLevels => {
            let paths = game_state.as_ref().map(|gs| gs.level_paths.clone()).unwrap_or_default();
            let completed = progress.as_ref().map(|p| p.completed.clone()).unwrap_or_default();
            let stars = progress.as_ref().map(|p| p.stars.clone()).unwrap_or_default();
            let pack = pack_state.as_ref().and_then(|ps| ps.pack.clone());
            if let Some(ref mut f) = focus {
                build_classic_levels(&mut commands, lang, &paths, &completed, &stars, &font, pack.as_ref(), f);
            }
        }
        MenuPageType::Settings => {
            let s = settings.as_deref().cloned().unwrap_or_else(SettingsData::load_or_default);
            build_settings(&mut commands, lang, &s, &font);
        }
    }
}

// ============================================================
//  Button interaction — key fix: no Changed<Interaction> filter
// ============================================================

pub fn menu_button_interaction(
    mut query: Query<(&Interaction, Option<&MenuButton>, Option<&LevelButton>, &mut BackgroundColor)>,
    mut next_state: ResMut<NextState<AppState>>,
    mut menu_page: ResMut<MenuPage>,
    mut menu_spawned: ResMut<MenuSpawned>,
    mut game_state: Option<ResMut<GameState>>,
    mut dungeon_manager: Option<ResMut<DungeonManager>>,
    mut multi_floor: Option<ResMut<MultiFloorRun>>,
    progress: Option<Res<ProgressData>>,
    mut settings: Option<ResMut<SettingsData>>,
    mut locale: Option<ResMut<Locale>>,
    mut volume: Option<ResMut<crate::audio::GameVolume>>,
    keyboard: Res<ButtonInput<KeyCode>>,
    mut focus: Option<ResMut<LevelGridFocus>>,
) {
    if keyboard.just_pressed(KeyCode::Escape) {
        if menu_page.0 != MenuPageType::Main {
            menu_page.0 = MenuPageType::Main;
            menu_spawned.0 = false;
            return;
        }
    }

    if menu_page.0 == MenuPageType::ClassicLevels {
        if let Some(ref mut f) = focus {
            let total = game_state.as_ref().map(|gs| gs.level_paths.len()).unwrap_or(0);
            if total == 0 {
                // no-op
            } else if keyboard.just_pressed(KeyCode::ArrowLeft) {
                if f.index > 0 {
                    f.index -= 1;
                    menu_spawned.0 = false;
                }
            } else if keyboard.just_pressed(KeyCode::ArrowRight) {
                if f.index + 1 < total {
                    f.index += 1;
                    menu_spawned.0 = false;
                }
            } else if keyboard.just_pressed(KeyCode::ArrowUp) {
                if f.index >= f.columns {
                    f.index -= f.columns;
                    menu_spawned.0 = false;
                }
            } else if keyboard.just_pressed(KeyCode::ArrowDown) {
                let next = f.index + f.columns;
                if next < total {
                    f.index = next;
                    menu_spawned.0 = false;
                }
            } else if keyboard.just_pressed(KeyCode::Enter) || keyboard.just_pressed(KeyCode::Space) {
                if let Some(gs) = game_state.as_deref_mut() {
                    gs.current_level_index = f.index;
                    gs.is_daily = false;
                    if let Some(dm) = dungeon_manager.as_deref_mut() { dm.active = false; }
                    if let Some(mf) = multi_floor.as_deref_mut() { mf.active = false; }
                    menu_page.0 = MenuPageType::Main;
                    menu_spawned.0 = false;
                    next_state.set(AppState::Loading);
                    return;
                }
            }
        }
    }

    for (interaction, menu_btn, level_btn, mut bg) in &mut query {
        match *interaction {
            Interaction::Pressed => {
                if let Some(action) = menu_btn {
                    match action.0 {
                        MenuAction::Start => { menu_page.0 = MenuPageType::ModeSelect; menu_spawned.0 = false; }
                        MenuAction::Continue => {
                            if let Some(gs) = game_state.as_deref_mut() {
                                if let Some(save_data) = crate::save::MidLevelSave::load() {
                                    gs.current_level_index = save_data.current_level_index;
                                    gs.level_paths = save_data.level_paths;
                                    gs.is_daily = save_data.is_daily;
                                    gs.daily_seed = save_data.daily_seed;
                                    gs.is_dungeon_mode = save_data.is_dungeon_mode;
                                    gs.is_multifloor = save_data.is_multifloor;
                                    gs.current_floor = save_data.current_floor;
                                    gs.floor_count = save_data.floor_count;
                                    gs.scene_theme = save_data.scene_theme;
                                    gs.level_name = save_data.level_name;
                                    gs.par_steps = save_data.par_steps;
                                    gs.dungeon_current = save_data.dungeon_current;
                                    gs.dungeon_total = save_data.dungeon_total;
                                    gs.dungeon_room_name = save_data.dungeon_room_name;
                                    gs.grid.restore(&save_data.grid_snapshot);
                                    gs.initial_snapshot = save_data.initial_snapshot;
                                    gs.history = save_data.history;
                                    gs.replay = save_data.replay;
                                    if let Some(dm) = dungeon_manager.as_deref_mut() {
                                        dm.active = save_data.is_dungeon_mode;
                                    }
                                    if let Some(mf) = multi_floor.as_deref_mut() {
                                        mf.active = save_data.is_multifloor;
                                    }
                                    gs.is_resuming = true;
                                } else {
                                    let last = progress.as_ref().and_then(|p| p.completed.iter().max().copied()).unwrap_or(0);
                                    gs.current_level_index = last;
                                    gs.is_daily = false;
                                    if let Some(dm) = dungeon_manager.as_deref_mut() { dm.active = false; }
                                    if let Some(mf) = multi_floor.as_deref_mut() { mf.active = false; }
                                }
                                menu_page.0 = MenuPageType::Main; menu_spawned.0 = false;
                                next_state.set(AppState::Loading);
                            }
                        }
                        MenuAction::Settings => { menu_page.0 = MenuPageType::Settings; menu_spawned.0 = false; }
                        MenuAction::Exit => std::process::exit(0),
                        MenuAction::Back => {
                            if menu_page.0 != MenuPageType::Main {
                                menu_page.0 = MenuPageType::Main; menu_spawned.0 = false;
                            }
                        }
                        MenuAction::Classic => { menu_page.0 = MenuPageType::ClassicLevels; menu_spawned.0 = false; }
                        MenuAction::Dungeon => {
                            if let (Some(dm), Some(mf), Some(gs)) = (dungeon_manager.as_deref_mut(), multi_floor.as_deref_mut(), game_state.as_deref_mut()) {
                                dm.load_demo(); gs.is_daily = false; mf.active = false;
                                menu_page.0 = MenuPageType::Main; menu_spawned.0 = false;
                                next_state.set(AppState::Loading);
                            }
                        }
                        MenuAction::Daily => {
                            if let (Some(dm), Some(mf), Some(gs)) = (dungeon_manager.as_deref_mut(), multi_floor.as_deref_mut(), game_state.as_deref_mut()) {
                                gs.is_daily = true; gs.daily_seed = today_daily_seed();
                                dm.active = false; mf.active = false;
                                menu_page.0 = MenuPageType::Main; menu_spawned.0 = false;
                                next_state.set(AppState::Loading);
                            }
                        }
                        MenuAction::MultiFloor => {
                            if let (Some(dm), Some(mf), Some(gs)) = (dungeon_manager.as_deref_mut(), multi_floor.as_deref_mut(), game_state.as_deref_mut()) {
                                mf.load_demo(); dm.active = false; gs.is_daily = false;
                                menu_page.0 = MenuPageType::Main; menu_spawned.0 = false;
                                next_state.set(AppState::Loading);
                            }
                        }
                        MenuAction::ToggleLang => {
                            if let (Some(loc), Some(s)) = (locale.as_deref_mut(), settings.as_deref_mut()) {
                                loc.lang = if loc.lang == "zh" { "en".into() } else { "zh".into() };
                                s.language = loc.lang.clone();
                                s.save();
                                menu_spawned.0 = false;
                            }
                        }
                        MenuAction::ToggleColorblind => {
                            if let Some(s) = settings.as_deref_mut() { s.colorblind_mode = !s.colorblind_mode; s.save(); }
                            menu_spawned.0 = false;
                        }
                        MenuAction::ToggleHighContrast => {
                            if let Some(s) = settings.as_deref_mut() { s.high_contrast = !s.high_contrast; s.save(); }
                            menu_spawned.0 = false;
                        }
                        MenuAction::CycleCameraMode => {
                            if let Some(s) = settings.as_deref_mut() {
                                use crate::save::CameraMode;
                                s.camera_mode = match s.camera_mode {
                                    CameraMode::FreeOrbit => CameraMode::Fixed,
                                    CameraMode::Fixed => CameraMode::FollowPlayer,
                                    CameraMode::FollowPlayer => CameraMode::FocusLayer,
                                    CameraMode::FocusLayer => CameraMode::FreeOrbit,
                                };
                                s.save();
                            }
                            menu_spawned.0 = false;
                        }
                        MenuAction::CycleFontSize => {
                            if let Some(s) = settings.as_deref_mut() {
                                s.font_size = if s.font_size <= 13.0 { 16.0 } else if s.font_size >= 20.0 { 12.0 } else { 22.0 };
                                s.save();
                            }
                            menu_spawned.0 = false;
                        }
                        MenuAction::VolMusicUp => { if let Some(s) = settings.as_deref_mut() { s.music_volume = (s.music_volume + 0.1).min(1.0); } menu_spawned.0 = false; }
                        MenuAction::VolMusicDown => { if let Some(s) = settings.as_deref_mut() { s.music_volume = (s.music_volume - 0.1).max(0.0); } menu_spawned.0 = false; }
                        MenuAction::VolSfxUp => {
                            if let (Some(s), Some(v)) = (settings.as_deref_mut(), volume.as_deref_mut()) { s.sound_volume = (s.sound_volume + 0.1).min(1.0); v.0 = s.sound_volume; }
                            menu_spawned.0 = false;
                        }
                        MenuAction::VolSfxDown => {
                            if let (Some(s), Some(v)) = (settings.as_deref_mut(), volume.as_deref_mut()) { s.sound_volume = (s.sound_volume - 0.1).max(0.0); v.0 = s.sound_volume; }
                            menu_spawned.0 = false;
                        }
                    }
                } else if let Some(button) = level_btn {
                    if let Some(gs) = game_state.as_deref_mut() {
                        if button.0 < gs.level_paths.len() {
                            gs.current_level_index = button.0;
                            if let Some(dm) = dungeon_manager.as_deref_mut() { dm.active = false; }
                            gs.is_daily = false;
                            if let Some(mf) = multi_floor.as_deref_mut() { mf.active = false; }
                            next_state.set(AppState::Loading);
                            return;
                        }
                    }
                }
            }
            Interaction::Hovered => { bg.0 = BTN_HOVER; }
            Interaction::None => {
                let base = if level_btn.is_some() {
                    let completed = progress.as_ref().map_or(false, |p| level_btn.map_or(false, |lb| p.completed.contains(&lb.0)));
                    if completed { Color::srgb(0.10, 0.22, 0.13) } else { BTN_DEFAULT }
                } else { BTN_DEFAULT };
                bg.0 = base;
            }
        }
    }
}

// ============================================================
//  Cleanup
// ============================================================

pub fn despawn_menu(
    mut commands: Commands, query: Query<Entity, With<MenuRoot>>,
    mut menu_spawned: ResMut<MenuSpawned>,
) {
    for entity in &query { commands.entity(entity).despawn(); }
    menu_spawned.0 = false;
}
