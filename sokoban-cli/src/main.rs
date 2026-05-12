mod daily;
mod generate;
mod replay;
mod solve;
mod validate;
mod validate_batch;

use std::env;

fn main() {
    let args: Vec<String> = env::args().collect();

    if args.len() < 2 {
        print_usage();
        return;
    }

    match args[1].as_str() {
        "validate" => validate::cmd_validate(&args[2..]),
        "validate-batch" => validate_batch::cmd_validate_batch(&args[2..]),
        "generate" => generate::cmd_generate(&args[2..]),
        "solve" => solve::cmd_solve(&args[2..]),
        "replay" => replay::cmd_replay(&args[2..]),
        "daily" => daily::cmd_daily(&args[2..]),
        "help" | "--help" | "-h" => print_usage(),
        other => {
            eprintln!("Unknown command: {}", other);
            print_usage();
        }
    }
}

fn print_usage() {
    println!("sokoban-cli - Sokoban 3D command line tools");
    println!();
    println!("USAGE:");
    println!("  sokoban-cli <COMMAND> [ARGS]");
    println!();
    println!("COMMANDS:");
    println!("  validate <file.ron>              Validate a single level file");
    println!("  validate-batch <directory>       Validate all .ron files in a directory");
    println!("  generate <params.json>           Generate levels from JSON parameters");
    println!("  solve <file.ron>                 Solve a level and print the solution");
    println!("  replay <file.ron> <replay_str>   Replay a solution on a level");
    println!("  daily [date_str]                 Generate and solve today's daily challenge");
    println!("  help                             Show this help");
}
