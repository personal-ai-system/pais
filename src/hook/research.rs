//! Research path validation hook
//!
//! Validates that writes to the research directory follow the expected pattern:
//! `~/.config/pais/research/<category>/<topic>/<YYYY-MM-DD>.md`
//!
//! This enforces consistent research storage so Claude doesn't randomly place
//! research files in incorrect locations.

use lazy_regex::regex_is_match;
use std::path::PathBuf;

use super::{HookEvent, HookHandler, HookResult};

/// Known research categories
const KNOWN_CATEGORIES: &[&str] = &["tech", "building", "football", "writing", "management", "youtube"];

/// Research path validator hook handler
pub struct ResearchPathValidator {
    enabled: bool,
    research_dir: PathBuf,
}

impl ResearchPathValidator {
    pub fn new(enabled: bool) -> Self {
        let research_dir = dirs::config_dir()
            .unwrap_or_else(|| PathBuf::from("."))
            .join("pais")
            .join("research");

        Self { enabled, research_dir }
    }

    #[allow(dead_code)]
    pub fn with_research_dir(mut self, dir: PathBuf) -> Self {
        self.research_dir = dir;
        self
    }

    /// Check if a path is within the research directory
    fn is_research_path(&self, path: &str) -> bool {
        let expanded = shellexpand::tilde(path);
        let path_buf = PathBuf::from(expanded.as_ref());

        // Check various ways the research path might be specified
        path.contains(".config/pais/research/")
            || path.contains("config/pais/research/")
            || path_buf.starts_with(&self.research_dir)
    }

    /// Validate research path follows expected pattern
    /// Expected: <category>/<topic>/<YYYY-MM-DD>.md
    fn validate_research_path(&self, path: &str) -> Result<(), String> {
        let expanded = shellexpand::tilde(path);
        let path_str = expanded.as_ref();

        // Extract the part after .config/pais/research/
        let research_part = if let Some(idx) = path_str.find(".config/pais/research/") {
            &path_str[idx + ".config/pais/research/".len()..]
        } else if let Some(idx) = path_str.find("config/pais/research/") {
            &path_str[idx + "config/pais/research/".len()..]
        } else {
            return Err(format!("Could not parse research path: {}", path));
        };

        // Split into components: category/topic/filename
        let parts: Vec<&str> = research_part.split('/').collect();

        if parts.len() < 3 {
            return Err(format!(
                "Research path must have at least 3 components: <category>/<topic>/<date>.md\n\
                 Got {} components: {:?}\n\
                 Expected pattern: ~/.config/pais/research/<category>/<topic>/<YYYY-MM-DD>.md",
                parts.len(),
                parts
            ));
        }

        let category = parts[0];
        let topic = parts[1];
        let filename = parts[parts.len() - 1];

        // Validate category (should be lowercase, known category)
        if !category.chars().all(|c| c.is_ascii_lowercase() || c == '-') {
            return Err(format!(
                "Category must be lowercase with hyphens only: '{}'\n\
                 Known categories: {:?}",
                category, KNOWN_CATEGORIES
            ));
        }

        // Validate topic (should be lowercase with hyphens)
        if !topic
            .chars()
            .all(|c| c.is_ascii_lowercase() || c.is_ascii_digit() || c == '-')
        {
            return Err(format!(
                "Topic must be lowercase with hyphens: '{}'\n\
                 Example: 'zapier-ceo-ai-stack', 'slack-mcp'",
                topic
            ));
        }

        // Validate filename is YYYY-MM-DD.md
        if !regex_is_match!(r"^\d{4}-\d{2}-\d{2}\.md$", filename) {
            return Err(format!(
                "Filename must be date format: '<YYYY-MM-DD>.md'\n\
                 Got: '{}'\n\
                 Example: '2026-01-05.md'",
                filename
            ));
        }

        Ok(())
    }
}

impl HookHandler for ResearchPathValidator {
    fn name(&self) -> &'static str {
        "research"
    }

    fn handles(&self, event: HookEvent) -> bool {
        self.enabled && event == HookEvent::PreToolUse
    }

    fn handle(&self, _event: HookEvent, payload: &serde_json::Value) -> HookResult {
        let tool_name = payload.get("tool_name").and_then(|v| v.as_str()).unwrap_or("");

        // Only check Write and Edit tools
        if !matches!(tool_name, "Write" | "Edit" | "MultiEdit") {
            return HookResult::Allow;
        }

        // Get the file path from tool input
        let file_path = payload
            .get("tool_input")
            .and_then(|v| v.get("file_path"))
            .and_then(|v| v.as_str())
            .unwrap_or("");

        // If not a research path, allow
        if !self.is_research_path(file_path) {
            return HookResult::Allow;
        }

        // Validate the research path structure
        match self.validate_research_path(file_path) {
            Ok(()) => {
                log::debug!("Research path validated: {}", file_path);
                HookResult::Allow
            }
            Err(message) => {
                log::warn!("Invalid research path blocked: {}", file_path);
                HookResult::Block {
                    message: format!(
                        "❌ INVALID RESEARCH PATH\n\n\
                         {}\n\n\
                         📁 Expected structure:\n\
                         ~/.config/pais/research/<category>/<topic>/<YYYY-MM-DD>.md\n\n\
                         📂 Example:\n\
                         ~/.config/pais/research/tech/zapier-ceo-ai-stack/2026-01-05.md\n\n\
                         🔍 Check the skill's Output Storage section for the correct path.",
                        message
                    ),
                }
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn make_payload(tool: &str, path: &str) -> serde_json::Value {
        serde_json::json!({
            "tool_name": tool,
            "tool_input": {
                "file_path": path
            }
        })
    }

    #[test]
    fn test_allows_non_research_paths() {
        let validator = ResearchPathValidator::new(true);
        let payload = make_payload("Write", "/home/user/code/foo.rs");
        let result = validator.handle(HookEvent::PreToolUse, &payload);
        assert!(matches!(result, HookResult::Allow));
    }

    #[test]
    fn test_allows_valid_research_path() {
        let validator = ResearchPathValidator::new(true);
        let payload = make_payload("Write", "~/.config/pais/research/tech/slack-mcp/2026-01-05.md");
        let result = validator.handle(HookEvent::PreToolUse, &payload);
        assert!(matches!(result, HookResult::Allow));
    }

    #[test]
    fn test_allows_valid_research_path_absolute() {
        let validator = ResearchPathValidator::new(true);
        let payload = make_payload(
            "Write",
            "/home/saidler/.config/pais/research/tech/zapier-ceo/2026-01-05.md",
        );
        let result = validator.handle(HookEvent::PreToolUse, &payload);
        assert!(matches!(result, HookResult::Allow));
    }

    #[test]
    fn test_blocks_missing_topic() {
        let validator = ResearchPathValidator::new(true);
        let payload = make_payload("Write", "~/.config/pais/research/tech/2026-01-05.md");
        let result = validator.handle(HookEvent::PreToolUse, &payload);
        assert!(matches!(result, HookResult::Block { .. }));
    }

    #[test]
    fn test_blocks_wrong_filename_format() {
        let validator = ResearchPathValidator::new(true);
        let payload = make_payload("Write", "~/.config/pais/research/tech/slack-mcp/my-research.md");
        let result = validator.handle(HookEvent::PreToolUse, &payload);
        assert!(matches!(result, HookResult::Block { .. }));
    }

    #[test]
    fn test_blocks_uppercase_category() {
        let validator = ResearchPathValidator::new(true);
        let payload = make_payload("Write", "~/.config/pais/research/Tech/slack-mcp/2026-01-05.md");
        let result = validator.handle(HookEvent::PreToolUse, &payload);
        assert!(matches!(result, HookResult::Block { .. }));
    }

    #[test]
    fn test_blocks_underscore_in_topic() {
        let validator = ResearchPathValidator::new(true);
        let payload = make_payload("Write", "~/.config/pais/research/tech/slack_mcp/2026-01-05.md");
        let result = validator.handle(HookEvent::PreToolUse, &payload);
        assert!(matches!(result, HookResult::Block { .. }));
    }

    #[test]
    fn test_ignores_non_write_tools() {
        let validator = ResearchPathValidator::new(true);
        let payload = make_payload("Read", "~/.config/pais/research/invalid");
        let result = validator.handle(HookEvent::PreToolUse, &payload);
        assert!(matches!(result, HookResult::Allow));
    }

    #[test]
    fn test_disabled_validator_allows_all() {
        let validator = ResearchPathValidator::new(false);
        let _payload = make_payload("Write", "~/.config/pais/research/invalid");
        // When disabled, handles() returns false so handle() won't be called
        assert!(!validator.handles(HookEvent::PreToolUse));
    }

    #[test]
    fn test_only_handles_pre_tool_use() {
        let validator = ResearchPathValidator::new(true);
        assert!(validator.handles(HookEvent::PreToolUse));
        assert!(!validator.handles(HookEvent::PostToolUse));
        assert!(!validator.handles(HookEvent::Stop));
    }
}
