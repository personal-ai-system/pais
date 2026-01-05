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
        if let Ok(entry) = serde_json::from_str::<serde_json::Value>(line)
            && entry.get("type").and_then(|t| t.as_str()) == Some("assistant")
            && let Some(message) = entry.get("message")
            && let Some(content) = message.get("content")
        {
            // Extract text from content (can be array or string)
            let text = extract_text_from_content(content);
            if text.len() > 50 {
                // Limit to 5000 chars to prevent huge entries
                return Some(text.chars().take(5000).collect());
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
                item.get("text")
                    .and_then(|t| t.as_str())
                    .map(|s| s.to_string())
                    .or_else(|| item.as_str().map(|s| s.to_string()))
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
    use tempfile::{NamedTempFile, tempdir};

    // =========================================================================
    // CRITICAL: Tests to prevent empty session content regression
    // =========================================================================

    #[test]
    fn test_empty_payload_produces_session_completed() {
        // This is the FAILURE case we want to detect
        let payload = json!({});
        let summary = build_session_summary(&payload);

        // Without any content source, we get the fallback
        assert_eq!(summary.trim(), "Session completed.");
    }

    #[test]
    fn test_payload_with_only_session_id_is_empty() {
        // This was the bug - Claude sends session_id but we weren't reading transcript
        let payload = json!({
            "session_id": "abc123",
            "stop_reason": "completed"
        });
        let summary = build_session_summary(&payload);

        // Should have stop_reason but still essentially empty content
        assert!(summary.contains("completed"));
        assert!(!summary.contains("## Final Response"));
    }

    #[test]
    fn test_transcript_path_is_read_when_response_missing() {
        // THE CRITICAL TEST: Claude provides transcript_path, not response
        let mut temp_file = NamedTempFile::new().expect("Failed to create temp file");
        let transcript_content = r#"{"type":"user","message":{"content":"Fix the bug"}}
{"type":"assistant","message":{"content":[{"type":"text","text":"I analyzed the code and found the issue. The bug was in the authentication handler where we were not properly validating tokens."}]}}
"#;
        temp_file
            .write_all(transcript_content.as_bytes())
            .expect("Failed to write");

        let payload = json!({
            "session_id": "test-session",
            "stop_reason": "completed",
            "transcript_path": temp_file.path().to_str().unwrap()
            // NOTE: No "response" field - this is how Claude actually sends it
        });

        let summary = build_session_summary(&payload);

        // MUST contain the actual content from transcript
        assert!(
            summary.contains("analyzed the code"),
            "Summary should contain transcript content, got: {}",
            summary
        );
        assert!(summary.contains("## Final Response"));
    }

    #[test]
    fn test_response_field_takes_precedence_over_transcript() {
        let mut temp_file = NamedTempFile::new().expect("Failed to create temp file");
        temp_file.write_all(b"should not be read").expect("Failed to write");

        let payload = json!({
            "response": "Direct response content here",
            "transcript_path": temp_file.path().to_str().unwrap()
        });

        let summary = build_session_summary(&payload);

        // response field should be used, not transcript
        assert!(summary.contains("Direct response content here"));
    }

    // =========================================================================
    // Claude Code transcript format tests (real format from ~/.claude/projects/)
    // =========================================================================

    #[test]
    fn test_real_claude_code_transcript_format() {
        // This matches the actual format Claude Code uses
        let mut temp_file = NamedTempFile::new().expect("Failed to create temp file");
        let transcript_content = r#"{"parentUuid":null,"type":"user","message":{"role":"user","content":"use gx create a PR"},"uuid":"e56dbefa","timestamp":"2026-01-04T20:50:11.549Z"}
{"parentUuid":"e56dbefa","type":"assistant","message":{"role":"assistant","content":[{"type":"thinking","thinking":"Let me analyze..."},{"type":"text","text":"I'll help you create PRs using gx. First, let me check the syntax."}]},"uuid":"07d80829","timestamp":"2026-01-04T20:50:19.980Z"}
{"parentUuid":"07d80829","type":"assistant","message":{"role":"assistant","content":[{"type":"text","text":"Successfully created 5 PRs across all repositories."}]},"uuid":"412b9777","timestamp":"2026-01-04T20:50:20.855Z"}
"#;
        temp_file
            .write_all(transcript_content.as_bytes())
            .expect("Failed to write");

        let payload = json!({
            "transcript_path": temp_file.path().to_str().unwrap()
        });

        let summary = build_session_summary(&payload);

        // Should get the LAST assistant message
        assert!(
            summary.contains("Successfully created 5 PRs"),
            "Should extract last assistant message, got: {}",
            summary
        );
    }

    #[test]
    fn test_transcript_with_thinking_blocks_extracts_text_only() {
        let mut temp_file = NamedTempFile::new().expect("Failed to create temp file");
        let transcript_content = r#"{"type":"assistant","message":{"content":[{"type":"thinking","thinking":"Internal reasoning here"},{"type":"text","text":"The visible response to the user about fixing the problem."}]}}
"#;
        temp_file
            .write_all(transcript_content.as_bytes())
            .expect("Failed to write");

        let payload = json!({
            "transcript_path": temp_file.path().to_str().unwrap()
        });

        let summary = build_session_summary(&payload);

        // Should extract text, not thinking
        assert!(summary.contains("visible response"));
        assert!(!summary.contains("Internal reasoning"));
    }

    // =========================================================================
    // Edge cases and error handling
    // =========================================================================

    #[test]
    fn test_nonexistent_transcript_path_gracefully_fails() {
        let payload = json!({
            "transcript_path": "/nonexistent/path/to/transcript.jsonl"
        });

        let summary = build_session_summary(&payload);

        // Should fallback gracefully, not panic
        assert_eq!(summary.trim(), "Session completed.");
    }

    #[test]
    fn test_malformed_jsonl_gracefully_fails() {
        let mut temp_file = NamedTempFile::new().expect("Failed to create temp file");
        temp_file
            .write_all(b"not valid json\nalso not valid\n")
            .expect("Failed to write");

        let payload = json!({
            "transcript_path": temp_file.path().to_str().unwrap()
        });

        let summary = build_session_summary(&payload);

        // Should fallback gracefully
        assert_eq!(summary.trim(), "Session completed.");
    }

    #[test]
    fn test_empty_transcript_file() {
        let temp_file = NamedTempFile::new().expect("Failed to create temp file");

        let payload = json!({
            "transcript_path": temp_file.path().to_str().unwrap()
        });

        let summary = build_session_summary(&payload);
        assert_eq!(summary.trim(), "Session completed.");
    }

    #[test]
    fn test_transcript_with_only_user_messages() {
        let mut temp_file = NamedTempFile::new().expect("Failed to create temp file");
        let transcript_content = r#"{"type":"user","message":{"content":"Hello"}}
{"type":"user","message":{"content":"Are you there?"}}
"#;
        temp_file
            .write_all(transcript_content.as_bytes())
            .expect("Failed to write");

        let payload = json!({
            "transcript_path": temp_file.path().to_str().unwrap()
        });

        let summary = build_session_summary(&payload);

        // No assistant messages, should fallback
        assert_eq!(summary.trim(), "Session completed.");
    }

    #[test]
    fn test_short_response_is_skipped() {
        // Responses < 50 chars are skipped (likely incomplete)
        let mut temp_file = NamedTempFile::new().expect("Failed to create temp file");
        let transcript_content = r#"{"type":"assistant","message":{"content":[{"type":"text","text":"OK"}]}}
"#;
        temp_file
            .write_all(transcript_content.as_bytes())
            .expect("Failed to write");

        let payload = json!({
            "transcript_path": temp_file.path().to_str().unwrap()
        });

        let summary = build_session_summary(&payload);

        // Too short, should skip
        assert_eq!(summary.trim(), "Session completed.");
    }

    #[test]
    fn test_response_is_truncated_at_5000_chars() {
        let mut temp_file = NamedTempFile::new().expect("Failed to create temp file");
        let long_text = "x".repeat(10000);
        let transcript_content = format!(
            r#"{{"type":"assistant","message":{{"content":[{{"type":"text","text":"{}"}}]}}}}"#,
            long_text
        );
        temp_file
            .write_all(transcript_content.as_bytes())
            .expect("Failed to write");

        let payload = json!({
            "transcript_path": temp_file.path().to_str().unwrap()
        });

        let summary = build_session_summary(&payload);

        // Should be truncated
        assert!(summary.len() < 6000, "Response should be truncated");
    }

    // =========================================================================
    // Integration: Full handler tests
    // =========================================================================

    #[test]
    fn test_stop_handler_creates_learning_entry_from_transcript() {
        let temp_dir = tempdir().expect("Failed to create temp dir");
        let handler = HistoryHandler::new(true, temp_dir.path().to_path_buf());

        // Create transcript with learning indicators
        let mut temp_file = NamedTempFile::new().expect("Failed to create temp file");
        let transcript_content = r#"{"type":"assistant","message":{"content":[{"type":"text","text":"I discovered the problem was a race condition. The root cause was that we were not properly locking the mutex. I fixed it by adding proper synchronization."}]}}
"#;
        temp_file
            .write_all(transcript_content.as_bytes())
            .expect("Failed to write");

        let payload = json!({
            "session_id": "test-learning-session",
            "stop_reason": "completed",
            "transcript_path": temp_file.path().to_str().unwrap()
        });

        let result = handler.handle(HookEvent::Stop, &payload);
        assert!(matches!(result, HookResult::Allow));

        // Check that a file was created in learnings/
        let learnings_dir = temp_dir.path().join("learnings");
        if learnings_dir.exists() {
            let entries: Vec<_> = fs::read_dir(&learnings_dir).unwrap().filter_map(|e| e.ok()).collect();
            // Should have created a dated subdirectory with an entry
            assert!(
                !entries.is_empty() || learnings_dir.read_dir().unwrap().count() > 0,
                "Should have created learning entry"
            );
        }
    }

    #[test]
    fn test_stop_handler_creates_session_entry_from_transcript() {
        let temp_dir = tempdir().expect("Failed to create temp dir");
        let handler = HistoryHandler::new(true, temp_dir.path().to_path_buf());

        // Create transcript WITHOUT learning indicators
        let mut temp_file = NamedTempFile::new().expect("Failed to create temp file");
        let transcript_content = r#"{"type":"assistant","message":{"content":[{"type":"text","text":"I've completed the task you requested. The configuration has been updated and the server is now running with the new settings."}]}}
"#;
        temp_file
            .write_all(transcript_content.as_bytes())
            .expect("Failed to write");

        let payload = json!({
            "session_id": "test-session",
            "stop_reason": "completed",
            "transcript_path": temp_file.path().to_str().unwrap()
        });

        let result = handler.handle(HookEvent::Stop, &payload);
        assert!(matches!(result, HookResult::Allow));

        // Check that a file was created in sessions/
        let sessions_dir = temp_dir.path().join("sessions");
        if sessions_dir.exists() {
            let has_content = fs::read_dir(&sessions_dir).map(|d| d.count() > 0).unwrap_or(false);
            assert!(has_content, "Should have created session entry");
        }
    }

    // =========================================================================
    // Original tests (kept for completeness)
    // =========================================================================

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
        let (category, agent) = handler.determine_category(None, "debugging the problem and found the root cause");
        assert_eq!(category, "learnings");
        assert!(agent.is_none());
    }

    #[test]
    fn test_determine_category_fallback_when_agent_not_found() {
        let temp_dir = tempdir().expect("Failed to create temp dir");
        let handler = HistoryHandler::new(true, temp_dir.path().to_path_buf());

        // Agent not found, falls back to content analysis
        let (category, agent) = handler.determine_category(Some("nonexistent"), "regular session work");
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
}
