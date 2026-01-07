use eyre::{Context, Result};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::fs;
use std::path::{Path, PathBuf};

/// Log level for RUST_LOG
#[derive(Debug, Clone, Deserialize, Serialize, PartialEq, Eq, Default)]
#[serde(rename_all = "lowercase")]
pub enum LogLevel {
    Trace,
    Debug,
    #[default]
    Info,
    Warn,
    Error,
    Off,
}

impl LogLevel {
    /// Convert to RUST_LOG filter string
    pub fn as_filter(&self) -> &'static str {
        match self {
            LogLevel::Trace => "trace",
            LogLevel::Debug => "debug",
            LogLevel::Info => "info",
            LogLevel::Warn => "warn",
            LogLevel::Error => "error",
            LogLevel::Off => "off",
        }
    }
}

/// Main PAIS configuration
#[derive(Debug, Clone, Deserialize, Serialize)]
#[serde(default)]
pub struct Config {
    /// Log level (can be overridden by RUST_LOG env var)
    #[serde(rename = "log-level")]
    pub log_level: LogLevel,
    pub paths: PathsConfig,
    pub hooks: HooksConfig,
    pub observability: ObservabilityConfig,
    pub environment: EnvironmentConfig,
    pub mcp: McpConfig,
    pub skills: SkillsConfig,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
#[serde(default)]
pub struct PathsConfig {
    pub plugins: PathBuf,
    pub skills: PathBuf,
    pub history: PathBuf,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
#[serde(default, rename_all = "kebab-case")]
pub struct HooksConfig {
    pub security_enabled: bool,
    pub history_enabled: bool,
    pub ui_enabled: bool,
    pub research_enabled: bool,
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
#[serde(default, rename_all = "kebab-case")]
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
            log_level: LogLevel::default(),
            paths: PathsConfig {
                plugins: pais_dir.join("plugins"),
                skills: pais_dir.join("skills"),
                history: pais_dir.join("history"),
            },
            hooks: HooksConfig::default(),
            observability: ObservabilityConfig::default(),
            environment: EnvironmentConfig::default(),
            mcp: McpConfig::default(),
            skills: SkillsConfig::default(),
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
        }
    }
}

impl Default for HooksConfig {
    fn default() -> Self {
        Self {
            security_enabled: true,
            history_enabled: true,
            ui_enabled: true,
            research_enabled: true,
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

/// Environment awareness configuration
#[derive(Debug, Clone, Deserialize, Serialize, Default)]
#[serde(default, rename_all = "kebab-case")]
pub struct EnvironmentConfig {
    /// Directory where repos are cloned (e.g., ~/repos/)
    /// Repos are expected at: {repos-dir}/{org}/{repo}
    pub repos_dir: Option<PathBuf>,

    /// Modern tool preferences (e.g., ls -> eza, grep -> rg)
    pub tool_preferences: HashMap<String, String>,

    /// Custom tools with install info
    pub tools: HashMap<String, ToolConfig>,
}

/// Configuration for a custom tool
#[derive(Debug, Clone, Deserialize, Serialize, Default)]
#[serde(default, rename_all = "kebab-case")]
pub struct ToolConfig {
    /// GitHub repository (e.g., "scottidler/otto")
    pub github: Option<String>,

    /// Description of what the tool does
    pub description: Option<String>,

    /// Custom install command (defaults to cargo install --git)
    pub install: Option<String>,
}

/// MCP (Model Context Protocol) server configuration
#[derive(Debug, Clone, Deserialize, Serialize, Default)]
#[serde(default, rename_all = "kebab-case")]
pub struct McpConfig {
    /// Source files for MCP server definitions
    /// First one found with the server will be used
    pub sources: Vec<PathBuf>,

    /// Named profiles mapping to lists of MCP server names
    pub profiles: HashMap<String, Vec<String>>,

    /// Default profile to use when none specified
    pub default_profile: Option<String>,

    /// Additional MCP server definitions (supplements sources)
    pub servers: HashMap<String, McpServerConfig>,
}

/// Configuration for a single MCP server
#[derive(Debug, Clone, Deserialize, Serialize)]
#[serde(rename_all = "kebab-case")]
pub struct McpServerConfig {
    /// Command to run the MCP server
    pub command: String,

    /// Arguments to pass to the command
    #[serde(default)]
    pub args: Vec<String>,

    /// Environment variables for the server
    #[serde(default)]
    pub env: HashMap<String, String>,
}

/// Skills configuration for dynamic skill loading
#[derive(Debug, Clone, Deserialize, Serialize, Default)]
#[serde(default, rename_all = "kebab-case")]
pub struct SkillsConfig {
    /// Named profiles mapping to lists of skill names
    pub profiles: HashMap<String, Vec<String>>,

    /// Default profile to use when none specified
    pub default_profile: Option<String>,
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

    /// Claude Code settings path (for display)
    pub const CLAUDE_SETTINGS_JSON: &'static str = "~/.claude/settings.json";

    /// Get the Claude Code skills directory (~/.claude/skills)
    pub fn claude_skills_dir() -> Option<PathBuf> {
        dirs::home_dir().map(|h| h.join(".claude/skills"))
    }

    /// Get the Claude Code settings file
    pub fn claude_settings_file() -> Option<PathBuf> {
        dirs::home_dir().map(|h| h.join(".claude/settings.json"))
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
        assert_eq!(config.log_level, LogLevel::Info);
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
        assert_eq!(parsed.log_level, config.log_level);
    }

    #[test]
    fn test_load_returns_config() {
        // Just test that load returns something (default or from file)
        let result = Config::load(None);
        assert!(result.is_ok());
    }

    #[test]
    fn test_parse_kebab_case_hooks_config() {
        // This is how users write config files - kebab-case keys
        let yaml = r#"
security-enabled: false
history-enabled: true
ui-enabled: false
"#;
        let config: HooksConfig = serde_yaml::from_str(yaml).expect("Failed to parse kebab-case HooksConfig");
        assert!(!config.security_enabled);
        assert!(config.history_enabled);
        assert!(!config.ui_enabled);
    }

    #[test]
    fn test_parse_kebab_case_observability_config() {
        let yaml = r#"
enabled: true
sinks:
  - file
  - stdout
http-endpoint: "https://example.com/logs"
include-payload: true
"#;
        let config: ObservabilityConfig =
            serde_yaml::from_str(yaml).expect("Failed to parse kebab-case ObservabilityConfig");
        assert!(config.enabled);
        assert_eq!(config.sinks.len(), 2);
        assert_eq!(config.http_endpoint, Some("https://example.com/logs".to_string()));
        assert!(config.include_payload);
    }

    #[test]
    fn test_parse_kebab_case_environment_config() {
        let yaml = r#"
repos-dir: ~/repos/
tool-preferences:
  ls: eza
  grep: rg
tools:
  otto:
    github: otto-rs/otto
    description: CI runner
"#;
        let config: EnvironmentConfig =
            serde_yaml::from_str(yaml).expect("Failed to parse kebab-case EnvironmentConfig");
        assert_eq!(config.repos_dir, Some(PathBuf::from("~/repos/")));
        assert_eq!(config.tool_preferences.get("ls"), Some(&"eza".to_string()));
        assert_eq!(config.tool_preferences.get("grep"), Some(&"rg".to_string()));
        assert!(config.tools.contains_key("otto"));
    }

    #[test]
    fn test_parse_realistic_config_file() {
        // Test a realistic config file as users would write it
        let yaml = r#"
log-level: debug

paths:
  plugins: ~/.config/pais/plugins
  skills: ~/.config/pais/skills
  history: ~/.config/pais/history

hooks:
  security-enabled: true
  history-enabled: true
  ui-enabled: true

observability:
  enabled: true
  sinks:
    - file
  include-payload: false

environment:
  repos-dir: ~/repos/
  tool-preferences:
    ls: eza
    cat: bat
  tools:
    clone:
      github: scottidler/clone
      description: Git clone helper
"#;
        let config: Config = serde_yaml::from_str(yaml).expect("Failed to parse realistic config file");

        // Verify log level
        assert_eq!(config.log_level, LogLevel::Debug);

        // Verify paths
        assert_eq!(config.paths.plugins, PathBuf::from("~/.config/pais/plugins"));

        // Verify hooks (kebab-case keys)
        assert!(config.hooks.security_enabled);
        assert!(config.hooks.history_enabled);
        assert!(config.hooks.ui_enabled);

        // Verify observability (kebab-case keys)
        assert!(config.observability.enabled);
        assert!(!config.observability.include_payload);

        // Verify environment (kebab-case keys)
        assert_eq!(config.environment.repos_dir, Some(PathBuf::from("~/repos/")));
        assert_eq!(config.environment.tool_preferences.get("ls"), Some(&"eza".to_string()));
        assert!(config.environment.tools.contains_key("clone"));
    }

    #[test]
    fn test_log_level_parsing() {
        let yaml = "log-level: trace";
        let config: Config = serde_yaml::from_str(yaml).expect("Failed to parse log level");
        assert_eq!(config.log_level, LogLevel::Trace);
        assert_eq!(config.log_level.as_filter(), "trace");

        let yaml = "log-level: debug";
        let config: Config = serde_yaml::from_str(yaml).expect("Failed to parse log level");
        assert_eq!(config.log_level, LogLevel::Debug);
        assert_eq!(config.log_level.as_filter(), "debug");

        let yaml = "log-level: warn";
        let config: Config = serde_yaml::from_str(yaml).expect("Failed to parse log level");
        assert_eq!(config.log_level, LogLevel::Warn);
        assert_eq!(config.log_level.as_filter(), "warn");
    }
}
