use bevy::prelude::*;

use crate::camera::GameCamera;
use crate::game::{GameState, SokobanEntity};
use crate::particles::AmbientParticle;
use crate::spawner::{GridObject, SceneEntity};
use sokoban_core::rules::PLAYER_ENTITY_ID;

// ============================================================
//  Resources
// ============================================================

#[derive(Resource, Default)]
pub struct EffectState {
    pub prev_level_complete: bool,
}

/// 每帧待处理的特效事件（由 player_input 写入，由 process_pending_effects 消费）
#[derive(Resource)]
pub struct PendingEffects {
    pub explosions: Vec<Vec3>,
    pub portal_flash: bool,
}

impl Default for PendingEffects {
    fn default() -> Self {
        Self {
            explosions: Vec::new(),
            portal_flash: false,
        }
    }
}

/// 传送闪屏计时器
#[derive(Resource)]
pub struct PortalFlash {
    pub timer: f32,
    pub duration: f32,
}

impl Default for PortalFlash {
    fn default() -> Self {
        Self {
            timer: 0.0,
            duration: 0.25,
        }
    }
}

// ============================================================
//  Components
// ============================================================

#[derive(Component)]
pub struct ParticleEffect {
    pub lifetime: f32,
    pub age: f32,
    pub velocity: Vec3,
}

/// 全屏白色闪屏覆盖层（在 setup_hud 中生成）
#[derive(Component)]
pub struct PortalFlashOverlay;

// ============================================================
//  箱子目标点辉光
// ============================================================

pub fn box_glow(
    game_state: Option<Res<GameState>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
    query: Query<(&SokobanEntity, &MeshMaterial3d<StandardMaterial>)>,
) {
    let Some(ref gs) = game_state else {
        return;
    };

    for (sok, mat3d) in &query {
        if sok.0 == PLAYER_ENTITY_ID {
            continue;
        }

        let on_target = gs
            .grid
            .box_positions
            .iter()
            .find(|b| b.entity_id == sok.0)
            .map(|b| gs.grid.is_target(b.pos.pos))
            .unwrap_or(false);

        if let Some(mat) = materials.get_mut(&mat3d.0) {
            if on_target {
                mat.emissive = LinearRgba::new(0.0, 0.6, 0.25, 1.0);
            } else {
                mat.emissive = LinearRgba::new(0.0, 0.0, 0.0, 1.0);
            }
        }
    }
}

// ============================================================
//  通关粒子爆发
// ============================================================

pub fn trigger_effects(
    mut commands: Commands,
    game_state: Option<Res<GameState>>,
    mut effect_state: ResMut<EffectState>,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
) {
    let Some(ref gs) = game_state else {
        return;
    };

    if gs.level_complete && !effect_state.prev_level_complete {
        let pp = gs.grid.player_pos.to_world(gs.cell_size, 0.0);
        spawn_completion_burst(
            &mut commands,
            &mut meshes,
            &mut materials,
            Vec3::new(pp[0], 0.0, pp[2]),
        );
    }

    effect_state.prev_level_complete = gs.level_complete;
}

fn spawn_completion_burst(
    commands: &mut Commands,
    meshes: &mut ResMut<Assets<Mesh>>,
    materials: &mut ResMut<Assets<StandardMaterial>>,
    center: Vec3,
) {
    let palette: &[(f32, f32, f32)] = &[
        (1.0, 0.85, 0.2),
        (0.3, 0.9, 0.5),
        (0.35, 0.6, 0.95),
        (0.95, 0.35, 0.35),
        (0.75, 0.35, 0.85),
    ];

    for i in 0..50 {
        let angle = (i as f32 / 50.0) * std::f32::consts::TAU;
        let speed = 3.0 + (i as f32 * 0.17) % 4.5;
        let vx = angle.cos() * speed;
        let vz = angle.sin() * speed;
        let vy = 5.0 + (i as f32 * 0.13) % 6.0;

        let (r, g, b) = palette[i % palette.len()];
        let color = Color::srgb(r, g, b);
        let size = 0.12 + (i as f32 * 0.006) % 0.12;

        commands.spawn((
            Mesh3d(meshes.add(Cuboid::new(size, size, size))),
            MeshMaterial3d(materials.add(StandardMaterial {
                base_color: color,
                emissive: LinearRgba::new(r * 0.6, g * 0.6, b * 0.6, 1.0),
                ..default()
            })),
            Transform::from_translation(center + Vec3::Y * 1.5),
            ParticleEffect {
                lifetime: 2.5,
                age: 0.0,
                velocity: Vec3::new(vx, vy, vz),
            },
            SceneEntity,
        ));
    }
}

// ============================================================
//  爆炸粒子（箱子销毁时触发）
// ============================================================

fn spawn_explosion_burst(
    commands: &mut Commands,
    meshes: &mut ResMut<Assets<Mesh>>,
    materials: &mut ResMut<Assets<StandardMaterial>>,
    center: Vec3,
) {
    let palette: &[(f32, f32, f32)] = &[
        (1.0, 0.6, 0.1),
        (0.95, 0.3, 0.1),
        (1.0, 0.85, 0.2),
        (0.9, 0.2, 0.1),
    ];

    for i in 0..20 {
        let angle = (i as f32 / 20.0) * std::f32::consts::TAU;
        let speed = 4.0 + (i as f32 * 0.23) % 3.0;
        let vx = angle.cos() * speed;
        let vz = angle.sin() * speed;
        let vy = 6.0 + (i as f32 * 0.17) % 4.0;

        let (r, g, b) = palette[i % palette.len()];
        let color = Color::srgb(r, g, b);
        let size = 0.1 + (i as f32 * 0.005) % 0.08;

        commands.spawn((
            Mesh3d(meshes.add(Cuboid::new(size, size, size))),
            MeshMaterial3d(materials.add(StandardMaterial {
                base_color: color,
                emissive: LinearRgba::new(r * 0.8, g * 0.8, b * 0.8, 1.0),
                ..default()
            })),
            Transform::from_translation(center + Vec3::Y * 0.5),
            ParticleEffect {
                lifetime: 1.5,
                age: 0.0,
                velocity: Vec3::new(vx, vy, vz),
            },
            SceneEntity,
        ));
    }
}

// ============================================================
//  处理待定特效（每帧消费 PendingEffects）
// ============================================================

pub fn process_pending_effects(
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
    mut pending: ResMut<PendingEffects>,
    mut flash: ResMut<PortalFlash>,
) {
    let explosions: Vec<Vec3> = pending.explosions.drain(..).collect();
    for pos in explosions {
        spawn_explosion_burst(&mut commands, &mut meshes, &mut materials, pos);
    }

    if pending.portal_flash {
        flash.timer = flash.duration;
        pending.portal_flash = false;
    }
}

// ============================================================
//  传送闪屏更新
// ============================================================

pub fn update_portal_flash(
    time: Res<Time>,
    mut flash: ResMut<PortalFlash>,
    mut query: Query<(&mut Visibility, &mut BackgroundColor), With<PortalFlashOverlay>>,
) {
    if flash.timer > 0.0 {
        flash.timer -= time.delta_secs();
        let alpha = (flash.timer / flash.duration).clamp(0.0, 1.0) * 0.5;
        for (mut vis, mut bg) in &mut query {
            *vis = Visibility::Visible;
            bg.0 = Color::srgba(1.0, 1.0, 1.0, alpha);
        }
    } else {
        for (mut vis, _) in &mut query {
            *vis = Visibility::Hidden;
        }
    }
}

// ============================================================
//  粒子动画更新
// ============================================================

pub fn update_particles(
    mut commands: Commands,
    time: Res<Time>,
    mut query: Query<
        (Entity, &mut ParticleEffect, &mut Transform),
        (
            Without<AmbientParticle>,
            Without<SokobanEntity>,
            Without<GameCamera>,
            Without<GridObject>,
        ),
    >,
) {
    let dt = time.delta_secs();
    for (entity, mut particle, mut transform) in &mut query {
        particle.age += dt;
        if particle.age >= particle.lifetime {
            commands.entity(entity).despawn();
            continue;
        }

        transform.translation += particle.velocity * dt;
        particle.velocity.y -= 9.8 * dt;

        let t = particle.age / particle.lifetime;
        let scale = (1.0 - t * t).max(0.01);
        transform.scale = Vec3::splat(scale);

        transform.rotate_y(dt * 4.0);
    }
}
