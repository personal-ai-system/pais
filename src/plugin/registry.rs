//! Plugin registry management
//!
//! Registries are sources of plugin metadata for discovery.

use serde::{Deserialize, Serialize};
use std::collections::HashMap;

/// A plugin registry entry
#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct RegistryEntry {
    pub source: String,
    pub version: Option<String>,
    pub description: Option<String>,

    #[serde(default)]
    pub provides: Vec<String>,

    #[serde(default)]
    pub keywords: Vec<String>,
}

/// A plugin registry
#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct Registry {
    #[serde(default)]
    pub plugins: HashMap<String, RegistryEntry>,
}

impl Registry {
    /// Load a registry from a TOML file or URL
    pub fn load(source: &str) -> eyre::Result<Self> {
        // TODO: Handle URL sources (git repos, HTTP)
        let content = std::fs::read_to_string(source)?;
        let registry: Self = serde_yaml::from_str(&content)?;
        Ok(registry)
    }

    /// Search for plugins matching a query
    pub fn search(&self, query: &str) -> Vec<(&str, &RegistryEntry)> {
        let query_lower = query.to_lowercase();
        self.plugins
            .iter()
            .filter(|(name, entry)| {
                name.to_lowercase().contains(&query_lower)
                    || entry
                        .description
                        .as_ref()
                        .map(|d| d.to_lowercase().contains(&query_lower))
                        .unwrap_or(false)
                    || entry.keywords.iter().any(|k| k.to_lowercase().contains(&query_lower))
            })
            .map(|(name, entry)| (name.as_str(), entry))
            .collect()
    }
}
