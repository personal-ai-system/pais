#![allow(dead_code)]

use colored::*;
use eyre::{Context, Result};
use std::io::{self, Read};

use crate::cli::HookAction;
use crate::config::Config;
use crate::hook::history::HistoryHandler;
use crate::hook::security::SecurityValidator;
use crate::hook::ui::UiHandler;
use crate::hook::{HookEvent, HookHandler, HookResult};
use crate::observability::EventEmitter;

pub fn run(action: HookAction, config: &Config) -> Result<()> {
    match action {
        HookAction::Dispatch { event, payload } => dispatch(&event, payload.as_deref(), config),
        HookAction::List { event } => list(event.as_deref(), config),
    }
}

fn dispatch(event: &str, payload: Option<&str>, config: &Config) -> Result<()> {
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

    // Parse event type
    let hook_event = match HookEvent::from_str(event) {
        Some(e) => e,
        None => {
            log::warn!("Unknown hook event: {}", event);
            std::process::exit(0); // Unknown events are allowed
        }
    };

    log::info!("Dispatching hook event: {:?}", hook_event);
    log::debug!("Payload: {}", payload);

    // Emit event to observability sinks (file, stdout, http)
    let history_path = Config::expand_path(&config.paths.history);
    let emitter = EventEmitter::new(config.observability.clone(), history_path.clone());
    emitter.emit(hook_event, &payload);

    // Build handlers list
    let security_enabled = config.hooks.security_enabled;
    let history_enabled = config.hooks.history_enabled;
    let ui_enabled = config.hooks.ui_enabled;

    let handlers: Vec<Box<dyn HookHandler>> = vec![
        Box::new(SecurityValidator::new(security_enabled).with_log_path(history_path.clone())),
        Box::new(HistoryHandler::new(history_enabled, history_path)),
        Box::new(UiHandler::new(ui_enabled)),
    ];

    // Run all handlers for this event
    for handler in &handlers {
        if handler.handles(hook_event) {
            let result = handler.handle(hook_event, &payload);

            match &result {
                HookResult::Block { message } => {
                    // Print block message to stderr (Claude Code reads this)
                    eprintln!("{}", message);
                    std::process::exit(result.exit_code());
                }
                HookResult::Error { message } => {
                    log::error!("Hook error: {}", message);
                    // Continue - errors don't block
                }
                HookResult::Allow => {
                    // Continue to next handler
                }
            }
        }
    }

    // All handlers passed
    std::process::exit(0);
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
