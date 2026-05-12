use std::fs;
use std::path::Path;

use sokoban_core::grid::*;
use sokoban_core::level::*;
use sokoban_core::solver::*;

pub fn cmd_validate_batch(args: &[String]) {
    if args.is_empty() {
        eprintln!("Usage: sokoban-cli validate-batch <directory>");
        return;
    }

    let dir = &args[0];
    let path = Path::new(dir);

    if !path.is_dir() {
        eprintln!("Not a directory: {}", dir);
        std::process::exit(1);
    }

    let mut files: Vec<String> = Vec::new();
    collect_ron_files(path, &mut files);
    files.sort();

    if files.is_empty() {
        println!("No .ron files found in {}", dir);
        return;
    }

    let mut total = 0;
    let mut passed = 0;
    let mut failed = 0;
    let mut solvable = 0;
    let mut unsolvable = 0;
    let mut failures: Vec<(String, Vec<String>)> = Vec::new();

    let solver_config = SolverConfig {
        max_states: 200_000,
        timeout_ms: 5_000,
    };

    for file in &files {
        total += 1;

        let level = match LevelData::load_from_ron(file) {
            Ok(l) => l,
            Err(e) => {
                failed += 1;
                failures.push((file.clone(), vec![format!("Load error: {}", e)]));
                continue;
            }
        };

        let validation = validate_level(&level);
        let mut issues: Vec<String> = validation
            .issues
            .iter()
            .map(|i| format!("{}", i))
            .collect();

        let grid = level.get_grid();
        let grid_state = GridState::from_grid(&grid, 0);
        let result = solve(&grid_state, &solver_config);

        if result.solution.is_some() {
            solvable += 1;
            let steps = result.solution.as_ref().unwrap().len();
            issues.push(format!("SOLVABLE ({} steps, {} states)", steps, result.states_explored));
        } else {
            unsolvable += 1;
            issues.push(format!(
                "UNSOLVABLE ({} states, timeout={})",
                result.states_explored, solver_config.timeout_ms
            ));
        }

        if validation.is_valid && result.solution.is_some() {
            passed += 1;
        } else {
            failed += 1;
            failures.push((file.clone(), issues));
        }
    }

    println!("=== Validation Report ===");
    println!("Total files: {}", total);
    println!("Passed:      {}", passed);
    println!("Failed:      {}", failed);
    println!("Solvable:    {}", solvable);
    println!("Unsolvable:  {}", unsolvable);

    if !failures.is_empty() {
        println!();
        println!("--- Failures ---");
        for (file, issues) in &failures {
            println!("{}:", file);
            for issue in issues {
                println!("  {}", issue);
            }
        }
    }

    if failed > 0 {
        std::process::exit(1);
    }
}

fn collect_ron_files(dir: &Path, out: &mut Vec<String>) {
    if let Ok(entries) = fs::read_dir(dir) {
        for entry in entries.flatten() {
            let path = entry.path();
            if path.is_dir() {
                collect_ron_files(&path, out);
            } else if path.extension().map_or(false, |e| e == "ron") {
                out.push(path.to_string_lossy().to_string());
            }
        }
    }
}
