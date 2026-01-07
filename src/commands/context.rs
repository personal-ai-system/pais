//! Context injection commands for Claude Code hooks
//!
//! This module provides the `pais context inject` command that outputs
//! skill context as a `<system-reminder>` for Claude Code's SessionStart hook.
//!
//! ## Tier Loading
//!
//! - **Tier 0 (Core)**: Full skill content loaded at session start
//!   - Skills with `tier: core` in frontmatter
//!   - The `core` skill (always force-loaded)
//! - **Tier 1 (Deferred)**: Only name/description/triggers in context
//!   - Full content loaded when skill is invoked
//!
//! ## Skill Filtering
//!
//! Skills are filtered based on what symlinks exist in `~/.claude/skills/`.
//! This is set up by `pais session` before Claude Code launches.
//! If no symlinks exist, all skills from the PAIS skills directory are shown.

use eyre::{Context, Result};
use std::collections::HashSet;
use std::fs;
use std::path::Path;
use std::process::Command;

use crate::cli::ContextAction;
use crate::config::Config;
use crate::skill::indexer::{SkillIndex, generate_index};
use crate::skill::parser::SkillTier;

/// Run a context subcommand
pub fn run(action: ContextAction, config: &Config) -> Result<()> {
    match action {
        ContextAction::Inject { raw } => inject_context(raw, config),
    }
}

/// Get current local timestamp
fn get_local_timestamp() -> String {
    let now = chrono::Local::now();
    now.format("%Y-%m-%d %H:%M:%S %Z").to_string()
}

const FRONTMATTER_DELIMITER: &str = "---";
const FRONTMATTER_DELIMITER_LEN: usize = 3;

/// Extract skill body content (everything after frontmatter)
fn extract_skill_body(content: &str) -> Option<String> {
    if let Some(start) = content.find(FRONTMATTER_DELIMITER)
        && let Some(end) = content[start + FRONTMATTER_DELIMITER_LEN..].find(FRONTMATTER_DELIMITER)
    {
        let body_start = start + FRONTMATTER_DELIMITER_LEN + end + FRONTMATTER_DELIMITER_LEN;
        let body = content[body_start..].trim();
        if !body.is_empty() {
            return Some(body.to_string());
        }
    }
    None
}

/// Read the skill filter from ~/.claude/skills/ symlinks
///
/// Returns Some(set of skill names) if symlinks exist, None if directory
/// doesn't exist or is empty (meaning no filtering - load all skills).
fn get_skill_filter() -> Option<HashSet<String>> {
    let home = dirs::home_dir()?;
    let claude_skills_dir = home.join(".claude").join("skills");

    if !claude_skills_dir.exists() {
        return None;
    }

    let entries = fs::read_dir(&claude_skills_dir).ok()?;
    let symlinks: HashSet<String> = entries
        .flatten()
        .filter(|e| e.path().is_symlink())
        .filter_map(|e| e.file_name().to_str().map(String::from))
        .collect();

    if symlinks.is_empty() { None } else { Some(symlinks) }
}

/// Check if a skill should be included based on the filter
fn should_include_skill(name: &str, filter: &Option<HashSet<String>>) -> bool {
    match filter {
        Some(allowed) => allowed.contains(name),
        None => true, // No filter = include all
    }
}

/// Load all core-tier skills (Tier 0 - always present)
///
/// Returns a list of (name, body) tuples for all skills marked as tier: core
/// If skill_filter is Some, only includes skills in the filter set
fn load_core_skills(
    skills_dir: &Path,
    index: &SkillIndex,
    skill_filter: &Option<HashSet<String>>,
) -> Vec<(String, String)> {
    let mut core_skills = Vec::new();

    // Get all core-tier skills from the index, applying filter
    let mut core_entries: Vec<_> = index
        .skills
        .values()
        .filter(|s| s.tier == SkillTier::Core && should_include_skill(&s.name, skill_filter))
        .collect();

    // Sort to ensure consistent ordering (put "core" first)
    core_entries.sort_by(|a, b| {
        if a.name.to_lowercase() == "core" {
            std::cmp::Ordering::Less
        } else if b.name.to_lowercase() == "core" {
            std::cmp::Ordering::Greater
        } else {
            a.name.cmp(&b.name)
        }
    });

    for entry in core_entries {
        let skill_path = skills_dir.join(&entry.name).join("SKILL.md");
        if skill_path.exists()
            && let Ok(content) = fs::read_to_string(&skill_path)
            && let Some(body) = extract_skill_body(&content)
        {
            core_skills.push((entry.name.clone(), body));
        }
    }

    core_skills
}

/// Generate deferred skills section from index, applying filter
fn generate_deferred_skills_content(index: &SkillIndex, skill_filter: &Option<HashSet<String>>) -> Option<String> {
    // Get deferred skills, applying filter
    let mut deferred_entries: Vec<_> = index
        .skills
        .values()
        .filter(|s| s.tier == SkillTier::Deferred && should_include_skill(&s.name, skill_filter))
        .collect();

    if deferred_entries.is_empty() {
        return None;
    }

    // Sort alphabetically
    deferred_entries.sort_by_key(|s| &s.name);

    // Skills table
    let mut lines = vec![
        "## Available Skills".to_string(),
        String::new(),
        "| Skill | Description | Triggers |".to_string(),
        "|-------|-------------|----------|".to_string(),
    ];

    for entry in &deferred_entries {
        let triggers = entry.triggers.join(", ");
        let triggers_display = if triggers.is_empty() { "-".to_string() } else { triggers };
        // Truncate description for table
        let desc = if entry.description.len() > 50 {
            format!("{}...", &entry.description[..47])
        } else {
            entry.description.clone()
        };
        lines.push(format!("| **{}** | {} | {} |", entry.name, desc, triggers_display));
    }

    // Routing instructions
    lines.push(String::new());
    lines.push("## Routing Instructions".to_string());
    lines.push(String::new());
    lines.push("When a user request matches a skill's triggers:".to_string());
    lines.push(
        "1. Read the full SKILL.md file from `/home/saidler/.config/pais/skills/[skill-name]/SKILL.md`".to_string(),
    );
    lines.push("2. Follow the skill's instructions and conventions".to_string());
    lines.push("3. No need to ask for permission - the skill is pre-approved".to_string());

    Some(lines.join("\n"))
}

/// Check if a tool is available in PATH
fn check_tool_available(tool: &str) -> Option<String> {
    // For tool preferences (like "eza --tree"), just check the first word
    let binary = tool.split_whitespace().next().unwrap_or(tool);

    Command::new("which")
        .arg(binary)
        .output()
        .ok()
        .filter(|o| o.status.success())
        .and_then(|_| {
            // Try to get version
            let version_output = Command::new(binary).arg("--version").output().ok()?;
            let version_str = String::from_utf8_lossy(&version_output.stdout);
            // Extract first line, first few words
            let version = version_str
                .lines()
                .next()
                .unwrap_or("")
                .split_whitespace()
                .take(3)
                .collect::<Vec<_>>()
                .join(" ");
            Some(version)
        })
}

/// Generate environment context section from config
fn generate_environment_context(config: &Config) -> Option<String> {
    let env = &config.environment;

    // Only generate if there's something to show
    if env.repos_dir.is_none() && env.tool_preferences.is_empty() && env.tools.is_empty() {
        return None;
    }

    let mut lines = Vec::new();
    lines.push("## Environment".to_string());
    lines.push(String::new());

    // Repos section
    if let Some(ref repos_dir) = env.repos_dir {
        let expanded = Config::expand_path(repos_dir);
        let path_str = expanded.display().to_string();
        let path_str = path_str.trim_end_matches('/');
        lines.push("### Repos".to_string());
        lines.push(format!("All repositories are at `{}/{{org}}/{{repo}}`.", path_str));
        lines.push("Use `clone` to checkout new repos (e.g., `clone scottidler/otto`).".to_string());
        lines.push(String::new());
    }

    // Tool preferences section
    if !env.tool_preferences.is_empty() {
        lines.push("### Preferred Tools".to_string());
        lines.push("Use modern alternatives when available:".to_string());

        let mut prefs: Vec<_> = env.tool_preferences.iter().collect();
        prefs.sort_by_key(|(k, _)| *k);

        for (legacy, modern) in prefs {
            let available = check_tool_available(modern).is_some();
            let status = if available { "✓" } else { "✗" };
            lines.push(format!("- `{}` instead of `{}` {}", modern, legacy, status));
        }

        lines.push(String::new());
        lines.push("Fallback to standard tools if modern ones unavailable.".to_string());
        lines.push(String::new());
    }

    // Custom tools section
    if !env.tools.is_empty() {
        lines.push("### Custom Tools".to_string());

        let mut tools: Vec<_> = env.tools.iter().collect();
        tools.sort_by_key(|(k, _)| *k);

        for (name, tool_config) in tools {
            let available = check_tool_available(name);
            let status = if available.is_some() { "✓" } else { "✗" };
            let desc = tool_config.description.as_deref().unwrap_or("");
            let github = tool_config
                .github
                .as_ref()
                .map(|g| format!(" ({})", g))
                .unwrap_or_default();

            lines.push(format!("- `{}` - {}{} {}", name, desc, github, status));
        }

        lines.push(String::new());
        lines.push("Check `which <tool>` before using if uncertain.".to_string());
        lines.push(String::new());
    }

    Some(lines.join("\n"))
}

/// Inject skill context for SessionStart hook
fn inject_context(raw: bool, config: &Config) -> Result<()> {
    log::debug!("Injecting context (raw={})", raw);

    let skills_dir = Config::expand_path(&config.paths.skills);
    log::debug!("Skills directory: {}", skills_dir.display());

    let context_path = skills_dir.join("context-snippet.md");

    // Check for skill filter from ~/.claude/skills/ symlinks
    let skill_filter = get_skill_filter();
    if let Some(ref filter) = skill_filter {
        log::info!("Skill filter from symlinks: {} skills", filter.len());
    } else {
        log::debug!("No skill filter - loading all skills");
    }

    // Generate or load the index
    let index = generate_index(&skills_dir).context("Failed to generate skill index")?;
    log::debug!(
        "Index generated: {} skills ({} core, {} deferred)",
        index.total_skills,
        index.core_count,
        index.deferred_count
    );

    // Load core-tier skills (Tier 0), applying filter
    let core_skills = load_core_skills(&skills_dir, &index, &skill_filter);
    log::debug!(
        "Loaded {} core skills: [{}]",
        core_skills.len(),
        core_skills
            .iter()
            .map(|(n, _)| n.as_str())
            .collect::<Vec<_>>()
            .join(", ")
    );

    // Generate environment context
    let env_context = generate_environment_context(config);
    log::debug!(
        "Environment context: {}",
        if env_context.is_some() { "generated" } else { "none" }
    );

    // Generate deferred skills content (Tier 1)
    // If skill filter is active, generate dynamically to apply the filter
    // Otherwise, use the static context-snippet.md if available
    let context_content = if skill_filter.is_some() {
        log::debug!("Generating filtered deferred skills content");
        generate_deferred_skills_content(&index, &skill_filter)
    } else if context_path.exists() {
        log::debug!("Loading deferred skills context from: {}", context_path.display());
        Some(
            fs::read_to_string(&context_path)
                .with_context(|| format!("Failed to read context file: {}", context_path.display()))?,
        )
    } else {
        log::debug!("Generating deferred skills content (no static file)");
        generate_deferred_skills_content(&index, &skill_filter)
    };

    // If neither exists, warn and exit
    if core_skills.is_empty() && context_content.is_none() {
        log::warn!("No skills found - run 'pais skill index' first");
        eprintln!("[PAIS] No skills found. Run 'pais skill index' first.");
        return Ok(());
    }

    if raw {
        // Output raw content without wrapper
        if let Some(ref env) = env_context {
            println!("{}", env);
            println!();
        }
        for (name, body) in &core_skills {
            println!("# {} (Tier 0 - Core)", name);
            println!();
            println!("{}", body);
            println!();
        }
        if let Some(ref context) = context_content {
            println!("{}", context);
        }
    } else {
        // Calculate actual loaded counts
        let loaded_core_count = core_skills.len();
        let loaded_deferred_count = context_content.as_ref().map(|c| c.matches("| **").count()).unwrap_or(0);
        let loaded_total = loaded_core_count + loaded_deferred_count;

        // Output with system-reminder wrapper for Claude Code
        println!("<system-reminder>");
        println!("PAIS CONTEXT (Auto-loaded at Session Start)");
        println!();
        println!("📅 Current Time: {}", get_local_timestamp());
        if skill_filter.is_some() {
            println!(
                "📦 Skills: {} loaded (filtered), {} core-tier",
                loaded_total, loaded_core_count
            );
        } else {
            println!(
                "📦 Skills: {} total, {} core-tier",
                index.total_skills, index.core_count
            );
        }

        // Environment context (if configured)
        if let Some(ref env) = env_context {
            println!();
            println!("═══════════════════════════════════════════════════════════");
            println!("                    ENVIRONMENT");
            println!("═══════════════════════════════════════════════════════════");
            println!();
            println!("{}", env);
        }

        // Core-tier skills (Tier 0) - full content loaded
        if !core_skills.is_empty() {
            println!();
            println!("═══════════════════════════════════════════════════════════");
            println!("                 CORE SKILLS (Tier 0)");
            println!("═══════════════════════════════════════════════════════════");

            for (name, body) in &core_skills {
                println!();
                if name.to_lowercase() == "core" {
                    println!("### CORE PRINCIPLES");
                } else {
                    println!("### {}", name.to_uppercase());
                }
                println!();
                println!("{}", body);
            }
        }

        // Deferred skills (Tier 1) - only frontmatter/triggers
        if let Some(ref context) = context_content {
            println!();
            println!("═══════════════════════════════════════════════════════════");
            println!("              DEFERRED SKILLS (Tier 1)");
            println!("═══════════════════════════════════════════════════════════");
            println!();
            println!("{}", context);
        }

        println!();
        println!("</system-reminder>");
        println!();
        if skill_filter.is_some() {
            println!(
                "✅ PAIS context loaded ({} skills filtered, {} core-tier)",
                loaded_total, loaded_core_count
            );
        } else {
            println!(
                "✅ PAIS context loaded ({} skills, {} core-tier)",
                index.total_skills, index.core_count
            );
        }
    }

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_get_local_timestamp() {
        let ts = get_local_timestamp();
        // Should be in format like "2026-01-02 14:30:00 PST"
        assert!(ts.contains("-"));
        assert!(ts.contains(":"));
    }

    // === Skill filter tests ===

    #[test]
    fn test_should_include_skill_no_filter() {
        let filter: Option<HashSet<String>> = None;
        assert!(should_include_skill("rust-coder", &filter));
        assert!(should_include_skill("otto", &filter));
        assert!(should_include_skill("anything", &filter));
    }

    #[test]
    fn test_should_include_skill_with_filter_positive() {
        let filter: Option<HashSet<String>> = Some(["rust-coder", "otto"].iter().map(|s| s.to_string()).collect());
        assert!(should_include_skill("rust-coder", &filter));
        assert!(should_include_skill("otto", &filter));
    }

    #[test]
    fn test_should_include_skill_with_filter_negative() {
        let filter: Option<HashSet<String>> = Some(["rust-coder", "otto"].iter().map(|s| s.to_string()).collect());
        assert!(!should_include_skill("fabric", &filter));
        assert!(!should_include_skill("unknown", &filter));
    }

    #[test]
    fn test_should_include_skill_empty_filter() {
        let filter: Option<HashSet<String>> = Some(HashSet::new());
        // Empty filter set means nothing matches
        assert!(!should_include_skill("rust-coder", &filter));
        assert!(!should_include_skill("anything", &filter));
    }

    #[test]
    fn test_extract_skill_body_valid() {
        let content = r#"---
name: test-skill
description: A test skill
---

# Test Skill

This is the body content.
"#;
        let body = extract_skill_body(content);
        assert!(body.is_some());
        let body = body.unwrap();
        assert!(body.contains("# Test Skill"));
        assert!(body.contains("This is the body content."));
    }

    #[test]
    fn test_extract_skill_body_no_frontmatter() {
        let content = "# Just content\n\nNo frontmatter here.";
        let body = extract_skill_body(content);
        assert!(body.is_none());
    }

    #[test]
    fn test_extract_skill_body_empty_body() {
        let content = r#"---
name: test-skill
---
"#;
        let body = extract_skill_body(content);
        assert!(body.is_none()); // Empty body after frontmatter
    }
}
