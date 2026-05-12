use bevy::prelude::*;
use std::collections::HashSet;

use crate::game::GameState;
use crate::stats::{GameStats, SessionStats};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum Achievement {
    FirstClear,
    AllClear,
    SpeedRunner,
    NoUndo,
    NoHint,
    ThreeStars,
    DungeonClear,
    DailyComplete,
    IceSlideMaster,
    KeyCollector,
}

impl Achievement {
    pub fn id(&self) -> &'static str {
        match self {
            Achievement::FirstClear => "first_clear",
            Achievement::AllClear => "all_clear",
            Achievement::SpeedRunner => "speed_runner",
            Achievement::NoUndo => "no_undo",
            Achievement::NoHint => "no_hint",
            Achievement::ThreeStars => "three_stars",
            Achievement::DungeonClear => "dungeon_clear",
            Achievement::DailyComplete => "daily_complete",
            Achievement::IceSlideMaster => "ice_slide_master",
            Achievement::KeyCollector => "key_collector",
        }
    }

    pub fn name(&self) -> &'static str {
        match self {
            Achievement::FirstClear => "First Steps",
            Achievement::AllClear => "Completionist",
            Achievement::SpeedRunner => "Speed Runner",
            Achievement::NoUndo => "Perfect Memory",
            Achievement::NoHint => "Self-Reliant",
            Achievement::ThreeStars => "Star Collector",
            Achievement::DungeonClear => "Dungeon Explorer",
            Achievement::DailyComplete => "Daily Devotee",
            Achievement::IceSlideMaster => "Ice Skater",
            Achievement::KeyCollector => "Keymaster",
        }
    }

    pub fn name_zh(&self) -> &'static str {
        match self {
            Achievement::FirstClear => "初次通关",
            Achievement::AllClear => "全部通关",
            Achievement::SpeedRunner => "速通达人",
            Achievement::NoUndo => "过目不忘",
            Achievement::NoHint => "自力更生",
            Achievement::ThreeStars => "三星收藏家",
            Achievement::DungeonClear => "地牢探索者",
            Achievement::DailyComplete => "每日坚持",
            Achievement::IceSlideMaster => "冰上滑手",
            Achievement::KeyCollector => "钥匙大师",
        }
    }

    pub fn description(&self) -> &'static str {
        match self {
            Achievement::FirstClear => "Complete your first level",
            Achievement::AllClear => "Complete all 10 levels",
            Achievement::SpeedRunner => "Complete a level under par steps",
            Achievement::NoUndo => "Complete a level without using undo",
            Achievement::NoHint => "Complete a level without using hints",
            Achievement::ThreeStars => "Earn 3 stars on any level",
            Achievement::DungeonClear => "Complete the dungeon",
            Achievement::DailyComplete => "Complete a daily challenge",
            Achievement::IceSlideMaster => "Push a box across 3+ ice tiles",
            Achievement::KeyCollector => "Collect 10 keys total",
        }
    }

    pub fn description_zh(&self) -> &'static str {
        match self {
            Achievement::FirstClear => "通关第一个关卡",
            Achievement::AllClear => "通关全部 10 个关卡",
            Achievement::SpeedRunner => "在标准步数内通关",
            Achievement::NoUndo => "不使用撤销通关",
            Achievement::NoHint => "不使用提示通关",
            Achievement::ThreeStars => "在任意关卡获得三星",
            Achievement::DungeonClear => "通关地牢模式",
            Achievement::DailyComplete => "完成一次每日挑战",
            Achievement::IceSlideMaster => "将箱子推过 3 格以上冰面",
            Achievement::KeyCollector => "累计拾取 10 把钥匙",
        }
    }

    #[allow(dead_code)]
    pub fn all() -> &'static [Achievement] {
        &[
            Achievement::FirstClear,
            Achievement::AllClear,
            Achievement::SpeedRunner,
            Achievement::NoUndo,
            Achievement::NoHint,
            Achievement::ThreeStars,
            Achievement::DungeonClear,
            Achievement::DailyComplete,
            Achievement::IceSlideMaster,
            Achievement::KeyCollector,
        ]
    }
}

#[derive(Resource)]
pub struct AchievementState {
    pub unlocked: HashSet<String>,
    pub pending_notification: Vec<Achievement>,
}

impl Default for AchievementState {
    fn default() -> Self {
        Self {
            unlocked: HashSet::new(),
            pending_notification: Vec::new(),
        }
    }
}

impl AchievementState {
    pub fn try_unlock(&mut self, achievement: Achievement) {
        if !self.unlocked.contains(achievement.id()) {
            self.unlocked.insert(achievement.id().to_string());
            self.pending_notification.push(achievement);
        }
    }

    #[allow(dead_code)]
    pub fn is_unlocked(&self, achievement: &Achievement) -> bool {
        self.unlocked.contains(achievement.id())
    }
}

pub fn check_achievements(
    mut ach_state: ResMut<AchievementState>,
    game_state: Res<GameState>,
    stats: Res<GameStats>,
    session: Res<SessionStats>,
) {
    if !game_state.level_complete {
        return;
    }

    ach_state.try_unlock(Achievement::FirstClear);

    if stats.levels_completed >= 10 {
        ach_state.try_unlock(Achievement::AllClear);
    }

    if let Some(par) = game_state.par_steps {
        if par > 0 && game_state.grid.current_step <= par {
            ach_state.try_unlock(Achievement::SpeedRunner);
        }
    }

    if session.undos_this_session == 0 {
        ach_state.try_unlock(Achievement::NoUndo);
    }

    if session.hints_this_session == 0 {
        ach_state.try_unlock(Achievement::NoHint);
    }

    if game_state.stars_earned >= 3 {
        ach_state.try_unlock(Achievement::ThreeStars);
    }

    if game_state.is_dungeon_mode {
        ach_state.try_unlock(Achievement::DungeonClear);
    }

    if game_state.is_daily {
        ach_state.try_unlock(Achievement::DailyComplete);
    }

    if stats.keys_collected >= 10 {
        ach_state.try_unlock(Achievement::KeyCollector);
    }

    if game_state.max_slide_streak >= 3 {
        ach_state.try_unlock(Achievement::IceSlideMaster);
    }
}

#[derive(Component)]
pub struct AchievementPopup {
    pub timer: f32,
}

pub fn show_achievement_popups(
    mut commands: Commands,
    mut ach_state: ResMut<AchievementState>,
    locale: Option<Res<crate::locale::Locale>>,
    existing: Query<Entity, With<AchievementPopup>>,
) {
    if ach_state.pending_notification.is_empty() {
        return;
    }

    if !existing.is_empty() {
        return;
    }

    let achievement = ach_state.pending_notification.remove(0);
    let lang = locale.as_ref().map(|l| l.lang.as_str()).unwrap_or("en");
    let title = if lang == "zh" {
        achievement.name_zh()
    } else {
        achievement.name()
    };
    let desc = if lang == "zh" {
        achievement.description_zh()
    } else {
        achievement.description()
    };

    commands
        .spawn((
            Node {
                position_type: PositionType::Absolute,
                top: Val::Px(80.0),
                right: Val::Px(20.0),
                width: Val::Px(280.0),
                padding: UiRect::all(Val::Px(16.0)),
                flex_direction: FlexDirection::Column,
                row_gap: Val::Px(4.0),
                ..default()
            },
            BackgroundColor(Color::srgba(0.15, 0.15, 0.2, 0.92)),
            AchievementPopup { timer: 3.5 },
        ))
        .with_children(|root| {
            root.spawn((
                Text::new(format!("\u{2605} {}", title)),
                TextFont {
                    font_size: 20.0,
                    ..default()
                },
                TextColor(Color::srgb(0.9, 0.85, 0.3)),
            ));
            root.spawn((
                Text::new(desc),
                TextFont {
                    font_size: 14.0,
                    ..default()
                },
                TextColor(Color::srgba(1.0, 1.0, 1.0, 0.7)),
            ));
        });
}

pub fn update_achievement_popups(
    mut commands: Commands,
    time: Res<Time>,
    mut query: Query<(Entity, &mut AchievementPopup, &mut BackgroundColor)>,
) {
    for (entity, mut popup, mut bg) in &mut query {
        popup.timer -= time.delta_secs();

        if popup.timer < 0.5 {
            let alpha = (popup.timer / 0.5).max(0.0);
            bg.0 = Color::srgba(0.15, 0.15, 0.2, alpha * 0.92);
        }

        if popup.timer <= 0.0 {
            commands.entity(entity).despawn();
        }
    }
}
