//! Plugin manifest parsing (plugin.toml)

use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::path::Path;

/// Plugin manifest structure
#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct PluginManifest {
    pub plugin: PluginInfo,

    #[serde(default)]
    pub paii: PaiiRequirements,

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
pub struct PaiiRequirements {
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

    pub default: Option<toml::Value>,

    pub env: Option<String>,

    #[serde(default)]
    pub secret: bool,
}

#[derive(Debug, Clone, Default, Deserialize, Serialize)]
pub struct HooksSpec {
    #[serde(default)]
    pub pre_tool_use: bool,

    #[serde(default)]
    pub post_tool_use: bool,

    #[serde(default)]
    pub stop: bool,

    #[serde(default)]
    pub session_start: bool,

    #[serde(default)]
    pub session_end: bool,

    #[serde(default)]
    pub subagent_stop: bool,
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
    Pip,
    Cargo,
    Custom,
}

impl PluginManifest {
    /// Load a manifest from a TOML file
    pub fn load<P: AsRef<Path>>(path: P) -> eyre::Result<Self> {
        let content = std::fs::read_to_string(&path)?;
        let manifest: Self = toml::from_str(&content)?;
        Ok(manifest)
    }
}
