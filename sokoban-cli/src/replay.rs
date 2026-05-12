use sokoban_core::grid::*;
use sokoban_core::level::*;
use sokoban_core::replay::*;
use sokoban_core::rules::*;
use sokoban_core::types::*;

pub fn cmd_replay(args: &[String]) {
    if args.len() < 2 {
        eprintln!("Usage: sokoban-cli replay <file.ron> <replay_string>");
        return;
    }

    let path = &args[0];
    let encoded = &args[1];

    let level = match LevelData::load_from_ron(path) {
        Ok(l) => l,
        Err(e) => {
            eprintln!("Could not load '{}': {}", path, e);
            std::process::exit(1);
        }
    };

    let replay = match ReplayData::decode(encoded) {
        Some(r) => r,
        None => {
            eprintln!("Invalid replay string");
            std::process::exit(1);
        }
    };

    println!("Level: {}", level.meta.name);
    println!("Replay: {} steps", replay.total_steps);
    println!();

    let grid = level.get_grid();
    let mut state = GridState::from_grid(&grid, 0);
    let scene = SceneTheme::default();

    let mut player = ReplayPlayer::new(replay);
    player.play();

    let mut step_num = 0;
    while let Some(dir) = player.next_move() {
        step_num += 1;
        let intent = MoveIntent { direction: dir };
        let result = resolve_move(&mut state, intent, &scene);

        if result.success {
            println!("Step {}: {} -> OK (player at {})", step_num, dir, state.player_pos.pos);
        } else {
            println!("Step {}: {} -> FAILED", step_num, dir);
            eprintln!("Replay failed at step {}!", step_num);
            std::process::exit(1);
        }
    }

    println!();
    if state.all_boxes_on_targets() {
        println!("REPLAY VALID: All boxes on targets!");
    } else {
        println!("REPLAY INVALID: Not all boxes on targets.");
        println!("  Boxes on targets: {}/{}", state.boxes_on_targets(), state.box_count());
    }
}
