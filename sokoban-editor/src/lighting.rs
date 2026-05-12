use bevy::prelude::*;

pub fn setup_environment_light(mut commands: Commands, mut images: ResMut<Assets<Image>>) {
    let w = 256u32;
    let h = 128u32;
    let mut data: Vec<u8> = Vec::with_capacity((w * h * 16) as usize);

    for y in 0..h {
        let t = y as f32 / h as f32;
        let sky = t < 0.5;
        let horizon_factor = (1.0 - (t - 0.5).abs() * 4.0).clamp(0.0, 1.0);

        let r = if sky { 0.3 + t * 0.5 } else { 0.25 * (1.0 - t) + 0.05 };
        let g = if sky { 0.4 + t * 0.4 } else { 0.2 * (1.0 - t) + 0.05 };
        let b = if sky { 0.6 + t * 0.4 } else { 0.15 * (1.0 - t) + 0.03 };
        let brightness = 0.8 + horizon_factor * 1.5;

        for _x in 0..w {
            let rf = r * brightness;
            let gf = g * brightness;
            let bf = b * brightness;
            data.extend_from_slice(&rf.to_le_bytes());
            data.extend_from_slice(&gf.to_le_bytes());
            data.extend_from_slice(&bf.to_le_bytes());
            data.extend_from_slice(&1.0f32.to_le_bytes());
        }
    }

    let image = Image::new(
        bevy::render::render_resource::Extent3d { width: w, height: h, depth_or_array_layers: 1 },
        bevy::render::render_resource::TextureDimension::D2,
        data,
        bevy::render::render_resource::TextureFormat::Rgba32Float,
        bevy::asset::RenderAssetUsages::RENDER_WORLD,
    );
    let handle = images.add(image);

    commands.spawn(EnvironmentMapLight {
        diffuse_map: handle.clone(),
        specular_map: handle.clone(),
        intensity: 800.0,
        affects_lightmapped_mesh_diffuse: false,
        rotation: Quat::IDENTITY,
    });
}

pub fn spawn_lights(mut commands: Commands) {
    commands.spawn((
        DirectionalLight {
            illuminance: 15000.0,
            shadows_enabled: true,
            ..default()
        },
        Transform::from_rotation(Quat::from_euler(EulerRot::XYZ, -0.8, 0.4, 0.0)),
    ));
    commands.spawn((
        DirectionalLight {
            illuminance: 5000.0,
            shadows_enabled: false,
            ..default()
        },
        Transform::from_rotation(Quat::from_euler(EulerRot::XYZ, -0.3, -2.0, 0.0)),
    ));
    commands.spawn(AmbientLight {
        brightness: 1500.0,
        ..default()
    });
}