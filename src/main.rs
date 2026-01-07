use clap::Parser;
use eyre::{Context, Result};
use log::info;
use std::fs;
use std::path::PathBuf;

mod agent;
mod architecture;
mod bundle;
mod cli;
mod commands;
mod config;
mod contract;
mod history;
mod hook;
mod migrate;
mod observability;
mod plugin;
mod skill;

use cli::{Cli, Commands};
use config::{Config, LogLevel};

fn setup_logging(log_level: &LogLevel) -> Result<()> {
    // Create log directory
    let log_dir = dirs::data_local_dir()
        .unwrap_or_else(|| PathBuf::from("."))
        .join("pais")
        .join("logs");

    fs::create_dir_all(&log_dir).context("Failed to create log directory")?;

    let log_file = log_dir.join("pais.log");

    // Setup env_logger with file output
    let target = Box::new(
        fs::OpenOptions::new()
            .create(true)
            .append(true)
            .open(&log_file)
            .context("Failed to open log file")?,
    );

    // RUST_LOG env var takes precedence, otherwise use config log_level
    let mut builder = env_logger::Builder::new();

    if std::env::var("RUST_LOG").is_ok() {
        // Let env_logger parse RUST_LOG
        builder.parse_default_env();
    } else {
        // Use log level from config
        builder.filter_level(match log_level {
            LogLevel::Trace => log::LevelFilter::Trace,
            LogLevel::Debug => log::LevelFilter::Debug,
            LogLevel::Info => log::LevelFilter::Info,
            LogLevel::Warn => log::LevelFilter::Warn,
            LogLevel::Error => log::LevelFilter::Error,
            LogLevel::Off => log::LevelFilter::Off,
        });
    }

    builder.target(env_logger::Target::Pipe(target)).init();

    info!("Logging initialized, writing to: {}", log_file.display());
    info!(
        "Log level: {} (from {})",
        log_level.as_filter(),
        if std::env::var("RUST_LOG").is_ok() { "RUST_LOG env" } else { "config" }
    );
    Ok(())
}

fn run(cli: Cli, config: Config) -> Result<()> {
    match cli.command {
        Commands::Init { path, force, no_git } => commands::init::run(path, force, no_git),
        Commands::Doctor => commands::doctor::run(&config),
        Commands::Plugin { action } => commands::plugin::run(action, &config),
        Commands::Skill { action } => commands::skill::run(action, &config),
        Commands::Hook { action } => commands::hook::run(action, &config),
        Commands::History { action } => commands::history::run(action, &config),
        Commands::Config { action } => commands::config::run(action, &config),
        Commands::Context { action } => commands::context::run(action, &config),
        Commands::Security { action } => commands::security::run(action, &config),
        Commands::Observe { filter, last, payload } => {
            commands::observe::run(filter.as_deref(), last, payload, &config)
        }
        Commands::Agent { action } => commands::agent::run(action, &config),
        Commands::Bundle { action } => commands::bundle::run(action, &config),
        Commands::Image { action } => commands::image::run(action, &config),
        Commands::Diagram { action } => commands::diagram::run(action, &config),
        Commands::Run { plugin, action, args } => commands::run::run(&plugin, &action, &args, &config),
        Commands::Session {
            mcp,
            skill,
            list,
            dry_run,
            format,
            claude_args,
        } => commands::session::run(mcp, skill, list, dry_run, format, claude_args, &config),
        Commands::Status { format } => commands::status::run(cli::OutputFormat::resolve(format), &config),
        Commands::Sync { dry_run, clean } => commands::sync::run(dry_run, clean, &config),
        Commands::Upgrade { dry_run, status } => commands::upgrade::run(dry_run, status, &config),
        Commands::Completions { shell } => commands::completions::run(shell),
    }
}

fn main() -> Result<()> {
    // Parse CLI arguments first
    let cli = Cli::parse();

    // Load configuration (before logging, so log messages in Config::load are silent)
    let config = Config::load(cli.config.as_ref()).context("Failed to load configuration")?;

    // Setup logging with log level from config (or RUST_LOG env var)
    setup_logging(&config.log_level).context("Failed to setup logging")?;

    info!("Starting pais with config from: {:?}", cli.config);

    // Run the command
    run(cli, config).context("Command failed")?;

    Ok(())
}
