//! Plugin discovery, loading, and management
//!
//! This module handles:
//! - Discovering plugins from the plugins directory
//! - Parsing plugin manifests (plugin.yaml)
//! - Loading and initializing plugins
//! - Executing plugin hooks
//! - Managing plugin lifecycle

#![allow(dead_code)] // Plugin lifecycle states and methods - for full plugin management

use eyre::{Context, Result};
use std::collections::HashMap;
use std::fs;
use std::path::PathBuf;

pub mod executor;
pub mod loader;
pub mod manifest;
pub mod registry;

use crate::hook::{HookEvent, HookResult};
use manifest::PluginManifest;

/// A loaded plugin
#[derive(Debug)]
pub struct Plugin {
    pub manifest: PluginManifest,
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
    pub fn discover(&mut self) -> Result<usize> {
        self.plugins.clear();

        if !self.plugins_dir.exists() {
            return Ok(0);
        }

        let mut count = 0;

        for entry in fs::read_dir(&self.plugins_dir).context("Failed to read plugins directory")? {
            let entry = entry?;
            let path = entry.path();

            if !path.is_dir() {
                continue;
            }

            let manifest_path = path.join("plugin.yaml");
            if !manifest_path.exists() {
                continue;
            }

            match PluginManifest::load(&manifest_path) {
                Ok(manifest) => {
                    let name = manifest.plugin.name.clone();
                    self.plugins.insert(
                        name,
                        Plugin {
                            manifest,
                            path,
                            state: PluginState::Discovered,
                        },
                    );
                    count += 1;
                }
                Err(e) => {
                    log::warn!("Failed to load plugin manifest {}: {}", manifest_path.display(), e);
                }
            }
        }

        Ok(count)
    }

    /// Load all discovered plugins
    pub fn load_all(&mut self) -> Result<()> {
        for plugin in self.plugins.values_mut() {
            if plugin.state == PluginState::Discovered {
                plugin.state = PluginState::Loaded;
            }
        }
        Ok(())
    }

    /// Get a plugin by name
    pub fn get(&self, name: &str) -> Option<&Plugin> {
        self.plugins.get(name)
    }

    /// Get all plugins with hooks for a given event
    pub fn plugins_for_event(&self, event: HookEvent) -> Vec<&Plugin> {
        self.plugins
            .values()
            .filter(|p| !p.manifest.hooks.scripts_for_event(&event.to_string()).is_empty())
            .collect()
    }

    /// Execute all plugin hooks for an event
    pub fn execute_hooks(&self, event: HookEvent, payload: &serde_json::Value) -> Vec<HookResult> {
        let mut results = Vec::new();

        for plugin in self.plugins_for_event(event) {
            let hook_results = executor::execute_plugin_hooks(&plugin.path, &plugin.manifest, event, payload);

            for result in hook_results {
                let hook_result = result.to_hook_result();

                // Log non-trivial results
                match &hook_result {
                    HookResult::Block { message } => {
                        log::warn!("Plugin '{}' blocked: {}", result.plugin_name, message);
                    }
                    HookResult::Error { message } => {
                        log::error!("Plugin '{}' error: {}", result.plugin_name, message);
                    }
                    HookResult::Allow => {}
                }

                // Print any stdout from the plugin
                if !result.stdout.is_empty() {
                    print!("{}", result.stdout);
                }

                results.push(hook_result);
            }
        }

        results
    }

    /// List all plugins
    pub fn list(&self) -> impl Iterator<Item = &Plugin> {
        self.plugins.values()
    }

    /// Check if a plugin exists
    pub fn has(&self, name: &str) -> bool {
        self.plugins.contains_key(name)
    }

    /// Remove a plugin
    pub fn remove(&mut self, name: &str) -> Result<()> {
        if let Some(plugin) = self.plugins.remove(name) {
            fs::remove_dir_all(&plugin.path)
                .with_context(|| format!("Failed to remove plugin directory: {}", plugin.path.display()))?;
            Ok(())
        } else {
            eyre::bail!("Plugin '{}' not found", name)
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::tempdir;

    fn create_test_plugin(dir: &std::path::Path, name: &str) {
        let plugin_dir = dir.join(name);
        fs::create_dir_all(&plugin_dir).unwrap();

        let manifest = format!(
            r#"
plugin:
  name: {}
  version: 0.1.0
  description: Test plugin
"#,
            name
        );
        fs::write(plugin_dir.join("plugin.yaml"), manifest).unwrap();
    }

    #[test]
    fn test_discover_plugins() {
        let temp = tempdir().unwrap();
        create_test_plugin(temp.path(), "plugin-a");
        create_test_plugin(temp.path(), "plugin-b");

        let mut manager = PluginManager::new(temp.path().to_path_buf());
        let count = manager.discover().unwrap();

        assert_eq!(count, 2);
        assert!(manager.has("plugin-a"));
        assert!(manager.has("plugin-b"));
    }

    #[test]
    fn test_discover_empty_directory() {
        let temp = tempdir().unwrap();

        let mut manager = PluginManager::new(temp.path().to_path_buf());
        let count = manager.discover().unwrap();

        assert_eq!(count, 0);
    }

    #[test]
    fn test_remove_plugin() {
        let temp = tempdir().unwrap();
        create_test_plugin(temp.path(), "test-plugin");

        let mut manager = PluginManager::new(temp.path().to_path_buf());
        manager.discover().unwrap();

        assert!(manager.has("test-plugin"));
        manager.remove("test-plugin").unwrap();
        assert!(!manager.has("test-plugin"));
        assert!(!temp.path().join("test-plugin").exists());
    }
}
