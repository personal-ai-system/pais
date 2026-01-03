//! Context injection commands for Claude Code hooks
//!
//! This module provides the `pais context inject` command that outputs
//! skill context as a `<system-reminder>` for Claude Code's SessionStart hook.

use eyre::{Context, Result};
use std::fs;

use crate::cli::ContextAction;
use crate::config::Config;

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

/// Load core skill content (Tier 0 - always present)
fn load_core_skill(skills_dir: &std::path::Path) -> Option<String> {
    let core_path = skills_dir.join("core").join("SKILL.md");
    if core_path.exists() {
        // Read and extract body (skip frontmatter)
        if let Ok(content) = fs::read_to_string(&core_path) {
            // Find end of frontmatter (second ---)
            if let Some(start) = content.find(FRONTMATTER_DELIMITER)
                && let Some(end) = content[start + FRONTMATTER_DELIMITER_LEN..].find(FRONTMATTER_DELIMITER)
            {
                let body_start = start + FRONTMATTER_DELIMITER_LEN + end + FRONTMATTER_DELIMITER_LEN;
                let body = content[body_start..].trim();
                if !body.is_empty() {
                    return Some(body.to_string());
                }
            }
        }
    }
    None
}

/// Inject skill context for SessionStart hook
fn inject_context(raw: bool, config: &Config) -> Result<()> {
    let skills_dir = Config::expand_path(&config.paths.skills);
    let context_path = skills_dir.join("context-snippet.md");
    let index_path = skills_dir.join("skill-index.yaml");

    // Load CORE skill (Tier 0 - always present)
    let core_content = load_core_skill(&skills_dir);

    // Check if context file exists
    let context_content = if context_path.exists() {
        Some(
            fs::read_to_string(&context_path)
                .with_context(|| format!("Failed to read context file: {}", context_path.display()))?,
        )
    } else {
        None
    };

    // If neither exists, warn and exit
    if core_content.is_none() && context_content.is_none() {
        eprintln!("[PAIS] No CORE skill or context file found. Run 'pais skill index' first.");
        return Ok(());
    }

    // Get skill count from index if available
    let skill_count = if index_path.exists() {
        if let Ok(index_content) = fs::read_to_string(&index_path) {
            if let Ok(index) = serde_json::from_str::<serde_json::Value>(&index_content) {
                index.get("total_skills").and_then(|v| v.as_u64()).unwrap_or(0)
            } else {
                0
            }
        } else {
            0
        }
    } else {
        0
    };

    if raw {
        // Output raw content without wrapper
        if let Some(ref core) = core_content {
            println!("{}", core);
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
        println!("📦 Skills Available: {}", skill_count);

        // CORE skill first (Tier 0)
        if let Some(ref core) = core_content {
            println!();
            println!("═══════════════════════════════════════════════════════════");
            println!("                    CORE PRINCIPLES");
            println!("═══════════════════════════════════════════════════════════");
            println!();
            println!("{}", core);
        }

        // Then skill index (Tier 1 - frontmatter/triggers)
        if let Some(ref context) = context_content {
            println!();
            println!("═══════════════════════════════════════════════════════════");
            println!("                    AVAILABLE SKILLS");
            println!("═══════════════════════════════════════════════════════════");
            println!();
            println!("{}", context);
        }

        println!();
        println!("</system-reminder>");
        println!();
        println!("✅ PAIS context loaded ({} skills available)", skill_count);
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
