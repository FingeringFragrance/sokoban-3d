use bevy::prelude::*;
use rand::Rng;

use crate::camera::GameCamera;
use crate::effects::ParticleEffect;
use crate::environment::SceneEnvironment;
use crate::game::{GameState, SokobanEntity};
use crate::spawner::{GridObject, SceneEntity};

#[derive(Resource)]
pub struct AmbientParticleSpawned(pub bool);

impl Default for AmbientParticleSpawned {
    fn default() -> Self {
        Self(false)
    }
}

#[derive(Component)]
pub struct AmbientParticle {
    pub velocity: Vec3,
    pub lifetime: f32,
    pub age: f32,
    pub sway_speed: f32,
    pub sway_amount: f32,
    pub initial_x: f32,
}

pub fn spawn_ambient_particles(
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
    game_state: Option<Res<GameState>>,
    mut spawned: ResMut<AmbientParticleSpawned>,
) {
    let Some(ref gs) = game_state else { return; };

    if spawned.0 {
        return;
    }

    let env = SceneEnvironment::for_theme(&gs.scene_theme);
    if env.particle_count == 0 {
        spawned.0 = true;
        return;
    }

    spawned.0 = true;

    let mut rng = rand::thread_rng();
    let center_x = gs.grid.width as f32 * gs.cell_size / 2.0;
    let center_z = gs.grid.height as f32 * gs.cell_size / 2.0;
    let spread = env.particle_spread;

    let particle_mesh = meshes.add(Cuboid::new(0.08, 0.08, 0.08));
    let particle_material = materials.add(StandardMaterial {
        base_color: env.particle_color,
        emissive: LinearRgba::new(
            env.particle_color.to_srgba().red * 0.3,
            env.particle_color.to_srgba().green * 0.3,
            env.particle_color.to_srgba().blue * 0.3,
            1.0,
        ),
        alpha_mode: AlphaMode::Blend,
        ..default()
    });

    for _ in 0..env.particle_count {
        let x = center_x + rng.gen_range(-spread..spread);
        let y = rng.gen_range(2.0..8.0);
        let z = center_z + rng.gen_range(-spread..spread);

        let vx = rng.gen_range(-0.3..0.3);
        let vy = -env.particle_speed * rng.gen_range(0.5..1.5);
        let vz = rng.gen_range(-0.3..0.3);

        let lifetime = rng.gen_range(4.0..10.0);
        let sway_speed = rng.gen_range(1.0..3.0);
        let sway_amount = rng.gen_range(0.3..1.0);

        commands.spawn((
            Mesh3d(particle_mesh.clone()),
            MeshMaterial3d(particle_material.clone()),
            Transform::from_xyz(x, y, z).with_scale(Vec3::splat(rng.gen_range(0.5..1.5))),
            AmbientParticle {
                velocity: Vec3::new(vx, vy, vz),
                lifetime,
                age: 0.0,
                sway_speed,
                sway_amount,
                initial_x: x,
            },
            SceneEntity,
        ));
    }
}

pub fn update_ambient_particles(
    time: Res<Time>,
    mut query: Query<
        (&mut AmbientParticle, &mut Transform),
        (
            Without<ParticleEffect>,
            Without<SokobanEntity>,
            Without<GameCamera>,
            Without<GridObject>,
        ),
    >,
    game_state: Option<Res<GameState>>,
) {
    let dt = time.delta_secs();
    let elapsed = time.elapsed_secs();

    let Some(ref gs) = game_state else { return; };
    let center_x = gs.grid.width as f32 * gs.cell_size / 2.0;
    let center_z = gs.grid.height as f32 * gs.cell_size / 2.0;
    let env = SceneEnvironment::for_theme(&gs.scene_theme);
    let spread = env.particle_spread;

    for (mut particle, mut transform) in &mut query {
        particle.age += dt;

        if particle.age >= particle.lifetime {
            let mut rng = rand::thread_rng();
            particle.age = 0.0;
            particle.lifetime = rng.gen_range(4.0..10.0);
            transform.translation.x = center_x + rng.gen_range(-spread..spread);
            transform.translation.y = rng.gen_range(5.0..10.0);
            transform.translation.z = center_z + rng.gen_range(-spread..spread);
            particle.initial_x = transform.translation.x;
            particle.velocity = Vec3::new(
                rng.gen_range(-0.3..0.3),
                -env.particle_speed * rng.gen_range(0.5..1.5),
                rng.gen_range(-0.3..0.3),
            );
            continue;
        }

        let sway = (elapsed * particle.sway_speed + particle.initial_x).sin() * particle.sway_amount;
        transform.translation.x = particle.initial_x + sway;
        transform.translation += particle.velocity * dt;

        transform.rotate_y(dt * 2.0);
        transform.rotate_x(dt * 0.7);
    }
}
