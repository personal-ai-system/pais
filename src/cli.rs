use clap::{Parser, Subcommand, ValueEnum};
use std::io::IsTerminal;
use std::path::PathBuf;

/// Output format for commands
#[derive(Debug, Clone, Copy, PartialEq, Eq, ValueEnum)]
pub enum OutputFormat {
    /// Human-readable text
    Text,
    /// JSON format
    Json,
    /// YAML format
    Yaml,
}

impl OutputFormat {
    /// Resolve the effective output format.
    /// If user specified a format, use it.
    /// Otherwise: TTY → Text, non-TTY (pipe) → Json
    pub fn resolve(user_choice: Option<OutputFormat>) -> OutputFormat {
        match user_choice {
            Some(fmt) => fmt,
            None => {
                if std::io::stdout().is_terminal() {
                    OutputFormat::Text
                } else {
                    OutputFormat::Json
                }
            }
        }
    }
}

#[derive(Parser)]
#[command(
    name = "pais",
    about = "Personal AI Infrastructure - A modular plugin system for Claude Code",
    version = env!("GIT_DESCRIBE"),
    after_help = "Logs are written to: ~/.local/share/pais/logs/pais.log\n\nDocumentation: https://github.com/scottidler/pais"
)]
pub struct Cli {
    /// Path to config file
    #[arg(short, long, global = true, help = "Path to pais.toml config file")]
    pub config: Option<PathBuf>,

    /// Enable verbose output
    #[arg(short, long, global = true, help = "Enable verbose output")]
    pub verbose: bool,

    /// Suppress non-error output
    #[arg(short, long, global = true, help = "Suppress non-error output")]
    pub quiet: bool,

    #[command(subcommand)]
    pub command: Commands,
}

#[derive(Subcommand)]
pub enum Commands {
    /// Initialize PAIS configuration
    Init {
        /// Directory to initialize (defaults to ~/.config/pais)
        #[arg(long)]
        path: Option<PathBuf>,

        /// Overwrite existing configuration
        #[arg(long)]
        force: bool,
    },

    /// Diagnose setup issues
    Doctor,

    /// Manage plugins
    Plugin {
        #[command(subcommand)]
        action: PluginAction,
    },

    /// Manage skills
    Skill {
        #[command(subcommand)]
        action: SkillAction,
    },

    /// Handle hook events from Claude Code
    Hook {
        #[command(subcommand)]
        action: HookAction,
    },

    /// Query and manage history
    History {
        #[command(subcommand)]
        action: HistoryAction,
    },

    /// Manage configuration
    Config {
        #[command(subcommand)]
        action: ConfigAction,
    },

    /// Manage plugin registries
    Registry {
        #[command(subcommand)]
        action: RegistryAction,
    },

    /// Run a plugin action directly
    Run {
        /// Plugin name
        plugin: String,

        /// Action to run
        action: String,

        /// Action arguments
        #[arg(trailing_var_arg = true)]
        args: Vec<String>,
    },

    /// Show system status
    Status {
        /// Output format (default: text for TTY, json for pipes)
        #[arg(long, short = 'o', value_enum)]
        format: Option<OutputFormat>,
    },

    /// Sync skills to Claude Code (~/.claude/skills/)
    Sync {
        /// Show what would happen without making changes
        #[arg(long)]
        dry_run: bool,

        /// Remove orphaned symlinks from Claude skills directory
        #[arg(long)]
        clean: bool,
    },

    /// Generate shell completions
    Completions {
        /// Shell to generate completions for
        shell: clap_complete::Shell,
    },
}

#[derive(Subcommand)]
pub enum PluginAction {
    /// List installed plugins
    List {
        /// Output format (default: text for TTY, json for pipes)
        #[arg(long, short = 'o', value_enum)]
        format: Option<OutputFormat>,
    },

    /// Install a plugin
    Install {
        /// Plugin source (name, git URL, or local path)
        source: String,

        /// Symlink for development (don't copy)
        #[arg(long)]
        dev: bool,

        /// Overwrite existing installation
        #[arg(long)]
        force: bool,
    },

    /// Remove a plugin
    Remove {
        /// Plugin name
        name: String,

        /// Remove even if other plugins depend on it
        #[arg(long)]
        force: bool,
    },

    /// Update a plugin
    Update {
        /// Plugin name (or "all")
        name: String,
    },

    /// Show plugin details
    Info {
        /// Plugin name
        name: String,
    },

    /// Create a new plugin
    New {
        /// Plugin name
        name: String,

        /// Language (python or rust)
        #[arg(long, default_value = "python")]
        language: String,

        /// Plugin type (foundation, integration, skill)
        #[arg(long, default_value = "skill")]
        r#type: String,

        /// Output path
        #[arg(long)]
        path: Option<PathBuf>,
    },

    /// Verify plugin installation
    Verify {
        /// Plugin name
        name: String,
    },
}

#[derive(Subcommand)]
pub enum SkillAction {
    /// List all skills (simple and plugin-based)
    List {
        /// Output format (default: text for TTY, json for pipes)
        #[arg(long, short = 'o', value_enum)]
        format: Option<OutputFormat>,

        /// Show only simple skills (no plugin.toml)
        #[arg(long)]
        simple: bool,

        /// Show only plugin skills
        #[arg(long)]
        plugin: bool,
    },

    /// Create a new skill from template
    Add {
        /// Skill name
        name: String,

        /// Open in $EDITOR after creation
        #[arg(long, short = 'e')]
        edit: bool,
    },

    /// Show skill details
    Info {
        /// Skill name
        name: String,
    },

    /// Edit a skill in $EDITOR
    Edit {
        /// Skill name
        name: String,
    },

    /// Remove a skill
    Remove {
        /// Skill name
        name: String,

        /// Remove without confirmation
        #[arg(long)]
        force: bool,
    },

    /// Validate SKILL.md format
    Validate {
        /// Skill name (or "all" to validate all skills)
        name: String,
    },

    /// Scan directories for .pais/SKILL.md files
    Scan {
        /// Directory to scan (defaults to ~/repos)
        path: Option<PathBuf>,

        /// Maximum depth to scan (default: 4)
        #[arg(long, default_value = "4")]
        depth: usize,

        /// Register found skills (create symlinks in ~/.config/pais/skills/)
        #[arg(long)]
        register: bool,

        /// Output format (default: text for TTY, json for pipes)
        #[arg(long, short = 'o', value_enum)]
        format: Option<OutputFormat>,
    },
}

#[derive(Subcommand)]
pub enum HookAction {
    /// Dispatch a hook event to handlers
    Dispatch {
        /// Event type (pre-tool-use, post-tool-use, stop, session-start, etc.)
        event: String,

        /// Event payload JSON (reads from stdin if not provided)
        #[arg(long)]
        payload: Option<String>,
    },

    /// List registered hook handlers
    List {
        /// Filter by event type
        #[arg(long)]
        event: Option<String>,
    },
}

#[derive(Subcommand)]
pub enum HistoryAction {
    /// Search history
    Query {
        /// Search query (regex)
        query: String,

        /// Category to search
        #[arg(long)]
        category: Option<String>,

        /// Max results
        #[arg(long, default_value = "10")]
        limit: usize,

        /// Only entries after this date
        #[arg(long)]
        since: Option<String>,

        /// Output format (default: text for TTY, json for pipes)
        #[arg(long, short = 'o', value_enum)]
        format: Option<OutputFormat>,
    },

    /// Show recent entries
    Recent {
        /// Category
        #[arg(long)]
        category: Option<String>,

        /// Number of entries
        #[arg(long, default_value = "5")]
        count: usize,
    },

    /// List available categories
    Categories,
}

#[derive(Subcommand)]
pub enum ConfigAction {
    /// Show current configuration
    Show {
        /// Output format (default: text for TTY, json for pipes)
        #[arg(long, short = 'o', value_enum)]
        format: Option<OutputFormat>,
    },

    /// Get a configuration value
    Get {
        /// Configuration key (dot notation)
        key: String,
    },

    /// Set a configuration value
    Set {
        /// Configuration key
        key: String,

        /// New value
        value: String,
    },
}

#[derive(Subcommand)]
pub enum RegistryAction {
    /// List configured registries
    List,

    /// Add a registry
    Add {
        /// Registry name
        name: String,

        /// Registry URL
        url: String,
    },

    /// Remove a registry
    Remove {
        /// Registry name
        name: String,
    },

    /// Update registry listings
    Update {
        /// Registry name (updates all if omitted)
        name: Option<String>,
    },

    /// Search for plugins in cached registries
    Search {
        /// Search query (matches name, description, tags)
        query: String,

        /// Output format (default: text for TTY, json for pipes)
        #[arg(long, short = 'o', value_enum)]
        format: Option<OutputFormat>,
    },

    /// Show all plugins in a cached registry
    Show {
        /// Registry name
        name: String,

        /// Output format (default: text for TTY, json for pipes)
        #[arg(long, short = 'o', value_enum)]
        format: Option<OutputFormat>,
    },
}
