//! Session command - Launch Claude Code with dynamic MCP and skill configuration
//!
//! This module provides `pais session` which wraps Claude Code startup,
//! allowing selective MCP server and skill loading to reduce context token usage.
//!
//! ## Usage
//!
//! ```bash
//! # Launch with specific MCPs (names or profiles)
//! pais session -m github -m slack
//! pais session -m work  # expands profile
//!
//! # Launch with specific skills (names or profiles)
//! pais session -s rust-coder -s otto
//! pais session -s dev  # expands profile
//!
//! # Combined
//! pais session -m work -s dev
//!
//! # List available MCPs, skills, and profiles
//! pais session --list
//! ```

use colored::Colorize;
use eyre::{Context, Result, eyre};
use indexmap::IndexMap;
use serde::{Deserialize, Serialize};
use std::collections::{HashMap, HashSet};
use std::fs;
use std::os::unix::process::CommandExt;
use std::path::PathBuf;
use std::process::Command;

use crate::cli::OutputFormat;
use crate::config::{Config, McpServerConfig};
use crate::skill::indexer::generate_index;

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

/// Information about an available skill
#[derive(Debug, Clone, Serialize)]
struct SkillInfo {
    name: String,
    description: String,
    tier: String,
}

/// Run the session command
pub fn run(
    mcp: Option<Vec<String>>,
    skill: Option<Vec<String>>,
    list: bool,
    dry_run: bool,
    format: Option<OutputFormat>,
    claude_args: Vec<String>,
    config: &Config,
) -> Result<()> {
    if list {
        return list_all(OutputFormat::resolve(format), config);
    }

    // Resolve which MCPs to load (expand profiles, apply defaults)
    let mcp_list = resolve_list(mcp, &config.mcp.profiles);

    // Resolve which skills to load (expand profiles, apply defaults)
    let skill_list = resolve_list(skill, &config.skills.profiles);

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
        println!("  MCP servers found: {}", server_count);
        println!(
            "  Skills: {}",
            if skill_list.is_empty() {
                "all (no filter)".to_string()
            } else {
                skill_list.join(", ")
            }
        );
        if let Some(ref path) = temp_path {
            println!("  MCP config file: {}", path.display());
            if let Ok(content) = fs::read_to_string(path) {
                println!("\n{}", "Generated MCP config:".dimmed());
                println!("{}", content);
            }
        }
        if !skill_list.is_empty() {
            println!("  PAIS_SKILLS: {}", skill_list.join(","));
        }
        println!("  Extra args: {:?}", claude_args);
        return Ok(());
    }

    // Build and exec claude command
    launch_claude(temp_path, skill_list, claude_args)
}

/// Expand a list of names, replacing profile names with their contents
/// If input is None, returns the first profile's contents as default
fn resolve_list(input: Option<Vec<String>>, profiles: &IndexMap<String, Vec<String>>) -> Vec<String> {
    match input {
        Some(names) => expand_names(&names, profiles),
        None => get_default(profiles),
    }
}

/// Expand names, replacing profile names with their contents
fn expand_names(names: &[String], profiles: &IndexMap<String, Vec<String>>) -> Vec<String> {
    let mut result = Vec::new();
    for name in names {
        if let Some(profile_contents) = profiles.get(name) {
            // It's a profile - expand it
            result.extend(profile_contents.iter().cloned());
        } else {
            // It's a direct name
            result.push(name.clone());
        }
    }
    // Deduplicate while preserving order
    let mut seen = HashSet::new();
    result.retain(|x| seen.insert(x.clone()));
    result
}

/// Get default from first profile (if any)
fn get_default(profiles: &IndexMap<String, Vec<String>>) -> Vec<String> {
    profiles.values().next().cloned().unwrap_or_default()
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

/// Launch Claude Code with the specified MCP config and skill filter
fn launch_claude(mcp_config_path: Option<PathBuf>, skill_list: Vec<String>, extra_args: Vec<String>) -> Result<()> {
    let mut cmd = Command::new("claude");

    // Always use strict mode - only load what we specify
    cmd.arg("--strict-mcp-config");

    // Add our MCP config if we have one
    if let Some(ref path) = mcp_config_path {
        cmd.arg("--mcp-config");
        cmd.arg(path);
    }

    // Set skill filter via environment variable
    // Empty list means "no filter" (load all skills)
    // Non-empty list means "only these skills"
    if !skill_list.is_empty() {
        cmd.env("PAIS_SKILLS", skill_list.join(","));
    }

    // Pass through any extra args
    cmd.args(&extra_args);

    log::info!("Launching Claude with args: {:?}", cmd.get_args().collect::<Vec<_>>());
    if !skill_list.is_empty() {
        log::info!("PAIS_SKILLS={}", skill_list.join(","));
    }

    // exec() replaces this process with claude
    // This never returns on success
    let err = cmd.exec();

    Err(eyre!("Failed to exec claude: {}", err))
}

/// List available MCPs, skills, and profiles
fn list_all(format: OutputFormat, config: &Config) -> Result<()> {
    let all_servers = load_all_mcp_servers(config);

    // Load skills from index
    let skills_dir = Config::expand_path(&config.paths.skills);
    let skill_index = generate_index(&skills_dir).ok();

    match format {
        OutputFormat::Json => {
            #[derive(Serialize)]
            struct ListOutput {
                mcp_servers: Vec<McpServerInfo>,
                mcp_profiles: IndexMap<String, Vec<String>>,
                mcp_default: Option<String>,
                skills: Vec<SkillInfo>,
                skill_profiles: IndexMap<String, Vec<String>>,
                skill_default: Option<String>,
            }

            let servers: Vec<McpServerInfo> = all_servers
                .iter()
                .map(|(name, (cfg, source))| McpServerInfo {
                    name: name.clone(),
                    command: cfg.command.clone(),
                    source: source.clone(),
                })
                .collect();

            let skills: Vec<SkillInfo> = skill_index
                .as_ref()
                .map(|idx| {
                    idx.skills
                        .values()
                        .map(|s| SkillInfo {
                            name: s.name.clone(),
                            description: s.description.clone(),
                            tier: format!("{:?}", s.tier),
                        })
                        .collect()
                })
                .unwrap_or_default();

            let output = ListOutput {
                mcp_servers: servers,
                mcp_profiles: config.mcp.profiles.clone(),
                mcp_default: config.mcp.profiles.keys().next().cloned(),
                skills,
                skill_profiles: config.skills.profiles.clone(),
                skill_default: config.skills.profiles.keys().next().cloned(),
            };

            println!("{}", serde_json::to_string_pretty(&output)?);
        }
        OutputFormat::Yaml => {
            #[derive(Serialize)]
            struct ListOutput {
                mcp_servers: Vec<McpServerInfo>,
                mcp_profiles: IndexMap<String, Vec<String>>,
                mcp_default: Option<String>,
                skills: Vec<SkillInfo>,
                skill_profiles: IndexMap<String, Vec<String>>,
                skill_default: Option<String>,
            }

            let servers: Vec<McpServerInfo> = all_servers
                .iter()
                .map(|(name, (cfg, source))| McpServerInfo {
                    name: name.clone(),
                    command: cfg.command.clone(),
                    source: source.clone(),
                })
                .collect();

            let skills: Vec<SkillInfo> = skill_index
                .as_ref()
                .map(|idx| {
                    idx.skills
                        .values()
                        .map(|s| SkillInfo {
                            name: s.name.clone(),
                            description: s.description.clone(),
                            tier: format!("{:?}", s.tier),
                        })
                        .collect()
                })
                .unwrap_or_default();

            let output = ListOutput {
                mcp_servers: servers,
                mcp_profiles: config.mcp.profiles.clone(),
                mcp_default: config.mcp.profiles.keys().next().cloned(),
                skills,
                skill_profiles: config.skills.profiles.clone(),
                skill_default: config.skills.profiles.keys().next().cloned(),
            };

            println!("{}", serde_yaml::to_string(&output)?);
        }
        OutputFormat::Text => {
            // MCP Servers section
            println!("{}", "MCP Servers:".bold());
            if all_servers.is_empty() {
                println!("  {}", "(none found)".dimmed());
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

            // MCP Profiles section
            println!();
            println!("{}", "MCP Profiles:".bold());
            if config.mcp.profiles.is_empty() {
                println!("  {}", "(none defined)".dimmed());
            } else {
                let default_name = config.mcp.profiles.keys().next();
                for (name, servers) in &config.mcp.profiles {
                    let default_marker = if Some(name) == default_name {
                        " (default)".green().to_string()
                    } else {
                        String::new()
                    };

                    let server_str = if servers.is_empty() {
                        "(empty)".dimmed().to_string()
                    } else {
                        servers.join(", ")
                    };

                    println!("  {}{}: {}", name.yellow(), default_marker, server_str);
                }
            }

            // Skills section
            println!();
            println!("{}", "Skills:".bold());
            if let Some(ref idx) = skill_index {
                let mut skills: Vec<_> = idx.skills.values().collect();
                skills.sort_by_key(|s| &s.name);

                for skill in skills {
                    let tier_str = match skill.tier {
                        crate::skill::parser::SkillTier::Core => "(core)".green(),
                        crate::skill::parser::SkillTier::Deferred => "(deferred)".dimmed(),
                    };
                    println!("  {} {} {}", skill.name.cyan(), tier_str, skill.description.dimmed());
                }
            } else {
                println!("  {}", "(unable to load skill index)".dimmed());
            }

            // Skill Profiles section
            println!();
            println!("{}", "Skill Profiles:".bold());
            if config.skills.profiles.is_empty() {
                println!("  {}", "(none defined)".dimmed());
            } else {
                let default_name = config.skills.profiles.keys().next();
                for (name, skills) in &config.skills.profiles {
                    let default_marker = if Some(name) == default_name {
                        " (default)".green().to_string()
                    } else {
                        String::new()
                    };

                    let skill_str = if skills.is_empty() {
                        "(empty - loads all skills)".dimmed().to_string()
                    } else {
                        skills.join(", ")
                    };

                    println!("  {}{}: {}", name.yellow(), default_marker, skill_str);
                }
            }

            // Usage hints
            println!();
            println!("{}", "Usage:".bold());
            println!("  pais session                    # Use default profiles");
            println!("  pais session -m work -s dev     # Use specific profiles");
            println!("  pais session -m github,slack    # Load specific MCPs");
            println!("  pais session -s rust-coder,otto # Load specific skills");
            println!("  pais session --dry-run          # Show what would happen");
        }
    }

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_expand_names_direct() {
        let profiles = IndexMap::new();
        let result = expand_names(&["foo".to_string(), "bar".to_string()], &profiles);
        assert_eq!(result, vec!["foo", "bar"]);
    }

    #[test]
    fn test_expand_names_with_profile() {
        let mut profiles = IndexMap::new();
        profiles.insert("dev".to_string(), vec!["rust-coder".to_string(), "otto".to_string()]);

        let result = expand_names(&["dev".to_string()], &profiles);
        assert_eq!(result, vec!["rust-coder", "otto"]);
    }

    #[test]
    fn test_expand_names_mixed() {
        let mut profiles = IndexMap::new();
        profiles.insert("dev".to_string(), vec!["rust-coder".to_string(), "otto".to_string()]);

        let result = expand_names(&["dev".to_string(), "fabric".to_string()], &profiles);
        assert_eq!(result, vec!["rust-coder", "otto", "fabric"]);
    }

    #[test]
    fn test_expand_names_deduplicates() {
        let mut profiles = IndexMap::new();
        profiles.insert("dev".to_string(), vec!["rust-coder".to_string(), "otto".to_string()]);

        // rust-coder appears in profile and as direct name
        let result = expand_names(&["dev".to_string(), "rust-coder".to_string()], &profiles);
        assert_eq!(result, vec!["rust-coder", "otto"]);
    }

    #[test]
    fn test_get_default_empty() {
        let profiles: IndexMap<String, Vec<String>> = IndexMap::new();
        let result = get_default(&profiles);
        assert!(result.is_empty());
    }

    #[test]
    fn test_get_default_first_profile() {
        let mut profiles = IndexMap::new();
        profiles.insert("first".to_string(), vec!["a".to_string(), "b".to_string()]);
        profiles.insert("second".to_string(), vec!["c".to_string()]);

        let result = get_default(&profiles);
        assert_eq!(result, vec!["a", "b"]);
    }

    #[test]
    fn test_resolve_list_with_input() {
        let mut profiles = IndexMap::new();
        profiles.insert("dev".to_string(), vec!["rust-coder".to_string()]);

        let result = resolve_list(Some(vec!["fabric".to_string()]), &profiles);
        assert_eq!(result, vec!["fabric"]);
    }

    #[test]
    fn test_resolve_list_without_input() {
        let mut profiles = IndexMap::new();
        profiles.insert("dev".to_string(), vec!["rust-coder".to_string()]);

        let result = resolve_list(None, &profiles);
        assert_eq!(result, vec!["rust-coder"]);
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

    // === Additional positive tests ===

    #[test]
    fn test_expand_names_multiple_profiles() {
        let mut profiles = IndexMap::new();
        profiles.insert("dev".to_string(), vec!["rust-coder".to_string(), "otto".to_string()]);
        profiles.insert(
            "research".to_string(),
            vec!["fabric".to_string(), "youtube".to_string()],
        );

        let result = expand_names(&["dev".to_string(), "research".to_string()], &profiles);
        assert_eq!(result, vec!["rust-coder", "otto", "fabric", "youtube"]);
    }

    #[test]
    fn test_expand_names_profile_with_overlapping_items() {
        let mut profiles = IndexMap::new();
        profiles.insert("dev".to_string(), vec!["rust-coder".to_string(), "fabric".to_string()]);
        profiles.insert(
            "research".to_string(),
            vec!["fabric".to_string(), "youtube".to_string()],
        );

        // fabric appears in both profiles - should deduplicate
        let result = expand_names(&["dev".to_string(), "research".to_string()], &profiles);
        assert_eq!(result, vec!["rust-coder", "fabric", "youtube"]);
    }

    #[test]
    fn test_resolve_list_expands_profile_in_input() {
        let mut profiles = IndexMap::new();
        profiles.insert("dev".to_string(), vec!["rust-coder".to_string(), "otto".to_string()]);

        // When user provides -s dev, it should expand the profile
        let result = resolve_list(Some(vec!["dev".to_string()]), &profiles);
        assert_eq!(result, vec!["rust-coder", "otto"]);
    }

    #[test]
    fn test_get_default_empty_first_profile() {
        let mut profiles = IndexMap::new();
        profiles.insert("minimal".to_string(), vec![]); // First = default, empty
        profiles.insert("dev".to_string(), vec!["rust-coder".to_string()]);

        let result = get_default(&profiles);
        assert!(result.is_empty()); // minimal profile is empty
    }

    #[test]
    fn test_resolve_list_none_with_empty_first_profile() {
        let mut profiles = IndexMap::new();
        profiles.insert("minimal".to_string(), vec![]); // First = default
        profiles.insert("dev".to_string(), vec!["rust-coder".to_string()]);

        // No input → uses default (first profile which is empty)
        let result = resolve_list(None, &profiles);
        assert!(result.is_empty());
    }

    // === Negative tests ===

    #[test]
    fn test_expand_names_unknown_profile_treated_as_literal() {
        let mut profiles = IndexMap::new();
        profiles.insert("dev".to_string(), vec!["rust-coder".to_string()]);

        // "unknown" is not a profile, so treated as a literal skill name
        let result = expand_names(&["unknown".to_string()], &profiles);
        assert_eq!(result, vec!["unknown"]);
    }

    #[test]
    fn test_expand_names_empty_input() {
        let mut profiles = IndexMap::new();
        profiles.insert("dev".to_string(), vec!["rust-coder".to_string()]);

        let result = expand_names(&[], &profiles);
        assert!(result.is_empty());
    }

    #[test]
    fn test_resolve_list_empty_vec_input() {
        let mut profiles = IndexMap::new();
        profiles.insert("dev".to_string(), vec!["rust-coder".to_string()]);

        // Empty vec provided → returns empty (not default)
        let result = resolve_list(Some(vec![]), &profiles);
        assert!(result.is_empty());
    }

    #[test]
    fn test_expand_names_preserves_order() {
        let profiles = IndexMap::new();

        let result = expand_names(&["c".to_string(), "a".to_string(), "b".to_string()], &profiles);
        assert_eq!(result, vec!["c", "a", "b"]);
    }

    #[test]
    fn test_expand_names_profile_order_preserved() {
        let mut profiles = IndexMap::new();
        profiles.insert(
            "dev".to_string(),
            vec!["z".to_string(), "a".to_string(), "m".to_string()],
        );

        let result = expand_names(&["dev".to_string()], &profiles);
        assert_eq!(result, vec!["z", "a", "m"]); // Order from profile preserved
    }

    #[test]
    fn test_get_default_respects_insertion_order() {
        let mut profiles = IndexMap::new();
        // Insert in specific order
        profiles.insert("second".to_string(), vec!["b".to_string()]);
        profiles.insert("first".to_string(), vec!["a".to_string()]);

        // IndexMap preserves insertion order, so "second" is first
        let result = get_default(&profiles);
        assert_eq!(result, vec!["b"]);
    }

    // === MCP-specific tests (same logic applies) ===

    #[test]
    fn test_mcp_profile_expansion() {
        let mut profiles = IndexMap::new();
        profiles.insert("work".to_string(), vec!["github".to_string(), "slack".to_string()]);
        profiles.insert("minimal".to_string(), vec![]);

        let result = resolve_list(Some(vec!["work".to_string()]), &profiles);
        assert_eq!(result, vec!["github", "slack"]);
    }

    #[test]
    fn test_mcp_mixed_profile_and_direct() {
        let mut profiles = IndexMap::new();
        profiles.insert("work".to_string(), vec!["github".to_string(), "slack".to_string()]);

        // User specifies profile + additional MCP
        let result = resolve_list(Some(vec!["work".to_string(), "jira".to_string()]), &profiles);
        assert_eq!(result, vec!["github", "slack", "jira"]);
    }

    #[test]
    fn test_mcp_default_is_first_profile() {
        let mut profiles = IndexMap::new();
        profiles.insert("minimal".to_string(), vec![]);
        profiles.insert("work".to_string(), vec!["github".to_string()]);

        // No flags → uses first profile (minimal = empty)
        let result = resolve_list(None, &profiles);
        assert!(result.is_empty());
    }
}
