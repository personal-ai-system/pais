use colored::*;
use eyre::{Context, Result};
use std::fs;

use crate::cli::{ConfigAction, OutputFormat};
use crate::config::Config;

pub fn run(action: ConfigAction, config: &Config) -> Result<()> {
    match action {
        ConfigAction::Show { format } => show(OutputFormat::resolve(format), config),
        ConfigAction::Get { key } => get(&key, config),
        ConfigAction::Set { key, value } => set(&key, &value, config),
    }
}

fn show(format: OutputFormat, config: &Config) -> Result<()> {
    match format {
        OutputFormat::Json => {
            println!("{}", serde_json::to_string_pretty(config)?);
        }
        OutputFormat::Yaml => {
            println!("{}", serde_yaml::to_string(config)?);
        }
        OutputFormat::Text => {
            println!("{}", "PAIS Configuration".bold());
            println!();

            println!("{}:", "paths".cyan());
            println!("  plugins: {}", config.paths.plugins.display());
            println!("  skills: {}", config.paths.skills.display());
            println!("  history: {}", config.paths.history.display());
            println!();

            println!("{}:", "hooks".cyan());
            println!("  security_enabled: {}", config.hooks.security_enabled);
            println!("  history_enabled: {}", config.hooks.history_enabled);
        }
    }

    Ok(())
}

fn get(key: &str, config: &Config) -> Result<()> {
    let value = match key {
        "paths.plugins" => Some(config.paths.plugins.display().to_string()),
        "paths.skills" => Some(config.paths.skills.display().to_string()),
        "paths.history" => Some(config.paths.history.display().to_string()),
        "hooks.security_enabled" => Some(config.hooks.security_enabled.to_string()),
        "hooks.history_enabled" => Some(config.hooks.history_enabled.to_string()),
        "log_level" | "log-level" => Some(config.log_level.as_filter().to_string()),
        _ => None,
    };

    match value {
        Some(v) => println!("{}", v),
        None => {
            eprintln!("{} Unknown config key: {}", "✗".red(), key);
            std::process::exit(1);
        }
    }

    Ok(())
}

fn set(key: &str, value: &str, config: &Config) -> Result<()> {
    println!("{} Setting {} = {}", "→".blue(), key.cyan(), value.green());

    let mut new_config = config.clone();

    match key {
        "paths.plugins" => new_config.paths.plugins = value.into(),
        "paths.skills" => new_config.paths.skills = value.into(),
        "paths.history" => new_config.paths.history = value.into(),
        "hooks.security_enabled" => {
            new_config.hooks.security_enabled =
                value.parse().context("Invalid boolean value (use 'true' or 'false')")?;
        }
        "hooks.history_enabled" => {
            new_config.hooks.history_enabled =
                value.parse().context("Invalid boolean value (use 'true' or 'false')")?;
        }
        _ => {
            eyre::bail!("Unknown config key: {}", key);
        }
    }

    let config_path = Config::pais_dir().join("pais.yaml");
    if let Some(parent) = config_path.parent() {
        fs::create_dir_all(parent)?;
    }

    let yaml_str = serde_yaml::to_string(&new_config).context("Failed to serialize config")?;
    fs::write(&config_path, yaml_str).context("Failed to write config file")?;

    println!("  {} Saved to {}", "✓".green(), config_path.display());

    Ok(())
}
