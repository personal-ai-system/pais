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

use eyre::{Context, Result};
use std::fs;
use std::path::Path;

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

/// Load all core-tier skills (Tier 0 - always present)
///
/// Returns a list of (name, body) tuples for all skills marked as tier: core
fn load_core_skills(skills_dir: &Path, index: &SkillIndex) -> Vec<(String, String)> {
    let mut core_skills = Vec::new();

    // Get all core-tier skills from the index
    let mut core_entries: Vec<_> = index.skills.values().filter(|s| s.tier == SkillTier::Core).collect();

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

/// Inject skill context for SessionStart hook
fn inject_context(raw: bool, config: &Config) -> Result<()> {
    let skills_dir = Config::expand_path(&config.paths.skills);
    let context_path = skills_dir.join("context-snippet.md");

    // Generate or load the index
    let index = generate_index(&skills_dir).context("Failed to generate skill index")?;

    // Load all core-tier skills (Tier 0)
    let core_skills = load_core_skills(&skills_dir, &index);

    // Check if context file exists (Tier 1 - deferred skills)
    let context_content = if context_path.exists() {
        Some(
            fs::read_to_string(&context_path)
                .with_context(|| format!("Failed to read context file: {}", context_path.display()))?,
        )
    } else {
        None
    };

    // If neither exists, warn and exit
    if core_skills.is_empty() && context_content.is_none() {
        eprintln!("[PAIS] No skills found. Run 'pais skill index' first.");
        return Ok(());
    }

    if raw {
        // Output raw content without wrapper
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
        // Output with system-reminder wrapper for Claude Code
        println!("<system-reminder>");
        println!("PAIS CONTEXT (Auto-loaded at Session Start)");
        println!();
        println!("📅 Current Time: {}", get_local_timestamp());
        println!(
            "📦 Skills: {} total, {} core-tier",
            index.total_skills, index.core_count
        );

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
        println!(
            "✅ PAIS context loaded ({} skills, {} core-tier)",
            index.total_skills, index.core_count
        );
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
}
