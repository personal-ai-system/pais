//! Plugin loading and initialization

use std::path::Path;

use super::manifest::PluginManifest;
use super::{Plugin, PluginState};

/// Load a plugin from a directory
pub fn load_plugin<P: AsRef<Path>>(path: P) -> eyre::Result<Plugin> {
    let path = path.as_ref();
    let manifest_path = path.join("plugin.yaml");

    if !manifest_path.exists() {
        eyre::bail!("No plugin.yaml found in {}", path.display());
    }

    let manifest = PluginManifest::load(&manifest_path)?;

    Ok(Plugin {
        manifest,
        path: path.to_path_buf(),
        state: PluginState::Discovered,
    })
}

/// Initialize a plugin (install deps, build if needed)
pub fn initialize_plugin(_plugin: &mut Plugin) -> eyre::Result<()> {
    // TODO: Based on plugin language:
    // - Python: create venv, install requirements
    // - Rust: cargo build --release
    Ok(())
}
