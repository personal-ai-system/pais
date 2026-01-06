//! Security CLI commands

use chrono::Local;
use colored::*;
use eyre::Result;
use serde::Serialize;
use std::fs;
use terminal_size::{Width, terminal_size};

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

/// Get terminal width, defaulting to 80 if not available
fn get_terminal_width() -> usize {
    terminal_size().map(|(Width(w), _)| w as usize).unwrap_or(80)
}

/// Wrap text to max_width, returning lines
fn wrap_text(s: &str, max_width: usize) -> Vec<String> {
    if max_width == 0 {
        return vec![s.to_string()];
    }

    let mut lines = Vec::new();
    let mut current_line = String::new();
    let mut current_len = 0;

    for word in s.split_whitespace() {
        let word_len = word.chars().count();

        if current_len == 0 {
            current_line = word.to_string();
            current_len = word_len;
        } else if current_len + 1 + word_len <= max_width {
            current_line.push(' ');
            current_line.push_str(word);
            current_len += 1 + word_len;
        } else {
            lines.push(current_line);
            current_line = word.to_string();
            current_len = word_len;
        }
    }

    if !current_line.is_empty() {
        lines.push(current_line);
    }

    if lines.is_empty() {
        lines.push(String::new());
    }

    lines
}

/// Show security tiers
fn show_tiers(format: OutputFormat) -> Result<()> {
    let tiers = get_security_summary();

    match format {
        OutputFormat::Text => {
            if tiers.is_empty() {
                println!("{}", "No security tiers defined".dimmed());
                return Ok(());
            }

            let term_width = get_terminal_width();

            // Calculate column widths
            let tier_width = 4; // "Tier" or max 2 digits
            let action_width = tiers.iter().map(|(_, _, a)| a.len()).max().unwrap_or(6);

            // Description gets remaining space (minus columns and gaps)
            let fixed_width = tier_width + 2 + action_width + 2;
            let desc_width = term_width.saturating_sub(fixed_width).max(20);

            // Header
            println!(
                "{:<tier_width$}  {:<action_width$}  {}",
                "TIER".bold(),
                "ACTION".bold(),
                "DESCRIPTION".bold(),
                tier_width = tier_width,
                action_width = action_width,
            );

            // Tiers
            let indent = " ".repeat(fixed_width);
            for (tier, desc, action) in &tiers {
                let desc_lines = wrap_text(desc, desc_width);
                let action_colored = match *action {
                    "Block" => action.red(),
                    "Warn" => action.yellow(),
                    "Log" => action.dimmed(),
                    _ => action.normal(),
                };

                // First line with tier and action
                println!(
                    "{:<tier_width$}  {:<action_width$}  {}",
                    tier.to_string().cyan(),
                    action_colored,
                    desc_lines.first().unwrap_or(&String::new()).dimmed(),
                    tier_width = tier_width,
                    action_width = action_width,
                );
                // Continuation lines indented under description
                for line in desc_lines.iter().skip(1) {
                    println!("{}{}", indent, line.dimmed());
                }
            }

            println!();
            println!("{}", format!("{} tiers", tiers.len()).dimmed());
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
