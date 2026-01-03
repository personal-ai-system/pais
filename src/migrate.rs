//! Config migration system
//!
//! Tracks PAIS config version using git tags and applies migrations when needed.

#![allow(dead_code)] // needs_migration - for future auto-migration on startup

use eyre::{Context, Result};
use std::process::Command;

use crate::config::Config;

/// Current config version
pub const CURRENT_VERSION: u32 = 1;

/// Get current version from git tags
pub fn get_current_version() -> Result<u32> {
    let pais_dir = Config::pais_dir();

    // Check if it's a git repo
    if !pais_dir.join(".git").exists() {
        return Ok(0); // Not a git repo, assume v0
    }

    let output = Command::new("git")
        .args(["-C", &pais_dir.to_string_lossy(), "describe", "--tags", "--abbrev=0"])
        .output()
        .context("Failed to run git describe")?;

    if !output.status.success() {
        // No tags exist yet
        return Ok(0);
    }

    let tag = String::from_utf8_lossy(&output.stdout).trim().to_string();

    // Parse version from tag (e.g., "v1" -> 1)
    parse_version_tag(&tag)
}

/// Parse a version tag like "v1" or "v2" into a number
fn parse_version_tag(tag: &str) -> Result<u32> {
    let version_str = tag.trim_start_matches('v');
    version_str
        .parse()
        .with_context(|| format!("Invalid version tag: {}", tag))
}

/// Set version by creating an annotated git tag
fn set_version(version: u32, message: &str) -> Result<()> {
    let pais_dir = Config::pais_dir();
    let tag = format!("v{}", version);

    // Create annotated tag
    let output = Command::new("git")
        .args(["-C", &pais_dir.to_string_lossy(), "tag", "-a", &tag, "-m", message])
        .output()
        .context("Failed to create git tag")?;

    if !output.status.success() {
        let stderr = String::from_utf8_lossy(&output.stderr);
        eyre::bail!("Failed to create tag {}: {}", tag, stderr);
    }

    Ok(())
}

/// A migration that upgrades from one version to the next
pub trait Migration {
    fn source_version(&self) -> u32;
    fn target_version(&self) -> u32;
    fn description(&self) -> &str;
    fn apply(&self, config: &Config) -> Result<()>;
}

/// Migration from v0 (unversioned) to v1
struct MigrationV0ToV1;

impl Migration for MigrationV0ToV1 {
    fn source_version(&self) -> u32 {
        0
    }

    fn target_version(&self) -> u32 {
        1
    }

    fn description(&self) -> &str {
        "Initial versioning"
    }

    fn apply(&self, _config: &Config) -> Result<()> {
        // v0 -> v1 is just establishing versioning, no config changes needed
        Ok(())
    }
}

/// Get all available migrations
fn get_migrations() -> Vec<Box<dyn Migration>> {
    vec![Box::new(MigrationV0ToV1)]
}

/// Check if migrations are needed
pub fn needs_migration() -> Result<bool> {
    let current = get_current_version()?;
    Ok(current < CURRENT_VERSION)
}

/// Get pending migrations
pub fn pending_migrations() -> Result<Vec<(u32, u32, String)>> {
    let current = get_current_version()?;
    let migrations = get_migrations();

    let pending: Vec<_> = migrations
        .iter()
        .filter(|m| m.source_version() >= current && m.target_version() <= CURRENT_VERSION)
        .map(|m| (m.source_version(), m.target_version(), m.description().to_string()))
        .collect();

    Ok(pending)
}

/// Run all pending migrations
pub fn run_migrations(config: &Config, dry_run: bool) -> Result<Vec<String>> {
    let mut current = get_current_version()?;
    let migrations = get_migrations();

    let mut applied = Vec::new();

    for migration in migrations {
        if migration.source_version() >= current && migration.target_version() <= CURRENT_VERSION {
            let desc = format!(
                "v{} → v{}: {}",
                migration.source_version(),
                migration.target_version(),
                migration.description()
            );

            if dry_run {
                applied.push(format!("[dry-run] {}", desc));
            } else {
                migration.apply(config)?;

                // Create git tag for this version
                set_version(migration.target_version(), migration.description())?;

                current = migration.target_version();
                applied.push(desc);
            }
        }
    }

    Ok(applied)
}

/// Get current version info
pub fn version_info() -> Result<(u32, u32)> {
    let current = get_current_version()?;
    Ok((current, CURRENT_VERSION))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_version_tag() {
        assert_eq!(parse_version_tag("v1").unwrap(), 1);
        assert_eq!(parse_version_tag("v42").unwrap(), 42);
        assert!(parse_version_tag("invalid").is_err());
    }

    #[test]
    fn test_migration_v0_to_v1() {
        let migration = MigrationV0ToV1;
        assert_eq!(migration.source_version(), 0);
        assert_eq!(migration.target_version(), 1);
    }
}
