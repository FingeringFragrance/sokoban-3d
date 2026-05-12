use bevy::prelude::*;
use sokoban_core::grid::*;
use sokoban_core::level::*;
use sokoban_core::rules::PLAYER_ENTITY_ID;
use sokoban_core::types::*;
use crate::camera::GameCamera;
use crate::effects::ParticleEffect;
use crate::assets::AssetCatalog;
use crate::environment::SceneEnvironment;
use crate::game::{SokobanEntity, CELL_SIZE};
use crate::particles::AmbientParticle;

#[derive(Component)]
pub struct SceneEntity;

#[derive(Component)]
pub struct GridObject {
    pub pos: GridPos,
    pub expected: ObjectType,
}

fn try_spawn_scene(
    commands: &mut Commands,
    catalog: Option<&AssetCatalog>,
    scene_theme: &str,
    object_type: &str,
    wx: f32,
    wy: f32,
    wz: f32,
) -> bool {
    let Some(cat) = catalog else { return false; };
    let Some(entry) = cat.get(scene_theme, object_type) else { return false; };
    commands.spawn((
        SceneRoot(entry.scene_handle.clone()),
        Transform::from_xyz(wx, wy, wz),
        SceneEntity,
    ));
    true
}

fn try_spawn_scene_ex(
    commands: &mut Commands,
    catalog: Option<&AssetCatalog>,
    scene_theme: &str,
    object_type: &str,
    transform: Transform,
    extra: impl Bundle,
) -> bool {
    let Some(cat) = catalog else { return false; };
    let Some(entry) = cat.get(scene_theme, object_type) else { return false; };
    commands.spawn((
        SceneRoot(entry.scene_handle.clone()),
        transform,
        extra,
        SceneEntity,
    ));
    true
}

pub fn spawn_level_from_file(
    commands: &mut Commands,
    meshes: &mut ResMut<Assets<Mesh>>,
    materials: &mut ResMut<Assets<StandardMaterial>>,
    path: &str,
    catalog: Option<&AssetCatalog>,
) -> Option<(GridState, LevelData)> {
    match LevelData::load_from_ron(path) {
        Ok(level) => {
            let grid = level.get_grid();
            let validation = validate_level(&level);
            if !validation.is_valid {
                println!("Level validation failed for '{}':", path);
                for issue in &validation.issues {
                    println!("  {}", issue);
                }
                return None;
            }
            let theme = level.scene_theme.clone();
            let grid_state = spawn_grid(commands, meshes, materials, &grid, catalog, &theme);
            Some((grid_state, level))
        }
        Err(e) => {
            println!("Failed to load level: {}", e);
            None
        }
    }
}

pub fn spawn_grid(
    commands: &mut Commands,
    meshes: &mut ResMut<Assets<Mesh>>,
    materials: &mut ResMut<Assets<StandardMaterial>>,
    grid: &Grid,
    catalog: Option<&AssetCatalog>,
    scene_theme: &str,
) -> GridState {
    let state = GridState::from_grid(grid, 0);
    spawn_world(commands, meshes, materials, &state, catalog, scene_theme);
    state
}

pub fn spawn_from_gridstate(
    commands: &mut Commands,
    meshes: &mut ResMut<Assets<Mesh>>,
    materials: &mut ResMut<Assets<StandardMaterial>>,
    state: &GridState,
    catalog: Option<&AssetCatalog>,
    scene_theme: &str,
) -> GridState {
    spawn_world(commands, meshes, materials, state, catalog, scene_theme);
    state.clone()
}

fn spawn_world(
    commands: &mut Commands,
    meshes: &mut ResMut<Assets<Mesh>>,
    materials: &mut ResMut<Assets<StandardMaterial>>,
    state: &GridState,
    catalog: Option<&AssetCatalog>,
    theme: &str,
) {
    let env = SceneEnvironment::for_theme(theme);

    for z in 0..state.height as i32 {
        for x in 0..state.width as i32 {
            let cell = &state.cells[z as usize][x as usize];
            let wx = x as f32 * CELL_SIZE;
            let wz = z as f32 * CELL_SIZE;
            let pos = GridPos::new(x, z);

            if spawn_wall_cell(commands, meshes, materials, catalog, theme, cell, wx, wz, &env) {
                continue;
            }

            spawn_floor_cell(commands, meshes, materials, catalog, theme, cell, wx, wz, &env);

            spawn_object_cell(commands, meshes, materials, catalog, theme, cell, pos, wx, wz);
        }
    }

    spawn_grid_lines(commands, meshes, materials, state, &env);
    spawn_player_entity(commands, meshes, materials, catalog, theme, state, &env);
    spawn_box_entities(commands, meshes, materials, catalog, theme, state, &env);
}

fn spawn_wall_cell(
    commands: &mut Commands,
    meshes: &mut ResMut<Assets<Mesh>>,
    materials: &mut ResMut<Assets<StandardMaterial>>,
    catalog: Option<&AssetCatalog>,
    theme: &str,
    cell: &Cell,
    wx: f32,
    wz: f32,
    env: &SceneEnvironment,
) -> bool {
    if cell.floor != FloorType::Empty
        || !matches!(cell.object, ObjectType::Wall | ObjectType::CrackedWall)
    {
        return false;
    }

    let obj_name = if cell.object == ObjectType::CrackedWall {
        "CrackedWall"
    } else {
        "Wall"
    };
    if try_spawn_scene(commands, catalog, theme, obj_name, wx, 1.5, wz) {
        return true;
    }
    let (color, roughness) = if cell.object == ObjectType::CrackedWall {
        (
            Color::srgb(
                env.wall_color.to_srgba().red * 0.8,
                env.wall_color.to_srgba().green * 0.8,
                env.wall_color.to_srgba().blue * 0.8,
            ),
            0.95,
        )
    } else {
        (env.wall_color, env.wall_roughness)
    };
    commands.spawn((
        Mesh3d(meshes.add(Cuboid::new(CELL_SIZE, 3.0, CELL_SIZE))),
        MeshMaterial3d(materials.add(StandardMaterial {
            base_color: color,
            metallic: 0.1,
            perceptual_roughness: roughness,
            ..default()
        })),
        Transform::from_xyz(wx, 1.5, wz),
        SceneEntity,
    ));
    true
}

fn spawn_floor_cell(
    commands: &mut Commands,
    meshes: &mut ResMut<Assets<Mesh>>,
    materials: &mut ResMut<Assets<StandardMaterial>>,
    catalog: Option<&AssetCatalog>,
    theme: &str,
    cell: &Cell,
    wx: f32,
    wz: f32,
    env: &SceneEnvironment,
) {
    if cell.floor == FloorType::Empty {
        return;
    }

    let floor_name = match cell.floor {
        FloorType::Target => "Target",
        FloorType::Ice => "Ice",
        FloorType::Water => "Water",
        FloorType::Pit => "Pit",
        FloorType::Mud => "Mud",
        FloorType::Glass => "Glass",
        FloorType::PressurePlate => "PressurePlate",
        FloorType::Portal(_) => "Portal",
        FloorType::Conveyor(_) => "Conveyor",
        FloorType::Ramp(_) => "Ramp",
        _ => "Normal",
    };
    if try_spawn_scene(commands, catalog, theme, floor_name, wx, 0.0, wz) {
        return;
    }

    let floor_color = match cell.floor {
        FloorType::Target => env.floor_target,
        FloorType::Ice => env.floor_ice,
        FloorType::Water => Color::srgb(0.2, 0.4, 0.8),
        FloorType::Pit => Color::srgb(0.1, 0.1, 0.12),
        FloorType::Mud => Color::srgb(0.5, 0.4, 0.25),
        FloorType::Glass => Color::srgba(0.85, 0.9, 0.95, 0.4),
        FloorType::PressurePlate => Color::srgb(0.75, 0.72, 0.65),
        FloorType::Portal(_) => Color::srgb(0.5, 0.3, 0.7),
        FloorType::Conveyor(_) => Color::srgb(0.6, 0.6, 0.55),
        FloorType::Ramp(_) => Color::srgb(0.8, 0.75, 0.65),
        _ => env.floor_normal,
    };
    let mut mat = StandardMaterial {
        base_color: floor_color,
        perceptual_roughness: 0.9,
        ..default()
    };
    if cell.floor == FloorType::Glass {
        mat.alpha_mode = AlphaMode::Blend;
    }
    commands.spawn((
        Mesh3d(meshes.add(Plane3d::default().mesh().size(CELL_SIZE, CELL_SIZE))),
        MeshMaterial3d(materials.add(mat)),
        Transform::from_xyz(wx, 0.0, wz),
        SceneEntity,
    ));
}

fn spawn_object_cell(
    commands: &mut Commands,
    meshes: &mut ResMut<Assets<Mesh>>,
    materials: &mut ResMut<Assets<StandardMaterial>>,
    catalog: Option<&AssetCatalog>,
    theme: &str,
    cell: &Cell,
    pos: GridPos,
    wx: f32,
    wz: f32,
) {
    match cell.object {
        ObjectType::Key(color) => spawn_key(commands, meshes, materials, catalog, theme, pos, color, wx, wz),
        ObjectType::Gate(color) => spawn_gate(commands, meshes, materials, catalog, theme, pos, color, wx, wz),
        ObjectType::Switch(_) => spawn_switch(commands, meshes, materials, catalog, theme, wx, wz),
        ObjectType::Spring => spawn_spring_obj(commands, meshes, materials, catalog, theme, wx, wz),
        ObjectType::Rock => spawn_rock_obj(commands, meshes, materials, catalog, theme, wx, wz),
        ObjectType::Pillar(id) => spawn_pillar_obj(commands, meshes, materials, catalog, theme, pos, id, wx, wz),
        ObjectType::Mirror(dir) => spawn_mirror_obj(commands, meshes, materials, catalog, theme, pos, dir, wx, wz),
        ObjectType::Magnet => spawn_magnet_obj(commands, meshes, materials, catalog, theme, pos, wx, wz),
        ObjectType::Spikes => spawn_spikes_obj(commands, meshes, materials, catalog, theme, pos, wx, wz),
        _ => {}
    }
}

fn spawn_key(
    commands: &mut Commands,
    meshes: &mut ResMut<Assets<Mesh>>,
    materials: &mut ResMut<Assets<StandardMaterial>>,
    catalog: Option<&AssetCatalog>,
    theme: &str,
    pos: GridPos,
    color: ItemColor,
    wx: f32,
    wz: f32,
) {
    if try_spawn_scene_ex(
        commands, catalog, theme, "Key",
        Transform::from_xyz(wx, 1.2, wz),
        GridObject { pos, expected: ObjectType::Key(color) },
    ) {
        return;
    }
    let c = item_color_to_bevy(color);
    commands.spawn((
        Mesh3d(meshes.add(Cuboid::new(0.5, 0.5, 0.5))),
        MeshMaterial3d(materials.add(StandardMaterial {
            base_color: c,
            metallic: 0.6,
            perceptual_roughness: 0.3,
            ..default()
        })),
        Transform::from_xyz(wx, 1.2, wz),
        GridObject { pos, expected: ObjectType::Key(color) },
        SceneEntity,
    ));
}

fn spawn_gate(
    commands: &mut Commands,
    meshes: &mut ResMut<Assets<Mesh>>,
    materials: &mut ResMut<Assets<StandardMaterial>>,
    catalog: Option<&AssetCatalog>,
    theme: &str,
    pos: GridPos,
    color: ItemColor,
    wx: f32,
    wz: f32,
) {
    if try_spawn_scene_ex(
        commands, catalog, theme, "Gate",
        Transform::from_xyz(wx, 1.5, wz),
        GridObject { pos, expected: ObjectType::Gate(color) },
    ) {
        return;
    }
    let c = item_color_to_bevy(color);
    commands.spawn((
        Mesh3d(meshes.add(Cuboid::new(CELL_SIZE * 0.9, 3.0, CELL_SIZE * 0.3))),
        MeshMaterial3d(materials.add(StandardMaterial {
            base_color: c,
            metallic: 0.5,
            perceptual_roughness: 0.4,
            ..default()
        })),
        Transform::from_xyz(wx, 1.5, wz),
        GridObject { pos, expected: ObjectType::Gate(color) },
        SceneEntity,
    ));
}

fn spawn_switch(
    commands: &mut Commands,
    meshes: &mut ResMut<Assets<Mesh>>,
    materials: &mut ResMut<Assets<StandardMaterial>>,
    catalog: Option<&AssetCatalog>,
    theme: &str,
    wx: f32,
    wz: f32,
) {
    if try_spawn_scene(commands, catalog, theme, "Switch", wx, 0.05, wz) {
        return;
    }
    commands.spawn((
        Mesh3d(meshes.add(Cuboid::new(0.8, 0.1, 0.8))),
        MeshMaterial3d(materials.add(StandardMaterial {
            base_color: Color::srgb(0.9, 0.8, 0.2),
            metallic: 0.4,
            perceptual_roughness: 0.5,
            ..default()
        })),
        Transform::from_xyz(wx, 0.05, wz),
        SceneEntity,
    ));
}

fn spawn_spring_obj(
    commands: &mut Commands,
    meshes: &mut ResMut<Assets<Mesh>>,
    materials: &mut ResMut<Assets<StandardMaterial>>,
    catalog: Option<&AssetCatalog>,
    theme: &str,
    wx: f32,
    wz: f32,
) {
    if try_spawn_scene(commands, catalog, theme, "Spring", wx, 0.25, wz) {
        return;
    }
    commands.spawn((
        Mesh3d(meshes.add(Cuboid::new(0.8, 0.5, 0.8))),
        MeshMaterial3d(materials.add(StandardMaterial {
            base_color: Color::srgb(0.3, 0.8, 0.4),
            metallic: 0.3,
            perceptual_roughness: 0.6,
            ..default()
        })),
        Transform::from_xyz(wx, 0.25, wz),
        SceneEntity,
    ));
}

fn spawn_rock_obj(
    commands: &mut Commands,
    meshes: &mut ResMut<Assets<Mesh>>,
    materials: &mut ResMut<Assets<StandardMaterial>>,
    catalog: Option<&AssetCatalog>,
    theme: &str,
    wx: f32,
    wz: f32,
) {
    if try_spawn_scene(commands, catalog, theme, "Rock", wx, 0.8, wz) {
        return;
    }
    commands.spawn((
        Mesh3d(meshes.add(Cuboid::new(1.6, 1.6, 1.6))),
        MeshMaterial3d(materials.add(StandardMaterial {
            base_color: Color::srgb(0.45, 0.4, 0.35),
            metallic: 0.05,
            perceptual_roughness: 0.95,
            ..default()
        })),
        Transform::from_xyz(wx, 0.8, wz),
        SceneEntity,
    ));
}

fn spawn_pillar_obj(
    commands: &mut Commands,
    meshes: &mut ResMut<Assets<Mesh>>,
    materials: &mut ResMut<Assets<StandardMaterial>>,
    catalog: Option<&AssetCatalog>,
    theme: &str,
    pos: GridPos,
    id: u8,
    wx: f32,
    wz: f32,
) {
    if try_spawn_scene_ex(
        commands, catalog, theme, "Pillar",
        Transform::from_xyz(wx, 1.5, wz),
        GridObject { pos, expected: ObjectType::Pillar(id) },
    ) {
        return;
    }
    commands.spawn((
        Mesh3d(meshes.add(Cuboid::new(0.6, 3.0, 0.6))),
        MeshMaterial3d(materials.add(StandardMaterial {
            base_color: Color::srgb(0.5, 0.5, 0.55),
            metallic: 0.2,
            perceptual_roughness: 0.7,
            ..default()
        })),
        Transform::from_xyz(wx, 1.5, wz),
        GridObject { pos, expected: ObjectType::Pillar(id) },
        SceneEntity,
    ));
}

fn spawn_mirror_obj(
    commands: &mut Commands,
    meshes: &mut ResMut<Assets<Mesh>>,
    materials: &mut ResMut<Assets<StandardMaterial>>,
    catalog: Option<&AssetCatalog>,
    theme: &str,
    pos: GridPos,
    dir: Direction,
    wx: f32,
    wz: f32,
) {
    let rotation = match dir {
        Direction::Up | Direction::Down => Quat::from_rotation_y(0.0),
        Direction::Left | Direction::Right => Quat::from_rotation_y(std::f32::consts::FRAC_PI_2),
    };
    if try_spawn_scene_ex(
        commands, catalog, theme, "Mirror",
        Transform {
            translation: Vec3::new(wx, 0.7, wz),
            rotation,
            ..default()
        },
        GridObject { pos, expected: ObjectType::Mirror(dir) },
    ) {
        return;
    }
    commands.spawn((
        Mesh3d(meshes.add(Cuboid::new(0.1, 1.4, 1.4))),
        MeshMaterial3d(materials.add(StandardMaterial {
            base_color: Color::srgb(0.85, 0.9, 0.95),
            metallic: 0.9,
            perceptual_roughness: 0.05,
            ..default()
        })),
        Transform {
            translation: Vec3::new(wx, 0.7, wz),
            rotation,
            ..default()
        },
        GridObject { pos, expected: ObjectType::Mirror(dir) },
        SceneEntity,
    ));
}

fn spawn_magnet_obj(
    commands: &mut Commands,
    meshes: &mut ResMut<Assets<Mesh>>,
    materials: &mut ResMut<Assets<StandardMaterial>>,
    catalog: Option<&AssetCatalog>,
    theme: &str,
    pos: GridPos,
    wx: f32,
    wz: f32,
) {
    if try_spawn_scene_ex(
        commands, catalog, theme, "Magnet",
        Transform::from_xyz(wx, 0.5, wz),
        GridObject { pos, expected: ObjectType::Magnet },
    ) {
        return;
    }
    commands.spawn((
        Mesh3d(meshes.add(Cylinder::new(0.35, 1.0))),
        MeshMaterial3d(materials.add(StandardMaterial {
            base_color: Color::srgb(0.7, 0.2, 0.2),
            metallic: 0.8,
            perceptual_roughness: 0.2,
            ..default()
        })),
        Transform::from_xyz(wx, 0.5, wz),
        GridObject { pos, expected: ObjectType::Magnet },
        SceneEntity,
    ));
}

fn spawn_spikes_obj(
    commands: &mut Commands,
    meshes: &mut ResMut<Assets<Mesh>>,
    materials: &mut ResMut<Assets<StandardMaterial>>,
    catalog: Option<&AssetCatalog>,
    theme: &str,
    pos: GridPos,
    wx: f32,
    wz: f32,
) {
    if try_spawn_scene_ex(
        commands, catalog, theme, "Spikes",
        Transform::from_xyz(wx, 0.08, wz),
        GridObject { pos, expected: ObjectType::Spikes },
    ) {
        return;
    }
    commands.spawn((
        Mesh3d(meshes.add(Cuboid::new(1.4, 0.15, 1.4))),
        MeshMaterial3d(materials.add(StandardMaterial {
            base_color: Color::srgb(0.6, 0.15, 0.15),
            metallic: 0.5,
            perceptual_roughness: 0.4,
            ..default()
        })),
        Transform::from_xyz(wx, 0.08, wz),
        GridObject { pos, expected: ObjectType::Spikes },
        SceneEntity,
    ));
}

fn spawn_grid_lines(
    commands: &mut Commands,
    meshes: &mut ResMut<Assets<Mesh>>,
    materials: &mut ResMut<Assets<StandardMaterial>>,
    state: &GridState,
    env: &SceneEnvironment,
) {
    let grid_mat = materials.add(StandardMaterial {
        base_color: env.grid_line_color,
        unlit: true,
        ..default()
    });

    for z in 0..=state.height as i32 {
        let z_world = z as f32 * CELL_SIZE;
        commands.spawn((
            Mesh3d(meshes.add(Cuboid::new(state.width as f32 * CELL_SIZE, 0.005, 0.02))),
            MeshMaterial3d(grid_mat.clone()),
            Transform::from_xyz(state.width as f32 * CELL_SIZE / 2.0, 0.005, z_world),
            SceneEntity,
        ));
    }

    for x in 0..=state.width as i32 {
        let x_world = x as f32 * CELL_SIZE;
        commands.spawn((
            Mesh3d(meshes.add(Cuboid::new(0.02, 0.005, state.height as f32 * CELL_SIZE))),
            MeshMaterial3d(grid_mat.clone()),
            Transform::from_xyz(x_world, 0.005, state.height as f32 * CELL_SIZE / 2.0),
            SceneEntity,
        ));
    }
}

fn spawn_player_entity(
    commands: &mut Commands,
    meshes: &mut ResMut<Assets<Mesh>>,
    materials: &mut ResMut<Assets<StandardMaterial>>,
    catalog: Option<&AssetCatalog>,
    theme: &str,
    state: &GridState,
    env: &SceneEnvironment,
) {
    let pp = state.player_pos.to_world(CELL_SIZE, 0.0);
    if try_spawn_scene_ex(
        commands, catalog, theme, "Player",
        Transform::from_xyz(pp[0], 0.9, pp[2]),
        SokobanEntity(PLAYER_ENTITY_ID),
    ) {
        return;
    }
    commands.spawn((
        Mesh3d(meshes.add(Cuboid::new(1.4, 1.8, 1.0))),
        MeshMaterial3d(materials.add(StandardMaterial {
            base_color: env.player_color,
            metallic: 0.3,
            perceptual_roughness: 0.5,
            ..default()
        })),
        Transform::from_xyz(pp[0], 0.9, pp[2]),
        SokobanEntity(PLAYER_ENTITY_ID),
        SceneEntity,
    ));
}

fn spawn_box_entities(
    commands: &mut Commands,
    meshes: &mut ResMut<Assets<Mesh>>,
    materials: &mut ResMut<Assets<StandardMaterial>>,
    catalog: Option<&AssetCatalog>,
    theme: &str,
    state: &GridState,
    env: &SceneEnvironment,
) {
    for b in &state.box_positions {
        let bp = b.pos.to_world(CELL_SIZE, 0.0);
        let box_type_name = match b.box_type {
            ObjectType::HeavyBox => "HeavyBox",
            ObjectType::FragileBox => "FragileBox",
            ObjectType::IceBox => "IceBox",
            ObjectType::Bomb => "Bomb",
            _ => "Box",
        };
        let sy: f32 = match b.box_type {
            ObjectType::HeavyBox => 1.7,
            ObjectType::FragileBox => 1.4,
            ObjectType::IceBox => 1.5,
            ObjectType::Bomb => 1.3,
            _ => 1.5,
        };
        if try_spawn_scene_ex(
            commands, catalog, theme, box_type_name,
            Transform::from_xyz(bp[0], sy / 2.0, bp[2]),
            SokobanEntity(b.entity_id),
        ) {
            continue;
        }
        let (color, metallic, roughness) = match b.box_type {
            ObjectType::HeavyBox => (Color::srgb(0.4, 0.4, 0.45), 0.3, 0.5),
            ObjectType::FragileBox => (Color::srgb(0.9, 0.6, 0.6), 0.1, 0.8),
            ObjectType::IceBox => (Color::srgb(0.6, 0.85, 0.95), 0.5, 0.2),
            ObjectType::Bomb => (Color::srgb(0.9, 0.3, 0.2), 0.2, 0.6),
            _ => (env.box_color, 0.15, 0.7),
        };
        commands.spawn((
            Mesh3d(meshes.add(Cuboid::new(sy, sy, sy))),
            MeshMaterial3d(materials.add(StandardMaterial {
                base_color: color,
                metallic,
                perceptual_roughness: roughness,
                ..default()
            })),
            Transform::from_xyz(bp[0], sy / 2.0, bp[2]),
            SokobanEntity(b.entity_id),
            SceneEntity,
        ));
    }
}

pub fn sync_grid_objects(
    mut commands: Commands,
    game_state: Option<Res<crate::game::GameState>>,
    mut query: Query<
        (Entity, &GridObject, &mut Transform),
        (
            Without<AmbientParticle>,
            Without<SokobanEntity>,
            Without<GameCamera>,
            Without<ParticleEffect>,
        ),
    >,
) {
    let Some(ref gs) = game_state else {
        return;
    };
    for (entity, go, mut transform) in &mut query {
        if let ObjectType::Pillar(id) = go.expected {
            let active = gs.grid.is_switch_active(id);
            let target_y = if active { -1.3 } else { 1.5 };
            transform.translation.y += (target_y - transform.translation.y) * 0.15;
            continue;
        }
        if gs.grid.object_at(go.pos) != go.expected {
            commands.entity(entity).despawn();
        }
    }
}

pub fn sync_destroyed_boxes(
    mut commands: Commands,
    game_state: Option<Res<crate::game::GameState>>,
    query: Query<(Entity, &SokobanEntity)>,
) {
    let Some(ref gs) = game_state else {
        return;
    };
    for (entity, sok) in &query {
        if sok.0 == PLAYER_ENTITY_ID {
            continue;
        }
        if gs
            .grid
            .box_positions
            .iter()
            .all(|b| b.entity_id != sok.0)
        {
            commands.entity(entity).despawn();
        }
    }
}

fn item_color_to_bevy(color: ItemColor) -> Color {
    match color {
        ItemColor::Red => Color::srgb(0.9, 0.2, 0.2),
        ItemColor::Blue => Color::srgb(0.2, 0.4, 0.9),
        ItemColor::Green => Color::srgb(0.2, 0.8, 0.3),
        ItemColor::Yellow => Color::srgb(0.9, 0.85, 0.2),
        ItemColor::Purple => Color::srgb(0.7, 0.3, 0.8),
    }
}
