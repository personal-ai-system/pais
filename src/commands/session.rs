//! Session command - Launch Claude Code with dynamic MCP and skill configuration
//!
//! This module provides `pais session` which wraps Claude Code startup,
//! allowing selective MCP server and skill loading to reduce context token usage.
//!
//! ## Usage
//!
//! ```bash
//! # Launch with specific MCPs (repeated flags)
//! pais session -m github -m slack
//!
//! # Launch with specific skills
//! pais session -s rust-coder -s otto
//!
//! # Combine MCPs and skills
//! pais session -m github -s rust-coder -s otto
//!
//! # Use named profiles
//! pais session --mcp-profile work --skill-profile dev
//!
//! # List available MCPs, skills, and profiles
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

/// Information about an available skill
#[derive(Debug, Clone, Serialize)]
struct SkillInfo {
    name: String,
    description: String,
}

/// MCP-related options
pub struct McpOptions {
    pub servers: Option<Vec<String>>,
    pub profile: Option<String>,
}

/// Skill-related options
pub struct SkillOptions {
    pub skills: Option<Vec<String>>,
    pub profile: Option<String>,
}

/// Session command options
pub struct SessionOptions {
    pub mcp: McpOptions,
    pub skill: SkillOptions,
    pub list: bool,
    pub dry_run: bool,
    pub format: Option<OutputFormat>,
    pub claude_args: Vec<String>,
}

/// Run the session command
pub fn run(opts: SessionOptions, config: &Config) -> Result<()> {
    if opts.list {
        return list_all(OutputFormat::resolve(opts.format), config);
    }

    // Determine which MCPs to load
    let mcp_list = resolve_mcp_list(opts.mcp.servers, opts.mcp.profile, config)?;

    // Determine which skills to load
    let skill_list = resolve_skill_list(opts.skill.skills, opts.skill.profile, config)?;

    // Build the MCP config JSON
    let (temp_path, server_count) = if mcp_list.is_empty() {
        (None, 0)
    } else {
        let (path, count) = build_mcp_config(&mcp_list, config)?;
        (Some(path), count)
    };

    if opts.dry_run {
        println!("{}", "Dry run - would launch Claude with:".yellow());
        println!(
            "  MCPs: {}",
            if mcp_list.is_empty() { "none".to_string() } else { mcp_list.join(", ") }
        );
        println!("  MCP servers found: {}", server_count);
        println!(
            "  Skills: {}",
            if skill_list.is_empty() { "(all)".to_string() } else { skill_list.join(", ") }
        );
        if let Some(ref path) = temp_path {
            println!("  MCP config file: {}", path.display());
            if let Ok(content) = fs::read_to_string(path) {
                println!("\n{}", "Generated MCP config:".dimmed());
                println!("{}", content);
            }
        }
        if !skill_list.is_empty() {
            println!("  PAIS_SKILLS env: {}", skill_list.join(","));
        }
        println!("  Extra args: {:?}", opts.claude_args);
        return Ok(());
    }

    // Build and exec claude command
    launch_claude(temp_path, skill_list, opts.claude_args)
}

/// Resolve which MCPs to load based on flags and config
fn resolve_mcp_list(mcp: Option<Vec<String>>, mcp_profile: Option<String>, config: &Config) -> Result<Vec<String>> {
    // Explicit --mcp flag takes precedence
    if let Some(mcps) = mcp {
        return Ok(mcps);
    }

    // Then check for --mcp-profile
    if let Some(profile_name) = mcp_profile {
        return config.mcp.profiles.get(&profile_name).cloned().ok_or_else(|| {
            eyre!(
                "Unknown MCP profile '{}'. Use --list to see available profiles.",
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

/// Resolve which skills to load based on flags and config
fn resolve_skill_list(
    skill: Option<Vec<String>>,
    skill_profile: Option<String>,
    config: &Config,
) -> Result<Vec<String>> {
    // Explicit --skill flag takes precedence
    if let Some(skills) = skill {
        return Ok(skills);
    }

    // Then check for --skill-profile
    if let Some(profile_name) = skill_profile {
        return config.skills.profiles.get(&profile_name).cloned().ok_or_else(|| {
            eyre!(
                "Unknown skill profile '{}'. Use --list to see available profiles.",
                profile_name
            )
        });
    }

    // Then check for default profile
    if let Some(ref default_name) = config.skills.default_profile
        && let Some(skills) = config.skills.profiles.get(default_name)
    {
        return Ok(skills.clone());
    }

    // No skills specified - return empty (means "load all skills")
    Ok(vec![])
}

/// Load all available skills from the skills directory
fn load_all_skills(config: &Config) -> Vec<SkillInfo> {
    let skills_dir = Config::expand_path(&config.paths.skills);

    let mut skills = Vec::new();

    if !skills_dir.exists() {
        return skills;
    }

    if let Ok(entries) = fs::read_dir(&skills_dir) {
        for entry in entries.flatten() {
            let path = entry.path();
            if path.is_dir() {
                let skill_md = path.join("SKILL.md");
                if skill_md.exists() {
                    let name = path
                        .file_name()
                        .map(|n| n.to_string_lossy().to_string())
                        .unwrap_or_default();

                    // Try to extract description from SKILL.md frontmatter
                    let description = fs::read_to_string(&skill_md)
                        .ok()
                        .and_then(|content| extract_skill_description(&content))
                        .unwrap_or_default();

                    skills.push(SkillInfo { name, description });
                }
            }
        }
    }

    skills.sort_by(|a, b| a.name.cmp(&b.name));
    skills
}

/// Extract description from SKILL.md frontmatter
fn extract_skill_description(content: &str) -> Option<String> {
    // Look for YAML frontmatter
    let stripped = content.strip_prefix("---")?;
    let end = stripped.find("---")?;
    let frontmatter = &stripped[..end];

    for line in frontmatter.lines() {
        if let Some(desc) = line.strip_prefix("description:") {
            return Some(desc.trim().trim_matches('"').trim_matches('\'').to_string());
        }
    }
    None
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

/// Launch Claude Code with the specified MCP and skill config
fn launch_claude(mcp_config_path: Option<PathBuf>, skill_list: Vec<String>, extra_args: Vec<String>) -> Result<()> {
    let mut cmd = Command::new("claude");

    // Always use strict mode - only load what we specify
    cmd.arg("--strict-mcp-config");

    // Add our MCP config if we have one
    if let Some(ref path) = mcp_config_path {
        cmd.arg("--mcp-config");
        cmd.arg(path);
    }

    // Set PAIS_SKILLS env var for skill filtering
    // Empty list means "load all skills" (no filtering)
    if !skill_list.is_empty() {
        let skills_env = skill_list.join(",");
        cmd.env("PAIS_SKILLS", &skills_env);
        log::info!("Setting PAIS_SKILLS={}", skills_env);
    }

    // Pass through any extra args
    cmd.args(&extra_args);

    log::info!("Launching Claude with args: {:?}", cmd.get_args().collect::<Vec<_>>());

    // exec() replaces this process with claude
    // This never returns on success
    let err = cmd.exec();

    Err(eyre!("Failed to exec claude: {}", err))
}

/// List available MCPs, skills, and profiles
fn list_all(format: OutputFormat, config: &Config) -> Result<()> {
    let all_servers = load_all_mcp_servers(config);
    let all_skills = load_all_skills(config);

    match format {
        OutputFormat::Json => {
            #[derive(Serialize)]
            struct ListOutput {
                mcp_servers: Vec<McpServerInfo>,
                mcp_profiles: HashMap<String, Vec<String>>,
                mcp_default_profile: Option<String>,
                skills: Vec<SkillInfo>,
                skill_profiles: HashMap<String, Vec<String>>,
                skill_default_profile: Option<String>,
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
                mcp_servers: servers,
                mcp_profiles: config.mcp.profiles.clone(),
                mcp_default_profile: config.mcp.default_profile.clone(),
                skills: all_skills,
                skill_profiles: config.skills.profiles.clone(),
                skill_default_profile: config.skills.default_profile.clone(),
            };

            println!("{}", serde_json::to_string_pretty(&output)?);
        }
        OutputFormat::Yaml => {
            #[derive(Serialize)]
            struct ListOutput {
                mcp_servers: Vec<McpServerInfo>,
                mcp_profiles: HashMap<String, Vec<String>>,
                mcp_default_profile: Option<String>,
                skills: Vec<SkillInfo>,
                skill_profiles: HashMap<String, Vec<String>>,
                skill_default_profile: Option<String>,
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
                mcp_servers: servers,
                mcp_profiles: config.mcp.profiles.clone(),
                mcp_default_profile: config.mcp.default_profile.clone(),
                skills: all_skills,
                skill_profiles: config.skills.profiles.clone(),
                skill_default_profile: config.skills.default_profile.clone(),
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
                let mut profile_list: Vec<_> = config.mcp.profiles.iter().collect();
                profile_list.sort_by_key(|(name, _)| name.as_str());

                for (name, servers) in profile_list {
                    let default_marker = if config.mcp.default_profile.as_ref() == Some(name) {
                        " (default)".green().to_string()
                    } else {
                        String::new()
                    };

                    let server_str = if servers.is_empty() {
                        "(none)".dimmed().to_string()
                    } else {
                        servers.join(", ")
                    };

                    println!("  {}{}: {}", name.yellow(), default_marker, server_str);
                }
            }

            // Skills section
            println!();
            println!("{}", "Skills:".bold());
            if all_skills.is_empty() {
                println!("  {}", "(none found)".dimmed());
            } else {
                for skill in &all_skills {
                    let desc = if skill.description.is_empty() {
                        String::new()
                    } else {
                        // Truncate long descriptions
                        let truncated = if skill.description.len() > 50 {
                            format!("{}...", &skill.description[..47])
                        } else {
                            skill.description.clone()
                        };
                        format!(" - {}", truncated).dimmed().to_string()
                    };
                    println!("  {}{}", skill.name.cyan(), desc);
                }
            }

            // Skill Profiles section
            println!();
            println!("{}", "Skill Profiles:".bold());
            if config.skills.profiles.is_empty() {
                println!("  {}", "(none defined)".dimmed());
            } else {
                let mut profile_list: Vec<_> = config.skills.profiles.iter().collect();
                profile_list.sort_by_key(|(name, _)| name.as_str());

                for (name, skills) in profile_list {
                    let default_marker = if config.skills.default_profile.as_ref() == Some(name) {
                        " (default)".green().to_string()
                    } else {
                        String::new()
                    };

                    let skill_str = if skills.is_empty() {
                        "(none)".dimmed().to_string()
                    } else {
                        skills.join(", ")
                    };

                    println!("  {}{}: {}", name.yellow(), default_marker, skill_str);
                }
            }

            // Usage hints
            println!();
            println!("{}", "Usage:".bold());
            println!("  pais session                         # Use default profiles");
            println!("  pais session -m github -m slack      # Load specific MCPs");
            println!("  pais session -s rust-coder -s otto   # Load specific skills");
            println!("  pais session --mcp-profile work --skill-profile dev");
            println!("  pais session --dry-run               # Show what would happen");
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
