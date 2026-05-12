use bevy::prelude::*;
use bevy::core_pipeline::tonemapping::Tonemapping;
use bevy::window::PrimaryWindow;

use crate::game::GameState;
use crate::multifloor::MultiFloorRun;
use crate::save::CameraMode;

#[derive(Component)]
pub struct GameCamera;

#[derive(Resource)]
pub struct CameraConfig {
    pub center: Vec3,
    pub distance: f32,
    pub min_distance: f32,
    pub max_distance: f32,
    pub azimuth: f32,
    pub elevation: f32,
    pub min_elevation: f32,
    pub max_elevation: f32,
    pub smooth_speed: f32,
    pub mode: CameraMode,
}

impl Default for CameraConfig {
    fn default() -> Self {
        Self {
            center: Vec3::new(6.0, 0.0, 6.0),
            distance: 18.0,
            min_distance: 5.0,
            max_distance: 40.0,
            azimuth: 0.5,
            elevation: 1.0,
            min_elevation: 0.15,
            max_elevation: 1.5,
            smooth_speed: 5.0,
            mode: CameraMode::FreeOrbit,
        }
    }
}

/// 创建摄像机，根据地图尺寸计算初始距离
pub fn setup_camera(commands: &mut Commands, width: u32, height: u32, cell_size: f32) {
    let cx = width as f32 * cell_size / 2.0;
    let cz = height as f32 * cell_size / 2.0;
    let diag = ((width.pow(2) + height.pow(2)) as f32).sqrt();
    let distance = (diag * cell_size * 0.85 + 3.0).clamp(5.0, 40.0);

    let config = CameraConfig {
        center: Vec3::new(cx, 0.0, cz),
        distance,
        ..default()
    };

    let pos = camera_position(&config);
    commands.spawn((
        Camera3d::default(),
        Tonemapping::AgX,
        Transform::from_translation(pos).looking_at(config.center, Vec3::Y),
        GameCamera,
    ));
    commands.insert_resource(config);
}

/// 自动更新中心点（多层模式楼层切换时高度变化）
pub fn auto_center_camera(
    game_state: Option<Res<GameState>>,
    multi_floor: Option<Res<MultiFloorRun>>,
    mut cam_config: ResMut<CameraConfig>,
    keyboard: Res<ButtonInput<KeyCode>>,
) {
    let Some(ref gs) = game_state else {
        return;
    };

    // Tab 键切换摄像机模式
    if keyboard.just_pressed(KeyCode::Tab) {
        cam_config.mode = match cam_config.mode {
            CameraMode::FreeOrbit => CameraMode::FollowPlayer,
            CameraMode::FollowPlayer => CameraMode::Fixed,
            CameraMode::Fixed => CameraMode::FocusLayer,
            CameraMode::FocusLayer => CameraMode::FreeOrbit,
        };
    }

    let cx = gs.grid.width as f32 * gs.cell_size / 2.0;
    let cz = gs.grid.height as f32 * gs.cell_size / 2.0;

    let floor_height = multi_floor
        .as_ref()
        .filter(|mf| mf.active)
        .map(|mf| mf.current_elevation())
        .unwrap_or(0.0);

    match cam_config.mode {
        CameraMode::FollowPlayer => {
            let player_world = gs.grid.player_pos.to_world(gs.cell_size, floor_height);
            cam_config.center.x += (player_world[0] - cam_config.center.x) * 0.1;
            cam_config.center.z += (player_world[2] - cam_config.center.z) * 0.1;
            cam_config.center.y += (player_world[1] - cam_config.center.y) * 0.1;
            cam_config.distance = 12.0;
            cam_config.elevation = 0.9;
        }
        CameraMode::Fixed => {
            // 固定俯视（小地图视角）
            cam_config.center = Vec3::new(cx, floor_height, cz);
            cam_config.azimuth = 0.0;
            cam_config.elevation = 1.5;
            cam_config.distance = (gs.grid.width.max(gs.grid.height) as f32 * gs.cell_size * 0.6)
                .clamp(cam_config.min_distance, cam_config.max_distance);
        }
        CameraMode::FocusLayer => {
            // 聚焦当前层
            cam_config.center = Vec3::new(cx, floor_height, cz);
        }
        CameraMode::FreeOrbit => {
            cam_config.center = Vec3::new(cx, floor_height, cz);
        }
    }
}

/// 处理摄像机输入：右键拖拽旋转、Q/E 旋转、+/- 缩放、F 重置
pub fn camera_input(
    keyboard: Res<ButtonInput<KeyCode>>,
    mouse_button: Res<ButtonInput<MouseButton>>,
    windows: Query<&Window, With<PrimaryWindow>>,
    mut cam_config: ResMut<CameraConfig>,
    mut last_cursor: Local<Option<Vec2>>,
) {
    let Ok(window) = windows.single() else {
        *last_cursor = None;
        return;
    };
    let cursor_pos = window.cursor_position();

    // 右键拖拽旋转
    if mouse_button.pressed(MouseButton::Right) {
        if let (Some(current), Some(last)) = (cursor_pos, *last_cursor) {
            let delta = current - last;
            let sens = 0.005;
            cam_config.azimuth += delta.x * sens;
            cam_config.elevation = (cam_config.elevation - delta.y * sens)
                .clamp(cam_config.min_elevation, cam_config.max_elevation);
        }
        *last_cursor = cursor_pos;
    } else {
        *last_cursor = None;
    }

    // Q/E 键盘旋转
    if keyboard.pressed(KeyCode::KeyQ) {
        cam_config.azimuth -= 0.03;
    }
    if keyboard.pressed(KeyCode::KeyE) {
        cam_config.azimuth += 0.03;
    }

    // +/- 缩放
    if keyboard.pressed(KeyCode::Equal) || keyboard.pressed(KeyCode::NumpadAdd) {
        cam_config.distance = (cam_config.distance - 0.5)
            .clamp(cam_config.min_distance, cam_config.max_distance);
    }
    if keyboard.pressed(KeyCode::Minus) || keyboard.pressed(KeyCode::NumpadSubtract) {
        cam_config.distance = (cam_config.distance + 0.5)
            .clamp(cam_config.min_distance, cam_config.max_distance);
    }

    // F 重置视角
    if keyboard.just_pressed(KeyCode::KeyF) {
        cam_config.azimuth = 0.5;
        cam_config.elevation = 1.0;
    }
}

/// 平滑更新摄像机位置到轨道坐标
pub fn update_camera(
    cam_config: Res<CameraConfig>,
    time: Res<Time>,
    mut query: Query<&mut Transform, With<GameCamera>>,
) {
    let Ok(mut transform) = query.single_mut() else {
        return;
    };

    let target = camera_position(&cam_config);
    let t = 1.0 - (-cam_config.smooth_speed * time.delta_secs()).exp();
    transform.translation = transform.translation.lerp(target, t);
    transform.look_at(cam_config.center, Vec3::Y);
}

/// 根据球坐标计算摄像机世界位置
fn camera_position(config: &CameraConfig) -> Vec3 {
    let h_dist = config.distance * config.elevation.cos();
    let height = config.distance * config.elevation.sin();
    Vec3::new(
        config.center.x + h_dist * config.azimuth.sin(),
        config.center.y + height,
        config.center.z + h_dist * config.azimuth.cos(),
    )
}
