use colored::*;
use eyre::Result;

use crate::cli::ConfigAction;
use crate::config::Config;

pub fn run(action: ConfigAction, config: &Config) -> Result<()> {
    match action {
        ConfigAction::Show { json } => show(json, config),
        ConfigAction::Get { key } => get(&key, config),
        ConfigAction::Set { key, value } => set(&key, &value, config),
    }
}

fn show(json: bool, config: &Config) -> Result<()> {
    if json {
        let json_str = serde_json::to_string_pretty(config)?;
        println!("{}", json_str);
    } else {
        println!("{}", "PAII Configuration".bold());
        println!();

        println!("{}:", "paii".cyan());
        println!("  version: {}", config.paii.version);
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

    Ok(())
}

fn get(key: &str, config: &Config) -> Result<()> {
    // Simple dot-notation lookup
    let value = match key {
        "paii.version" => Some(config.paii.version.clone()),
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

fn set(key: &str, value: &str, _config: &Config) -> Result<()> {
    println!("{} Setting {} = {}", "→".blue(), key.cyan(), value.green());

    // TODO: Implement config writing
    println!("  {} Config writing not yet implemented", "⚠".yellow());

    Ok(())
}
