use colored::*;
use eyre::{Context, Result};
use std::io::{self, Read};

use crate::cli::HookAction;
use crate::config::Config;
use crate::hook::history::HistoryHandler;
use crate::hook::research::ResearchPathValidator;
use crate::hook::security::SecurityValidator;
use crate::hook::ui::UiHandler;
use crate::hook::{HookEvent, HookHandler, HookResult};
use crate::observability::EventEmitter;
use crate::plugin::PluginManager;

pub fn run(action: HookAction, config: &Config) -> Result<()> {
    match action {
        HookAction::Dispatch { event, payload } => dispatch(&event, payload.as_deref(), config),
        HookAction::List { event } => list(event.as_deref(), config),
    }
}

fn dispatch(event: &str, payload: Option<&str>, config: &Config) -> Result<()> {
    log::debug!("Hook dispatch started: event={}", event);

    // Read payload from stdin if not provided
    let payload_str = match payload {
        Some(p) => {
            log::debug!("Payload provided as argument ({} bytes)", p.len());
            p.to_string()
        }
        None => {
            log::debug!("Reading payload from stdin...");
            let mut buffer = String::new();
            io::stdin()
                .read_to_string(&mut buffer)
                .context("Failed to read payload from stdin")?;
            log::debug!("Read {} bytes from stdin", buffer.len());
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
    let research_enabled = config.hooks.research_enabled;

    log::debug!(
        "Handler config: security={}, history={}, ui={}, research={}",
        security_enabled,
        history_enabled,
        ui_enabled,
        research_enabled
    );

    let handlers: Vec<Box<dyn HookHandler>> = vec![
        Box::new(SecurityValidator::new(security_enabled).with_log_path(history_path.clone())),
        Box::new(ResearchPathValidator::new(research_enabled)),
        Box::new(HistoryHandler::new(history_enabled, history_path)),
        Box::new(UiHandler::new(ui_enabled)),
    ];

    // Run all built-in handlers for this event
    for handler in &handlers {
        if handler.handles(hook_event) {
            log::debug!("Running handler: {}", handler.name());
            let result = handler.handle(hook_event, &payload);

            match &result {
                HookResult::Block { message } => {
                    log::warn!("Handler {} blocked: {}", handler.name(), message);
                    // Print block message to stderr (Claude Code reads this)
                    eprintln!("{}", message);
                    std::process::exit(result.exit_code());
                }
                HookResult::Error { message } => {
                    log::error!("Hook error from {}: {}", handler.name(), message);
                    // Continue - errors don't block
                }
                HookResult::Allow => {
                    log::debug!("Handler {} allowed", handler.name());
                    // Continue to next handler
                }
            }
        } else {
            log::trace!("Handler {} does not handle {:?}", handler.name(), hook_event);
        }
    }

    // Run plugin hooks
    let plugins_dir = Config::expand_path(&config.paths.plugins);
    log::debug!("Checking plugin hooks in: {}", plugins_dir.display());

    let mut plugin_manager = PluginManager::new(plugins_dir);

    if plugin_manager.discover().is_ok() {
        log::debug!("Found {} plugins with hooks", plugin_manager.plugins.len());

        let plugin_results = plugin_manager.execute_hooks(hook_event, &payload);

        for result in plugin_results {
            match &result {
                HookResult::Block { message } => {
                    log::warn!("Plugin hook blocked: {}", message);
                    eprintln!("{}", message);
                    std::process::exit(result.exit_code());
                }
                HookResult::Error { message } => {
                    log::error!("Plugin hook error: {}", message);
                }
                HookResult::Allow => {
                    log::debug!("Plugin hook allowed");
                }
            }
        }
    } else {
        log::debug!("No plugins discovered");
    }

    log::debug!("Hook dispatch complete, all handlers passed");
    // All handlers passed
    std::process::exit(0);
}

/// Information about a built-in hook handler
struct HandlerInfo {
    name: &'static str,
    description: &'static str,
    events: &'static [&'static str],
    enabled: bool,
}

fn list(event_filter: Option<&str>, config: &Config) -> Result<()> {
    println!("{}", "Hook Handlers".bold());
    println!();

    // Parse event filter if provided
    let filter_event = event_filter.and_then(HookEvent::from_str);
    if let Some(filter_str) = event_filter
        && filter_event.is_none()
    {
        println!(
            "  {} Unknown event '{}', showing all handlers",
            "⚠".yellow(),
            filter_str
        );
        println!();
    }

    // Built-in handlers
    let handlers = vec![
        HandlerInfo {
            name: "security",
            description: "Blocks dangerous commands before execution",
            events: &["PreToolUse"],
            enabled: config.hooks.security_enabled,
        },
        HandlerInfo {
            name: "history",
            description: "Captures session lifecycle events",
            events: &["SessionStart", "Stop", "SubagentStop", "SessionEnd"],
            enabled: config.hooks.history_enabled,
        },
        HandlerInfo {
            name: "ui",
            description: "Updates terminal tab title",
            events: &["UserPromptSubmit"],
            enabled: config.hooks.ui_enabled,
        },
        HandlerInfo {
            name: "research",
            description: "Validates research directory path structure",
            events: &["PreToolUse"],
            enabled: config.hooks.research_enabled,
        },
    ];

    // Filter handlers if event specified
    let filtered_handlers: Vec<_> = handlers
        .into_iter()
        .filter(|h| {
            filter_event
                .map(|e| h.events.contains(&e.to_string().as_str()))
                .unwrap_or(true)
        })
        .collect();

    // Print built-in handlers
    println!("  {}", "Built-in Handlers".cyan().bold());
    println!();

    if filtered_handlers.is_empty() {
        println!("    (none match filter)");
    } else {
        for handler in &filtered_handlers {
            let status = if handler.enabled { "●".green() } else { "○".bright_black() };
            let state = if handler.enabled { "enabled".green() } else { "disabled".bright_black() };

            println!("    {} {} ({})", status, handler.name.bold(), state);
            println!("      {}", handler.description.bright_black());
            println!("      Events: {}", handler.events.join(", ").cyan());
            println!();
        }
    }

    // Plugin handlers
    let plugins_dir = Config::expand_path(&config.paths.plugins);
    let mut plugin_manager = PluginManager::new(plugins_dir);

    println!("  {}", "Plugin Handlers".cyan().bold());
    println!();

    if plugin_manager.discover().is_ok() {
        let plugins_with_hooks: Vec<_> = plugin_manager
            .list()
            .filter(|p| p.manifest.hooks.has_hooks())
            .filter(|p| {
                filter_event
                    .map(|e| !p.manifest.hooks.scripts_for_event(&e.to_string()).is_empty())
                    .unwrap_or(true)
            })
            .collect();

        if plugins_with_hooks.is_empty() {
            println!("    (no plugins with hooks)");
        } else {
            for plugin in plugins_with_hooks {
                println!(
                    "    {} {} v{}",
                    "●".green(),
                    plugin.manifest.plugin.name.bold(),
                    plugin.manifest.plugin.version
                );
                println!("      {}", plugin.manifest.plugin.description.bright_black());

                // List events this plugin handles
                let mut events = Vec::new();
                if !plugin.manifest.hooks.pre_tool_use.is_empty() {
                    events.push("PreToolUse");
                }
                if !plugin.manifest.hooks.post_tool_use.is_empty() {
                    events.push("PostToolUse");
                }
                if !plugin.manifest.hooks.stop.is_empty() {
                    events.push("Stop");
                }
                if !plugin.manifest.hooks.session_start.is_empty() {
                    events.push("SessionStart");
                }
                if !plugin.manifest.hooks.session_end.is_empty() {
                    events.push("SessionEnd");
                }
                if !plugin.manifest.hooks.subagent_stop.is_empty() {
                    events.push("SubagentStop");
                }

                println!("      Events: {}", events.join(", ").cyan());
                println!();
            }
        }
    } else {
        println!("    (plugins directory not found)");
    }

    // Show available events for reference
    if filter_event.is_none() {
        println!("  {}", "Available Events".cyan().bold());
        println!();
        println!("    PreToolUse      Before a tool runs (can block)");
        println!("    PostToolUse     After a tool completes");
        println!("    SessionStart    When a session begins");
        println!("    Stop            When main session stops");
        println!("    SubagentStop    When a subagent stops");
        println!("    SessionEnd      When session fully ends");
        println!("    UserPromptSubmit When user submits a prompt");
        println!("    Notification    When a notification is sent");
        println!("    PermissionRequest When permission is requested");
        println!("    PreCompact      Before context compaction");
        println!();
    }

    Ok(())
}
