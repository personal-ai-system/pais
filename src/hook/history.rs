//! History hook handler
//!
//! Captures session summaries on Stop events.

use std::path::PathBuf;

use super::{HookEvent, HookHandler, HookResult};
use crate::history::{HistoryEntry, HistoryStore};

/// History hook handler - captures session data
pub struct HistoryHandler {
    enabled: bool,
    history_path: PathBuf,
}

impl HistoryHandler {
    pub fn new(enabled: bool, history_path: PathBuf) -> Self {
        Self { enabled, history_path }
    }

    fn capture_session(&self, payload: &serde_json::Value) -> HookResult {
        // Extract session info from payload
        let session_id = payload.get("session_id").and_then(|v| v.as_str()).unwrap_or("unknown");

        // Get the stop reason and any summary
        let stop_reason = payload
            .get("stop_reason")
            .and_then(|v| v.as_str())
            .unwrap_or("completed");

        // Build summary from available info
        let summary = build_session_summary(payload);

        // Create history entry
        let title = format!("Session {}", &session_id[..8.min(session_id.len())]);
        let entry = HistoryEntry::new("sessions", &title, &summary)
            .with_tag(stop_reason)
            .with_metadata("session_id", session_id);

        // Store it
        let store = HistoryStore::new(self.history_path.clone());
        match store.store(&entry) {
            Ok(path) => {
                log::info!("Captured session to: {}", path.display());
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
}

impl HookHandler for HistoryHandler {
    fn handles(&self, event: HookEvent) -> bool {
        self.enabled && event == HookEvent::Stop
    }

    fn handle(&self, event: HookEvent, payload: &serde_json::Value) -> HookResult {
        match event {
            HookEvent::Stop => self.capture_session(payload),
            _ => HookResult::Allow,
        }
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

    // Add any assistant message that might be the final response
    if let Some(response) = payload.get("response").and_then(|v| v.as_str()) {
        summary.push_str("## Final Response\n\n");
        summary.push_str(response);
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
}
