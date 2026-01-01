use colored::*;
use eyre::Result;

use crate::config::Config;

pub fn run(plugin: &str, action: &str, args: &[String], _config: &Config) -> Result<()> {
    println!(
        "{} Running: {} {} {}",
        "→".blue(),
        plugin.cyan(),
        action.green(),
        args.join(" ").dimmed()
    );

    // TODO: Implement plugin action execution
    println!("  {} Plugin action execution not yet implemented", "⚠".yellow());

    Ok(())
}
