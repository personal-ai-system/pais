use clap::{Parser, Subcommand};
use std::path::PathBuf;

#[derive(Parser)]
#[command(
    name = "paii",
    about = "Personal AI Infrastructure - A modular plugin system for Claude Code",
    version = env!("GIT_DESCRIBE"),
    after_help = "Logs are written to: ~/.local/share/paii/logs/paii.log\n\nDocumentation: https://github.com/scottidler/paii"
)]
pub struct Cli {
    /// Path to config file
    #[arg(short, long, global = true, help = "Path to paii.toml config file")]
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
    /// Manage plugins
    Plugin {
        #[command(subcommand)]
        action: PluginAction,
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
        /// Output as JSON
        #[arg(long)]
        json: bool,
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
        /// Output as JSON
        #[arg(long)]
        json: bool,
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

        /// Output as JSON
        #[arg(long)]
        json: bool,
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
        /// Output as JSON
        #[arg(long)]
        json: bool,
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
}
