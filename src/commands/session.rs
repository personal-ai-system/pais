//! Session command - Launch Claude Code with dynamic MCP and skill configuration
//!
//! This module provides `pais session` which wraps Claude Code startup,
//! allowing selective MCP server and skill loading to reduce context token usage.
//!
//! ## How It Works
//!
//! ### MCP Filtering
//! Claude Code supports `--mcp-config` and `--strict-mcp-config` flags natively.
//! We generate a temporary MCP config file with only the requested servers.
//!
//! ### Skill Filtering
//! Claude Code has NO native skill filtering - it loads all skills from `~/.claude/skills/`.
//! We work around this by managing symlinks in that directory:
//!
//! 1. Read current symlinks in `~/.claude/skills/`
//! 2. Compute requested skills from profile/flags
//! 3. Diff: remove symlinks not in requested set, add symlinks that are missing
//! 4. This avoids unnecessary churn - symlinks common to both sets are untouched
//!
//! Skills are sourced from:
//! - `~/.config/pais/skills/<name>/` (dedicated skills)
//! - `~/.config/pais/plugins/<name>/` (plugins with SKILL.md)
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
use std::os::unix::fs as unix_fs;
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

    // Sync skill symlinks in ~/.claude/skills/
    let sync_result = sync_skill_symlinks(&skill_list, config)?;

    if dry_run {
        println!("{}", "Dry run - would launch Claude with:".yellow());
        println!(
            "  MCPs: {}",
            if mcp_list.is_empty() { "none".to_string() } else { mcp_list.join(", ") }
        );
        println!("  MCP servers found: {}", server_count);
        println!(
            "  Skills: {}",
            if skill_list.is_empty() { "all".to_string() } else { skill_list.join(", ") }
        );
        println!();
        println!("{}", "Skill symlink changes:".bold());
        if sync_result.added.is_empty() && sync_result.removed.is_empty() {
            println!("  {}", "(no changes needed)".dimmed());
        } else {
            for name in &sync_result.removed {
                println!("  {} {}", "-".red(), name);
            }
            for name in &sync_result.added {
                println!("  {} {}", "+".green(), name);
            }
        }
        println!("  Unchanged: {}", sync_result.unchanged.len());
        if let Some(ref path) = temp_path {
            println!();
            println!("  MCP config file: {}", path.display());
            if let Ok(content) = fs::read_to_string(path) {
                println!("\n{}", "Generated MCP config:".dimmed());
                println!("{}", content);
            }
        }
        println!("  Extra args: {:?}", claude_args);
        return Ok(());
    }

    // Log what we did
    if !sync_result.added.is_empty() || !sync_result.removed.is_empty() {
        log::info!(
            "Synced skill symlinks: +{} -{} (unchanged: {})",
            sync_result.added.len(),
            sync_result.removed.len(),
            sync_result.unchanged.len()
        );
    }

    // Build and exec claude command
    launch_claude(temp_path, claude_args)
}

/// Result of syncing skill symlinks
#[derive(Debug, Default)]
struct SyncResult {
    added: Vec<String>,
    removed: Vec<String>,
    unchanged: Vec<String>,
    not_found: Vec<String>,
}

/// Get Claude Code's skills directory (~/.claude/skills/)
fn get_claude_skills_dir() -> Result<PathBuf> {
    let home = dirs::home_dir().ok_or_else(|| eyre!("Could not determine home directory"))?;
    Ok(home.join(".claude").join("skills"))
}

/// Find the source path for a skill (checks skills dir, then plugins dir)
/// Returns None if skill doesn't exist in either location
fn find_skill_source(name: &str, config: &Config) -> Option<PathBuf> {
    // Check dedicated skills directory first
    let skills_dir = Config::expand_path(&config.paths.skills);
    let skill_path = skills_dir.join(name);
    if skill_path.exists() && skill_path.join("SKILL.md").exists() {
        return Some(skill_path);
    }

    // Check plugins directory (plugins can have SKILL.md too)
    let plugins_dir = Config::expand_path(&config.paths.plugins);
    let plugin_path = plugins_dir.join(name);
    if plugin_path.exists() && plugin_path.join("SKILL.md").exists() {
        return Some(plugin_path);
    }

    None
}

/// Get all available skill names from both skills and plugins directories
fn get_all_skill_names(config: &Config) -> HashSet<String> {
    let mut names = HashSet::new();

    // From skills directory
    let skills_dir = Config::expand_path(&config.paths.skills);
    if let Ok(entries) = fs::read_dir(&skills_dir) {
        for entry in entries.flatten() {
            let path = entry.path();
            if path.is_dir()
                && path.join("SKILL.md").exists()
                && let Some(name) = path.file_name().and_then(|n| n.to_str())
            {
                names.insert(name.to_string());
            }
        }
    }

    // From plugins directory
    let plugins_dir = Config::expand_path(&config.paths.plugins);
    if let Ok(entries) = fs::read_dir(&plugins_dir) {
        for entry in entries.flatten() {
            let path = entry.path();
            if path.is_dir()
                && path.join("SKILL.md").exists()
                && let Some(name) = path.file_name().and_then(|n| n.to_str())
            {
                names.insert(name.to_string());
            }
        }
    }

    names
}

/// Get current skill symlinks in Claude's skills directory
/// Returns a map of symlink name -> target path
fn get_current_symlinks(claude_skills_dir: &PathBuf) -> HashMap<String, PathBuf> {
    let mut symlinks = HashMap::new();

    if let Ok(entries) = fs::read_dir(claude_skills_dir) {
        for entry in entries.flatten() {
            let path = entry.path();
            // Only consider symlinks, skip regular files like README.md
            if path.is_symlink()
                && let Some(name) = path.file_name().and_then(|n| n.to_str())
                && let Ok(target) = fs::read_link(&path)
            {
                symlinks.insert(name.to_string(), target);
            }
        }
    }

    symlinks
}

/// Sync skill symlinks in ~/.claude/skills/ to match the requested skill list
///
/// This performs a smart diff:
/// - Symlinks for skills not in the requested list are removed
/// - Symlinks for skills in the requested list but not present are added
/// - Symlinks that already exist and are in the requested list are left alone
///
/// If skill_list is empty, this loads ALL available skills (no filtering).
fn sync_skill_symlinks(skill_list: &[String], config: &Config) -> Result<SyncResult> {
    let claude_skills_dir = get_claude_skills_dir()?;

    // Ensure the directory exists
    fs::create_dir_all(&claude_skills_dir).context("Failed to create ~/.claude/skills/")?;

    // Get current state
    let current_symlinks = get_current_symlinks(&claude_skills_dir);
    let current_names: HashSet<String> = current_symlinks.keys().cloned().collect();

    // Determine requested skills
    let requested_names: HashSet<String> = if skill_list.is_empty() {
        // Empty list = load all available skills
        get_all_skill_names(config)
    } else {
        skill_list.iter().cloned().collect()
    };

    // Compute diff
    let to_remove: HashSet<_> = current_names.difference(&requested_names).cloned().collect();
    let to_add: HashSet<_> = requested_names.difference(&current_names).cloned().collect();
    let unchanged: HashSet<_> = current_names.intersection(&requested_names).cloned().collect();

    let mut result = SyncResult {
        unchanged: unchanged.into_iter().collect(),
        ..Default::default()
    };

    // Remove symlinks that shouldn't be there
    for name in &to_remove {
        let symlink_path = claude_skills_dir.join(name);
        if let Err(e) = fs::remove_file(&symlink_path) {
            log::warn!("Failed to remove symlink {}: {}", symlink_path.display(), e);
        } else {
            log::debug!("Removed skill symlink: {}", name);
            result.removed.push(name.clone());
        }
    }

    // Add symlinks that should be there
    for name in &to_add {
        if let Some(source_path) = find_skill_source(name, config) {
            let symlink_path = claude_skills_dir.join(name);
            if let Err(e) = unix_fs::symlink(&source_path, &symlink_path) {
                log::warn!(
                    "Failed to create symlink {} -> {}: {}",
                    symlink_path.display(),
                    source_path.display(),
                    e
                );
            } else {
                log::debug!("Created skill symlink: {} -> {}", name, source_path.display());
                result.added.push(name.clone());
            }
        } else {
            log::warn!("Skill not found: {}", name);
            result.not_found.push(name.clone());
        }
    }

    // Sort for consistent output
    result.added.sort();
    result.removed.sort();
    result.unchanged.sort();
    result.not_found.sort();

    Ok(result)
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

/// Launch Claude Code with the specified MCP config
///
/// Skill filtering is handled by sync_skill_symlinks() before this is called -
/// Claude Code loads whatever symlinks exist in ~/.claude/skills/.
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

    // === Symlink management tests ===
    //
    // These tests use isolated temp directories to avoid:
    // - Interfering with real ~/.claude/skills/
    // - Race conditions between parallel tests
    // - Leftover state from failed tests

    use std::sync::atomic::{AtomicU64, Ordering};
    use tempfile::TempDir;

    /// Atomic counter for unique test directory names
    static TEST_COUNTER: AtomicU64 = AtomicU64::new(0);

    /// Create an isolated test environment with:
    /// - A temp "claude skills" directory (simulates ~/.claude/skills/)
    /// - A temp "pais skills" directory (simulates ~/.config/pais/skills/)
    /// - A temp "pais plugins" directory (simulates ~/.config/pais/plugins/)
    struct TestEnv {
        _temp_dir: TempDir, // Holds the temp dir alive
        claude_skills_dir: PathBuf,
        pais_skills_dir: PathBuf,
        pais_plugins_dir: PathBuf,
    }

    impl TestEnv {
        fn new() -> Self {
            let id = TEST_COUNTER.fetch_add(1, Ordering::SeqCst);
            let temp_dir = TempDir::with_prefix(format!("pais-test-{}-", id)).unwrap();
            let base = temp_dir.path();

            let claude_skills_dir = base.join("claude-skills");
            let pais_skills_dir = base.join("pais-skills");
            let pais_plugins_dir = base.join("pais-plugins");

            fs::create_dir_all(&claude_skills_dir).unwrap();
            fs::create_dir_all(&pais_skills_dir).unwrap();
            fs::create_dir_all(&pais_plugins_dir).unwrap();

            TestEnv {
                _temp_dir: temp_dir,
                claude_skills_dir,
                pais_skills_dir,
                pais_plugins_dir,
            }
        }

        /// Create a skill in the pais skills directory
        fn create_skill(&self, name: &str) -> PathBuf {
            let skill_dir = self.pais_skills_dir.join(name);
            fs::create_dir_all(&skill_dir).unwrap();
            fs::write(skill_dir.join("SKILL.md"), format!("# {}\nTest skill", name)).unwrap();
            skill_dir
        }

        /// Create a plugin skill in the pais plugins directory
        fn create_plugin_skill(&self, name: &str) -> PathBuf {
            let plugin_dir = self.pais_plugins_dir.join(name);
            fs::create_dir_all(&plugin_dir).unwrap();
            fs::write(plugin_dir.join("SKILL.md"), format!("# {}\nTest plugin skill", name)).unwrap();
            plugin_dir
        }

        /// Create a symlink in the claude skills directory
        fn create_symlink(&self, name: &str, target: &PathBuf) {
            let link_path = self.claude_skills_dir.join(name);
            unix_fs::symlink(target, &link_path).unwrap();
        }

        /// Create a regular file (not a symlink) in claude skills directory
        fn create_regular_file(&self, name: &str, content: &str) {
            let file_path = self.claude_skills_dir.join(name);
            fs::write(&file_path, content).unwrap();
        }

        /// List symlinks in claude skills directory
        fn list_symlinks(&self) -> Vec<String> {
            let mut names: Vec<String> = fs::read_dir(&self.claude_skills_dir)
                .unwrap()
                .flatten()
                .filter(|e| e.path().is_symlink())
                .filter_map(|e| e.file_name().to_str().map(String::from))
                .collect();
            names.sort();
            names
        }

        /// List all files (including non-symlinks) in claude skills directory
        fn list_all_files(&self) -> Vec<String> {
            let mut names: Vec<String> = fs::read_dir(&self.claude_skills_dir)
                .unwrap()
                .flatten()
                .filter_map(|e| e.file_name().to_str().map(String::from))
                .collect();
            names.sort();
            names
        }
    }

    #[test]
    fn test_get_current_symlinks_empty_directory() {
        let env = TestEnv::new();
        let result = get_current_symlinks(&env.claude_skills_dir);
        assert!(result.is_empty());
    }

    #[test]
    fn test_get_current_symlinks_with_symlinks() {
        let env = TestEnv::new();

        // Create skills and symlinks
        let rust_coder = env.create_skill("rust-coder");
        let otto = env.create_skill("otto");
        env.create_symlink("rust-coder", &rust_coder);
        env.create_symlink("otto", &otto);

        let result = get_current_symlinks(&env.claude_skills_dir);
        assert_eq!(result.len(), 2);
        assert!(result.contains_key("rust-coder"));
        assert!(result.contains_key("otto"));
    }

    #[test]
    fn test_get_current_symlinks_ignores_regular_files() {
        let env = TestEnv::new();

        // Create a symlink
        let rust_coder = env.create_skill("rust-coder");
        env.create_symlink("rust-coder", &rust_coder);

        // Create a regular file (should be ignored)
        env.create_regular_file("README.md", "# Skills\nThis is a readme.");

        let result = get_current_symlinks(&env.claude_skills_dir);
        assert_eq!(result.len(), 1);
        assert!(result.contains_key("rust-coder"));
        assert!(!result.contains_key("README.md"));
    }

    #[test]
    fn test_get_current_symlinks_nonexistent_directory() {
        let env = TestEnv::new();
        let nonexistent = env.claude_skills_dir.join("does-not-exist");
        let result = get_current_symlinks(&nonexistent);
        assert!(result.is_empty());
    }

    #[test]
    fn test_find_skill_source_in_skills_dir() {
        let env = TestEnv::new();
        env.create_skill("rust-coder");

        // Create a mock config pointing to our test directories
        let config = create_test_config(&env);

        let result = find_skill_source("rust-coder", &config);
        assert!(result.is_some());
        assert!(result.unwrap().ends_with("rust-coder"));
    }

    #[test]
    fn test_find_skill_source_in_plugins_dir() {
        let env = TestEnv::new();
        env.create_plugin_skill("fabric");

        let config = create_test_config(&env);

        let result = find_skill_source("fabric", &config);
        assert!(result.is_some());
        assert!(result.unwrap().ends_with("fabric"));
    }

    #[test]
    fn test_find_skill_source_prefers_skills_over_plugins() {
        let env = TestEnv::new();
        // Create same skill in both directories
        env.create_skill("otto");
        env.create_plugin_skill("otto");

        let config = create_test_config(&env);

        let result = find_skill_source("otto", &config);
        assert!(result.is_some());
        // Should find the one in skills dir (checked first)
        let path = result.unwrap();
        assert!(path.to_string_lossy().contains("pais-skills"));
    }

    #[test]
    fn test_find_skill_source_not_found() {
        let env = TestEnv::new();
        let config = create_test_config(&env);

        let result = find_skill_source("nonexistent-skill", &config);
        assert!(result.is_none());
    }

    #[test]
    fn test_find_skill_source_requires_skill_md() {
        let env = TestEnv::new();
        // Create directory without SKILL.md
        let skill_dir = env.pais_skills_dir.join("incomplete-skill");
        fs::create_dir_all(&skill_dir).unwrap();
        // No SKILL.md file!

        let config = create_test_config(&env);

        let result = find_skill_source("incomplete-skill", &config);
        assert!(result.is_none());
    }

    #[test]
    fn test_get_all_skill_names_empty() {
        let env = TestEnv::new();
        let config = create_test_config(&env);

        let result = get_all_skill_names(&config);
        assert!(result.is_empty());
    }

    #[test]
    fn test_get_all_skill_names_from_skills_dir() {
        let env = TestEnv::new();
        env.create_skill("rust-coder");
        env.create_skill("otto");

        let config = create_test_config(&env);

        let result = get_all_skill_names(&config);
        assert_eq!(result.len(), 2);
        assert!(result.contains("rust-coder"));
        assert!(result.contains("otto"));
    }

    #[test]
    fn test_get_all_skill_names_from_plugins_dir() {
        let env = TestEnv::new();
        env.create_plugin_skill("fabric");
        env.create_plugin_skill("youtube");

        let config = create_test_config(&env);

        let result = get_all_skill_names(&config);
        assert_eq!(result.len(), 2);
        assert!(result.contains("fabric"));
        assert!(result.contains("youtube"));
    }

    #[test]
    fn test_get_all_skill_names_combined() {
        let env = TestEnv::new();
        env.create_skill("rust-coder");
        env.create_skill("otto");
        env.create_plugin_skill("fabric");

        let config = create_test_config(&env);

        let result = get_all_skill_names(&config);
        assert_eq!(result.len(), 3);
        assert!(result.contains("rust-coder"));
        assert!(result.contains("otto"));
        assert!(result.contains("fabric"));
    }

    #[test]
    fn test_get_all_skill_names_deduplicates() {
        let env = TestEnv::new();
        // Same skill in both directories
        env.create_skill("otto");
        env.create_plugin_skill("otto");

        let config = create_test_config(&env);

        let result = get_all_skill_names(&config);
        assert_eq!(result.len(), 1);
        assert!(result.contains("otto"));
    }

    #[test]
    fn test_sync_symlinks_add_new_skills() {
        let env = TestEnv::new();
        env.create_skill("rust-coder");
        env.create_skill("otto");

        let config = create_test_config(&env);

        // Sync with requested skills
        let result = sync_skill_symlinks_with_dir(
            &["rust-coder".to_string(), "otto".to_string()],
            &config,
            &env.claude_skills_dir,
        )
        .unwrap();

        assert_eq!(result.added.len(), 2);
        assert!(result.removed.is_empty());
        assert!(result.unchanged.is_empty());
        assert!(result.not_found.is_empty());

        // Verify symlinks were created
        let symlinks = env.list_symlinks();
        assert_eq!(symlinks, vec!["otto", "rust-coder"]);
    }

    #[test]
    fn test_sync_symlinks_remove_unwanted_skills() {
        let env = TestEnv::new();
        let rust_coder = env.create_skill("rust-coder");
        let otto = env.create_skill("otto");
        let fabric = env.create_skill("fabric");

        // Pre-create symlinks for all three
        env.create_symlink("rust-coder", &rust_coder);
        env.create_symlink("otto", &otto);
        env.create_symlink("fabric", &fabric);

        let config = create_test_config(&env);

        // Sync with only rust-coder requested
        let result =
            sync_skill_symlinks_with_dir(&["rust-coder".to_string()], &config, &env.claude_skills_dir).unwrap();

        assert!(result.added.is_empty());
        assert_eq!(result.removed.len(), 2);
        assert!(result.removed.contains(&"otto".to_string()));
        assert!(result.removed.contains(&"fabric".to_string()));
        assert_eq!(result.unchanged, vec!["rust-coder"]);

        // Verify only rust-coder remains
        let symlinks = env.list_symlinks();
        assert_eq!(symlinks, vec!["rust-coder"]);
    }

    #[test]
    fn test_sync_symlinks_leaves_unchanged() {
        let env = TestEnv::new();
        let rust_coder = env.create_skill("rust-coder");
        let otto = env.create_skill("otto");

        // Pre-create symlinks
        env.create_symlink("rust-coder", &rust_coder);
        env.create_symlink("otto", &otto);

        let config = create_test_config(&env);

        // Sync with same skills
        let result = sync_skill_symlinks_with_dir(
            &["rust-coder".to_string(), "otto".to_string()],
            &config,
            &env.claude_skills_dir,
        )
        .unwrap();

        assert!(result.added.is_empty());
        assert!(result.removed.is_empty());
        assert_eq!(result.unchanged.len(), 2);

        // Symlinks should still exist
        let symlinks = env.list_symlinks();
        assert_eq!(symlinks, vec!["otto", "rust-coder"]);
    }

    #[test]
    fn test_sync_symlinks_mixed_add_remove_unchanged() {
        let env = TestEnv::new();
        let rust_coder = env.create_skill("rust-coder");
        let otto = env.create_skill("otto");
        let fabric = env.create_skill("fabric");
        env.create_skill("clone");

        // Start with rust-coder and otto
        env.create_symlink("rust-coder", &rust_coder);
        env.create_symlink("otto", &otto);
        env.create_symlink("fabric", &fabric);

        let config = create_test_config(&env);

        // Request rust-coder (keep), clone (add), remove otto and fabric
        let result = sync_skill_symlinks_with_dir(
            &["rust-coder".to_string(), "clone".to_string()],
            &config,
            &env.claude_skills_dir,
        )
        .unwrap();

        assert_eq!(result.added, vec!["clone"]);
        assert!(result.removed.contains(&"fabric".to_string()));
        assert!(result.removed.contains(&"otto".to_string()));
        assert_eq!(result.removed.len(), 2);
        assert_eq!(result.unchanged, vec!["rust-coder"]);

        // Verify final state
        let symlinks = env.list_symlinks();
        assert_eq!(symlinks, vec!["clone", "rust-coder"]);
    }

    #[test]
    fn test_sync_symlinks_handles_not_found() {
        let env = TestEnv::new();
        env.create_skill("rust-coder");
        // "nonexistent" skill does NOT exist

        let config = create_test_config(&env);

        let result = sync_skill_symlinks_with_dir(
            &["rust-coder".to_string(), "nonexistent".to_string()],
            &config,
            &env.claude_skills_dir,
        )
        .unwrap();

        assert_eq!(result.added, vec!["rust-coder"]);
        assert_eq!(result.not_found, vec!["nonexistent"]);

        // Only rust-coder should be created
        let symlinks = env.list_symlinks();
        assert_eq!(symlinks, vec!["rust-coder"]);
    }

    #[test]
    fn test_sync_symlinks_empty_list_loads_all() {
        let env = TestEnv::new();
        env.create_skill("rust-coder");
        env.create_skill("otto");
        env.create_plugin_skill("fabric");

        let config = create_test_config(&env);

        // Empty list = load all available
        let result = sync_skill_symlinks_with_dir(&[], &config, &env.claude_skills_dir).unwrap();

        assert_eq!(result.added.len(), 3);
        assert!(result.removed.is_empty());

        let symlinks = env.list_symlinks();
        assert_eq!(symlinks, vec!["fabric", "otto", "rust-coder"]);
    }

    #[test]
    fn test_sync_symlinks_preserves_regular_files() {
        let env = TestEnv::new();
        env.create_skill("rust-coder");

        // Create a regular file that should NOT be touched
        env.create_regular_file("README.md", "# Skills");

        let config = create_test_config(&env);

        let result =
            sync_skill_symlinks_with_dir(&["rust-coder".to_string()], &config, &env.claude_skills_dir).unwrap();

        assert_eq!(result.added, vec!["rust-coder"]);

        // Both should exist - README.md should not be removed
        let all_files = env.list_all_files();
        assert!(all_files.contains(&"README.md".to_string()));
        assert!(all_files.contains(&"rust-coder".to_string()));
    }

    #[test]
    fn test_sync_symlinks_creates_directory_if_needed() {
        let env = TestEnv::new();
        env.create_skill("rust-coder");

        // Remove the claude skills directory
        fs::remove_dir_all(&env.claude_skills_dir).unwrap();
        assert!(!env.claude_skills_dir.exists());

        let config = create_test_config(&env);

        // This should create the directory
        let result =
            sync_skill_symlinks_with_dir(&["rust-coder".to_string()], &config, &env.claude_skills_dir).unwrap();

        assert!(env.claude_skills_dir.exists());
        assert_eq!(result.added, vec!["rust-coder"]);
    }

    #[test]
    fn test_sync_symlinks_results_are_sorted() {
        let env = TestEnv::new();
        env.create_skill("zebra");
        env.create_skill("apple");
        env.create_skill("mango");

        let config = create_test_config(&env);

        let result = sync_skill_symlinks_with_dir(
            &["zebra".to_string(), "apple".to_string(), "mango".to_string()],
            &config,
            &env.claude_skills_dir,
        )
        .unwrap();

        // Results should be sorted alphabetically
        assert_eq!(result.added, vec!["apple", "mango", "zebra"]);
    }

    /// Helper: create a test config pointing to the test environment's directories
    fn create_test_config(env: &TestEnv) -> Config {
        let mut config = Config::default();
        config.paths.skills = env.pais_skills_dir.clone();
        config.paths.plugins = env.pais_plugins_dir.clone();
        config
    }

    /// Test-friendly version of sync_skill_symlinks that takes the target dir as a parameter
    /// (the real function uses ~/.claude/skills/ which we can't override)
    fn sync_skill_symlinks_with_dir(
        skill_list: &[String],
        config: &Config,
        claude_skills_dir: &PathBuf,
    ) -> eyre::Result<SyncResult> {
        // Ensure the directory exists
        fs::create_dir_all(claude_skills_dir).context("Failed to create skills directory")?;

        // Get current state
        let current_symlinks = get_current_symlinks(claude_skills_dir);
        let current_names: HashSet<String> = current_symlinks.keys().cloned().collect();

        // Determine requested skills
        let requested_names: HashSet<String> = if skill_list.is_empty() {
            get_all_skill_names(config)
        } else {
            skill_list.iter().cloned().collect()
        };

        // Compute diff
        let to_remove: HashSet<_> = current_names.difference(&requested_names).cloned().collect();
        let to_add: HashSet<_> = requested_names.difference(&current_names).cloned().collect();
        let unchanged: HashSet<_> = current_names.intersection(&requested_names).cloned().collect();

        let mut result = SyncResult {
            unchanged: unchanged.into_iter().collect(),
            ..Default::default()
        };

        // Remove symlinks that shouldn't be there
        for name in &to_remove {
            let symlink_path = claude_skills_dir.join(name);
            if let Err(e) = fs::remove_file(&symlink_path) {
                log::warn!("Failed to remove symlink {}: {}", symlink_path.display(), e);
            } else {
                result.removed.push(name.clone());
            }
        }

        // Add symlinks that should be there
        for name in &to_add {
            if let Some(source_path) = find_skill_source(name, config) {
                let symlink_path = claude_skills_dir.join(name);
                if let Err(e) = unix_fs::symlink(&source_path, &symlink_path) {
                    log::warn!(
                        "Failed to create symlink {} -> {}: {}",
                        symlink_path.display(),
                        source_path.display(),
                        e
                    );
                } else {
                    result.added.push(name.clone());
                }
            } else {
                result.not_found.push(name.clone());
            }
        }

        // Sort for consistent output
        result.added.sort();
        result.removed.sort();
        result.unchanged.sort();
        result.not_found.sort();

        Ok(result)
    }
}
