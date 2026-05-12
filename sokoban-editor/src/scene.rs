use bevy::prelude::*;
use bevy::window::PrimaryWindow;
use crate::grid::{CellKind, GridData, CELL};
use crate::editor_camera::{EditorCam, mouse_to_grid};

#[derive(Component)]
pub struct SceneObj;

#[derive(Component)]
pub struct DecorationObj;

#[derive(Component)]
pub struct HoverObj;

#[derive(Resource)]
pub struct SceneMaterials {
    pub wall: Handle<StandardMaterial>,
    pub player: Handle<StandardMaterial>,
    pub box_mat: Handle<StandardMaterial>,
    pub target: Handle<StandardMaterial>,
    pub key: Handle<StandardMaterial>,
    pub gate: Handle<StandardMaterial>,
    pub hover: Handle<StandardMaterial>,
}

impl SceneMaterials {
    pub fn new(mats: &mut Assets<StandardMaterial>) -> Self {
        Self {
            wall: mats.add(StandardMaterial {
                base_color: Color::srgb(0.35, 0.35, 0.42),
                perceptual_roughness: 0.8,
                ..default()
            }),
            player: mats.add(StandardMaterial {
                base_color: Color::srgb(0.3, 0.55, 0.85),
                perceptual_roughness: 0.6,
                ..default()
            }),
            box_mat: mats.add(StandardMaterial {
                base_color: Color::srgb(0.82, 0.52, 0.2),
                perceptual_roughness: 0.7,
                ..default()
            }),
            target: mats.add(StandardMaterial {
                base_color: Color::srgb(0.4, 0.78, 0.5),
                perceptual_roughness: 0.9,
                ..default()
            }),
            key: mats.add(StandardMaterial {
                base_color: Color::srgb(0.9, 0.2, 0.2),
                perceptual_roughness: 0.4,
                ..default()
            }),
            gate: mats.add(StandardMaterial {
                base_color: Color::srgb(0.85, 0.3, 0.3),
                perceptual_roughness: 0.4,
                ..default()
            }),
            hover: mats.add(StandardMaterial {
                base_color: Color::srgba(1.0, 1.0, 1.0, 0.15),
                unlit: true,
                ..default()
            }),
        }
    }
}

pub fn init_scene_materials(mut commands: Commands, mut mats: ResMut<Assets<StandardMaterial>>) {
    commands.insert_resource(SceneMaterials::new(&mut mats));
}

pub fn sync_world(
    mut commands: Commands,
    grid: Res<GridData>,
    existing: Query<Entity, With<SceneObj>>,
    mut meshes: ResMut<Assets<Mesh>>,
    mats: Res<SceneMaterials>,
    decorations: Res<crate::ui::DecorationAssets>,
    mut last_version: Local<u64>,
) {
    if grid.version == *last_version && !existing.is_empty() {
        return;
    }
    *last_version = grid.version;

    for e in &existing {
        commands.entity(e).despawn();
    }

    for x in 0..grid.width as i32 {
        for z in 0..grid.height as i32 {
            let wx = x as f32 * CELL;
            let wz = z as f32 * CELL;
            match grid.get(x, z) {
                CellKind::Wall => {
                    commands.spawn((
                        Mesh3d(meshes.add(Cuboid::new(CELL * 0.9, 3.0, CELL * 0.9))),
                        MeshMaterial3d(mats.wall.clone()),
                        Transform::from_xyz(wx, 1.5, wz),
                        SceneObj,
                    ));
                }
                CellKind::Player => {
                    commands.spawn((
                        Mesh3d(meshes.add(Cuboid::new(1.4, 1.8, 1.0))),
                        MeshMaterial3d(mats.player.clone()),
                        Transform::from_xyz(wx, 0.9, wz),
                        SceneObj,
                    ));
                }
                CellKind::Box => {
                    commands.spawn((
                        Mesh3d(meshes.add(Cuboid::new(1.5, 1.5, 1.5))),
                        MeshMaterial3d(mats.box_mat.clone()),
                        Transform::from_xyz(wx, 0.75, wz),
                        SceneObj,
                    ));
                }
                CellKind::Target => {
                    commands.spawn((
                        Mesh3d(meshes.add(Plane3d::default().mesh().size(CELL * 0.95, CELL * 0.95))),
                        MeshMaterial3d(mats.target.clone()),
                        Transform::from_xyz(wx, 0.005, wz),
                        SceneObj,
                    ));
                }
                CellKind::Key => {
                    commands.spawn((
                        Mesh3d(meshes.add(Cuboid::new(0.5, 0.5, 0.5))),
                        MeshMaterial3d(mats.key.clone()),
                        Transform::from_xyz(wx, 1.2, wz),
                        SceneObj,
                    ));
                }
                CellKind::Gate => {
                    commands.spawn((
                        Mesh3d(meshes.add(Cuboid::new(CELL * 0.9, 3.0, CELL * 0.3))),
                        MeshMaterial3d(mats.gate.clone()),
                        Transform::from_xyz(wx, 1.5, wz),
                        SceneObj,
                    ));
                }
                CellKind::Decoration => {
                    commands.spawn((
                        SceneRoot(decorations.scene.clone()),
                        Transform::from_xyz(wx, 0.0, wz),
                        SceneObj,
                        DecorationObj,
                    ));
                }
                CellKind::Empty => {}
            }
        }
    }
}

pub fn adjust_decoration_materials(
    decorations: Query<&Children, With<DecorationObj>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
    mesh_mats: Query<&MeshMaterial3d<StandardMaterial>>,
    children: Query<&Children>,
    mut processed: Local<Vec<Entity>>,
    mut logged: Local<bool>,
) {
    if decorations.is_empty() {
        processed.clear();
        return;
    }

    for deco_children in &decorations {
        for child in deco_children.iter() {
            if processed.contains(&child) {
                continue;
            }
            if let Ok(mat_handle) = mesh_mats.get(child) {
                if let Some(mat) = materials.get_mut(mat_handle) {
                    if !*logged {
                        let c = mat.base_color.to_srgba();
                        eprintln!("[DECO-MAT] name=? base_color=({:.3},{:.3},{:.3},{:.3}) metallic={:.2} roughness={:.2} reflectance={:.2}",
                            c.red, c.green, c.blue, c.alpha,
                            mat.metallic, mat.perceptual_roughness, mat.reflectance);
                    }
                    mat.perceptual_roughness = 0.3;
                    mat.metallic = 0.0;
                    mat.reflectance = 0.3;
                }
                processed.push(child);
            }
            if let Ok(grandchildren) = children.get(child) {
                for gc in grandchildren.iter() {
                    if processed.contains(&gc) {
                        continue;
                    }
                    if let Ok(mat_handle) = mesh_mats.get(gc) {
                        if let Some(mat) = materials.get_mut(mat_handle) {
                            if !*logged {
                                let c = mat.base_color.to_srgba();
                                eprintln!("[DECO-MAT] name=? base_color=({:.3},{:.3},{:.3},{:.3}) metallic={:.2} roughness={:.2} reflectance={:.2}",
                                    c.red, c.green, c.blue, c.alpha,
                                    mat.metallic, mat.perceptual_roughness, mat.reflectance);
                            }
                            mat.perceptual_roughness = 0.3;
                            mat.metallic = 0.0;
                            mat.reflectance = 0.3;
                        }
                        processed.push(gc);
                    }
                }
            }
        }
    }
    *logged = true;
}

#[derive(Component)]
pub struct GridLine;

pub fn sync_grid_lines(
    mut commands: Commands,
    grid: Res<GridData>,
    existing: Query<Entity, With<GridLine>>,
    mut meshes: ResMut<Assets<Mesh>>,
    mut mats: ResMut<Assets<StandardMaterial>>,
    mut last_dims: Local<(u32, u32)>,
) {
    let dims = (grid.width, grid.height);
    if *last_dims == dims && !existing.is_empty() {
        return;
    }
    *last_dims = dims;

    for e in &existing {
        commands.entity(e).despawn();
    }

    let w = grid.width as f32 * CELL;
    let h = grid.height as f32 * CELL;

    let line_mat = mats.add(StandardMaterial {
        base_color: Color::srgba(1.0, 1.0, 1.0, 0.08),
        unlit: true,
        ..default()
    });

    for x in 0..=grid.width {
        let wx = x as f32 * CELL;
        commands.spawn((
            Mesh3d(meshes.add(Cuboid::new(0.03, 0.005, h))),
            MeshMaterial3d(line_mat.clone()),
            Transform::from_xyz(wx, 0.003, h / 2.0),
            GridLine,
        ));
    }
    for z in 0..=grid.height {
        let wz = z as f32 * CELL;
        commands.spawn((
            Mesh3d(meshes.add(Cuboid::new(w, 0.005, 0.03))),
            MeshMaterial3d(line_mat.clone()),
            Transform::from_xyz(w / 2.0, 0.003, wz),
            GridLine,
        ));
    }
}

pub fn spawn_axes(
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut mats: ResMut<Assets<StandardMaterial>>,
) {
    let len = 4.0;
    let t = 0.08;

    commands.spawn((
        Mesh3d(meshes.add(Cuboid::new(len, t, t))),
        MeshMaterial3d(mats.add(StandardMaterial {
            base_color: Color::srgb(1.0, 0.15, 0.15),
            unlit: true,
            ..default()
        })),
        Transform::from_xyz(len / 2.0, t / 2.0, 0.0),
    ));

    commands.spawn((
        Mesh3d(meshes.add(Cuboid::new(t, len, t))),
        MeshMaterial3d(mats.add(StandardMaterial {
            base_color: Color::srgb(0.15, 1.0, 0.15),
            unlit: true,
            ..default()
        })),
        Transform::from_xyz(0.0, len / 2.0, 0.0),
    ));

    commands.spawn((
        Mesh3d(meshes.add(Cuboid::new(t, t, len))),
        MeshMaterial3d(mats.add(StandardMaterial {
            base_color: Color::srgb(0.2, 0.4, 1.0),
            unlit: true,
            ..default()
        })),
        Transform::from_xyz(0.0, t / 2.0, len / 2.0),
    ));
}

pub fn editor_hover(
    mut commands: Commands,
    playtest: Res<crate::playtest::PlaytestState>,
    windows: Query<&Window, With<PrimaryWindow>>,
    cameras: Query<(&Camera, &GlobalTransform), With<EditorCam>>,
    grid: Res<GridData>,
    existing: Query<Entity, With<HoverObj>>,
    mut meshes: ResMut<Assets<Mesh>>,
    mats: Res<SceneMaterials>,
    mut last_pos: Local<Option<(i32, i32)>>,
) {
    if playtest.active {
        if last_pos.is_some() {
            for e in &existing { commands.entity(e).despawn(); }
            *last_pos = None;
        }
        return;
    }
    let Ok(w) = windows.single() else { return };
    let Some(cursor) = w.cursor_position() else {
        if last_pos.is_some() {
            for e in &existing { commands.entity(e).despawn(); }
            *last_pos = None;
        }
        return;
    };
    let Ok((cam, cam_t)) = cameras.single() else { return };

    let pos = mouse_to_grid(cursor, w, cam, cam_t, grid.width, grid.height);
    if pos == *last_pos { return }

    for e in &existing { commands.entity(e).despawn(); }
    *last_pos = pos;

    if let Some((gx, gz)) = pos {
        if grid.get(gx, gz) == CellKind::Empty {
            commands.spawn((
                Mesh3d(meshes.add(Cuboid::new(CELL * 0.9, 0.02, CELL * 0.9))),
                MeshMaterial3d(mats.hover.clone()),
                Transform::from_xyz(gx as f32 * CELL, 0.01, gz as f32 * CELL),
                HoverObj,
            ));
        }
    }
}