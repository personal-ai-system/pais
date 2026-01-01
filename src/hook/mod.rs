//! Hook event handling
//!
//! Hooks are events fired by Claude Code that PAII can intercept.
//! This module handles dispatching those events to plugin handlers.

#![allow(dead_code)]

use serde::{Deserialize, Serialize};

pub mod dispatch;
pub mod security;

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
