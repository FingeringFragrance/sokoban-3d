use sokoban_core::types::*;

pub fn build_scene_theme(id: &str) -> SceneTheme {
    match id {
        "forest" => SceneTheme {
            id: "forest".to_string(),
            name: "Emerald Forest".to_string(),
            environment_rules: EnvironmentRules {
                friction: 0.8,
                gravity_multiplier: 1.0,
                visibility_range: 80.0,
                time_scale: 1.0,
            },
            exclusive_mechanics: vec![
                // 落叶堆：定时出现/消失的地板（模拟隐藏路径）
                ExclusiveMechanic::AppearingFloor {
                    positions: vec![],
                    appear_interval: 7,
                    disappear_interval: 3,
                },
            ],
        },
        "volcano" => SceneTheme {
            id: "volcano".to_string(),
            name: "Lava Volcano".to_string(),
            environment_rules: EnvironmentRules {
                friction: 0.4,
                gravity_multiplier: 1.2,
                visibility_range: 60.0,
                time_scale: 1.0,
            },
            exclusive_mechanics: vec![
                ExclusiveMechanic::LavaCycle {
                    rise_interval: 8,
                    retreat_interval: 4,
                    pattern: vec![],
                },
            ],
        },
        "ice_palace" => SceneTheme {
            id: "ice_palace".to_string(),
            name: "Ice Palace".to_string(),
            environment_rules: EnvironmentRules {
                friction: 0.05,
                gravity_multiplier: 1.0,
                visibility_range: 70.0,
                time_scale: 1.0,
            },
            exclusive_mechanics: vec![
                ExclusiveMechanic::WindGust {
                    interval: 5,
                    direction: Direction::Right,
                    strength: 1,
                },
                ExclusiveMechanic::AppearingFloor {
                    positions: vec![],
                    appear_interval: 6,
                    disappear_interval: 2,
                },
            ],
        },
        "sky_temple" => SceneTheme {
            id: "sky_temple".to_string(),
            name: "Sky Temple".to_string(),
            environment_rules: EnvironmentRules {
                friction: 0.3,
                gravity_multiplier: 0.7,
                visibility_range: 100.0,
                time_scale: 1.0,
            },
            exclusive_mechanics: vec![
                ExclusiveMechanic::AppearingFloor {
                    positions: vec![],
                    appear_interval: 5,
                    disappear_interval: 5,
                },
                ExclusiveMechanic::BalanceScale {
                    left_positions: vec![],
                    right_positions: vec![],
                    max_weight_diff: 1,
                    linked_gate: 0,
                },
            ],
        },
        "ruins" => SceneTheme {
            id: "ruins".to_string(),
            name: "Ancient Ruins".to_string(),
            environment_rules: EnvironmentRules {
                friction: 0.5,
                gravity_multiplier: 1.0,
                visibility_range: 60.0,
                time_scale: 1.0,
            },
            exclusive_mechanics: vec![
                ExclusiveMechanic::WaterLevel {
                    initial_level: 0,
                    max_level: 3,
                },
                ExclusiveMechanic::LightBeam {
                    source_pos: GridPos::new(0, 0),
                    source_dir: Direction::Right,
                    target_pos: GridPos::new(0, 0),
                },
            ],
        },
        "void" => SceneTheme {
            id: "void".to_string(),
            name: "Void Space".to_string(),
            environment_rules: EnvironmentRules {
                friction: 0.0,
                gravity_multiplier: 0.5,
                visibility_range: 50.0,
                time_scale: 1.0,
            },
            exclusive_mechanics: vec![
                ExclusiveMechanic::MirrorZone {
                    zone_a_origin: GridPos::new(0, 0),
                    zone_a_size: (3, 3),
                    zone_b_origin: GridPos::new(6, 6),
                    zone_b_size: (3, 3),
                },
            ],
        },
        _ => SceneTheme {
            id: "default".to_string(),
            name: "Default".to_string(),
            environment_rules: EnvironmentRules::default(),
            exclusive_mechanics: vec![],
        },
    }
}
