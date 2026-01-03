//! Security CLI commands

use chrono::Local;
use colored::*;
use eyre::Result;
use serde::Serialize;
use std::fs;

use crate::cli::{OutputFormat, SecurityAction as CliSecurityAction};
use crate::config::Config;
use crate::hook::security::{SecurityEvent, get_security_summary};

pub fn run(action: CliSecurityAction, config: &Config) -> Result<()> {
    match action {
        CliSecurityAction::Tiers { format } => show_tiers(OutputFormat::resolve(format)),
        CliSecurityAction::Log { days, format } => show_log(days, OutputFormat::resolve(format), config),
        CliSecurityAction::Test { command } => test_command(&command, config),
    }
}

/// Show security tiers
fn show_tiers(format: OutputFormat) -> Result<()> {
    let tiers = get_security_summary();

    match format {
        OutputFormat::Text => {
            println!("{}", "Security Tiers:".bold());
            println!();
            println!("  {:4} {:30} {:8}", "Tier", "Description", "Action");
            println!("  {:4} {:30} {:8}", "----", "------------------------------", "------");

            for (tier, desc, action) in &tiers {
                let action_colored = match *action {
                    "Block" => action.red(),
                    "Warn" => action.yellow(),
                    "Log" => action.dimmed(),
                    _ => action.normal(),
                };
                println!("  {:4} {:30} {}", tier, desc, action_colored);
            }
        }
        OutputFormat::Json => {
            #[derive(Serialize)]
            struct TierInfo {
                tier: u8,
                description: String,
                action: String,
            }
            let output: Vec<TierInfo> = tiers
                .iter()
                .map(|(t, d, a)| TierInfo {
                    tier: *t,
                    description: d.to_string(),
                    action: a.to_string(),
                })
                .collect();
            println!("{}", serde_json::to_string_pretty(&output)?);
        }
        OutputFormat::Yaml => {
            #[derive(Serialize)]
            struct TierInfo {
                tier: u8,
                description: String,
                action: String,
            }
            let output: Vec<TierInfo> = tiers
                .iter()
                .map(|(t, d, a)| TierInfo {
                    tier: *t,
                    description: d.to_string(),
                    action: a.to_string(),
                })
                .collect();
            println!("{}", serde_yaml::to_string(&output)?);
        }
    }

    Ok(())
}

/// Show security log
fn show_log(days: usize, format: OutputFormat, config: &Config) -> Result<()> {
    let history_path = Config::expand_path(&config.paths.history);
    let security_dir = history_path.join("security");

    let mut events = Vec::new();

    // Collect events from the last N days
    let today = Local::now().date_naive();
    for i in 0..days {
        let date = today - chrono::Duration::days(i as i64);
        let month_dir = security_dir.join(date.format("%Y-%m").to_string());
        let log_file = month_dir.join(format!("{}.jsonl", date.format("%Y-%m-%d")));

        if log_file.exists()
            && let Ok(content) = fs::read_to_string(&log_file)
        {
            for line in content.lines() {
                if line.trim().is_empty() {
                    continue;
                }
                if let Ok(event) = serde_json::from_str::<SecurityEvent>(line) {
                    events.push(event);
                }
            }
        }
    }

    // Sort by timestamp (newest first)
    events.sort_by(|a, b| b.timestamp.cmp(&a.timestamp));

    match format {
        OutputFormat::Text => {
            println!("{} Security events (last {} days):", "🔒".blue(), days);
            println!();

            if events.is_empty() {
                println!("  {}", "(no security events)".dimmed());
            } else {
                for event in &events {
                    let action_colored = match event.action.as_str() {
                        "Block" => event.action.red(),
                        "Warn" => event.action.yellow(),
                        _ => event.action.dimmed(),
                    };

                    println!(
                        "  {} [Tier {}] {} - {}",
                        event.timestamp[..19].dimmed(), // Just date and time
                        event.tier,
                        action_colored,
                        event.description.bold()
                    );

                    // Truncate command
                    let cmd_display = if event.command.len() > 60 {
                        format!("{}...", &event.command[..57])
                    } else {
                        event.command.clone()
                    };
                    println!("    {}", cmd_display.dimmed());
                }

                println!();
                println!("Total: {} event(s)", events.len());
            }
        }
        OutputFormat::Json => {
            println!("{}", serde_json::to_string_pretty(&events)?);
        }
        OutputFormat::Yaml => {
            println!("{}", serde_yaml::to_string(&events)?);
        }
    }

    Ok(())
}

/// Test a command against security patterns
fn test_command(command: &str, _config: &Config) -> Result<()> {
    use crate::hook::security::SecurityValidator;
    use crate::hook::{HookHandler, HookResult};

    let validator = SecurityValidator::new(true);

    // Create a mock payload
    let payload = serde_json::json!({
        "tool_name": "Bash",
        "tool_input": {
            "command": command
        }
    });

    let result = validator.handle(crate::hook::HookEvent::PreToolUse, &payload);

    match result {
        HookResult::Block { message } => {
            println!("{}", message);
            println!();
            println!("Command: {}", command.dimmed());
            std::process::exit(2);
        }
        HookResult::Allow => {
            println!("{} Command allowed", "✓".green());
            println!();
            println!("Command: {}", command.dimmed());
        }
        HookResult::Error { message } => {
            println!("{} Error: {}", "✗".red(), message);
        }
    }

    Ok(())
}
