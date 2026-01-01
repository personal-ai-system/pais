#![allow(dead_code)]

use colored::*;
use eyre::{Context, Result};
use std::io::{self, Read};

use crate::cli::HookAction;
use crate::config::Config;

/// Exit codes for hook dispatch
/// These match Claude Code's expectations
pub const EXIT_ALLOW: i32 = 0;
pub const EXIT_BLOCK: i32 = 2;

pub fn run(action: HookAction, config: &Config) -> Result<()> {
    match action {
        HookAction::Dispatch { event, payload } => dispatch(&event, payload.as_deref(), config),
        HookAction::List { event } => list(event.as_deref(), config),
    }
}

fn dispatch(event: &str, payload: Option<&str>, _config: &Config) -> Result<()> {
    // Read payload from stdin if not provided
    let payload_str = match payload {
        Some(p) => p.to_string(),
        None => {
            let mut buffer = String::new();
            io::stdin()
                .read_to_string(&mut buffer)
                .context("Failed to read payload from stdin")?;
            buffer
        }
    };

    // Parse the payload
    let payload: serde_json::Value = serde_json::from_str(&payload_str).context("Failed to parse payload JSON")?;

    log::info!("Dispatching hook event: {} with payload", event);
    log::debug!("Payload: {}", payload);

    // TODO: Look up handlers for this event and dispatch
    // For now, just allow everything

    // Return appropriate exit code
    // EXIT_ALLOW (0) = continue
    // EXIT_BLOCK (2) = block the action
    std::process::exit(EXIT_ALLOW);
}

fn list(event_filter: Option<&str>, _config: &Config) -> Result<()> {
    println!("{}", "Registered hook handlers:".bold());
    println!();

    if let Some(event) = event_filter {
        println!("  Filtering by event: {}", event.cyan());
    }

    // TODO: Implement handler listing
    println!("  {} Hook handler listing not yet implemented", "⚠".yellow());

    Ok(())
}
