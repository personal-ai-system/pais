//! IntegrationProvider contract
//!
//! Plugins that connect to external services (Jira, Slack, etc.).

#![allow(dead_code)] // Contract trait - pending plugin implementation

use std::collections::HashMap;

/// Result from an integration action
pub type ActionResult = HashMap<String, serde_json::Value>;

/// IntegrationProvider contract interface
pub trait IntegrationProvider: Send + Sync {
    /// Service name (e.g., "jira", "slack")
    fn service_name(&self) -> &str;

    /// Check if the integration is configured
    fn is_configured(&self) -> bool;

    /// Health check - can we reach the service?
    fn health_check(&self) -> bool;

    /// Execute an action on the service
    fn execute(&self, action: &str, params: HashMap<String, serde_json::Value>) -> eyre::Result<ActionResult>;

    /// List supported actions
    fn list_actions(&self) -> Vec<String>;
}
