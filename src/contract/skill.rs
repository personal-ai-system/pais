//! SkillProvider contract
//!
//! Plugins that provide Claude Code skills.

use std::collections::HashMap;
use std::path::PathBuf;

/// SkillProvider contract interface
pub trait SkillProvider: Send + Sync {
    /// Skill name (matches SKILL.md name field)
    fn skill_name(&self) -> &str;

    /// Path to SKILL.md file
    fn skill_path(&self) -> PathBuf;

    /// Match confidence for an intent (0.0 - 1.0)
    fn match_intent(&self, intent: &str) -> f32;

    /// Execute a skill action
    fn execute(
        &self,
        action: &str,
        context: HashMap<String, serde_json::Value>,
    ) -> eyre::Result<HashMap<String, serde_json::Value>>;
}
