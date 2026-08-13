// Jabr CLI entry point.
//
// Usage:
//   jabr run <file.jabr>   — execute a Jabr source file
//   jabr                  — start REPL (future)

use std::env;
use std::fs;
use std::process;

fn main() {
    let args: Vec<String> = env::args().collect();

    if args.len() < 2 {
        eprintln!("Usage: jabr run <file.jabr>");
        process::exit(1);
    }

    match args[1].as_str() {
        "run" => {
            if args.len() < 3 {
                eprintln!("Usage: jabr run <file.jabr>");
                process::exit(1);
            }
            let path = &args[2];
            let source = match fs::read_to_string(path) {
                Ok(s) => s,
                Err(e) => {
                    eprintln!("Error reading '{}': {}", path, e);
                    process::exit(1);
                }
            };
            if let Err(e) = jabr::run_source(&source) {
                eprintln!("Error: {}", e);
                process::exit(1);
            }
        }
        other => {
            eprintln!("Unknown command '{}'. Usage: jabr run <file.jabr>", other);
            process::exit(1);
        }
    }
}
