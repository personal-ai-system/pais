//! Event emitter with multiple sink support

#![allow(dead_code)] // has_stdout_sink - for observe command deduplication

use chrono::{Local, Utc};
use colored::*;
use serde::{Deserialize, Serialize};
use std::fs::{self, OpenOptions};
use std::io::Write;

use crate::config::{ObservabilityConfig, ObservabilitySink};
use crate::hook::HookEvent;

/// An observable event
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Event {
    /// Timestamp (UTC ISO 8601)
    pub timestamp: String,
    /// Local time for display
    pub local_time: String,
    /// Event type
    pub event_type: String,
    /// Session ID if available
    #[serde(skip_serializing_if = "Option::is_none")]
    pub session_id: Option<String>,
    /// Tool name if applicable
    #[serde(skip_serializing_if = "Option::is_none")]
    pub tool_name: Option<String>,
    /// Event payload (optional, can be large)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub payload: Option<serde_json::Value>,
}

impl Event {
    /// Create a new event from a hook event and payload
    pub fn from_hook(hook_event: HookEvent, payload: &serde_json::Value, include_payload: bool) -> Self {
        let now = Utc::now();
        let local = Local::now();

        let session_id = payload
            .get("session_id")
            .or_else(|| payload.get("sessionId"))
            .and_then(|v| v.as_str())
            .map(|s| s.to_string());

        let tool_name = payload
            .get("tool_name")
            .or_else(|| payload.get("toolName"))
            .and_then(|v| v.as_str())
            .map(|s| s.to_string());

        Self {
            timestamp: now.to_rfc3339(),
            local_time: local.format("%Y-%m-%d %H:%M:%S").to_string(),
            event_type: format!("{:?}", hook_event),
            session_id,
            tool_name,
            payload: if include_payload { Some(payload.clone()) } else { None },
        }
    }

    /// Format for stdout display
    pub fn format_display(&self) -> String {
        let event_colored = match self.event_type.as_str() {
            "SessionStart" => self.event_type.green(),
            "SessionEnd" => self.event_type.red(),
            "PreToolUse" => self.event_type.cyan(),
            "PostToolUse" => self.event_type.blue(),
            "Stop" => self.event_type.yellow(),
            _ => self.event_type.normal(),
        };

        let mut parts = vec![self.local_time.dimmed().to_string(), event_colored.to_string()];

        if let Some(ref session) = self.session_id {
            parts.push(format!("[{}]", &session[..8.min(session.len())]).dimmed().to_string());
        }

        if let Some(ref tool) = self.tool_name {
            parts.push(tool.bold().to_string());
        }

        parts.join(" ")
    }
}

/// Event emitter that sends to multiple sinks
pub struct EventEmitter {
    config: ObservabilityConfig,
    history_path: std::path::PathBuf,
}

impl EventEmitter {
    /// Create a new event emitter
    pub fn new(config: ObservabilityConfig, history_path: std::path::PathBuf) -> Self {
        Self { config, history_path }
    }

    /// Emit an event to all configured sinks
    pub fn emit(&self, hook_event: HookEvent, payload: &serde_json::Value) {
        if !self.config.enabled {
            return;
        }

        let event = Event::from_hook(hook_event, payload, self.config.include_payload);

        for sink in &self.config.sinks {
            match sink {
                ObservabilitySink::File => {
                    if let Err(e) = self.emit_to_file(&event) {
                        log::warn!("Failed to emit to file sink: {}", e);
                    }
                }
                ObservabilitySink::Stdout => {
                    self.emit_to_stdout(&event);
                }
                ObservabilitySink::Http => {
                    if let Err(e) = self.emit_to_http(&event) {
                        log::warn!("Failed to emit to HTTP sink: {}", e);
                    }
                }
            }
        }
    }

    /// Write event to JSONL file
    fn emit_to_file(&self, event: &Event) -> std::io::Result<()> {
        let now = Local::now();
        let month_dir = self
            .history_path
            .join("raw-events")
            .join(now.format("%Y-%m").to_string());
        fs::create_dir_all(&month_dir)?;

        let log_file = month_dir.join(format!("{}.jsonl", now.format("%Y-%m-%d")));

        let mut file = OpenOptions::new().create(true).append(true).open(log_file)?;

        let json = serde_json::to_string(event).unwrap_or_default();
        writeln!(file, "{}", json)?;

        Ok(())
    }

    /// Print event to stdout
    fn emit_to_stdout(&self, event: &Event) {
        println!("{}", event.format_display());
    }

    /// POST event to HTTP endpoint
    fn emit_to_http(&self, event: &Event) -> Result<(), String> {
        let endpoint = self
            .config
            .http_endpoint
            .as_ref()
            .ok_or_else(|| "HTTP endpoint not configured".to_string())?;

        // Use ureq for HTTP POST
        let body = serde_json::to_string(event).map_err(|e| e.to_string())?;

        match ureq::post(endpoint)
            .header("Content-Type", "application/json")
            .send(body.as_bytes())
        {
            Ok(_) => Ok(()),
            Err(e) => Err(format!("HTTP request failed: {}", e)),
        }
    }
}

/// Check if observability is configured for stdout
/// (useful for `pais observe` command to avoid duplicate output)
pub fn has_stdout_sink(config: &ObservabilityConfig) -> bool {
    config.enabled && config.sinks.contains(&ObservabilitySink::Stdout)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_event_from_hook() {
        let payload = serde_json::json!({
            "session_id": "test-123",
            "tool_name": "Bash"
        });

        let event = Event::from_hook(HookEvent::PreToolUse, &payload, false);

        assert_eq!(event.event_type, "PreToolUse");
        assert_eq!(event.session_id, Some("test-123".to_string()));
        assert_eq!(event.tool_name, Some("Bash".to_string()));
        assert!(event.payload.is_none());
    }

    #[test]
    fn test_event_with_payload() {
        let payload = serde_json::json!({"key": "value"});

        let event = Event::from_hook(HookEvent::SessionStart, &payload, true);

        assert!(event.payload.is_some());
    }

    #[test]
    fn test_event_format_display() {
        let event = Event {
            timestamp: "2026-01-03T12:00:00Z".to_string(),
            local_time: "2026-01-03 12:00:00".to_string(),
            event_type: "SessionStart".to_string(),
            session_id: Some("abc12345".to_string()),
            tool_name: None,
            payload: None,
        };

        let display = event.format_display();
        assert!(display.contains("2026-01-03 12:00:00"));
        assert!(display.contains("abc12345"));
    }

    #[test]
    fn test_emitter_disabled() {
        let config = ObservabilityConfig {
            enabled: false,
            sinks: vec![ObservabilitySink::Stdout],
            http_endpoint: None,
            include_payload: false,
        };

        let emitter = EventEmitter::new(config, std::path::PathBuf::from("/tmp"));
        // Should not panic or do anything
        emitter.emit(HookEvent::SessionStart, &serde_json::json!({}));
    }
}
