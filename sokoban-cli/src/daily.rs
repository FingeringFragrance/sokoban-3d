use std::time::Instant;

use rand::rngs::StdRng;
use rand::SeedableRng;

use sokoban_core::generator::{generate, GenParams};
use sokoban_core::solver::SolverConfig;

pub fn cmd_daily(args: &[String]) {
    let seed_str = if args.is_empty() {
        let now = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap_or_default()
            .as_secs();
        let days = now / 86400;
        format!("{}", days)
    } else {
        args[0].clone()
    };

    let seed: u64 = {
        let mut h: u64 = 0;
        for b in seed_str.bytes() {
            h = h.wrapping_mul(31).wrapping_add(b as u64);
        }
        h
    };

    println!("Daily challenge seed: {} (from '{}')", seed, seed_str);

    let params = GenParams {
        min_width: 7,
        max_width: 10,
        min_height: 7,
        max_height: 10,
        min_boxes: 2,
        max_boxes: 4,
        wall_density: 0.12,
        max_retries: 200,
        scene_theme: "default".to_string(),
        target_difficulty: 2,
        special_floor_density: 0.0,
        available_items: vec![],
        solver_config: SolverConfig {
            max_states: 500_000,
            timeout_ms: 10_000,
        },
    };

    let mut rng = StdRng::seed_from_u64(seed);

    let start = Instant::now();
    match generate(&params, &mut rng) {
        Some(result) => {
            let elapsed = start.elapsed().as_millis();
            println!();
            println!("=== Daily Challenge ===");
            println!("Boxes: {}", result.box_count);
            println!("Optimal steps: {}", result.optimal_steps.unwrap_or(0));
            println!("Generation attempts: {}", result.attempts);
            println!("Generation time: {}ms", elapsed);
            println!();
            println!("ASCII:");
            println!("{}", result.grid.to_ascii());
        }
        None => {
            eprintln!("Could not generate daily challenge after {} retries", params.max_retries);
            std::process::exit(1);
        }
    }
}
