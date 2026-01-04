//! Architecture documentation generator
//!
//! Generates ARCHITECTURE.md with current system state.

use chrono::Local;
use eyre::{Context, Result};
use std::fs;

use crate::agent::loader::AgentLoader;
use crate::config::Config;
use crate::skill::loader::discover_simple_skills;

/// Generate ARCHITECTURE.md in the PAIS directory
pub fn generate_architecture_doc(config: &Config) -> Result<String> {
    let skills_dir = Config::expand_path(&config.paths.skills);
    let agents_dir = skills_dir.parent().unwrap_or(&skills_dir).join("agents");
    let pais_dir = Config::pais_dir();

    let mut doc = String::new();

    // Header
    doc.push_str("# PAIS Architecture\n\n");
    doc.push_str(&format!(
        "> Auto-generated on {} by `pais`\n\n",
        Local::now().format("%Y-%m-%d %H:%M:%S")
    ));

    // Overview
    doc.push_str("## Overview\n\n");
    doc.push_str(&format!("- **PAIS Directory:** `{}`\n", pais_dir.display()));
    doc.push_str(&format!("- **Skills Directory:** `{}`\n", skills_dir.display()));
    doc.push_str(&format!("- **Agents Directory:** `{}`\n", agents_dir.display()));
    doc.push('\n');

    // Skills section
    doc.push_str("## Skills\n\n");
    let skills = discover_simple_skills(&skills_dir).unwrap_or_default();
    if skills.is_empty() {
        doc.push_str("No skills installed.\n\n");
    } else {
        doc.push_str("| Skill | Tier | Description |\n");
        doc.push_str("|-------|------|-------------|\n");
        for skill in &skills {
            let skill_md = skill.path.join("SKILL.md");
            if let Ok(metadata) = crate::skill::parser::parse_skill_md(&skill_md) {
                doc.push_str(&format!(
                    "| {} | {} | {} |\n",
                    metadata.name,
                    metadata.tier,
                    truncate(&metadata.description, 50)
                ));
            }
        }
        doc.push('\n');
    }

    // Agents section
    doc.push_str("## Agents\n\n");
    let mut loader = AgentLoader::new(agents_dir.clone());
    let agents = loader.load_all().unwrap_or_default();
    if agents.is_empty() {
        doc.push_str("No agents configured.\n\n");
    } else {
        doc.push_str("| Agent | Traits | History Category |\n");
        doc.push_str("|-------|--------|------------------|\n");
        for agent in &agents {
            let traits: Vec<String> = agent.traits.iter().map(|t| t.to_string()).collect();
            let category = agent.history_category.as_deref().unwrap_or("-");
            doc.push_str(&format!("| {} | {} | {} |\n", agent.name, traits.join(", "), category));
        }
        doc.push('\n');
    }

    // Hooks section
    doc.push_str("## Hooks\n\n");
    doc.push_str("| Hook | Status |\n");
    doc.push_str("|------|--------|\n");
    doc.push_str(&format!(
        "| Security Validator | {} |\n",
        if config.hooks.security_enabled { "✅ Enabled" } else { "❌ Disabled" }
    ));
    doc.push_str(&format!(
        "| History Capture | {} |\n",
        if config.hooks.history_enabled { "✅ Enabled" } else { "❌ Disabled" }
    ));
    doc.push_str(&format!(
        "| UI (Tab Titles) | {} |\n",
        if config.hooks.ui_enabled { "✅ Enabled" } else { "❌ Disabled" }
    ));
    doc.push('\n');

    // Observability section
    doc.push_str("## Observability\n\n");
    doc.push_str(&format!(
        "- **Enabled:** {}\n",
        if config.observability.enabled { "Yes" } else { "No" }
    ));
    if config.observability.enabled {
        let sinks: Vec<String> = config
            .observability
            .sinks
            .iter()
            .map(|s| format!("{:?}", s).to_lowercase())
            .collect();
        doc.push_str(&format!("- **Sinks:** {}\n", sinks.join(", ")));
        if let Some(ref endpoint) = config.observability.http_endpoint {
            doc.push_str(&format!("- **HTTP Endpoint:** {}\n", endpoint));
        }
    }
    doc.push('\n');

    // Paths section
    doc.push_str("## Paths\n\n");
    doc.push_str(&format!("- **Plugins:** `{}`\n", config.paths.plugins.display()));
    doc.push_str(&format!("- **Skills:** `{}`\n", config.paths.skills.display()));
    doc.push_str(&format!("- **History:** `{}`\n", config.paths.history.display()));
    doc.push('\n');

    Ok(doc)
}

/// Write architecture.md to the PAIS directory
pub fn write_architecture_doc(config: &Config) -> Result<std::path::PathBuf> {
    let doc = generate_architecture_doc(config)?;
    let pais_dir = Config::pais_dir();
    let path = pais_dir.join("architecture.md");

    fs::write(&path, &doc).with_context(|| format!("Failed to write {}", path.display()))?;

    Ok(path)
}

/// Truncate a string with ellipsis
fn truncate(s: &str, max_len: usize) -> String {
    let s = s.lines().next().unwrap_or(s).trim();
    if s.len() <= max_len {
        s.to_string()
    } else {
        format!("{}...", &s[..max_len - 3])
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_truncate() {
        assert_eq!(truncate("short", 10), "short");
        assert_eq!(truncate("this is a long string", 10), "this is...");
    }

    #[test]
    fn test_truncate_multiline() {
        assert_eq!(truncate("first line\nsecond line", 20), "first line");
    }
}
