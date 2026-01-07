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
    about = "Personal AI System - A modular plugin system for Claude Code",
    version = env!("GIT_DESCRIBE"),
    after_help = "Logs are written to: ~/.local/share/pais/logs/pais.log\n\nDocumentation: https://github.com/scottidler/pais"
)]
pub struct Cli {
    /// Path to config file
    #[arg(short, long, global = true, help = "Path to pais.yaml config file")]
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

        /// Skip git repository initialization
        #[arg(long)]
        no_git: bool,
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

    /// Context injection for hooks
    Context {
        #[command(subcommand)]
        action: ContextAction,
    },

    /// Security validation tools
    Security {
        #[command(subcommand)]
        action: SecurityAction,
    },

    /// Live event stream (tail events in real-time)
    Observe {
        /// Filter by event type (e.g., PreToolUse, SessionStart)
        #[arg(long, short = 'f')]
        filter: Option<String>,

        /// Number of recent events to show before tailing
        #[arg(long, short = 'n', default_value = "10")]
        last: usize,

        /// Include full payload in output
        #[arg(long)]
        payload: bool,
    },

    /// Manage agent personalities
    Agent {
        #[command(subcommand)]
        action: AgentAction,
    },

    /// Manage plugin bundles
    Bundle {
        #[command(subcommand)]
        action: BundleAction,
    },

    /// Generate images using AI models
    Image {
        #[command(subcommand)]
        action: ImageAction,
    },

    /// Generate diagrams using Mermaid
    Diagram {
        #[command(subcommand)]
        action: DiagramAction,
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

    /// Launch Claude Code with dynamic MCP and skill configuration
    Session {
        /// MCP servers or profiles to load (comma-separated, profiles expand to their contents)
        #[arg(short, long, value_delimiter = ',')]
        mcp: Option<Vec<String>>,

        /// Skills or profiles to load (comma-separated, profiles expand to their contents)
        #[arg(short, long, value_delimiter = ',')]
        skill: Option<Vec<String>>,

        /// List available MCPs, skills, and profiles
        #[arg(short, long)]
        list: bool,

        /// Show what would happen without launching
        #[arg(long)]
        dry_run: bool,

        /// Output format for --list (default: text for TTY, json for pipes)
        #[arg(long, short = 'o', value_enum)]
        format: Option<OutputFormat>,

        /// Additional arguments to pass to Claude
        #[arg(last = true)]
        claude_args: Vec<String>,
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

    /// Upgrade PAIS configuration (run migrations)
    Upgrade {
        /// Show what would happen without making changes
        #[arg(long)]
        dry_run: bool,

        /// Show current version info only
        #[arg(long)]
        status: bool,
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

        /// Output format (default: text for TTY, json for pipes)
        #[arg(long, short = 'o', value_enum)]
        format: Option<OutputFormat>,
    },

    /// Show plugin installation guide
    InstallGuide {
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

        /// Show only simple skills (no plugin.yaml)
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

    /// Generate skill index for context injection
    Index {
        /// Output format (default: text for TTY, json for pipes)
        #[arg(long, short = 'o', value_enum)]
        format: Option<OutputFormat>,
    },

    /// Show or list workflows for a skill
    Workflow {
        /// Skill name
        skill: String,

        /// Workflow name/intent (if omitted, lists available workflows)
        workflow: Option<String>,

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

    /// Show a specific history entry
    Show {
        /// Entry ID
        id: String,
    },

    /// Show event statistics
    Stats {
        /// Number of days to include
        #[arg(long, default_value = "7")]
        days: usize,

        /// Output format (default: text for TTY, json for pipes)
        #[arg(long, short = 'o', value_enum)]
        format: Option<OutputFormat>,
    },

    /// List raw event dates
    Events {
        /// Number of recent dates to show
        #[arg(long, default_value = "10")]
        limit: usize,
    },
}

#[derive(Subcommand)]
pub enum AgentAction {
    /// List available agents
    List {
        /// Output format (default: text for TTY, json for pipes)
        #[arg(long, short = 'o', value_enum)]
        format: Option<OutputFormat>,
    },

    /// Show agent details
    Show {
        /// Agent name
        name: String,

        /// Output format (default: text for TTY, json for pipes)
        #[arg(long, short = 'o', value_enum)]
        format: Option<OutputFormat>,
    },

    /// List available traits
    Traits {
        /// Output format (default: text for TTY, json for pipes)
        #[arg(long, short = 'o', value_enum)]
        format: Option<OutputFormat>,
    },

    /// Generate prompt for an agent
    Prompt {
        /// Agent name
        name: String,
    },

    /// Create a new agent from template
    Create {
        /// Agent name
        name: String,
    },
}

#[derive(Subcommand)]
pub enum BundleAction {
    /// List available bundles
    List {
        /// Output format (default: text for TTY, json for pipes)
        #[arg(long, short = 'o', value_enum)]
        format: Option<OutputFormat>,
    },

    /// Show bundle details
    Show {
        /// Bundle name
        name: String,

        /// Output format (default: text for TTY, json for pipes)
        #[arg(long, short = 'o', value_enum)]
        format: Option<OutputFormat>,
    },

    /// Install a bundle
    Install {
        /// Bundle name
        name: String,

        /// Install only required plugins (skip optional)
        #[arg(long)]
        required_only: bool,

        /// Skip verification after installation
        #[arg(long)]
        skip_verify: bool,
    },

    /// Create a new bundle
    New {
        /// Bundle name
        name: String,

        /// Output path (default: ~/.config/pais/bundles/<name>)
        #[arg(long)]
        path: Option<PathBuf>,
    },
}

#[derive(Subcommand)]
pub enum ContextAction {
    /// Inject skill context for SessionStart hook
    Inject {
        /// Output raw content without system-reminder wrapper
        #[arg(long)]
        raw: bool,
    },
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
pub enum SecurityAction {
    /// Show security tiers and their actions
    Tiers {
        /// Output format (default: text for TTY, json for pipes)
        #[arg(long, short = 'o', value_enum)]
        format: Option<OutputFormat>,
    },

    /// View security event log
    Log {
        /// Number of days to show
        #[arg(long, default_value = "7")]
        days: usize,

        /// Output format (default: text for TTY, json for pipes)
        #[arg(long, short = 'o', value_enum)]
        format: Option<OutputFormat>,
    },

    /// Test a command against security patterns
    Test {
        /// Command to test
        command: String,
    },
}

#[derive(Subcommand)]
pub enum ImageAction {
    /// Generate an image using AI
    Generate {
        /// Image generation prompt
        #[arg(long, short = 'p')]
        prompt: String,

        /// AI model to use (gemini, flux, openai)
        #[arg(long, short = 'm', default_value = "gemini")]
        model: String,

        /// Image size (1K, 2K, 4K for gemini; 1024x1024 for openai)
        #[arg(long, short = 's')]
        size: Option<String>,

        /// Aspect ratio (16:9, 1:1, 3:2, 21:9, etc.)
        #[arg(long, short = 'a')]
        aspect_ratio: Option<String>,

        /// Output file path
        #[arg(long, short = 'o')]
        output: Option<PathBuf>,

        /// Remove background (requires REMOVEBG_API_KEY)
        #[arg(long)]
        remove_bg: bool,

        /// Create thumbnail version with dark background
        #[arg(long)]
        thumbnail: bool,
    },

    /// List available AI models
    Models {
        /// Output format (default: text for TTY, json for pipes)
        #[arg(long, short = 'o', value_enum)]
        format: Option<OutputFormat>,
    },
}

#[derive(Subcommand)]
pub enum DiagramAction {
    /// Render a Mermaid diagram from file or stdin
    Render {
        /// Path to .mmd file (reads from stdin if omitted)
        #[arg()]
        file: Option<PathBuf>,

        /// Raw mermaid string to render
        #[arg(short, long)]
        mermaid: Option<String>,

        /// Output format (svg, png)
        #[arg(long, short = 'f', default_value = "svg")]
        format: String,

        /// Output file path (prints to stdout if omitted)
        #[arg(long, short = 'o')]
        output: Option<PathBuf>,

        /// Image width
        #[arg(long)]
        width: Option<u32>,

        /// Image height
        #[arg(long)]
        height: Option<u32>,

        /// Scale factor
        #[arg(long)]
        scale: Option<f32>,

        /// Background color
        #[arg(long)]
        background: Option<String>,

        /// Mermaid.ink server URL
        #[arg(long, default_value = "https://mermaid.ink")]
        server: String,

        /// Copy output to clipboard
        #[arg(long)]
        clipboard: bool,

        /// Open result in browser/viewer
        #[arg(long)]
        open: bool,
    },

    /// Generate a flowchart diagram
    Flowchart {
        /// Direction: TB (top-bottom), BT, LR, RL
        #[arg(long, short = 'd', default_value = "TB")]
        direction: String,

        /// YAML config file for diagram definition
        #[arg(long, short = 'c')]
        config: Option<PathBuf>,

        /// Output format (svg, png, mermaid)
        #[arg(long, short = 'f', default_value = "svg")]
        format: String,

        /// Output file path
        #[arg(long, short = 'o')]
        output: Option<PathBuf>,

        /// Mermaid.ink server URL
        #[arg(long, default_value = "https://mermaid.ink")]
        server: String,
    },

    /// Generate a sequence diagram
    Sequence {
        /// YAML config file for diagram definition
        #[arg(long, short = 'c')]
        config: Option<PathBuf>,

        /// Output format (svg, png, mermaid)
        #[arg(long, short = 'f', default_value = "svg")]
        format: String,

        /// Output file path
        #[arg(long, short = 'o')]
        output: Option<PathBuf>,

        /// Mermaid.ink server URL
        #[arg(long, default_value = "https://mermaid.ink")]
        server: String,
    },

    /// Generate an ER (entity-relationship) diagram
    Er {
        /// YAML config file for diagram definition
        #[arg(long, short = 'c')]
        config: Option<PathBuf>,

        /// Output format (svg, png, mermaid)
        #[arg(long, short = 'f', default_value = "svg")]
        format: String,

        /// Output file path
        #[arg(long, short = 'o')]
        output: Option<PathBuf>,

        /// Mermaid.ink server URL
        #[arg(long, default_value = "https://mermaid.ink")]
        server: String,
    },

    /// Generate a state diagram
    State {
        /// YAML config file for diagram definition
        #[arg(long, short = 'c')]
        config: Option<PathBuf>,

        /// Output format (svg, png, mermaid)
        #[arg(long, short = 'f', default_value = "svg")]
        format: String,

        /// Output file path
        #[arg(long, short = 'o')]
        output: Option<PathBuf>,

        /// Mermaid.ink server URL
        #[arg(long, default_value = "https://mermaid.ink")]
        server: String,
    },

    /// Generate a mindmap diagram
    Mindmap {
        /// YAML config file for diagram definition
        #[arg(long, short = 'c')]
        config: Option<PathBuf>,

        /// Output format (svg, png, mermaid)
        #[arg(long, short = 'f', default_value = "svg")]
        format: String,

        /// Output file path
        #[arg(long, short = 'o')]
        output: Option<PathBuf>,

        /// Mermaid.ink server URL
        #[arg(long, default_value = "https://mermaid.ink")]
        server: String,
    },

    /// Generate a pie chart
    Pie {
        /// Chart title
        #[arg(long, short = 't')]
        title: Option<String>,

        /// Show legend
        #[arg(long)]
        show_data: bool,

        /// YAML config file for data
        #[arg(long, short = 'c')]
        config: Option<PathBuf>,

        /// Output format (svg, png, mermaid)
        #[arg(long, short = 'f', default_value = "svg")]
        format: String,

        /// Output file path
        #[arg(long, short = 'o')]
        output: Option<PathBuf>,

        /// Mermaid.ink server URL
        #[arg(long, default_value = "https://mermaid.ink")]
        server: String,
    },

    /// Generate a user journey diagram
    Journey {
        /// Journey title
        #[arg(long, short = 't')]
        title: Option<String>,

        /// YAML config file for diagram definition
        #[arg(long, short = 'c')]
        config: Option<PathBuf>,

        /// Output format (svg, png, mermaid)
        #[arg(long, short = 'f', default_value = "svg")]
        format: String,

        /// Output file path
        #[arg(long, short = 'o')]
        output: Option<PathBuf>,

        /// Mermaid.ink server URL
        #[arg(long, default_value = "https://mermaid.ink")]
        server: String,
    },

    /// List available diagram types
    Types {
        /// Output format (default: text for TTY, json for pipes)
        #[arg(long, short = 'o', value_enum)]
        format: Option<OutputFormat>,
    },
}
