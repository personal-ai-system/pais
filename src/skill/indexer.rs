//! Skill index generation for context injection
//!
//! Generates a skill-index.yaml file containing:
//! - All skill names and descriptions
//! - USE WHEN triggers extracted from descriptions
//! - File paths for deferred loading
//!
//! This index is used by the SessionStart hook to inject
//! skill routing context into Claude's system prompt.

use eyre::{Context, Result};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::fs;
use std::path::Path;

use super::parser::{SkillTier, parse_skill_md};
use super::workflow::{WorkflowRoute, discover_workflows};

/// A skill entry in the index
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SkillIndexEntry {
    /// Skill name
    pub name: String,
    /// Relative path to SKILL.md
    pub path: String,
    /// Full description from frontmatter
    pub description: String,
    /// Extracted trigger words from USE WHEN clause
    pub triggers: Vec<String>,
    /// Loading tier
    pub tier: SkillTier,
    /// Available workflows for this skill
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub workflows: Vec<WorkflowRoute>,
}

/// The complete skill index
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SkillIndex {
    /// When the index was generated
    pub generated: String,
    /// Total number of skills
    pub total_skills: usize,
    /// Number of core (always-loaded) skills
    pub core_count: usize,
    /// Number of deferred skills
    pub deferred_count: usize,
    /// Skills by name (lowercase)
    pub skills: HashMap<String, SkillIndexEntry>,
}

/// Skills that should always be loaded at session start (override frontmatter tier)
const FORCE_CORE_SKILLS: &[&str] = &["core"];

/// Extract trigger words from a USE WHEN clause
pub fn extract_triggers(description: &str) -> Vec<String> {
    let mut triggers = Vec::new();

    // Match "USE WHEN" followed by text until period or end
    let desc_lower = description.to_lowercase();

    // Find USE WHEN clauses
    let mut search_from = 0;
    while let Some(pos) = desc_lower[search_from..].find("use when") {
        let start = search_from + pos + 8; // Skip "use when"
        let end = desc_lower[start..]
            .find('.')
            .map(|p| start + p)
            .unwrap_or(desc_lower.len());
        let clause = &desc_lower[start..end];

        // Split on common delimiters and extract words
        let words: Vec<String> = clause
            .split([',', ' ', '\t', '\n'])
            .map(|w| w.trim().to_lowercase())
            .map(|w| {
                // Simple plural handling - strip trailing 's' for common cases
                if w.ends_with('s') && w.len() > 3 { w[..w.len() - 1].to_string() } else { w }
            })
            .filter(|w| {
                w.len() > 2
                    && ![
                        "the", "and", "for", "with", "when", "user", "asks", "about", "any", "or",
                    ]
                    .contains(&w.as_str())
            })
            .collect();
        triggers.extend(words);

        search_from = end;
    }

    // Also extract key nouns from the full description using compile-time regex
    let key_terms = [
        "rust",
        "python",
        "cli",
        "api",
        "test",
        "deploy",
        "build",
        "git",
        "docker",
        "kubernetes",
        "k8s",
        "terraform",
        "aws",
        "gcp",
        "azure",
        "security",
        "auth",
        "database",
        "sql",
        "web",
        "frontend",
        "backend",
        "server",
        "client",
        "config",
        "yaml",
        "json",
        "toml",
        "xml",
        "html",
        "css",
        "js",
        "typescript",
        "react",
        "vue",
        "angular",
        "node",
        "bun",
        "deno",
        "cargo",
        "pip",
        "npm",
        "yarn",
        "pnpm",
    ];

    for word in desc_lower.split_whitespace() {
        let clean = word.trim_matches(|c: char| !c.is_alphanumeric());
        if key_terms.contains(&clean) && !triggers.contains(&clean.to_string()) {
            triggers.push(clean.to_string());
        }
    }

    // Deduplicate
    triggers.sort();
    triggers.dedup();
    triggers
}

/// Generate a skill index from a skills directory
pub fn generate_index(skills_dir: &Path) -> Result<SkillIndex> {
    let mut index = SkillIndex {
        generated: chrono::Utc::now().to_rfc3339(),
        total_skills: 0,
        core_count: 0,
        deferred_count: 0,
        skills: HashMap::new(),
    };

    if !skills_dir.exists() {
        return Ok(index);
    }

    for entry in fs::read_dir(skills_dir)
        .with_context(|| format!("Failed to read skills directory: {}", skills_dir.display()))?
    {
        let entry = entry?;
        let path = entry.path();

        if !path.is_dir() {
            continue;
        }

        let skill_md = path.join("SKILL.md");
        if !skill_md.exists() {
            continue;
        }

        // Parse the skill
        match parse_skill_md(&skill_md) {
            Ok(metadata) => {
                let name_lower = metadata.name.to_lowercase();

                // Tier is determined by:
                // 1. Force-core list (always core regardless of frontmatter)
                // 2. Frontmatter tier field
                let tier = if FORCE_CORE_SKILLS.contains(&name_lower.as_str()) {
                    SkillTier::Core
                } else {
                    metadata.tier
                };

                let relative_path = path
                    .file_name()
                    .map(|n| format!("{}/SKILL.md", n.to_string_lossy()))
                    .unwrap_or_default();

                let triggers = extract_triggers(&metadata.description);

                // Discover workflows for this skill
                let workflows = discover_workflows(&path).map(|w| w.routes).unwrap_or_default();

                let entry = SkillIndexEntry {
                    name: metadata.name.clone(),
                    path: relative_path,
                    description: metadata.description.clone(),
                    triggers,
                    tier,
                    workflows,
                };

                if tier.is_core() {
                    index.core_count += 1;
                } else {
                    index.deferred_count += 1;
                }
                index.total_skills += 1;
                index.skills.insert(name_lower, entry);
            }
            Err(e) => {
                log::warn!("Failed to parse skill at {}: {}", skill_md.display(), e);
            }
        }
    }

    Ok(index)
}

/// Write the index to a file
pub fn write_index(index: &SkillIndex, output_path: &Path) -> Result<()> {
    let yaml = serde_yaml::to_string(index).context("Failed to serialize skill index")?;

    fs::write(output_path, yaml).with_context(|| format!("Failed to write index to {}", output_path.display()))?;

    Ok(())
}

/// Generate a context snippet for injection
pub fn generate_context_snippet(index: &SkillIndex, skills_dir: &Path) -> String {
    let mut lines = vec![
        "## Available Skills".to_string(),
        String::new(),
        "| Skill | Description | Triggers |".to_string(),
        "|-------|-------------|----------|".to_string(),
    ];

    // Sort skills by name
    let mut skills: Vec<_> = index.skills.values().collect();
    skills.sort_by(|a, b| a.name.cmp(&b.name));

    for skill in &skills {
        let triggers_str = if skill.triggers.is_empty() {
            "-".to_string()
        } else {
            skill.triggers.join(", ")
        };

        // Truncate description for table
        let desc = if skill.description.len() > 60 {
            format!("{}...", &skill.description[..57])
        } else {
            skill.description.clone()
        };

        lines.push(format!("| **{}** | {} | {} |", skill.name, desc, triggers_str));
    }

    // Add workflow routing section if any skills have workflows
    let skills_with_workflows: Vec<_> = skills.iter().filter(|s| !s.workflows.is_empty()).collect();

    if !skills_with_workflows.is_empty() {
        lines.push(String::new());
        lines.push("## Workflow Routing".to_string());
        lines.push(String::new());
        lines.push("Some skills have specific workflows for common tasks:".to_string());
        lines.push(String::new());

        for skill in skills_with_workflows {
            lines.push(format!("### {}", skill.name));
            lines.push(String::new());
            lines.push("| Intent | Workflow |".to_string());
            lines.push("|--------|----------|".to_string());

            for route in &skill.workflows {
                lines.push(format!("| {} | `{}` |", route.intent, route.workflow));
            }
            lines.push(String::new());
        }

        lines.push("When a request matches a workflow intent:".to_string());
        lines.push(format!(
            "1. Read the workflow file from `{}/[skill-name]/[workflow-path]`",
            skills_dir.display()
        ));
        lines.push("2. Follow the step-by-step instructions in the workflow".to_string());
        lines.push(String::new());
    }

    lines.push("## Routing Instructions".to_string());
    lines.push(String::new());
    lines.push("When a user request matches a skill's triggers:".to_string());
    lines.push(format!(
        "1. Read the full SKILL.md file from `{}/[skill-name]/SKILL.md`",
        skills_dir.display()
    ));
    lines.push("2. Follow the skill's instructions and conventions".to_string());
    lines.push("3. No need to ask for permission - the skill is pre-approved".to_string());

    lines.join("\n")
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_extract_triggers_use_when() {
        let desc = "Write Rust code. USE WHEN creating Rust CLIs, libraries, or cargo projects.";
        let triggers = extract_triggers(desc);
        assert!(triggers.contains(&"rust".to_string()));
        assert!(triggers.contains(&"cli".to_string()));
        assert!(triggers.contains(&"cargo".to_string()));
    }

    #[test]
    fn test_extract_triggers_key_words() {
        let desc = "Manage Python projects with pip and pytest.";
        let triggers = extract_triggers(desc);
        assert!(triggers.contains(&"python".to_string()));
        assert!(triggers.contains(&"pip".to_string()));
    }

    #[test]
    fn test_extract_triggers_empty() {
        let desc = "A simple skill with no triggers.";
        let triggers = extract_triggers(desc);
        // Should still find nothing specific
        assert!(triggers.is_empty() || triggers.len() < 3);
    }
}
