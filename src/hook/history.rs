//! History hook handler
//!
//! Captures session lifecycle events: SessionStart, Stop, SubagentStop, SessionEnd.
//!
//! On Stop/SubagentStop, content is analyzed to categorize as:
//! - Agent's `history_category` if agent detected
//! - `learnings`: If contains problem-solving narratives
//! - `sessions`: Default for regular work sessions
//!
//! ## Transcript Reading
//!
//! Claude Code provides `transcript_path` in Stop events, pointing to the session's
//! JSONL file. We read this to extract the actual conversation content.

#![allow(dead_code)] // with_agents_dir - for testing/custom config

use std::fs;
use std::path::PathBuf;

use super::{HookEvent, HookHandler, HookResult};
use crate::agent::loader::AgentLoader;
use crate::history::categorize::{categorize_content, extract_summary, extract_tags};
use crate::history::{HistoryEntry, HistoryStore};

/// History hook handler - captures session lifecycle data
pub struct HistoryHandler {
    enabled: bool,
    history_path: PathBuf,
    agents_dir: PathBuf,
}

impl HistoryHandler {
    pub fn new(enabled: bool, history_path: PathBuf) -> Self {
        // Default agents dir is sibling to history (e.g., ~/.config/pais/agents)
        let agents_dir = history_path
            .parent()
            .map(|p| p.join("agents"))
            .unwrap_or_else(|| history_path.join("../agents"));

        Self {
            enabled,
            history_path,
            agents_dir,
        }
    }

    /// Set a custom agents directory
    pub fn with_agents_dir(mut self, agents_dir: PathBuf) -> Self {
        self.agents_dir = agents_dir;
        self
    }

    fn on_session_start(&self, payload: &serde_json::Value) -> HookResult {
        let session_id = payload.get("session_id").and_then(|v| v.as_str()).unwrap_or("unknown");

        let cwd = payload.get("cwd").and_then(|v| v.as_str()).unwrap_or("unknown");

        let is_resumed = payload.get("is_resumed").and_then(|v| v.as_bool()).unwrap_or(false);

        let session_type = if is_resumed { "resumed" } else { "new" };

        // Log session start
        log::info!(
            "Session started: {} ({}) in {}",
            &session_id[..8.min(session_id.len())],
            session_type,
            cwd
        );

        // Create a brief entry for session starts
        let content = format!(
            "Session {} in `{}`\n\nType: {}",
            session_type,
            cwd,
            if is_resumed { "Resumed" } else { "New" }
        );

        let title = format!("Session {} started", &session_id[..8.min(session_id.len())]);
        let entry = HistoryEntry::new("events", &title, &content)
            .with_tag("session_start")
            .with_tag(session_type)
            .with_metadata("session_id", session_id)
            .with_metadata("cwd", cwd);

        let store = HistoryStore::new(self.history_path.clone());
        if let Err(e) = store.store(&entry) {
            log::error!("Failed to log session start: {}", e);
        }

        HookResult::Allow
    }

    fn on_stop(&self, payload: &serde_json::Value) -> HookResult {
        self.capture_stop_event(payload, None)
    }

    fn on_subagent_stop(&self, payload: &serde_json::Value) -> HookResult {
        // Extract agent type from payload
        let agent_type = payload
            .get("subagent_type")
            .or_else(|| payload.get("agent_type"))
            .or_else(|| payload.get("agent"))
            .and_then(|v| v.as_str());

        self.capture_stop_event(payload, agent_type)
    }

    /// Shared logic for Stop and SubagentStop events
    fn capture_stop_event(&self, payload: &serde_json::Value, agent_type: Option<&str>) -> HookResult {
        let session_id = payload.get("session_id").and_then(|v| v.as_str()).unwrap_or("unknown");

        let stop_reason = payload
            .get("stop_reason")
            .and_then(|v| v.as_str())
            .unwrap_or("completed");

        // Build summary from available info
        let summary = build_session_summary(payload);

        // Determine category - agent takes precedence over content analysis
        let (category_name, agent_name) = self.determine_category(agent_type, &summary);
        let extracted_title = extract_summary(&summary, 60);
        let tags = extract_tags(&summary);

        // Use extracted title or fallback to session ID
        let title = if extracted_title != "Untitled" && !extracted_title.is_empty() {
            extracted_title
        } else {
            format!("Session {}", &session_id[..8.min(session_id.len())])
        };

        // Create history entry with determined category
        let mut entry = HistoryEntry::new(&category_name, &title, &summary)
            .with_tag(stop_reason)
            .with_metadata("session_id", session_id)
            .with_metadata("category", &category_name);

        // Add agent metadata if present
        if let Some(agent) = agent_name {
            entry = entry
                .with_tag(&format!("agent:{}", agent))
                .with_metadata("agent", &agent);
        }

        // Add extracted tags
        for tag in tags {
            entry = entry.with_tag(&tag);
        }

        let store = HistoryStore::new(self.history_path.clone());
        match store.store(&entry) {
            Ok(path) => {
                log::info!("Captured {} to: {}", category_name, path.display());
                HookResult::Allow
            }
            Err(e) => {
                log::error!("Failed to capture session: {}", e);
                HookResult::Error {
                    message: format!("Failed to store session: {}", e),
                }
            }
        }
    }

    /// Determine history category from agent or content analysis
    /// Returns (category_name, optional_agent_name)
    fn determine_category(&self, agent_type: Option<&str>, content: &str) -> (String, Option<String>) {
        // If agent type provided, try to load agent and get its history_category
        if let Some(agent_name) = agent_type {
            let loader = AgentLoader::new(self.agents_dir.clone());
            let agent_path = self.agents_dir.join(format!("{}.yaml", agent_name.to_lowercase()));

            if let Ok(agent) = loader.load_agent(&agent_path) {
                if let Some(category) = agent.history_category {
                    log::info!("Using agent '{}' history category: {}", agent_name, category);
                    return (category, Some(agent_name.to_string()));
                }
            } else {
                log::debug!("Agent '{}' not found, falling back to content analysis", agent_name);
            }
        }

        // Fall back to content-based categorization
        let category = categorize_content(content);
        (category.dir_name().to_string(), None)
    }

    fn on_session_end(&self, payload: &serde_json::Value) -> HookResult {
        let session_id = payload.get("session_id").and_then(|v| v.as_str()).unwrap_or("unknown");

        log::info!("Session ended: {}", &session_id[..8.min(session_id.len())]);

        // Create an event entry for session end
        let title = format!("Session {} ended", &session_id[..8.min(session_id.len())]);
        let entry = HistoryEntry::new("events", &title, "Session completed.")
            .with_tag("session_end")
            .with_metadata("session_id", session_id);

        let store = HistoryStore::new(self.history_path.clone());
        if let Err(e) = store.store(&entry) {
            log::error!("Failed to log session end: {}", e);
        }

        HookResult::Allow
    }
}

impl HookHandler for HistoryHandler {
    fn name(&self) -> &'static str {
        "history"
    }

    fn handles(&self, event: HookEvent) -> bool {
        self.enabled
            && matches!(
                event,
                HookEvent::SessionStart | HookEvent::Stop | HookEvent::SubagentStop | HookEvent::SessionEnd
            )
    }

    fn handle(&self, event: HookEvent, payload: &serde_json::Value) -> HookResult {
        match event {
            HookEvent::SessionStart => self.on_session_start(payload),
            HookEvent::Stop => self.on_stop(payload),
            HookEvent::SubagentStop => self.on_subagent_stop(payload),
            HookEvent::SessionEnd => self.on_session_end(payload),
            _ => HookResult::Allow,
        }
    }
}

/// Extract the last assistant response from a Claude Code transcript file.
///
/// Claude Code provides `transcript_path` in Stop events, pointing to a JSONL file
/// containing the full conversation. We read backwards to find the last assistant message.
fn extract_response_from_transcript(transcript_path: &str) -> Option<String> {
    let content = fs::read_to_string(transcript_path).ok()?;
    let lines: Vec<&str> = content.lines().filter(|l| !l.trim().is_empty()).collect();

    // Parse backwards to find the last assistant message
    for line in lines.iter().rev() {
        if let Ok(entry) = serde_json::from_str::<serde_json::Value>(line) {
            // Check if this is an assistant message
            if entry.get("type").and_then(|t| t.as_str()) == Some("assistant") {
                if let Some(message) = entry.get("message") {
                    if let Some(content) = message.get("content") {
                        // Extract text from content (can be array or string)
                        let text = extract_text_from_content(content);
                        if text.len() > 50 {
                            // Limit to 5000 chars to prevent huge entries
                            return Some(text.chars().take(5000).collect());
                        }
                    }
                }
            }
        }
    }

    None
}

/// Extract text from Claude's message content (handles array of content blocks)
fn extract_text_from_content(content: &serde_json::Value) -> String {
    match content {
        serde_json::Value::String(s) => s.clone(),
        serde_json::Value::Array(arr) => arr
            .iter()
            .filter_map(|item| {
                // Handle {"type": "text", "text": "..."} blocks
                if let Some(text) = item.get("text").and_then(|t| t.as_str()) {
                    Some(text.to_string())
                } else if let Some(s) = item.as_str() {
                    Some(s.to_string())
                } else {
                    None
                }
            })
            .collect::<Vec<_>>()
            .join("\n"),
        _ => String::new(),
    }
}

/// Build a session summary from the Stop payload
fn build_session_summary(payload: &serde_json::Value) -> String {
    let mut summary = String::new();

    // Extract what we can from the payload
    if let Some(conversation) = payload.get("conversation")
        && let Some(messages) = conversation.as_array()
    {
        summary.push_str("## Conversation Summary\n\n");
        summary.push_str(&format!("Messages exchanged: {}\n\n", messages.len()));
    }

    // Add stop reason
    if let Some(reason) = payload.get("stop_reason").and_then(|v| v.as_str()) {
        summary.push_str(&format!("**Stop reason:** {}\n\n", reason));
    }

    // Try to get response from payload first, then from transcript_path
    let response = payload
        .get("response")
        .and_then(|v| v.as_str())
        .map(|s| s.to_string())
        .or_else(|| {
            payload
                .get("transcript_path")
                .and_then(|v| v.as_str())
                .and_then(extract_response_from_transcript)
        });

    if let Some(response_text) = response {
        summary.push_str("## Final Response\n\n");
        summary.push_str(&response_text);
        summary.push_str("\n\n");
    }

    // If we got tools used
    if let Some(tools) = payload.get("tools_used").and_then(|v| v.as_array())
        && !tools.is_empty()
    {
        summary.push_str("## Tools Used\n\n");
        for tool in tools {
            if let Some(name) = tool.as_str() {
                summary.push_str(&format!("- {}\n", name));
            }
        }
        summary.push('\n');
    }

    // If summary is empty, add a note
    if summary.is_empty() {
        summary.push_str("Session completed.\n");
    }

    summary
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;
    use std::io::Write;
    use tempfile::{tempdir, NamedTempFile};

    #[test]
    fn test_build_session_summary() {
        let payload = json!({
            "stop_reason": "user_request",
            "tools_used": ["Bash", "Edit"]
        });

        let summary = build_session_summary(&payload);
        assert!(summary.contains("user_request"));
        assert!(summary.contains("Bash"));
        assert!(summary.contains("Edit"));
    }

    #[test]
    fn test_build_session_summary_with_transcript() {
        // Create a fake transcript file
        let mut temp_file = NamedTempFile::new().expect("Failed to create temp file");
        let transcript_content = r#"{"type":"user","message":{"content":"Hello"}}
{"type":"assistant","message":{"content":[{"type":"text","text":"I fixed the bug by updating the configuration. The root cause was a missing environment variable that caused the authentication to fail."}]}}
"#;
        temp_file
            .write_all(transcript_content.as_bytes())
            .expect("Failed to write");

        let payload = json!({
            "stop_reason": "completed",
            "transcript_path": temp_file.path().to_str().unwrap()
        });

        let summary = build_session_summary(&payload);
        assert!(summary.contains("fixed the bug"));
        assert!(summary.contains("root cause"));
    }

    #[test]
    fn test_extract_text_from_content_string() {
        let content = json!("Hello world");
        let text = extract_text_from_content(&content);
        assert_eq!(text, "Hello world");
    }

    #[test]
    fn test_extract_text_from_content_array() {
        let content = json!([
            {"type": "text", "text": "First part"},
            {"type": "text", "text": "Second part"}
        ]);
        let text = extract_text_from_content(&content);
        assert!(text.contains("First part"));
        assert!(text.contains("Second part"));
    }

    #[test]
    fn test_determine_category_content_based() {
        let temp_dir = tempdir().expect("Failed to create temp dir");
        let handler = HistoryHandler::new(true, temp_dir.path().to_path_buf());

        // Content-based categorization (no agent)
        let (category, agent) =
            handler.determine_category(None, "debugging the problem and found the root cause");
        assert_eq!(category, "learnings");
        assert!(agent.is_none());
    }

    #[test]
    fn test_determine_category_fallback_when_agent_not_found() {
        let temp_dir = tempdir().expect("Failed to create temp dir");
        let handler = HistoryHandler::new(true, temp_dir.path().to_path_buf());

        // Agent not found, falls back to content analysis
        let (category, agent) =
            handler.determine_category(Some("nonexistent"), "regular session work");
        assert_eq!(category, "sessions");
        assert!(agent.is_none());
    }

    #[test]
    fn test_handles_subagent_stop() {
        let temp_dir = tempdir().expect("Failed to create temp dir");
        let handler = HistoryHandler::new(true, temp_dir.path().to_path_buf());

        assert!(handler.handles(HookEvent::SubagentStop));
        assert!(handler.handles(HookEvent::Stop));
        assert!(handler.handles(HookEvent::SessionStart));
        assert!(!handler.handles(HookEvent::PreToolUse));
    }

    #[test]
    fn test_transcript_with_learning_content_categorized_correctly() {
        // Create a transcript with learning indicators
        let mut temp_file = NamedTempFile::new().expect("Failed to create temp file");
        let transcript_content = r#"{"type":"assistant","message":{"content":[{"type":"text","text":"I discovered the problem was in the authentication module. The root cause was that we were using the wrong API endpoint. I fixed it by updating the configuration."}]}}
"#;
        temp_file
            .write_all(transcript_content.as_bytes())
            .expect("Failed to write");

        let payload = json!({
            "transcript_path": temp_file.path().to_str().unwrap()
        });

        let summary = build_session_summary(&payload);

        // The summary should contain learning indicators
        assert!(summary.contains("discovered"));
        assert!(summary.contains("root cause"));
        assert!(summary.contains("fixed"));
    }
}
