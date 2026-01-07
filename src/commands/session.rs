//! Session command - Launch Claude Code with dynamic MCP configuration
//!
//! This module provides `pais session` which wraps Claude Code startup,
//! allowing selective MCP server loading to reduce context token usage.
//!
//! ## Usage
//!
//! ```bash
//! # Launch with specific MCPs
//! pais session --mcp github,slack
//!
//! # Use a named profile
//! pais session --profile minimal
//!
//! # List available MCPs and profiles
//! pais session --list
//! ```

use colored::Colorize;
use eyre::{Context, Result, eyre};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::fs;
use std::os::unix::process::CommandExt;
use std::path::PathBuf;
use std::process::Command;

use crate::cli::OutputFormat;
use crate::config::{Config, McpServerConfig};

/// MCP server definition as stored in ~/.mcp.json or similar
#[derive(Debug, Clone, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
struct McpJsonFile {
    mcp_servers: HashMap<String, McpServerConfig>,
}

/// Information about an available MCP server
#[derive(Debug, Clone, Serialize)]
struct McpServerInfo {
    name: String,
    command: String,
    source: String,
}

/// Run the session command
pub fn run(
    mcp: Option<Vec<String>>,
    profile: Option<String>,
    list: bool,
    dry_run: bool,
    format: Option<OutputFormat>,
    claude_args: Vec<String>,
    config: &Config,
) -> Result<()> {
    if list {
        return list_mcps(OutputFormat::resolve(format), config);
    }

    // Determine which MCPs to load
    let mcp_list = resolve_mcp_list(mcp, profile, config)?;

    // Build the MCP config JSON
    let (temp_path, server_count) = if mcp_list.is_empty() {
        (None, 0)
    } else {
        let (path, count) = build_mcp_config(&mcp_list, config)?;
        (Some(path), count)
    };

    if dry_run {
        println!("{}", "Dry run - would launch Claude with:".yellow());
        println!(
            "  MCPs: {}",
            if mcp_list.is_empty() { "none".to_string() } else { mcp_list.join(", ") }
        );
        println!("  Servers found: {}", server_count);
        if let Some(ref path) = temp_path {
            println!("  Config file: {}", path.display());
            if let Ok(content) = fs::read_to_string(path) {
                println!("\n{}", "Generated config:".dimmed());
                println!("{}", content);
            }
        }
        println!("  Extra args: {:?}", claude_args);
        return Ok(());
    }

    // Build and exec claude command
    launch_claude(temp_path, claude_args)
}

/// Resolve which MCPs to load based on flags and config
fn resolve_mcp_list(mcp: Option<Vec<String>>, profile: Option<String>, config: &Config) -> Result<Vec<String>> {
    // Explicit --mcp flag takes precedence
    if let Some(mcps) = mcp {
        return Ok(mcps);
    }

    // Then check for --profile
    if let Some(profile_name) = profile {
        return config.mcp.profiles.get(&profile_name).cloned().ok_or_else(|| {
            eyre!(
                "Unknown profile '{}'. Use --list to see available profiles.",
                profile_name
            )
        });
    }

    // Then check for default profile
    if let Some(ref default_name) = config.mcp.default_profile
        && let Some(mcps) = config.mcp.profiles.get(default_name)
    {
        return Ok(mcps.clone());
    }

    // No MCPs specified - return empty (will use --strict-mcp-config with empty config)
    // This gives a "minimal" session with no MCPs
    Ok(vec![])
}

/// Load all available MCP servers from sources and config
fn load_all_mcp_servers(config: &Config) -> HashMap<String, (McpServerConfig, String)> {
    let mut servers: HashMap<String, (McpServerConfig, String)> = HashMap::new();

    // Load from source files (in order, first wins)
    for source_path in &config.mcp.sources {
        let expanded = Config::expand_path(source_path);
        if expanded.exists()
            && let Ok(content) = fs::read_to_string(&expanded)
            && let Ok(mcp_file) = serde_json::from_str::<McpJsonFile>(&content)
        {
            let source_str = expanded.display().to_string();
            for (name, server_config) in mcp_file.mcp_servers {
                servers.entry(name).or_insert((server_config, source_str.clone()));
            }
        }
    }

    // Also check ~/.mcp.json as a default source
    if let Some(home) = dirs::home_dir() {
        let default_mcp = home.join(".mcp.json");
        if default_mcp.exists()
            && !config.mcp.sources.iter().any(|p| Config::expand_path(p) == default_mcp)
            && let Ok(content) = fs::read_to_string(&default_mcp)
            && let Ok(mcp_file) = serde_json::from_str::<McpJsonFile>(&content)
        {
            let source_str = default_mcp.display().to_string();
            for (name, server_config) in mcp_file.mcp_servers {
                servers.entry(name).or_insert((server_config, source_str.clone()));
            }
        }
    }

    // Add servers defined directly in pais.yaml (highest priority - overwrites)
    for (name, server_config) in &config.mcp.servers {
        servers.insert(name.clone(), (server_config.clone(), "pais.yaml".to_string()));
    }

    servers
}

/// Build MCP config JSON file with only the requested servers
fn build_mcp_config(mcp_list: &[String], config: &Config) -> Result<(PathBuf, usize)> {
    let all_servers = load_all_mcp_servers(config);

    let mut selected_servers: HashMap<String, McpServerConfig> = HashMap::new();
    let mut not_found: Vec<String> = vec![];

    for name in mcp_list {
        if let Some((server_config, _source)) = all_servers.get(name) {
            selected_servers.insert(name.clone(), server_config.clone());
        } else {
            not_found.push(name.clone());
        }
    }

    if !not_found.is_empty() {
        return Err(eyre!(
            "MCP server(s) not found: {}. Use --list to see available servers.",
            not_found.join(", ")
        ));
    }

    let count = selected_servers.len();

    // Build the JSON structure
    let mcp_json = McpJsonFile {
        mcp_servers: selected_servers,
    };

    // Write to temp file
    let temp_dir = std::env::temp_dir();
    let temp_file = temp_dir.join(format!("pais-mcp-{}.json", std::process::id()));

    let json_content = serde_json::to_string_pretty(&mcp_json).context("Failed to serialize MCP config")?;

    fs::write(&temp_file, &json_content).context("Failed to write temp MCP config file")?;

    log::debug!("Wrote MCP config to: {}", temp_file.display());
    log::debug!("MCP config content:\n{}", json_content);

    Ok((temp_file, count))
}

/// Launch Claude Code with the specified MCP config
fn launch_claude(mcp_config_path: Option<PathBuf>, extra_args: Vec<String>) -> Result<()> {
    let mut cmd = Command::new("claude");

    // Always use strict mode - only load what we specify
    cmd.arg("--strict-mcp-config");

    // Add our MCP config if we have one
    if let Some(ref path) = mcp_config_path {
        cmd.arg("--mcp-config");
        cmd.arg(path);
    }

    // Pass through any extra args
    cmd.args(&extra_args);

    log::info!("Launching Claude with args: {:?}", cmd.get_args().collect::<Vec<_>>());

    // exec() replaces this process with claude
    // This never returns on success
    let err = cmd.exec();

    Err(eyre!("Failed to exec claude: {}", err))
}

/// List available MCPs and profiles
fn list_mcps(format: OutputFormat, config: &Config) -> Result<()> {
    let all_servers = load_all_mcp_servers(config);

    match format {
        OutputFormat::Json => {
            #[derive(Serialize)]
            struct ListOutput {
                servers: Vec<McpServerInfo>,
                profiles: HashMap<String, Vec<String>>,
                default_profile: Option<String>,
            }

            let servers: Vec<McpServerInfo> = all_servers
                .iter()
                .map(|(name, (cfg, source))| McpServerInfo {
                    name: name.clone(),
                    command: cfg.command.clone(),
                    source: source.clone(),
                })
                .collect();

            let output = ListOutput {
                servers,
                profiles: config.mcp.profiles.clone(),
                default_profile: config.mcp.default_profile.clone(),
            };

            println!("{}", serde_json::to_string_pretty(&output)?);
        }
        OutputFormat::Yaml => {
            #[derive(Serialize)]
            struct ListOutput {
                servers: Vec<McpServerInfo>,
                profiles: HashMap<String, Vec<String>>,
                default_profile: Option<String>,
            }

            let servers: Vec<McpServerInfo> = all_servers
                .iter()
                .map(|(name, (cfg, source))| McpServerInfo {
                    name: name.clone(),
                    command: cfg.command.clone(),
                    source: source.clone(),
                })
                .collect();

            let output = ListOutput {
                servers,
                profiles: config.mcp.profiles.clone(),
                default_profile: config.mcp.default_profile.clone(),
            };

            println!("{}", serde_yaml::to_string(&output)?);
        }
        OutputFormat::Text => {
            // Servers section
            println!("{}", "Available MCP Servers:".bold());
            if all_servers.is_empty() {
                println!("  {}", "(none found)".dimmed());
                println!();
                println!("  Add servers to ~/.mcp.json or configure in pais.yaml");
            } else {
                let mut server_list: Vec<_> = all_servers.iter().collect();
                server_list.sort_by_key(|(name, _)| name.as_str());

                for (name, (cfg, source)) in server_list {
                    println!(
                        "  {} {} {}",
                        name.cyan(),
                        format!("({})", cfg.command).dimmed(),
                        format!("[{}]", source).dimmed()
                    );
                }
            }

            // Profiles section
            println!();
            println!("{}", "Profiles:".bold());
            if config.mcp.profiles.is_empty() {
                println!("  {}", "(none defined)".dimmed());
                println!();
                println!("  Define profiles in pais.yaml under mcp.profiles");
            } else {
                let mut profile_list: Vec<_> = config.mcp.profiles.iter().collect();
                profile_list.sort_by_key(|(name, _)| name.as_str());

                for (name, servers) in profile_list {
                    let default_marker = if config.mcp.default_profile.as_ref() == Some(name) {
                        " (default)".green().to_string()
                    } else {
                        String::new()
                    };

                    let server_str = if servers.is_empty() {
                        "(no MCPs)".dimmed().to_string()
                    } else {
                        servers.join(", ")
                    };

                    println!("  {}{}: {}", name.yellow(), default_marker, server_str);
                }
            }

            // Usage hints
            println!();
            println!("{}", "Usage:".bold());
            println!("  pais session                    # Use default profile or no MCPs");
            println!("  pais session --profile minimal  # Use 'minimal' profile");
            println!("  pais session --mcp github,slack # Load specific MCPs");
            println!("  pais session --dry-run          # Show what would happen");
        }
    }

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_resolve_mcp_list_explicit() {
        let config = Config::default();
        let result = resolve_mcp_list(Some(vec!["github".to_string()]), None, &config);
        assert!(result.is_ok());
        assert_eq!(result.unwrap(), vec!["github".to_string()]);
    }

    #[test]
    fn test_resolve_mcp_list_empty() {
        let config = Config::default();
        let result = resolve_mcp_list(None, None, &config);
        assert!(result.is_ok());
        assert!(result.unwrap().is_empty());
    }

    #[test]
    fn test_mcp_json_deserialize() {
        let json = r#"{
            "mcpServers": {
                "slack": {
                    "command": "npx",
                    "args": ["-y", "@modelcontextprotocol/server-slack"],
                    "env": {
                        "SLACK_BOT_TOKEN": "${SLACK_BOT_TOKEN}"
                    }
                }
            }
        }"#;

        let parsed: McpJsonFile = serde_json::from_str(json).unwrap();
        assert!(parsed.mcp_servers.contains_key("slack"));
        assert_eq!(parsed.mcp_servers["slack"].command, "npx");
    }
}
