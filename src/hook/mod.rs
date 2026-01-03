//! Hook event handling
//!
//! Hooks are events fired by Claude Code that PAIS can intercept.
//! This module handles dispatching those events to plugin handlers.

#![allow(dead_code)]

use serde::{Deserialize, Serialize};

pub mod dispatch;
pub mod history;
pub mod security;
pub mod ui;

/// Hook event types
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Deserialize, Serialize)]
#[serde(rename_all = "PascalCase")]
pub enum HookEvent {
    PreToolUse,
    PostToolUse,
    Stop,
    SessionStart,
    SessionEnd,
    SubagentStop,
    Notification,
    PermissionRequest,
    UserPromptSubmit,
    PreCompact,
}

impl HookEvent {
    pub fn from_str(s: &str) -> Option<Self> {
        match s.to_lowercase().replace(['-', '_'], "").as_str() {
            "pretooluse" => Some(Self::PreToolUse),
            "posttooluse" => Some(Self::PostToolUse),
            "stop" => Some(Self::Stop),
            "sessionstart" => Some(Self::SessionStart),
            "sessionend" => Some(Self::SessionEnd),
            "subagentstop" => Some(Self::SubagentStop),
            "notification" => Some(Self::Notification),
            "permissionrequest" => Some(Self::PermissionRequest),
            "userpromptsubmit" => Some(Self::UserPromptSubmit),
            "precompact" => Some(Self::PreCompact),
            _ => None,
        }
    }
}

/// Result of a hook handler
#[derive(Debug, Clone)]
pub enum HookResult {
    /// Allow the action to proceed
    Allow,
    /// Block the action (exit code 2)
    Block { message: String },
    /// Error occurred (logged but allows action)
    Error { message: String },
}

impl HookResult {
    pub fn exit_code(&self) -> i32 {
        match self {
            HookResult::Allow => 0,
            HookResult::Block { .. } => 2,
            HookResult::Error { .. } => 0, // Errors don't block
        }
    }
}

/// A hook handler
pub trait HookHandler: Send + Sync {
    fn handles(&self, event: HookEvent) -> bool;
    fn handle(&self, event: HookEvent, payload: &serde_json::Value) -> HookResult;
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_hook_event_from_str_pascal_case() {
        assert_eq!(HookEvent::from_str("PreToolUse"), Some(HookEvent::PreToolUse));
        assert_eq!(HookEvent::from_str("PostToolUse"), Some(HookEvent::PostToolUse));
        assert_eq!(HookEvent::from_str("Stop"), Some(HookEvent::Stop));
        assert_eq!(HookEvent::from_str("SessionStart"), Some(HookEvent::SessionStart));
        assert_eq!(HookEvent::from_str("SessionEnd"), Some(HookEvent::SessionEnd));
    }

    #[test]
    fn test_hook_event_from_str_lowercase() {
        assert_eq!(HookEvent::from_str("pretooluse"), Some(HookEvent::PreToolUse));
        assert_eq!(HookEvent::from_str("posttooluse"), Some(HookEvent::PostToolUse));
        assert_eq!(HookEvent::from_str("stop"), Some(HookEvent::Stop));
    }

    #[test]
    fn test_hook_event_from_str_with_separators() {
        assert_eq!(HookEvent::from_str("pre-tool-use"), Some(HookEvent::PreToolUse));
        assert_eq!(HookEvent::from_str("pre_tool_use"), Some(HookEvent::PreToolUse));
        assert_eq!(HookEvent::from_str("session-start"), Some(HookEvent::SessionStart));
        assert_eq!(HookEvent::from_str("session_end"), Some(HookEvent::SessionEnd));
    }

    #[test]
    fn test_hook_event_from_str_unknown() {
        assert_eq!(HookEvent::from_str("unknown"), None);
        assert_eq!(HookEvent::from_str(""), None);
        assert_eq!(HookEvent::from_str("not-a-hook"), None);
    }

    #[test]
    fn test_hook_event_all_variants() {
        // Ensure all variants are parseable
        assert!(HookEvent::from_str("SubagentStop").is_some());
        assert!(HookEvent::from_str("Notification").is_some());
        assert!(HookEvent::from_str("PermissionRequest").is_some());
        assert!(HookEvent::from_str("UserPromptSubmit").is_some());
        assert!(HookEvent::from_str("PreCompact").is_some());
    }

    #[test]
    fn test_hook_result_exit_codes() {
        assert_eq!(HookResult::Allow.exit_code(), 0);
        assert_eq!(
            HookResult::Block {
                message: "blocked".to_string()
            }
            .exit_code(),
            2
        );
        assert_eq!(
            HookResult::Error {
                message: "error".to_string()
            }
            .exit_code(),
            0
        );
    }

    #[test]
    fn test_hook_event_serialization() {
        let event = HookEvent::PreToolUse;
        let json = serde_json::to_string(&event).unwrap();
        assert_eq!(json, "\"PreToolUse\"");

        let parsed: HookEvent = serde_json::from_str(&json).unwrap();
        assert_eq!(parsed, event);
    }
}
