#![allow(dead_code)]

use eyre::{Context, Result};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::fs;
use std::path::{Path, PathBuf};

/// Main PAII configuration
#[derive(Debug, Deserialize, Serialize)]
#[serde(default)]
pub struct Config {
    pub paii: PaiiConfig,
    pub paths: PathsConfig,
    pub defaults: DefaultsConfig,
    pub registries: HashMap<String, String>,
    pub hooks: HooksConfig,
}

#[derive(Debug, Deserialize, Serialize)]
#[serde(default)]
pub struct PaiiConfig {
    pub version: String,
}

#[derive(Debug, Deserialize, Serialize)]
#[serde(default)]
pub struct PathsConfig {
    pub plugins: PathBuf,
    pub history: PathBuf,
    pub registries: PathBuf,
}

#[derive(Debug, Deserialize, Serialize)]
#[serde(default)]
pub struct DefaultsConfig {
    pub language: String,
    pub log_level: String,
}

#[derive(Debug, Deserialize, Serialize)]
#[serde(default)]
pub struct HooksConfig {
    pub security_enabled: bool,
    pub history_enabled: bool,
}

impl Default for Config {
    fn default() -> Self {
        let paii_dir = dirs::config_dir().unwrap_or_else(|| PathBuf::from(".")).join("paii");

        Self {
            paii: PaiiConfig::default(),
            paths: PathsConfig {
                plugins: paii_dir.join("plugins"),
                history: paii_dir.join("history"),
                registries: paii_dir.join("registries"),
            },
            defaults: DefaultsConfig::default(),
            registries: HashMap::from([(
                "core".to_string(),
                "https://github.com/scottidler/paii/registry/plugins.toml".to_string(),
            )]),
            hooks: HooksConfig::default(),
        }
    }
}

impl Default for PaiiConfig {
    fn default() -> Self {
        Self {
            version: env!("CARGO_PKG_VERSION").to_string(),
        }
    }
}

impl Default for PathsConfig {
    fn default() -> Self {
        let paii_dir = dirs::config_dir().unwrap_or_else(|| PathBuf::from(".")).join("paii");

        Self {
            plugins: paii_dir.join("plugins"),
            history: paii_dir.join("history"),
            registries: paii_dir.join("registries"),
        }
    }
}

impl Default for DefaultsConfig {
    fn default() -> Self {
        Self {
            language: "python".to_string(),
            log_level: "info".to_string(),
        }
    }
}

impl Default for HooksConfig {
    fn default() -> Self {
        Self {
            security_enabled: true,
            history_enabled: true,
        }
    }
}

impl Config {
    /// Load configuration with fallback chain
    pub fn load(config_path: Option<&PathBuf>) -> Result<Self> {
        // If explicit config path provided, try to load it
        if let Some(path) = config_path {
            return Self::load_from_file(path).context(format!("Failed to load config from {}", path.display()));
        }

        // Check PAII_CONFIG env var
        if let Ok(env_path) = std::env::var("PAII_CONFIG") {
            let path = PathBuf::from(env_path);
            if path.exists() {
                match Self::load_from_file(&path) {
                    Ok(config) => return Ok(config),
                    Err(e) => {
                        log::warn!("Failed to load config from PAII_CONFIG: {}", e);
                    }
                }
            }
        }

        // Try PAII_DIR/paii.toml
        if let Ok(paii_dir) = std::env::var("PAII_DIR") {
            let path = PathBuf::from(paii_dir).join("paii.toml");
            if path.exists() {
                match Self::load_from_file(&path) {
                    Ok(config) => return Ok(config),
                    Err(e) => {
                        log::warn!("Failed to load config from PAII_DIR: {}", e);
                    }
                }
            }
        }

        // Try ~/.config/paii/paii.toml
        if let Some(config_dir) = dirs::config_dir() {
            let path = config_dir.join("paii").join("paii.toml");
            if path.exists() {
                match Self::load_from_file(&path) {
                    Ok(config) => return Ok(config),
                    Err(e) => {
                        log::warn!("Failed to load config from {}: {}", path.display(), e);
                    }
                }
            }
        }

        // Try ./paii.toml (for development)
        let local_config = PathBuf::from("paii.toml");
        if local_config.exists() {
            match Self::load_from_file(&local_config) {
                Ok(config) => return Ok(config),
                Err(e) => {
                    log::warn!("Failed to load local config: {}", e);
                }
            }
        }

        // No config file found, use defaults
        log::info!("No config file found, using defaults");
        Ok(Self::default())
    }

    fn load_from_file<P: AsRef<Path>>(path: P) -> Result<Self> {
        let content = fs::read_to_string(&path).context("Failed to read config file")?;

        let config: Self = toml::from_str(&content).context("Failed to parse config file")?;

        log::info!("Loaded config from: {}", path.as_ref().display());
        Ok(config)
    }

    /// Get the PAII directory (where plugins, history, etc. live)
    pub fn paii_dir() -> PathBuf {
        std::env::var("PAII_DIR")
            .map(PathBuf::from)
            .unwrap_or_else(|_| dirs::config_dir().unwrap_or_else(|| PathBuf::from(".")).join("paii"))
    }

    /// Expand a path that may contain ~ or env vars
    pub fn expand_path(path: &Path) -> PathBuf {
        let path_str = path.to_string_lossy();
        let expanded = shellexpand::full(&path_str).unwrap_or_else(|_| path_str.clone());
        PathBuf::from(expanded.as_ref())
    }
}
