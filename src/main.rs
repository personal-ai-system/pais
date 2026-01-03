use clap::Parser;
use eyre::{Context, Result};
use log::info;
use std::fs;
use std::path::PathBuf;

mod agent;
mod architecture;
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
use config::Config;

fn setup_logging() -> Result<()> {
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

    env_logger::Builder::from_default_env()
        .target(env_logger::Target::Pipe(target))
        .init();

    info!("Logging initialized, writing to: {}", log_file.display());
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
        Commands::Registry { action } => commands::registry::run(action, &config),
        Commands::Security { action } => commands::security::run(action, &config),
        Commands::Observe { filter, last, payload } => {
            commands::observe::run(filter.as_deref(), last, payload, &config)
        }
        Commands::Agent { action } => commands::agent::run(action, &config),
        Commands::Run { plugin, action, args } => commands::run::run(&plugin, &action, &args, &config),
        Commands::Status { format } => commands::status::run(cli::OutputFormat::resolve(format), &config),
        Commands::Sync { dry_run, clean } => commands::sync::run(dry_run, clean, &config),
        Commands::Upgrade { dry_run, status } => commands::upgrade::run(dry_run, status, &config),
        Commands::Completions { shell } => commands::completions::run(shell),
    }
}

fn main() -> Result<()> {
    // Setup logging first
    setup_logging().context("Failed to setup logging")?;

    // Parse CLI arguments
    let cli = Cli::parse();

    // Load configuration
    let config = Config::load(cli.config.as_ref()).context("Failed to load configuration")?;

    info!("Starting pais with config from: {:?}", cli.config);

    // Run the command
    run(cli, config).context("Command failed")?;

    Ok(())
}
