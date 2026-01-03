//! hello-rust - A PAIS plugin

use std::env;

fn main() {
    let args: Vec<String> = env::args().collect();

    if args.len() < 2 {
        eprintln!(r#"{{"error": "No action specified"}}"#);
        std::process::exit(1);
    }

    let action = &args[1];
    let action_args = &args[2..];

    match action.as_str() {
        "greet" => {
            let name = action_args.first().map(|s| s.as_str()).unwrap_or("World");
            println!(r#"{{"message": "Hello, {}!"}}"#, name);
        }
        "version" => {
            println!(r#"{{"version": "0.1.0"}}"#);
        }
        _ => {
            eprintln!(r#"{{"error": "Unknown action: {}"}}"#, action);
            std::process::exit(1);
        }
    }
}
