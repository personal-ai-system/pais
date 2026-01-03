//! SKILL.md frontmatter parsing
//!
//! Parses YAML frontmatter from SKILL.md files to extract metadata.
//!
//! # Format
//!
//! ```markdown
//! ---
//! name: terraform
//! description: Terraform best practices and patterns
//! ---
//!
//! # Terraform
//!
//! ## USE WHEN
//! ...
//! ```

use eyre::{Context, Result};
use serde::Deserialize;
use std::fs;
use std::path::Path;

/// Metadata extracted from SKILL.md frontmatter
#[derive(Debug, Clone, Deserialize)]
pub struct SkillMetadata {
    /// Skill name
    pub name: String,
    /// What this skill does
    #[serde(default)]
    pub description: String,
    /// Optional tags for categorization
    #[serde(default)]
    pub tags: Vec<String>,
    /// Optional version
    #[serde(default)]
    pub version: Option<String>,
}

/// Parse SKILL.md and extract frontmatter metadata
pub fn parse_skill_md(path: &Path) -> Result<SkillMetadata> {
    let content = fs::read_to_string(path).with_context(|| format!("Failed to read SKILL.md at {}", path.display()))?;

    parse_frontmatter(&content).with_context(|| format!("Failed to parse frontmatter in {}", path.display()))
}

/// Parse YAML frontmatter from markdown content
fn parse_frontmatter(content: &str) -> Result<SkillMetadata> {
    // Check for frontmatter delimiter
    let content = content.trim();
    if !content.starts_with("---") {
        eyre::bail!("SKILL.md must start with YAML frontmatter (---)");
    }

    // Find the end of frontmatter
    let rest = &content[3..];
    let end_pos = rest
        .find("\n---")
        .or_else(|| rest.find("\r\n---"))
        .ok_or_else(|| eyre::eyre!("No closing frontmatter delimiter (---) found"))?;

    let yaml_content = &rest[..end_pos].trim();

    // Parse YAML
    let metadata: SkillMetadata = serde_yaml::from_str(yaml_content).context("Failed to parse YAML frontmatter")?;

    Ok(metadata)
}

/// Check if a directory contains a SKILL.md file
pub fn has_skill_md(dir: &Path) -> bool {
    dir.join("SKILL.md").exists()
}

/// Check if a directory is a simple skill (SKILL.md but no plugin.yaml)
pub fn is_simple_skill(dir: &Path) -> bool {
    has_skill_md(dir) && !dir.join("plugin.yaml").exists()
}

/// Check if a directory is a full plugin (has plugin.yaml)
pub fn is_plugin(dir: &Path) -> bool {
    dir.join("plugin.yaml").exists()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_frontmatter_valid() {
        let content = r#"---
name: terraform
description: Terraform best practices
tags:
  - infrastructure
  - cloud
---

# Terraform

## USE WHEN

Working with infrastructure
"#;

        let metadata = parse_frontmatter(content).unwrap();
        assert_eq!(metadata.name, "terraform");
        assert_eq!(metadata.description, "Terraform best practices");
        assert_eq!(metadata.tags, vec!["infrastructure", "cloud"]);
    }

    #[test]
    fn test_parse_frontmatter_minimal() {
        let content = r#"---
name: simple
---

# Simple Skill
"#;

        let metadata = parse_frontmatter(content).unwrap();
        assert_eq!(metadata.name, "simple");
        assert_eq!(metadata.description, "");
        assert!(metadata.tags.is_empty());
    }

    #[test]
    fn test_parse_frontmatter_no_delimiter() {
        let content = "# No Frontmatter\n\nJust content";
        let result = parse_frontmatter(content);
        assert!(result.is_err());
    }

    #[test]
    fn test_parse_frontmatter_no_closing() {
        let content = "---\nname: broken\n# Missing closing";
        let result = parse_frontmatter(content);
        assert!(result.is_err());
    }

    #[test]
    fn test_parse_frontmatter_missing_name() {
        let content = r#"---
description: No name field
---

Content
"#;
        let result = parse_frontmatter(content);
        assert!(result.is_err());
    }
}
