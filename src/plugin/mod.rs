//! Plugin discovery, loading, and management
//!
//! This module handles:
//! - Discovering plugins from the plugins directory
//! - Parsing plugin manifests (plugin.toml)
//! - Loading and initializing plugins
//! - Managing plugin lifecycle

#![allow(dead_code)]

use std::collections::HashMap;
use std::path::PathBuf;

pub mod loader;
pub mod manifest;
pub mod registry;

/// A loaded plugin
#[derive(Debug)]
pub struct Plugin {
    pub manifest: manifest::PluginManifest,
    pub path: PathBuf,
    pub state: PluginState,
}

/// Plugin lifecycle state
#[derive(Debug, Clone, PartialEq)]
pub enum PluginState {
    Discovered,
    Loaded,
    Initialized,
    Failed(String),
}

/// Plugin manager responsible for all plugin operations
pub struct PluginManager {
    pub plugins: HashMap<String, Plugin>,
    pub plugins_dir: PathBuf,
}

impl PluginManager {
    pub fn new(plugins_dir: PathBuf) -> Self {
        Self {
            plugins: HashMap::new(),
            plugins_dir,
        }
    }

    /// Discover all plugins in the plugins directory
    pub fn discover(&mut self) -> eyre::Result<()> {
        // TODO: Scan plugins_dir for plugin.toml files
        Ok(())
    }

    /// Load all discovered plugins
    pub fn load_all(&mut self) -> eyre::Result<()> {
        // TODO: Parse manifests and resolve contracts
        Ok(())
    }

    /// Get a plugin by name
    pub fn get(&self, name: &str) -> Option<&Plugin> {
        self.plugins.get(name)
    }
}
