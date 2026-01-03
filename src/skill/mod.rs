//! Skill discovery, loading, and management
//!
//! This module handles:
//! - Discovering skills from SKILL.md files
//! - Skills can exist without a plugin.toml (simple skills)
//! - Skills can also be part of full plugins
//!
//! # Simple Skills vs Plugin Skills
//!
//! - **Simple Skills**: Just SKILL.md, no executable code
//! - **Plugin Skills**: Part of a plugin with plugin.toml, contracts, hooks

#![allow(dead_code)]

use std::path::PathBuf;

pub mod loader;
pub mod parser;
pub mod scanner;
pub mod template;

/// A skill that teaches Claude how to do something
#[derive(Debug, Clone)]
pub struct Skill {
    /// Skill name (from frontmatter or directory name)
    pub name: String,
    /// What this skill does
    pub description: String,
    /// Path to the skill directory
    pub path: PathBuf,
    /// Where this skill came from
    pub source: SkillSource,
}

/// Where a skill was discovered from
#[derive(Debug, Clone, PartialEq)]
pub enum SkillSource {
    /// Simple skill - just SKILL.md, no plugin.toml
    Simple,
    /// Part of a full plugin
    Plugin(String),
    /// Discovered via scan (from .pais/ in a repo)
    Discovered(PathBuf),
}

impl Skill {
    /// Create a new simple skill
    pub fn new_simple(name: String, description: String, path: PathBuf) -> Self {
        Self {
            name,
            description,
            path,
            source: SkillSource::Simple,
        }
    }

    /// Create a skill that's part of a plugin
    pub fn from_plugin(name: String, description: String, path: PathBuf, plugin_name: String) -> Self {
        Self {
            name,
            description,
            path,
            source: SkillSource::Plugin(plugin_name),
        }
    }

    /// Create a discovered skill (from .pais/ in a repo)
    pub fn discovered(name: String, description: String, path: PathBuf, repo_path: PathBuf) -> Self {
        Self {
            name,
            description,
            path,
            source: SkillSource::Discovered(repo_path),
        }
    }

    /// Check if this is a simple skill (no plugin)
    pub fn is_simple(&self) -> bool {
        matches!(self.source, SkillSource::Simple)
    }

    /// Check if this is part of a plugin
    pub fn is_plugin_skill(&self) -> bool {
        matches!(self.source, SkillSource::Plugin(_))
    }

    /// Check if this was discovered from a repo
    pub fn is_discovered(&self) -> bool {
        matches!(self.source, SkillSource::Discovered(_))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_new_simple_skill() {
        let skill = Skill::new_simple(
            "terraform".to_string(),
            "Terraform best practices".to_string(),
            PathBuf::from("/home/test/.config/pais/skills/terraform"),
        );
        assert_eq!(skill.name, "terraform");
        assert!(skill.is_simple());
        assert!(!skill.is_plugin_skill());
        assert!(!skill.is_discovered());
    }

    #[test]
    fn test_plugin_skill() {
        let skill = Skill::from_plugin(
            "incident".to_string(),
            "Incident management".to_string(),
            PathBuf::from("/home/test/.config/pais/plugins/incident"),
            "incident-plugin".to_string(),
        );
        assert_eq!(skill.name, "incident");
        assert!(!skill.is_simple());
        assert!(skill.is_plugin_skill());
        assert!(!skill.is_discovered());
    }

    #[test]
    fn test_discovered_skill() {
        let skill = Skill::discovered(
            "aka".to_string(),
            "Alias management".to_string(),
            PathBuf::from("/home/test/repos/aka/.pais"),
            PathBuf::from("/home/test/repos/aka"),
        );
        assert_eq!(skill.name, "aka");
        assert!(!skill.is_simple());
        assert!(!skill.is_plugin_skill());
        assert!(skill.is_discovered());
    }
}
