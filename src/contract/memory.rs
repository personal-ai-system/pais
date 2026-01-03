//! MemoryProvider contract
//!
//! Plugins that provide persistent memory/context storage.

#![allow(dead_code)] // Contract trait - pending plugin implementation

use serde::{Deserialize, Serialize};
use std::collections::HashMap;

/// Result from a memory query
#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct MemoryResult {
    pub path: String,
    pub category: String,
    pub timestamp: String,
    pub content: String,
    pub metadata: HashMap<String, serde_json::Value>,
}

/// MemoryProvider contract interface
pub trait MemoryProvider: Send + Sync {
    /// Store content in the specified category
    fn capture(
        &self,
        category: &str,
        content: &str,
        metadata: HashMap<String, serde_json::Value>,
    ) -> eyre::Result<String>;

    /// Search stored content
    fn query(&self, category: &str, query: &str, limit: usize) -> eyre::Result<Vec<MemoryResult>>;

    /// List available categories
    fn list_categories(&self) -> Vec<String>;

    /// Get most recent entries in a category
    fn get_recent(&self, category: &str, count: usize) -> eyre::Result<Vec<MemoryResult>>;
}
