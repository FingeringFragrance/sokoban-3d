use bevy::prelude::*;
use crate::grid::{CellKind, GridData, find_player, is_target, check_win};

#[derive(Resource, Default)]
pub struct PlaytestState {
    pub active: bool,
    pub steps: u32,
    pub won: bool,
    pub has_key: bool,
    pub targets: Vec<(i32, i32)>,
}

pub fn toggle_playtest(
    keyboard: Res<ButtonInput<KeyCode>>,
    mut state: ResMut<PlaytestState>,
    grid: Res<GridData>,
) {
    if !keyboard.just_pressed(KeyCode::Tab) { return; }
    state.active = !state.active;
    if state.active {
        state.steps = 0;
        state.won = false;
        state.has_key = false;
        state.targets.clear();
        for x in 0..grid.width as i32 {
            for z in 0..grid.height as i32 {
                if grid.get(x, z) == CellKind::Target {
                    state.targets.push((x, z));
                }
            }
        }
    }
}

pub fn playtest_move(
    keyboard: Res<ButtonInput<KeyCode>>,
    mut grid: ResMut<GridData>,
    mut state: ResMut<PlaytestState>,
) {
    if !state.active || state.won { return; }

    let dir: Option<(i32, i32)> = if keyboard.just_pressed(KeyCode::KeyW) { Some((0, -1)) }
        else if keyboard.just_pressed(KeyCode::KeyS) { Some((0, 1)) }
        else if keyboard.just_pressed(KeyCode::KeyA) { Some((-1, 0)) }
        else if keyboard.just_pressed(KeyCode::KeyD) { Some((1, 0)) }
        else { None };

    let Some((dx, dz)) = dir else { return };

    let (px, pz) = find_player(&grid.cells);
    let nx = px + dx;
    let nz = pz + dz;

    let target_cell = grid.get(nx, nz);

    if target_cell == CellKind::Wall || target_cell == CellKind::Decoration { return; }

    if target_cell == CellKind::Gate {
        if !state.has_key { return; }
        grid.set(px, pz, if is_target(px, pz, &state.targets) { CellKind::Target } else { CellKind::Empty });
        grid.set(nx, nz, CellKind::Player);
        state.steps += 1;
        return;
    }

    if target_cell == CellKind::Key {
        state.has_key = true;
        grid.set(px, pz, if is_target(px, pz, &state.targets) { CellKind::Target } else { CellKind::Empty });
        grid.set(nx, nz, CellKind::Player);
        state.steps += 1;
        return;
    }

    if target_cell == CellKind::Box {
        let bx = nx + dx;
        let bz = nz + dz;
        let beyond = grid.get(bx, bz);
        if beyond != CellKind::Empty && beyond != CellKind::Target {
            return;
        }
        grid.set(nx, nz, if is_target(nx, nz, &state.targets) { CellKind::Target } else { CellKind::Empty });
        grid.set(bx, bz, CellKind::Box);
    } else if target_cell != CellKind::Empty && target_cell != CellKind::Target {
        return;
    }

    grid.set(px, pz, if is_target(px, pz, &state.targets) { CellKind::Target } else { CellKind::Empty });
    grid.set(nx, nz, CellKind::Player);
    state.steps += 1;

    if check_win(&grid.cells, &state.targets) {
        state.won = true;
    }
}

#[derive(Component)]
pub struct PlaytestHud;

pub fn playtest_hud(
    mut commands: Commands,
    state: Res<PlaytestState>,
    font: Res<crate::ui::UiFont>,
    existing: Query<Entity, With<PlaytestHud>>,
    mut last: Local<(u32, bool, bool)>,
) {
    if !state.active {
        if !existing.is_empty() {
            for e in &existing { commands.entity(e).despawn(); }
            *last = (0, false, false);
        }
        return;
    }

    let cur = (state.steps, state.won, state.has_key);
    if *last == cur && !existing.is_empty() {
        return;
    }
    *last = cur;

    for e in &existing { commands.entity(e).despawn(); }

    commands.spawn((
        Node {
            position_type: PositionType::Absolute,
            top: Val::Px(10.0),
            left: Val::Px(0.0),
            right: Val::Px(0.0),
            justify_content: JustifyContent::Center,
            ..default()
        },
        PlaytestHud,
    ))
    .with_children(|parent| {
        parent.spawn((
            Text::new("试玩模式 - WASD移动 | Tab返回编辑"),
            TextFont { font: font.0.clone(), font_size: 16.0, ..default() },
            TextColor(Color::srgb(0.9, 0.9, 0.3)),
        ));
    });

    commands.spawn((
        Node {
            position_type: PositionType::Absolute,
            top: Val::Px(36.0),
            right: Val::Px(12.0),
            flex_direction: FlexDirection::Column,
            align_items: AlignItems::FlexEnd,
            row_gap: Val::Px(4.0),
            ..default()
        },
        PlaytestHud,
    ))
    .with_children(|parent| {
        parent.spawn((
            Text::new(format!("步数: {}", state.steps)),
            TextFont { font: font.0.clone(), font_size: 18.0, ..default() },
            TextColor(Color::srgb(0.9, 0.9, 0.9)),
        ));
        if state.has_key {
            parent.spawn((
                Text::new("已获得钥匙"),
                TextFont { font: font.0.clone(), font_size: 14.0, ..default() },
                TextColor(Color::srgb(0.9, 0.2, 0.2)),
            ));
        }
    });

    if state.won {
        commands.spawn((
            Node {
                position_type: PositionType::Absolute,
                top: Val::Px(0.0),
                bottom: Val::Px(0.0),
                left: Val::Px(0.0),
                right: Val::Px(0.0),
                justify_content: JustifyContent::Center,
                align_items: AlignItems::Center,
                ..default()
            },
            PlaytestHud,
        ))
        .with_children(|parent| {
            parent.spawn((
                Text::new(format!("通关! 共 {} 步", state.steps)),
                TextFont { font: font.0.clone(), font_size: 36.0, ..default() },
                TextColor(Color::srgb(0.3, 1.0, 0.3)),
            ));
        });
    }
}