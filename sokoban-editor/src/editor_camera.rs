use bevy::prelude::*;
use bevy::core_pipeline::tonemapping::Tonemapping;
use bevy::input::mouse::MouseWheel;
use bevy::window::PrimaryWindow;
use crate::grid::CELL;

#[derive(Component)]
pub struct EditorCam;

#[derive(Resource)]
pub struct EditorCamera {
    pub target: Vec3,
    pub distance: f32,
    pub yaw: f32,
    pub pitch: f32,
}

impl Default for EditorCamera {
    fn default() -> Self {
        Self {
            target: Vec3::new(12.0, 0.0, 12.0),
            distance: 28.0,
            yaw: std::f32::consts::FRAC_PI_4,
            pitch: 0.9,
        }
    }
}

pub fn camera_eye(cam: &EditorCamera) -> Vec3 {
    cam.target + Vec3::new(
        cam.yaw.sin() * cam.pitch.cos() * cam.distance,
        cam.pitch.sin() * cam.distance,
        cam.yaw.cos() * cam.pitch.cos() * cam.distance,
    )
}

pub fn spawn_camera(mut commands: Commands) {
    let cam = EditorCamera::default();
    let eye = camera_eye(&cam);
    commands.spawn((
        Camera::default(),
        Camera3d::default(),
        Tonemapping::AgX,
        Transform::from_translation(eye).looking_at(cam.target, Vec3::Y),
        EditorCam,
    ));
}

pub fn spawn_ui_camera(mut commands: Commands) {
    commands.spawn((Camera2d, Camera { order: 1, ..default() }));
}

pub fn control_camera(
    mut cam: ResMut<EditorCamera>,
    mouse: Res<ButtonInput<MouseButton>>,
    keyboard: Res<ButtonInput<KeyCode>>,
    playtest: Res<crate::playtest::PlaytestState>,
    mut scroll: MessageReader<MouseWheel>,
    windows: Query<&Window, With<PrimaryWindow>>,
    time: Res<Time>,
    mut q: Query<&mut Transform, With<EditorCam>>,
    mut last: Local<Option<Vec2>>,
) {
    let Ok(mut t) = q.single_mut() else { return };
    let Ok(w) = windows.single() else { return };
    let cur = w.cursor_position();
    let dt = time.delta_secs();

    for ev in scroll.read() {
        cam.distance = (cam.distance - ev.y * 3.0).clamp(5.0, 80.0);
    }

    if mouse.pressed(MouseButton::Right) {
        if let (Some(c), Some(l)) = (cur, *last) {
            let d = c - l;
            cam.yaw -= d.x * 0.005;
            cam.pitch = (cam.pitch + d.y * 0.005).clamp(0.1, 1.5);
        }
        *last = cur;
    } else {
        *last = None;
    }

    if !playtest.active {
        let spd = 12.0 * dt;
        let fwd = Vec3::new(cam.yaw.sin(), 0.0, cam.yaw.cos()).normalize();
        let right = Vec3::new(cam.yaw.cos(), 0.0, -cam.yaw.sin());
        if keyboard.pressed(KeyCode::KeyW) { cam.target -= fwd * spd; }
        if keyboard.pressed(KeyCode::KeyS) { cam.target += fwd * spd; }
        if keyboard.pressed(KeyCode::KeyA) { cam.target -= right * spd; }
        if keyboard.pressed(KeyCode::KeyD) { cam.target += right * spd; }
    }

    if keyboard.just_pressed(KeyCode::KeyF) {
        *cam = EditorCamera::default();
    }

    let eye = camera_eye(&cam);
    t.translation = eye;
    t.look_at(cam.target, Vec3::Y);
}

pub fn mouse_to_grid(cursor: Vec2, _w: &Window, cam: &Camera, cam_t: &GlobalTransform, grid_w: u32, grid_h: u32) -> Option<(i32, i32)> {
    let Ok(ray) = cam.viewport_to_world(cam_t, cursor) else { return None; };
    let t = -ray.origin.y / ray.direction.y;
    if t <= 0.0 { return None; }
    let hit = ray.origin + *ray.direction * t;
    let gx = (hit.x / CELL).floor() as i32;
    let gz = (hit.z / CELL).floor() as i32;
    if gx < 0 || gx >= grid_w as i32 || gz < 0 || gz >= grid_h as i32 { return None; }
    Some((gx, gz))
}