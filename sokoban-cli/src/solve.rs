use std::time::Instant;

use sokoban_core::grid::*;
use sokoban_core::level::*;
use sokoban_core::solver::*;

pub fn cmd_solve(args: &[String]) {
    if args.is_empty() {
        eprintln!("Usage: sokoban-cli solve <file.ron>");
        return;
    }

    let path = &args[0];
    let level = match LevelData::load_from_ron(path) {
        Ok(l) => l,
        Err(e) => {
            eprintln!("Could not load '{}': {}", path, e);
            std::process::exit(1);
        }
    };

    let grid = level.get_grid();
    let grid_state = GridState::from_grid(&grid, 0);

    println!("Level: {}", level.meta.name);
    println!("Size: {}x{}", grid.width, grid.height);
    println!("Boxes: {}", grid.count_boxes());
    println!("Targets: {}", grid.count_targets());
    println!();

    let config = SolverConfig::default();
    let start = Instant::now();
    let result = solve(&grid_state, &config);
    let elapsed = start.elapsed().as_millis();

    if let Some(solution) = &result.solution {
        println!("SOLVABLE");
        println!("  Steps: {}", solution.len());
        println!("  States explored: {}", result.states_explored);
        println!("  Time: {}ms", elapsed);
        println!();
        print!("  Solution: ");
        for dir in solution {
            print!("{} ", dir);
        }
        println!();
    } else {
        println!("UNSOLVABLE (or timed out)");
        println!("  States explored: {}", result.states_explored);
        println!("  Time: {}ms", elapsed);
    }
}
