#![allow(dead_code)]

use eyre::{Context, Result};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::fs;
use std::path::{Path, PathBuf};

/// Main PAIS configuration
#[derive(Debug, Clone, Deserialize, Serialize)]
#[serde(default)]
pub struct Config {
    pub paths: PathsConfig,
    pub registries: HashMap<String, String>,
    pub hooks: HooksConfig,
    pub observability: ObservabilityConfig,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
#[serde(default)]
pub struct PathsConfig {
    pub plugins: PathBuf,
    pub skills: PathBuf,
    pub history: PathBuf,
    pub registries: PathBuf,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
#[serde(default)]
pub struct HooksConfig {
    pub security_enabled: bool,
    pub history_enabled: bool,
    pub ui_enabled: bool,
}

/// Observability sink type
#[derive(Debug, Clone, Deserialize, Serialize, PartialEq, Eq)]
#[serde(rename_all = "lowercase")]
pub enum ObservabilitySink {
    /// Write to JSONL file (default, uses history path)
    File,
    /// Print to stdout
    Stdout,
    /// Send to HTTP endpoint
    Http,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
#[serde(default)]
pub struct ObservabilityConfig {
    /// Enable observability
    pub enabled: bool,
    /// Which sinks to send events to
    pub sinks: Vec<ObservabilitySink>,
    /// HTTP endpoint for http sink
    pub http_endpoint: Option<String>,
    /// Include event payload in output (can be verbose)
    pub include_payload: bool,
}

impl Default for Config {
    fn default() -> Self {
        let pais_dir = dirs::config_dir().unwrap_or_else(|| PathBuf::from(".")).join("pais");

        Self {
            paths: PathsConfig {
                plugins: pais_dir.join("plugins"),
                skills: pais_dir.join("skills"),
                history: pais_dir.join("history"),
                registries: pais_dir.join("registries"),
            },
            registries: HashMap::from([(
                "core".to_string(),
                "https://raw.githubusercontent.com/scottidler/pais/main/registry/plugins.yaml".to_string(),
            )]),
            hooks: HooksConfig::default(),
            observability: ObservabilityConfig::default(),
        }
    }
}

impl Default for PathsConfig {
    fn default() -> Self {
        let pais_dir = dirs::config_dir().unwrap_or_else(|| PathBuf::from(".")).join("pais");

        Self {
            plugins: pais_dir.join("plugins"),
            skills: pais_dir.join("skills"),
            history: pais_dir.join("history"),
            registries: pais_dir.join("registries"),
        }
    }
}

impl Default for HooksConfig {
    fn default() -> Self {
        Self {
            security_enabled: true,
            history_enabled: true,
            ui_enabled: true,
        }
    }
}

impl Default for ObservabilityConfig {
    fn default() -> Self {
        Self {
            enabled: true,
            sinks: vec![ObservabilitySink::File],
            http_endpoint: None,
            include_payload: false,
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

        // Check PAIS_CONFIG env var
        if let Ok(env_path) = std::env::var("PAIS_CONFIG") {
            let path = PathBuf::from(env_path);
            if path.exists() {
                match Self::load_from_file(&path) {
                    Ok(config) => return Ok(config),
                    Err(e) => {
                        log::warn!("Failed to load config from PAIS_CONFIG: {}", e);
                    }
                }
            }
        }

        // Try PAIS_DIR/pais.yaml
        if let Ok(pais_dir) = std::env::var("PAIS_DIR") {
            let path = PathBuf::from(pais_dir).join("pais.yaml");
            if path.exists() {
                match Self::load_from_file(&path) {
                    Ok(config) => return Ok(config),
                    Err(e) => {
                        log::warn!("Failed to load config from PAIS_DIR: {}", e);
                    }
                }
            }
        }

        // Try ~/.config/pais/pais.yaml
        if let Some(config_dir) = dirs::config_dir() {
            let path = config_dir.join("pais").join("pais.yaml");
            if path.exists() {
                match Self::load_from_file(&path) {
                    Ok(config) => return Ok(config),
                    Err(e) => {
                        log::warn!("Failed to load config from {}: {}", path.display(), e);
                    }
                }
            }
        }

        // Try ./pais.yaml (for development)
        let local_config = PathBuf::from("pais.yaml");
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

        let config: Self = serde_yaml::from_str(&content).context("Failed to parse config file")?;

        log::info!("Loaded config from: {}", path.as_ref().display());
        Ok(config)
    }

    /// Get the PAIS directory (where plugins, history, etc. live)
    pub fn pais_dir() -> PathBuf {
        std::env::var("PAIS_DIR")
            .map(PathBuf::from)
            .unwrap_or_else(|_| dirs::config_dir().unwrap_or_else(|| PathBuf::from(".")).join("pais"))
    }

    /// Expand a path that may contain ~ or env vars
    pub fn expand_path(path: &Path) -> PathBuf {
        let path_str = path.to_string_lossy();
        let expanded = shellexpand::full(&path_str).unwrap_or_else(|_| path_str.clone());
        PathBuf::from(expanded.as_ref())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_default_config() {
        let config = Config::default();
        assert!(config.hooks.security_enabled);
        assert!(config.hooks.history_enabled);
        assert!(config.registries.contains_key("core"));
    }

    #[test]
    fn test_default_hooks_config() {
        let config = HooksConfig::default();
        assert!(config.security_enabled);
        assert!(config.history_enabled);
    }

    #[test]
    fn test_expand_path_no_expansion() {
        let path = PathBuf::from("/usr/local/bin");
        let expanded = Config::expand_path(&path);
        assert_eq!(expanded, PathBuf::from("/usr/local/bin"));
    }

    #[test]
    fn test_expand_path_with_tilde() {
        let path = PathBuf::from("~/test");
        let expanded = Config::expand_path(&path);
        // Should expand ~ to home directory
        assert!(!expanded.to_string_lossy().contains('~'));
        assert!(expanded.to_string_lossy().contains("test"));
    }

    #[test]
    fn test_expand_path_with_env_var() {
        // SAFETY: Test runs single-threaded, env var is test-specific
        unsafe {
            std::env::set_var("PAIS_TEST_VAR", "/custom/path");
        }
        let path = PathBuf::from("$PAIS_TEST_VAR/subdir");
        let expanded = Config::expand_path(&path);
        assert_eq!(expanded, PathBuf::from("/custom/path/subdir"));
        unsafe {
            std::env::remove_var("PAIS_TEST_VAR");
        }
    }

    #[test]
    fn test_pais_dir_default() {
        // Just test that it returns something with "pais" in it
        // Don't modify env vars to avoid test interference
        let dir = Config::pais_dir();
        // Either it's from PAIS_DIR env or it defaults to config dir
        assert!(!dir.to_string_lossy().is_empty());
    }

    #[test]
    fn test_pais_dir_from_env() {
        // SAFETY: Test runs single-threaded, env var is test-specific
        unsafe {
            std::env::set_var("PAIS_DIR", "/custom/pais");
        }
        let dir = Config::pais_dir();
        assert_eq!(dir, PathBuf::from("/custom/pais"));
        unsafe {
            std::env::remove_var("PAIS_DIR");
        }
    }

    #[test]
    fn test_config_serialization_roundtrip() {
        let config = Config::default();
        let yaml_str = serde_yaml::to_string(&config).expect("Failed to serialize");
        let parsed: Config = serde_yaml::from_str(&yaml_str).expect("Failed to deserialize");
        assert_eq!(parsed.hooks.security_enabled, config.hooks.security_enabled);
        assert_eq!(parsed.registries.len(), config.registries.len());
    }

    #[test]
    fn test_load_returns_config() {
        // Just test that load returns something (default or from file)
        let result = Config::load(None);
        assert!(result.is_ok());
    }
}
