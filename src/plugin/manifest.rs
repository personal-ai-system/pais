//! Plugin manifest parsing (plugin.yaml)

#![allow(dead_code)] // Methods reserved for future use

use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::path::Path;

/// Plugin manifest structure
#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct PluginManifest {
    pub plugin: PluginInfo,

    #[serde(default)]
    pub pais: PaisRequirements,

    #[serde(default)]
    pub provides: HashMap<String, ProvideSpec>,

    #[serde(default)]
    pub consumes: HashMap<String, ConsumeSpec>,

    #[serde(default)]
    pub config: HashMap<String, ConfigSpec>,

    #[serde(default)]
    pub hooks: HooksSpec,

    #[serde(default)]
    pub build: BuildSpec,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct PluginInfo {
    pub name: String,
    pub version: String,
    pub description: String,

    #[serde(default)]
    pub authors: Vec<String>,

    #[serde(default)]
    pub language: PluginLanguage,

    pub license: Option<String>,
    pub repository: Option<String>,
    pub homepage: Option<String>,

    #[serde(default)]
    pub keywords: Vec<String>,
}

#[derive(Debug, Clone, Default, Deserialize, Serialize)]
#[serde(rename_all = "lowercase")]
pub enum PluginLanguage {
    #[default]
    Python,
    Rust,
    Mixed,
}

#[derive(Debug, Clone, Default, Deserialize, Serialize)]
pub struct PaisRequirements {
    #[serde(default)]
    pub core_version: Option<String>,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
#[serde(untagged)]
pub enum ProvideSpec {
    Simple(String),
    Detailed {
        contract: String,
        #[serde(default)]
        service: Option<String>,
    },
}

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct ConsumeSpec {
    pub contract: String,

    #[serde(default)]
    pub optional: bool,

    #[serde(default)]
    pub service: Option<String>,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct ConfigSpec {
    pub r#type: String,

    #[serde(default)]
    pub required: bool,

    pub default: Option<serde_yaml::Value>,

    pub env: Option<String>,

    #[serde(default)]
    pub secret: bool,
}

/// Hook configuration - maps event types to scripts
#[derive(Debug, Clone, Default, Deserialize, Serialize)]
pub struct HooksSpec {
    /// Scripts to run on PreToolUse
    #[serde(default, rename = "PreToolUse")]
    pub pre_tool_use: Vec<HookScript>,

    /// Scripts to run on PostToolUse
    #[serde(default, rename = "PostToolUse")]
    pub post_tool_use: Vec<HookScript>,

    /// Scripts to run on Stop
    #[serde(default, rename = "Stop")]
    pub stop: Vec<HookScript>,

    /// Scripts to run on SessionStart
    #[serde(default, rename = "SessionStart")]
    pub session_start: Vec<HookScript>,

    /// Scripts to run on SessionEnd
    #[serde(default, rename = "SessionEnd")]
    pub session_end: Vec<HookScript>,

    /// Scripts to run on SubagentStop
    #[serde(default, rename = "SubagentStop")]
    pub subagent_stop: Vec<HookScript>,
}

/// A hook script definition
#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct HookScript {
    /// Path to the script (relative to plugin directory)
    pub script: String,

    /// Optional matcher (e.g., "Bash" to only run on Bash tool)
    #[serde(default)]
    pub matcher: Option<String>,

    /// Optional timeout in seconds (default: 30)
    #[serde(default = "default_timeout")]
    pub timeout: u64,
}

fn default_timeout() -> u64 {
    30
}

impl HooksSpec {
    /// Check if this plugin has any hooks
    pub fn has_hooks(&self) -> bool {
        !self.pre_tool_use.is_empty()
            || !self.post_tool_use.is_empty()
            || !self.stop.is_empty()
            || !self.session_start.is_empty()
            || !self.session_end.is_empty()
            || !self.subagent_stop.is_empty()
    }

    /// Get scripts for a given event type
    pub fn scripts_for_event(&self, event: &str) -> &[HookScript] {
        match event {
            "PreToolUse" => &self.pre_tool_use,
            "PostToolUse" => &self.post_tool_use,
            "Stop" => &self.stop,
            "SessionStart" => &self.session_start,
            "SessionEnd" => &self.session_end,
            "SubagentStop" => &self.subagent_stop,
            _ => &[],
        }
    }
}

#[derive(Debug, Clone, Default, Deserialize, Serialize)]
pub struct BuildSpec {
    #[serde(default)]
    pub r#type: BuildType,

    pub requirements: Option<String>,
    pub install_command: Option<String>,
    pub build_command: Option<String>,
}

#[derive(Debug, Clone, Default, Deserialize, Serialize)]
#[serde(rename_all = "lowercase")]
pub enum BuildType {
    #[default]
    Uv,
    Cargo,
    Custom,
}

impl PluginManifest {
    /// Load a manifest from a YAML file
    pub fn load<P: AsRef<Path>>(path: P) -> eyre::Result<Self> {
        let content = std::fs::read_to_string(&path)?;
        let manifest: Self = serde_yaml::from_str(&content)?;
        Ok(manifest)
    }

    /// Parse a manifest from YAML string
    pub fn from_str(content: &str) -> eyre::Result<Self> {
        let manifest: Self = serde_yaml::from_str(content)?;
        Ok(manifest)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    const MINIMAL_MANIFEST: &str = r#"
plugin:
  name: test-plugin
  version: 0.1.0
  description: A test plugin
"#;

    const FULL_MANIFEST: &str = r#"
plugin:
  name: full-plugin
  version: 1.2.3
  description: A fully configured plugin
  authors:
    - Test Author <test@example.com>
  language: rust
  license: MIT
  repository: https://github.com/example/plugin
  keywords:
    - test
    - example

pais:
  core_version: ">=0.1.0"

provides:
  memory:
    contract: MemoryProvider
    service: sqlite-memory

consumes:
  config:
    contract: ConfigProvider
    optional: true

hooks:
  PreToolUse:
    - script: hooks/validate.py
      matcher: Bash
  Stop:
    - script: hooks/capture.py

build:
  type: cargo
"#;

    #[test]
    fn test_parse_minimal_manifest() {
        let manifest = PluginManifest::from_str(MINIMAL_MANIFEST).unwrap();
        assert_eq!(manifest.plugin.name, "test-plugin");
        assert_eq!(manifest.plugin.version, "0.1.0");
        assert_eq!(manifest.plugin.description, "A test plugin");
        assert!(manifest.plugin.authors.is_empty());
    }

    #[test]
    fn test_parse_full_manifest() {
        let manifest = PluginManifest::from_str(FULL_MANIFEST).unwrap();
        assert_eq!(manifest.plugin.name, "full-plugin");
        assert_eq!(manifest.plugin.version, "1.2.3");
        assert_eq!(manifest.plugin.authors.len(), 1);
        assert!(matches!(manifest.plugin.language, PluginLanguage::Rust));
        assert_eq!(manifest.plugin.license, Some("MIT".to_string()));
        assert!(manifest.provides.contains_key("memory"));
        assert!(manifest.consumes.contains_key("config"));
        assert_eq!(manifest.hooks.pre_tool_use.len(), 1);
        assert_eq!(manifest.hooks.pre_tool_use[0].script, "hooks/validate.py");
        assert_eq!(manifest.hooks.pre_tool_use[0].matcher, Some("Bash".to_string()));
        assert_eq!(manifest.hooks.stop.len(), 1);
        assert!(manifest.hooks.post_tool_use.is_empty());
        assert!(matches!(manifest.build.r#type, BuildType::Cargo));
    }

    #[test]
    fn test_default_plugin_language() {
        let lang = PluginLanguage::default();
        assert!(matches!(lang, PluginLanguage::Python));
    }

    #[test]
    fn test_default_build_type() {
        let build = BuildType::default();
        assert!(matches!(build, BuildType::Uv));
    }

    #[test]
    fn test_default_hooks_spec() {
        let hooks = HooksSpec::default();
        assert!(hooks.pre_tool_use.is_empty());
        assert!(hooks.post_tool_use.is_empty());
        assert!(hooks.stop.is_empty());
        assert!(hooks.session_start.is_empty());
        assert!(hooks.session_end.is_empty());
        assert!(!hooks.has_hooks());
    }

    #[test]
    fn test_consume_spec_optional() {
        let yaml_str = r#"
plugin:
  name: test
  version: 0.1.0
  description: test

consumes:
  optional_dep:
    contract: SomeContract
    optional: true
  required_dep:
    contract: OtherContract
"#;
        let manifest = PluginManifest::from_str(yaml_str).unwrap();
        assert!(manifest.consumes["optional_dep"].optional);
        assert!(!manifest.consumes["required_dep"].optional);
    }

    #[test]
    fn test_manifest_serialization_roundtrip() {
        let manifest = PluginManifest::from_str(FULL_MANIFEST).unwrap();
        let yaml_str = serde_yaml::to_string(&manifest).unwrap();
        let reparsed = PluginManifest::from_str(&yaml_str).unwrap();
        assert_eq!(reparsed.plugin.name, manifest.plugin.name);
        assert_eq!(reparsed.plugin.version, manifest.plugin.version);
    }
}
