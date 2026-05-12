use std::fs;

use rand::rngs::StdRng;
use rand::SeedableRng;

use sokoban_core::generator::{generate, GenParams};
use sokoban_core::level::*;
use sokoban_core::solver::SolverConfig;

pub fn cmd_generate(args: &[String]) {
    if args.is_empty() {
        eprintln!("Usage: sokoban-cli generate <params.json>");
        eprintln!("  params.json format:");
        eprintln!("    {{");
        eprintln!("      \"count\": 5,");
        eprintln!("      \"seed\": 12345,");
        eprintln!("      \"min_width\": 6, \"max_width\": 10,");
        eprintln!("      \"min_height\": 6, \"max_height\": 10,");
        eprintln!("      \"min_boxes\": 1, \"max_boxes\": 3,");
        eprintln!("      \"wall_density\": 0.15,");
        eprintln!("      \"output_dir\": \"generated/\"");
        eprintln!("    }}");
        return;
    }

    let path = &args[0];
    let content = match fs::read_to_string(path) {
        Ok(c) => c,
        Err(e) => {
            eprintln!("Could not read '{}': {}", path, e);
            std::process::exit(1);
        }
    };

    let json: serde_json::Value = match serde_json::from_str(&content) {
        Ok(v) => v,
        Err(e) => {
            eprintln!("Invalid JSON: {}", e);
            std::process::exit(1);
        }
    };

    let count = json["count"].as_u64().unwrap_or(5) as u32;
    let seed = json["seed"].as_u64().unwrap_or(42);
    let output_dir = json["output_dir"]
        .as_str()
        .unwrap_or("generated")
        .to_string();

    let params = GenParams {
        min_width: json["min_width"].as_u64().unwrap_or(6) as u32,
        max_width: json["max_width"].as_u64().unwrap_or(10) as u32,
        min_height: json["min_height"].as_u64().unwrap_or(6) as u32,
        max_height: json["max_height"].as_u64().unwrap_or(10) as u32,
        min_boxes: json["min_boxes"].as_u64().unwrap_or(1) as u32,
        max_boxes: json["max_boxes"].as_u64().unwrap_or(3) as u32,
        wall_density: json["wall_density"].as_f64().unwrap_or(0.15) as f32,
        max_retries: 100,
        scene_theme: "default".to_string(),
        target_difficulty: 2,
        special_floor_density: 0.0,
        available_items: vec![],
        solver_config: SolverConfig {
            max_states: 200_000,
            timeout_ms: 5_000,
        },
    };

    let mut rng = StdRng::seed_from_u64(seed);

    fs::create_dir_all(&output_dir).ok();

    let mut generated = 0;
    let mut failed = 0;

    for i in 0..count {
        match generate(&params, &mut rng) {
            Some(result) => {
                let level = LevelData {
                    meta: LevelMeta {
                        id: i + 1,
                        name: format!("Generated {}", i + 1),
                        author: "Generator".to_string(),
                        difficulty: 1,
                        par_steps: result.optimal_steps,
                        tags: vec!["generated".to_string()],
                        description: format!(
                            "Auto-generated ({} boxes, {} optimal steps, {} attempts)",
                            result.box_count,
                            result.optimal_steps.unwrap_or(0),
                            result.attempts,
                        ),
                    },
                    grid: Some(result.grid),
                    ascii: None,
                    scene_theme: "default".to_string(),
                };

                let out_path = format!("{}/gen_{:03}.ron", output_dir, i + 1);
                match level.save_to_ron(&out_path) {
                    Ok(()) => {
                        println!(
                            "OK: {} ({} boxes, {} steps, {} attempts)",
                            out_path,
                            result.box_count,
                            result.optimal_steps.unwrap_or(0),
                            result.attempts,
                        );
                        generated += 1;
                    }
                    Err(e) => {
                        eprintln!("FAIL: Could not save {}: {}", out_path, e);
                        failed += 1;
                    }
                }
            }
            None => {
                eprintln!("FAIL: Could not generate level {} after {} retries", i + 1, params.max_retries);
                failed += 1;
            }
        }
    }

    println!();
    println!("Generated: {}/{}", generated, count);
    if failed > 0 {
        println!("Failed: {}", failed);
    }
}
