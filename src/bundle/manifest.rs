//! Bundle manifest parsing (bundle.yaml)

use eyre::{Context, Result};
use indexmap::IndexMap;
use serde::{Deserialize, Serialize};
use std::path::Path;

/// Bundle manifest structure
#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct BundleManifest {
    pub bundle: Bundle,

    /// Plugins in this bundle (order preserved for installation)
    #[serde(default)]
    pub plugins: IndexMap<String, PluginRef>,

    /// Environment variables to set
    #[serde(default)]
    pub environment: IndexMap<String, String>,

    /// Commands to run after all plugins are installed
    #[serde(default, rename = "post-install")]
    pub post_install: Vec<PostInstallCommand>,

    /// Bundles that conflict with this one
    #[serde(default)]
    pub conflicts: Vec<String>,
}

/// Bundle metadata
#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct Bundle {
    pub name: String,
    pub version: String,
    pub description: String,

    #[serde(default)]
    pub author: Option<String>,

    #[serde(default)]
    pub license: Option<String>,

    /// Minimum PAIS version required
    #[serde(default, rename = "pais-version")]
    pub pais_version: Option<String>,
}

/// Reference to a plugin in a bundle
#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct PluginRef {
    /// Whether this plugin is required or optional
    #[serde(default = "default_required")]
    pub required: bool,

    /// Description of why this plugin is included
    #[serde(default)]
    pub description: Option<String>,

    /// Source for remote plugins (future)
    #[serde(default)]
    pub source: Option<String>,

    /// Path within source (for remote plugins)
    #[serde(default)]
    pub path: Option<String>,
}

fn default_required() -> bool {
    true
}

/// Command to run after installation
#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct PostInstallCommand {
    pub command: String,

    #[serde(default)]
    pub description: Option<String>,
}

impl BundleManifest {
    /// Load a manifest from a YAML file
    pub fn load<P: AsRef<Path>>(path: P) -> Result<Self> {
        let content = std::fs::read_to_string(&path)
            .with_context(|| format!("Failed to read bundle manifest: {}", path.as_ref().display()))?;
        Self::from_str(&content)
    }

    /// Parse a manifest from YAML string
    pub fn from_str(content: &str) -> Result<Self> {
        let manifest: Self = serde_yaml::from_str(content).context("Failed to parse bundle manifest")?;
        Ok(manifest)
    }

    /// Get required plugins in order
    pub fn required_plugins(&self) -> Vec<(&String, &PluginRef)> {
        self.plugins.iter().filter(|(_, p)| p.required).collect()
    }

    /// Get optional plugins in order
    pub fn optional_plugins(&self) -> Vec<(&String, &PluginRef)> {
        self.plugins.iter().filter(|(_, p)| !p.required).collect()
    }

    /// Get all plugins in order
    pub fn all_plugins(&self) -> Vec<(&String, &PluginRef)> {
        self.plugins.iter().collect()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    const MINIMAL_BUNDLE: &str = r#"
bundle:
  name: test-bundle
  version: 1.0.0
  description: A test bundle

plugins:
  plugin-a:
    required: true
  plugin-b:
    required: false
"#;

    const FULL_BUNDLE: &str = r#"
bundle:
  name: full-bundle
  version: 1.0.0
  description: A fully configured bundle
  author: Test Author
  license: MIT
  pais-version: ">=0.2.0"

plugins:
  plugin-a:
    required: true
    description: First plugin

  plugin-b:
    required: true

  optional-plugin:
    required: false
    description: Nice to have

environment:
  SOME_VAR: value
  OTHER_VAR: other

post-install:
  - command: pais skill index --rebuild
    description: Rebuild skill index

conflicts:
  - other-bundle
"#;

    #[test]
    fn test_parse_minimal_bundle() {
        let manifest = BundleManifest::from_str(MINIMAL_BUNDLE).unwrap();
        assert_eq!(manifest.bundle.name, "test-bundle");
        assert_eq!(manifest.bundle.version, "1.0.0");
        assert_eq!(manifest.plugins.len(), 2);
        assert!(manifest.plugins["plugin-a"].required);
        assert!(!manifest.plugins["plugin-b"].required);
    }

    #[test]
    fn test_parse_full_bundle() {
        let manifest = BundleManifest::from_str(FULL_BUNDLE).unwrap();
        assert_eq!(manifest.bundle.name, "full-bundle");
        assert_eq!(manifest.bundle.author, Some("Test Author".to_string()));
        assert_eq!(manifest.bundle.pais_version, Some(">=0.2.0".to_string()));
        assert_eq!(manifest.plugins.len(), 3);
        assert_eq!(manifest.environment.len(), 2);
        assert_eq!(manifest.post_install.len(), 1);
        assert_eq!(manifest.conflicts.len(), 1);
    }

    #[test]
    fn test_required_plugins() {
        let manifest = BundleManifest::from_str(FULL_BUNDLE).unwrap();
        let required = manifest.required_plugins();
        assert_eq!(required.len(), 2);
        assert_eq!(required[0].0, "plugin-a");
        assert_eq!(required[1].0, "plugin-b");
    }

    #[test]
    fn test_optional_plugins() {
        let manifest = BundleManifest::from_str(FULL_BUNDLE).unwrap();
        let optional = manifest.optional_plugins();
        assert_eq!(optional.len(), 1);
        assert_eq!(optional[0].0, "optional-plugin");
    }

    #[test]
    fn test_plugin_order_preserved() {
        let manifest = BundleManifest::from_str(FULL_BUNDLE).unwrap();
        let all: Vec<_> = manifest.all_plugins();
        assert_eq!(all[0].0, "plugin-a");
        assert_eq!(all[1].0, "plugin-b");
        assert_eq!(all[2].0, "optional-plugin");
    }
}
