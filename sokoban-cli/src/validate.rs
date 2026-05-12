use std::time::Instant;

use sokoban_core::grid::*;
use sokoban_core::level::*;
use sokoban_core::solver::*;

pub fn cmd_validate(args: &[String]) {
    if args.is_empty() {
        eprintln!("Usage: sokoban-cli validate <file.ron>");
        return;
    }

    let path = &args[0];
    let level = match LevelData::load_from_ron(path) {
        Ok(l) => l,
        Err(e) => {
            eprintln!("FAIL: Could not load '{}': {}", path, e);
            std::process::exit(1);
        }
    };

    let validation = validate_level(&level);

    if !validation.is_valid {
        println!("INVALID");
        for issue in &validation.issues {
            println!("  - {}", issue);
        }
    } else {
        println!("VALID");
    }

    let grid = level.get_grid();
    let grid_state = GridState::from_grid(&grid, 0);

    let config = SolverConfig {
        max_states: 500_000,
        timeout_ms: 10_000,
    };

    let start = Instant::now();
    let result = solve(&grid_state, &config);
    let elapsed = start.elapsed().as_millis();

    if let Some(solution) = &result.solution {
        println!("SOLVABLE");
        println!("  Optimal steps: {}", solution.len());
        println!("  States explored: {}", result.states_explored);
        println!("  Time: {}ms", elapsed);
    } else {
        println!("UNSOLVABLE (or timed out)");
        println!("  States explored: {}", result.states_explored);
        println!("  Time: {}ms", elapsed);
    }
}
