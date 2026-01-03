//! Upgrade command
//!
//! Run config migrations to upgrade PAIS configuration.

use colored::*;
use eyre::Result;

use crate::config::Config;
use crate::migrate;

/// Run the upgrade command
pub fn run(dry_run: bool, status_only: bool, config: &Config) -> Result<()> {
    if status_only {
        show_status()?;
        return Ok(());
    }

    let (current, target) = migrate::version_info()?;

    if current >= target {
        println!("{} PAIS is up to date (v{})", "✓".green(), target);
        return Ok(());
    }

    println!("{} Upgrading PAIS from v{} to v{}", "🔄".blue(), current, target);
    println!();

    // Show pending migrations
    let pending = migrate::pending_migrations()?;
    if pending.is_empty() {
        println!("{} No migrations to apply", "✓".green());
        return Ok(());
    }

    println!("Pending migrations:");
    for (from, to, desc) in &pending {
        println!("  • v{} → v{}: {}", from, to, desc);
    }
    println!();

    // Run migrations
    let applied = migrate::run_migrations(config, dry_run)?;

    println!();
    if dry_run {
        println!("{} Dry run - no changes applied", "📋".blue());
    } else {
        println!("{} Applied {} migration(s)", "✓".green(), applied.len());
        println!();
        println!("Version tags created in ~/.config/pais (git tags)");
    }

    Ok(())
}

fn show_status() -> Result<()> {
    let (current, target) = migrate::version_info()?;

    println!("{} PAIS Version Status", "📦".blue());
    println!();
    println!("  Current version: v{}", current);
    println!("  Latest version:  v{}", target);
    println!();

    if current < target {
        let pending = migrate::pending_migrations()?;
        println!("{} {} migration(s) pending", "⚠".yellow(), pending.len());
        println!();
        println!("Run `pais upgrade` to apply pending migrations.");
    } else {
        println!("{} Up to date", "✓".green());
    }

    Ok(())
}
