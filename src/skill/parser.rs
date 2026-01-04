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
//! tier: deferred  # optional: core, deferred (default)
//! ---
//!
//! # Terraform
//!
//! ## USE WHEN
//! ...
//! ```

use eyre::{Context, Result};
use serde::{Deserialize, Serialize};
use std::fmt;
use std::fs;
use std::path::Path;
use std::str::FromStr;

/// Skill loading tier
///
/// Determines when and how much of a skill is loaded into context.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum SkillTier {
    /// Tier 0: Always present, full content injected at session start
    Core,
    /// Tier 1: Name + description + triggers in context (default)
    /// Full body loaded on explicit invocation
    #[default]
    Deferred,
}

impl Serialize for SkillTier {
    fn serialize<S>(&self, serializer: S) -> std::result::Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        serializer.serialize_str(&self.to_string())
    }
}

impl<'de> Deserialize<'de> for SkillTier {
    fn deserialize<D>(deserializer: D) -> std::result::Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        struct TierVisitor;

        impl serde::de::Visitor<'_> for TierVisitor {
            type Value = SkillTier;

            fn expecting(&self, formatter: &mut fmt::Formatter) -> fmt::Result {
                formatter.write_str("a tier value (core, deferred, 0, or 1)")
            }

            fn visit_str<E>(self, v: &str) -> std::result::Result<Self::Value, E>
            where
                E: serde::de::Error,
            {
                SkillTier::from_str(v).map_err(serde::de::Error::custom)
            }

            fn visit_u64<E>(self, v: u64) -> std::result::Result<Self::Value, E>
            where
                E: serde::de::Error,
            {
                match v {
                    0 => Ok(SkillTier::Core),
                    1 => Ok(SkillTier::Deferred),
                    _ => Err(serde::de::Error::custom(format!(
                        "Unknown tier number {}. Valid values: 0 (core), 1 (deferred)",
                        v
                    ))),
                }
            }

            fn visit_i64<E>(self, v: i64) -> std::result::Result<Self::Value, E>
            where
                E: serde::de::Error,
            {
                self.visit_u64(v as u64)
            }
        }

        deserializer.deserialize_any(TierVisitor)
    }
}

impl SkillTier {
    /// Check if this is a core tier skill
    pub fn is_core(&self) -> bool {
        matches!(self, SkillTier::Core)
    }
}

impl fmt::Display for SkillTier {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            SkillTier::Core => write!(f, "core"),
            SkillTier::Deferred => write!(f, "deferred"),
        }
    }
}

impl FromStr for SkillTier {
    type Err = eyre::Error;

    fn from_str(s: &str) -> Result<Self> {
        match s.to_lowercase().as_str() {
            "core" | "0" | "tier0" => Ok(SkillTier::Core),
            "deferred" | "1" | "tier1" | "frontmatter" => Ok(SkillTier::Deferred),
            _ => eyre::bail!("Unknown tier '{}'. Valid values: core, deferred", s),
        }
    }
}

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
    /// Loading tier (core or deferred)
    #[serde(default)]
    pub tier: SkillTier,
    /// Explicit trigger phrases from frontmatter
    #[serde(default)]
    pub triggers: Vec<String>,
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
