use colored::*;
use eyre::Result;

use crate::cli::RegistryAction;
use crate::config::Config;

pub fn run(action: RegistryAction, config: &Config) -> Result<()> {
    match action {
        RegistryAction::List => list(config),
        RegistryAction::Add { name, url } => add(&name, &url, config),
        RegistryAction::Remove { name } => remove(&name, config),
        RegistryAction::Update { name } => update(name.as_deref(), config),
    }
}

fn list(config: &Config) -> Result<()> {
    println!("{}", "Configured registries:".bold());
    println!();

    if config.registries.is_empty() {
        println!("  {}", "(none)".dimmed());
        return Ok(());
    }

    for (name, url) in &config.registries {
        println!("  {}: {}", name.cyan(), url.dimmed());
    }

    Ok(())
}

fn add(name: &str, url: &str, _config: &Config) -> Result<()> {
    println!("{} Adding registry: {} → {}", "→".blue(), name.cyan(), url.dimmed());

    // TODO: Implement registry addition
    println!("  {} Registry addition not yet implemented", "⚠".yellow());

    Ok(())
}

fn remove(name: &str, _config: &Config) -> Result<()> {
    println!("{} Removing registry: {}", "→".blue(), name.cyan());

    // TODO: Implement registry removal
    println!("  {} Registry removal not yet implemented", "⚠".yellow());

    Ok(())
}

fn update(name: Option<&str>, _config: &Config) -> Result<()> {
    match name {
        Some(n) => println!("{} Updating registry: {}", "→".blue(), n.cyan()),
        None => println!("{} Updating all registries", "→".blue()),
    }

    // TODO: Implement registry update
    println!("  {} Registry update not yet implemented", "⚠".yellow());

    Ok(())
}
