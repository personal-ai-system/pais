//! Event capture for raw event logging
//!
//! Appends all hook events to daily JSONL files for later analysis.
//! Format: `history/raw-events/YYYY-MM/YYYY-MM-DD.jsonl`

#![allow(dead_code)] // Capture methods - used by observability, some pending CLI commands

use chrono::{DateTime, Local, Utc};
use eyre::{Context, Result};
use serde::{Deserialize, Serialize};
use std::fs::{self, OpenOptions};
use std::io::Write;
use std::path::{Path, PathBuf};

use crate::hook::HookEvent;

/// A captured event
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CapturedEvent {
    /// Timestamp of the event (UTC)
    pub timestamp: DateTime<Utc>,
    /// Local timestamp for display
    pub local_time: String,
    /// Event type
    pub event: String,
    /// Session ID (if available)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub session_id: Option<String>,
    /// Event payload
    pub payload: serde_json::Value,
    /// Duration in milliseconds (for end events)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub duration_ms: Option<u64>,
}

impl CapturedEvent {
    /// Create a new captured event
    pub fn new(event: HookEvent, payload: serde_json::Value) -> Self {
        let now = Utc::now();
        let local = Local::now();

        // Extract session_id from payload if present
        let session_id = payload
            .get("session_id")
            .or_else(|| payload.get("sessionId"))
            .and_then(|v| v.as_str())
            .map(|s| s.to_string());

        Self {
            timestamp: now,
            local_time: local.format("%Y-%m-%d %H:%M:%S").to_string(),
            event: format!("{:?}", event),
            session_id,
            payload,
            duration_ms: None,
        }
    }

    /// Add duration information
    pub fn with_duration(mut self, duration_ms: u64) -> Self {
        self.duration_ms = Some(duration_ms);
        self
    }
}

/// Event capture store
pub struct EventCapture {
    base_path: PathBuf,
    enabled: bool,
}

impl EventCapture {
    /// Create a new event capture store
    pub fn new(base_path: PathBuf, enabled: bool) -> Self {
        Self { base_path, enabled }
    }

    /// Get the path to today's event log
    fn today_log_path(&self) -> PathBuf {
        let now = Local::now();
        let month_dir = self.base_path.join("raw-events").join(now.format("%Y-%m").to_string());
        month_dir.join(format!("{}.jsonl", now.format("%Y-%m-%d")))
    }

    /// Capture an event
    pub fn capture(&self, event: HookEvent, payload: &serde_json::Value) -> Result<()> {
        if !self.enabled {
            return Ok(());
        }

        let captured = CapturedEvent::new(event, payload.clone());
        self.append_event(&captured)
    }

    /// Append an event to the daily log
    fn append_event(&self, event: &CapturedEvent) -> Result<()> {
        let log_path = self.today_log_path();

        // Ensure directory exists
        if let Some(parent) = log_path.parent() {
            fs::create_dir_all(parent).context("Failed to create raw-events directory")?;
        }

        // Serialize to JSON line
        let json_line = serde_json::to_string(event).context("Failed to serialize event")?;

        // Append to file
        let mut file = OpenOptions::new()
            .create(true)
            .append(true)
            .open(&log_path)
            .with_context(|| format!("Failed to open event log: {}", log_path.display()))?;

        writeln!(file, "{}", json_line).context("Failed to write event")?;

        log::debug!("Captured event: {} -> {}", event.event, log_path.display());
        Ok(())
    }

    /// Read events from a specific date
    pub fn read_events(&self, date: &str) -> Result<Vec<CapturedEvent>> {
        // Parse date to get the month directory
        let parts: Vec<&str> = date.split('-').collect();
        if parts.len() != 3 {
            eyre::bail!("Invalid date format. Expected YYYY-MM-DD");
        }
        let month = format!("{}-{}", parts[0], parts[1]);

        let log_path = self
            .base_path
            .join("raw-events")
            .join(&month)
            .join(format!("{}.jsonl", date));

        if !log_path.exists() {
            return Ok(Vec::new());
        }

        let content = fs::read_to_string(&log_path).context("Failed to read event log")?;

        let mut events = Vec::new();
        for line in content.lines() {
            if line.trim().is_empty() {
                continue;
            }
            match serde_json::from_str(line) {
                Ok(event) => events.push(event),
                Err(e) => log::warn!("Failed to parse event line: {}", e),
            }
        }

        Ok(events)
    }

    /// Get event counts by type for a date range
    pub fn stats(&self, days: usize) -> Result<EventStats> {
        let mut stats = EventStats::default();

        let today = Local::now().date_naive();
        for i in 0..days {
            let date = today - chrono::Duration::days(i as i64);
            let date_str = date.format("%Y-%m-%d").to_string();

            if let Ok(events) = self.read_events(&date_str) {
                for event in events {
                    stats.total += 1;
                    *stats.by_type.entry(event.event).or_insert(0) += 1;
                }
            }
        }

        Ok(stats)
    }

    /// List available event log dates
    pub fn list_dates(&self) -> Result<Vec<String>> {
        let raw_events_dir = self.base_path.join("raw-events");
        if !raw_events_dir.exists() {
            return Ok(Vec::new());
        }

        let mut dates = Vec::new();

        for month_entry in fs::read_dir(&raw_events_dir)? {
            let month_entry = month_entry?;
            let month_path = month_entry.path();

            if !month_path.is_dir() {
                continue;
            }

            for file_entry in fs::read_dir(&month_path)? {
                let file_entry = file_entry?;
                let file_path = file_entry.path();

                if file_path.extension().map(|e| e == "jsonl").unwrap_or(false)
                    && let Some(stem) = file_path.file_stem().and_then(|s| s.to_str())
                {
                    dates.push(stem.to_string());
                }
            }
        }

        dates.sort();
        dates.reverse(); // Most recent first
        Ok(dates)
    }
}

/// Event statistics
#[derive(Debug, Default)]
pub struct EventStats {
    pub total: usize,
    pub by_type: std::collections::HashMap<String, usize>,
}

/// Initialize the history directory structure
pub fn init_history_dirs(history_path: &Path) -> Result<()> {
    let dirs = [
        "raw-events",
        "sessions",
        "learnings",
        "research",
        "decisions",
        "execution",
    ];

    for dir in dirs {
        let path = history_path.join(dir);
        fs::create_dir_all(&path).with_context(|| format!("Failed to create history directory: {}", path.display()))?;
    }

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::TempDir;

    #[test]
    fn test_captured_event_creation() {
        let payload = serde_json::json!({
            "session_id": "test-123",
            "tool": "bash"
        });

        let event = CapturedEvent::new(HookEvent::PreToolUse, payload);

        assert_eq!(event.event, "PreToolUse");
        assert_eq!(event.session_id, Some("test-123".to_string()));
        assert!(event.duration_ms.is_none());
    }

    #[test]
    fn test_event_capture_disabled() {
        let temp = TempDir::new().unwrap();
        let capture = EventCapture::new(temp.path().to_path_buf(), false);

        let payload = serde_json::json!({});
        let result = capture.capture(HookEvent::SessionStart, &payload);

        assert!(result.is_ok());
        // Should not create any files when disabled
        assert!(!temp.path().join("raw-events").exists());
    }

    #[test]
    fn test_event_capture_enabled() {
        let temp = TempDir::new().unwrap();
        let capture = EventCapture::new(temp.path().to_path_buf(), true);

        let payload = serde_json::json!({"test": true});
        capture.capture(HookEvent::SessionStart, &payload).unwrap();

        // Should create the raw-events directory
        assert!(temp.path().join("raw-events").exists());

        // Should be able to read back the event
        let today = Local::now().format("%Y-%m-%d").to_string();
        let events = capture.read_events(&today).unwrap();
        assert_eq!(events.len(), 1);
        assert_eq!(events[0].event, "SessionStart");
    }

    #[test]
    fn test_init_history_dirs() {
        let temp = TempDir::new().unwrap();
        init_history_dirs(temp.path()).unwrap();

        assert!(temp.path().join("raw-events").exists());
        assert!(temp.path().join("sessions").exists());
        assert!(temp.path().join("learnings").exists());
        assert!(temp.path().join("research").exists());
        assert!(temp.path().join("decisions").exists());
        assert!(temp.path().join("execution").exists());
    }
}
