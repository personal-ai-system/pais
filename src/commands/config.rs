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

            println!("{}:", "pais".cyan());
            println!("  version: {}", config.pais.version);
            println!();

            println!("{}:", "paths".cyan());
            println!("  plugins: {}", config.paths.plugins.display());
            println!("  history: {}", config.paths.history.display());
            println!("  registries: {}", config.paths.registries.display());
            println!();

            println!("{}:", "defaults".cyan());
            println!("  language: {}", config.defaults.language);
            println!("  log_level: {}", config.defaults.log_level);
            println!();

            println!("{}:", "registries".cyan());
            for (name, url) in &config.registries {
                println!("  {}: {}", name, url.dimmed());
            }
            println!();

            println!("{}:", "hooks".cyan());
            println!("  security_enabled: {}", config.hooks.security_enabled);
            println!("  history_enabled: {}", config.hooks.history_enabled);
        }
    }

    Ok(())
}

fn get(key: &str, config: &Config) -> Result<()> {
    // Simple dot-notation lookup
    let value = match key {
        "pais.version" => Some(config.pais.version.clone()),
        "paths.plugins" => Some(config.paths.plugins.display().to_string()),
        "paths.history" => Some(config.paths.history.display().to_string()),
        "paths.registries" => Some(config.paths.registries.display().to_string()),
        "defaults.language" => Some(config.defaults.language.clone()),
        "defaults.log_level" => Some(config.defaults.log_level.clone()),
        "hooks.security_enabled" => Some(config.hooks.security_enabled.to_string()),
        "hooks.history_enabled" => Some(config.hooks.history_enabled.to_string()),
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

    // Clone config and modify
    let mut new_config = config.clone();

    // Update the value based on key
    match key {
        "pais.version" => new_config.pais.version = value.to_string(),
        "paths.plugins" => new_config.paths.plugins = value.into(),
        "paths.history" => new_config.paths.history = value.into(),
        "paths.registries" => new_config.paths.registries = value.into(),
        "defaults.language" => new_config.defaults.language = value.to_string(),
        "defaults.log_level" => new_config.defaults.log_level = value.to_string(),
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

    // Write config to file
    let config_path = Config::pais_dir().join("pais.toml");
    fs::create_dir_all(config_path.parent().unwrap())?;

    let toml_str = toml::to_string_pretty(&new_config).context("Failed to serialize config")?;
    fs::write(&config_path, toml_str).context("Failed to write config file")?;

    println!("  {} Saved to {}", "✓".green(), config_path.display());

    Ok(())
}
