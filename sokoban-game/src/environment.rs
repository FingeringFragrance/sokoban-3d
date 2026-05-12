use bevy::prelude::*;

use crate::spawner::SceneEntity;

pub struct SceneEnvironment {
    pub clear_color: Color,
    pub ambient_brightness: f32,
    pub ambient_color: Color,
    pub directional_illuminance: f32,
    pub directional_color: Color,
    #[allow(dead_code)]
    pub fog_color: Color,
    pub fog_density: f32,
    pub particle_color: Color,
    pub particle_count: u32,
    pub particle_spread: f32,
    pub particle_speed: f32,
    pub floor_normal: Color,
    pub floor_target: Color,
    pub floor_ice: Color,
    pub wall_color: Color,
    pub wall_roughness: f32,
    pub box_color: Color,
    pub player_color: Color,
    pub grid_line_color: Color,
}

impl SceneEnvironment {
    pub fn for_theme(theme: &str) -> Self {
        match theme {
            "forest" => Self::forest(),
            "volcano" => Self::volcano(),
            "ice_palace" => Self::ice_palace(),
            "sky_temple" => Self::sky_temple(),
            "ruins" => Self::ruins(),
            "void" => Self::void(),
            _ => Self::default_env(),
        }
    }

    fn default_env() -> Self {
        Self {
            clear_color: Color::srgb(0.05, 0.05, 0.08),
            ambient_brightness: 400.0,
            ambient_color: Color::WHITE,
            directional_illuminance: 8000.0,
            directional_color: Color::srgb(1.0, 0.98, 0.95),
            fog_color: Color::srgb(0.05, 0.05, 0.08),
            fog_density: 0.02,
            particle_color: Color::srgba(1.0, 1.0, 1.0, 0.0),
            particle_count: 0,
            particle_spread: 0.0,
            particle_speed: 0.0,
            floor_normal: Color::srgb(0.88, 0.83, 0.73),
            floor_target: Color::srgb(0.4, 0.78, 0.5),
            floor_ice: Color::srgb(0.7, 0.88, 0.95),
            wall_color: Color::srgb(0.35, 0.35, 0.42),
            wall_roughness: 0.8,
            box_color: Color::srgb(0.82, 0.52, 0.2),
            player_color: Color::srgb(0.3, 0.55, 0.85),
            grid_line_color: Color::srgba(1.0, 1.0, 1.0, 0.06),
        }
    }

    fn forest() -> Self {
        Self {
            clear_color: Color::srgb(0.15, 0.18, 0.1),
            ambient_brightness: 500.0,
            ambient_color: Color::srgb(0.85, 0.95, 0.8),
            directional_illuminance: 7000.0,
            directional_color: Color::srgb(1.0, 0.95, 0.8),
            fog_color: Color::srgb(0.18, 0.22, 0.14),
            fog_density: 0.03,
            particle_color: Color::srgb(0.4, 0.6, 0.2),
            particle_count: 40,
            particle_spread: 20.0,
            particle_speed: 0.8,
            floor_normal: Color::srgb(0.45, 0.55, 0.3),
            floor_target: Color::srgb(0.3, 0.7, 0.35),
            floor_ice: Color::srgb(0.7, 0.88, 0.95),
            wall_color: Color::srgb(0.35, 0.28, 0.18),
            wall_roughness: 0.95,
            box_color: Color::srgb(0.6, 0.42, 0.2),
            player_color: Color::srgb(0.25, 0.5, 0.3),
            grid_line_color: Color::srgba(0.3, 0.5, 0.2, 0.1),
        }
    }

    fn volcano() -> Self {
        Self {
            clear_color: Color::srgb(0.08, 0.03, 0.02),
            ambient_brightness: 300.0,
            ambient_color: Color::srgb(1.0, 0.7, 0.5),
            directional_illuminance: 5000.0,
            directional_color: Color::srgb(1.0, 0.6, 0.3),
            fog_color: Color::srgb(0.12, 0.05, 0.03),
            fog_density: 0.025,
            particle_color: Color::srgb(0.6, 0.3, 0.1),
            particle_count: 25,
            particle_spread: 15.0,
            particle_speed: 1.5,
            floor_normal: Color::srgb(0.25, 0.2, 0.18),
            floor_target: Color::srgb(0.8, 0.4, 0.2),
            floor_ice: Color::srgb(0.7, 0.88, 0.95),
            wall_color: Color::srgb(0.2, 0.12, 0.1),
            wall_roughness: 0.9,
            box_color: Color::srgb(0.7, 0.35, 0.15),
            player_color: Color::srgb(0.8, 0.5, 0.3),
            grid_line_color: Color::srgba(0.8, 0.3, 0.1, 0.08),
        }
    }

    fn ice_palace() -> Self {
        Self {
            clear_color: Color::srgb(0.08, 0.1, 0.15),
            ambient_brightness: 600.0,
            ambient_color: Color::srgb(0.8, 0.88, 1.0),
            directional_illuminance: 9000.0,
            directional_color: Color::srgb(0.85, 0.92, 1.0),
            fog_color: Color::srgb(0.15, 0.18, 0.25),
            fog_density: 0.015,
            particle_color: Color::srgb(0.85, 0.92, 1.0),
            particle_count: 50,
            particle_spread: 22.0,
            particle_speed: 0.5,
            floor_normal: Color::srgb(0.75, 0.82, 0.9),
            floor_target: Color::srgb(0.5, 0.75, 0.9),
            floor_ice: Color::srgb(0.8, 0.92, 1.0),
            wall_color: Color::srgb(0.6, 0.7, 0.85),
            wall_roughness: 0.3,
            box_color: Color::srgb(0.55, 0.65, 0.75),
            player_color: Color::srgb(0.4, 0.55, 0.8),
            grid_line_color: Color::srgba(0.5, 0.7, 0.9, 0.08),
        }
    }

    fn sky_temple() -> Self {
        Self {
            clear_color: Color::srgb(0.55, 0.7, 0.9),
            ambient_brightness: 800.0,
            ambient_color: Color::srgb(1.0, 0.98, 0.95),
            directional_illuminance: 12000.0,
            directional_color: Color::WHITE,
            fog_color: Color::srgb(0.65, 0.78, 0.92),
            fog_density: 0.008,
            particle_color: Color::srgba(1.0, 1.0, 1.0, 0.3),
            particle_count: 20,
            particle_spread: 25.0,
            particle_speed: 0.3,
            floor_normal: Color::srgb(0.9, 0.88, 0.82),
            floor_target: Color::srgb(0.6, 0.8, 0.95),
            floor_ice: Color::srgb(0.7, 0.88, 0.95),
            wall_color: Color::srgb(0.85, 0.82, 0.75),
            wall_roughness: 0.4,
            box_color: Color::srgb(0.75, 0.7, 0.6),
            player_color: Color::srgb(0.9, 0.85, 0.7),
            grid_line_color: Color::srgba(0.4, 0.5, 0.6, 0.06),
        }
    }

    fn ruins() -> Self {
        Self {
            clear_color: Color::srgb(0.06, 0.06, 0.05),
            ambient_brightness: 350.0,
            ambient_color: Color::srgb(0.9, 0.85, 0.7),
            directional_illuminance: 6000.0,
            directional_color: Color::srgb(0.95, 0.88, 0.7),
            fog_color: Color::srgb(0.08, 0.07, 0.06),
            fog_density: 0.02,
            particle_color: Color::srgb(0.5, 0.45, 0.35),
            particle_count: 15,
            particle_spread: 18.0,
            particle_speed: 0.2,
            floor_normal: Color::srgb(0.5, 0.48, 0.4),
            floor_target: Color::srgb(0.6, 0.7, 0.5),
            floor_ice: Color::srgb(0.7, 0.88, 0.95),
            wall_color: Color::srgb(0.4, 0.38, 0.32),
            wall_roughness: 0.95,
            box_color: Color::srgb(0.55, 0.48, 0.35),
            player_color: Color::srgb(0.7, 0.6, 0.4),
            grid_line_color: Color::srgba(0.6, 0.5, 0.3, 0.08),
        }
    }

    fn void() -> Self {
        Self {
            clear_color: Color::srgb(0.02, 0.01, 0.05),
            ambient_brightness: 200.0,
            ambient_color: Color::srgb(0.7, 0.5, 1.0),
            directional_illuminance: 4000.0,
            directional_color: Color::srgb(0.8, 0.6, 1.0),
            fog_color: Color::srgb(0.03, 0.01, 0.06),
            fog_density: 0.035,
            particle_color: Color::srgb(0.6, 0.3, 1.0),
            particle_count: 30,
            particle_spread: 20.0,
            particle_speed: 1.0,
            floor_normal: Color::srgb(0.1, 0.08, 0.15),
            floor_target: Color::srgb(0.5, 0.3, 0.8),
            floor_ice: Color::srgb(0.3, 0.5, 0.9),
            wall_color: Color::srgb(0.08, 0.05, 0.12),
            wall_roughness: 0.6,
            box_color: Color::srgb(0.4, 0.25, 0.7),
            player_color: Color::srgb(0.5, 0.3, 0.85),
            grid_line_color: Color::srgba(0.5, 0.2, 0.9, 0.06),
        }
    }
}

pub fn setup_environment(
    commands: &mut Commands,
    theme: &str,
    colorblind: bool,
    high_contrast: bool,
) {
    let mut env = SceneEnvironment::for_theme(theme);
    env.apply_accessibility(colorblind, high_contrast);

    commands.insert_resource(ClearColor(env.clear_color));

    commands.spawn((
        AmbientLight {
            color: env.ambient_color,
            brightness: env.ambient_brightness,
            affects_lightmapped_meshes: true,
        },
        SceneEntity,
    ));

    commands.spawn((
        DirectionalLight {
            illuminance: env.directional_illuminance,
            shadows_enabled: true,
            color: env.directional_color,
            ..default()
        },
        Transform::from_rotation(Quat::from_euler(EulerRot::XYZ, -0.8, 0.4, 0.0)),
        SceneEntity,
    ));
}

impl SceneEnvironment {
    /// 根据无障碍设置调整环境颜色
    pub fn apply_accessibility(&mut self, colorblind: bool, high_contrast: bool) {
        if high_contrast {
            // 提高对比度：加深暗色，提亮亮色
            self.fog_density *= 0.5;
            self.ambient_brightness *= 1.3;
            self.directional_illuminance *= 1.2;
            self.floor_normal = saturate_color(self.floor_normal, 1.2);
            self.floor_target = saturate_color(self.floor_target, 1.3);
            self.wall_color = saturate_color(self.wall_color, 1.15);
            self.box_color = saturate_color(self.box_color, 1.25);
            self.player_color = saturate_color(self.player_color, 1.3);
            self.grid_line_color.set_alpha(self.grid_line_color.alpha() * 2.0);
        }

        if colorblind {
            // 色盲模式：使用高区分度色调（蓝-橙-黄调色板）
            self.floor_target = Color::srgb(0.1, 0.5, 0.85);
            self.player_color = Color::srgb(0.9, 0.7, 0.1);
        }
    }
}

fn saturate_color(c: Color, factor: f32) -> Color {
    c.mix(&Color::WHITE, factor.clamp(0.0, 1.0) * 0.3)
}

#[derive(Resource)]
pub struct SceneEffectState {
    pub timer: f32,
}

impl Default for SceneEffectState {
    fn default() -> Self {
        Self { timer: 0.0 }
    }
}

pub fn update_scene_effects(
    time: Res<Time>,
    mut effect_state: ResMut<SceneEffectState>,
    mut ambient_query: Query<&mut AmbientLight, With<SceneEntity>>,
    game_state: Option<Res<crate::game::GameState>>,
) {
    effect_state.timer += time.delta_secs();

    let Some(ref gs) = game_state else { return; };
    let env = SceneEnvironment::for_theme(&gs.scene_theme);

    for mut ambient in &mut ambient_query {
        match gs.scene_theme.as_str() {
            "volcano" => {
                let pulse = (effect_state.timer * 2.5).sin() * 0.15 + 0.85;
                ambient.brightness = env.ambient_brightness * pulse;
            }
            "void" => {
                let flicker = (effect_state.timer * 3.7).sin() * 0.2 + 0.8;
                ambient.brightness = env.ambient_brightness * flicker;
            }
            "ice_palace" => {
                let shimmer = (effect_state.timer * 1.8).sin() * 0.08 + 0.92;
                ambient.brightness = env.ambient_brightness * shimmer;
            }
            _ => {}
        }
    }
}
