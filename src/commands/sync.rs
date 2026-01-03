//! Sync skills to Claude Code
//!
//! Syncs PAIS skills to ~/.claude/skills/ using symlinks so Claude Code can discover them.

use eyre::{Context, Result};
use std::fs;
use std::path::{Path, PathBuf};

use crate::config::Config;
use crate::skill::loader::{discover_plugin_skills, discover_simple_skills};
use crate::skill::parser::has_skill_md;

/// Run the sync command
pub fn run(dry_run: bool, clean: bool, config: &Config) -> Result<()> {
    let claude_skills_dir = get_claude_skills_dir()?;

    if clean {
        clean_orphaned_symlinks(&claude_skills_dir, dry_run, config)?;
    } else {
        sync_skills(&claude_skills_dir, dry_run, config)?;
    }

    Ok(())
}

/// Get the Claude Code skills directory
fn get_claude_skills_dir() -> Result<PathBuf> {
    let home = dirs::home_dir().ok_or_else(|| eyre::eyre!("Could not determine home directory"))?;
    Ok(home.join(".claude").join("skills"))
}

/// Sync all PAIS skills to Claude Code
fn sync_skills(claude_skills_dir: &Path, dry_run: bool, config: &Config) -> Result<()> {
    let skills_dir = Config::expand_path(&config.paths.skills);
    let plugins_dir = Config::expand_path(&config.paths.plugins);

    // Ensure Claude skills directory exists
    if !dry_run {
        fs::create_dir_all(claude_skills_dir).with_context(|| {
            format!(
                "Failed to create Claude skills directory: {}",
                claude_skills_dir.display()
            )
        })?;
    }

    let mut synced_count = 0;
    let mut skipped_count = 0;

    // Sync simple skills
    if skills_dir.exists() {
        let simple_skills = discover_simple_skills(&skills_dir).context("Failed to discover simple skills")?;

        for skill in simple_skills {
            match sync_skill(&skill.path, &skill.name, claude_skills_dir, dry_run) {
                Ok(true) => synced_count += 1,
                Ok(false) => skipped_count += 1,
                Err(e) => {
                    log::warn!("Failed to sync skill '{}': {}", skill.name, e);
                }
            }
        }
    }

    // Sync plugin skills (only those with SKILL.md)
    if plugins_dir.exists() {
        let plugin_skills = discover_plugin_skills(&plugins_dir).context("Failed to discover plugin skills")?;

        for skill in plugin_skills {
            match sync_skill(&skill.path, &skill.name, claude_skills_dir, dry_run) {
                Ok(true) => synced_count += 1,
                Ok(false) => skipped_count += 1,
                Err(e) => {
                    log::warn!("Failed to sync plugin skill '{}': {}", skill.name, e);
                }
            }
        }
    }

    // Summary
    println!();
    if dry_run {
        println!("Dry run complete:");
        println!("  Would sync: {} skill(s)", synced_count);
        println!("  Already synced: {} skill(s)", skipped_count);
    } else {
        println!("Sync complete:");
        println!("  Synced: {} skill(s)", synced_count);
        println!("  Already synced: {} skill(s)", skipped_count);
        println!();
        println!("Claude Code skills directory: {}", claude_skills_dir.display());
    }

    Ok(())
}

/// Sync a single skill to Claude Code
fn sync_skill(source: &Path, name: &str, claude_skills_dir: &Path, dry_run: bool) -> Result<bool> {
    let target = claude_skills_dir.join(name);

    // Check if already correctly linked
    if target.exists() || target.symlink_metadata().is_ok() {
        if fs::read_link(&target)
            .map(|link_target| link_target == source)
            .unwrap_or(false)
        {
            // Already correctly linked
            return Ok(false);
        }

        // Remove existing (wrong link or regular file/dir)
        if dry_run {
            println!("Would remove existing: {}", target.display());
        } else if target.is_dir()
            && !target
                .symlink_metadata()
                .map(|m| m.file_type().is_symlink())
                .unwrap_or(false)
        {
            fs::remove_dir_all(&target)
                .with_context(|| format!("Failed to remove existing directory: {}", target.display()))?;
        } else {
            fs::remove_file(&target)
                .with_context(|| format!("Failed to remove existing file/link: {}", target.display()))?;
        }
    }

    // Create symlink
    if dry_run {
        println!("Would link: {} -> {}", name, source.display());
    } else {
        #[cfg(unix)]
        {
            std::os::unix::fs::symlink(source, &target)
                .with_context(|| format!("Failed to create symlink: {} -> {}", target.display(), source.display()))?;
        }

        #[cfg(not(unix))]
        {
            // On non-Unix, copy the SKILL.md file instead
            let skill_md = source.join("SKILL.md");
            if skill_md.exists() {
                fs::create_dir_all(&target)?;
                fs::copy(&skill_md, target.join("SKILL.md"))?;
            }
        }

        println!("Linked: {} -> {}", name, source.display());
    }

    Ok(true)
}

/// Remove orphaned symlinks from Claude skills directory
fn clean_orphaned_symlinks(claude_skills_dir: &Path, dry_run: bool, config: &Config) -> Result<()> {
    if !claude_skills_dir.exists() {
        println!(
            "Claude skills directory does not exist: {}",
            claude_skills_dir.display()
        );
        return Ok(());
    }

    let skills_dir = Config::expand_path(&config.paths.skills);
    let plugins_dir = Config::expand_path(&config.paths.plugins);

    let mut removed_count = 0;
    let mut valid_count = 0;

    for entry in fs::read_dir(claude_skills_dir).context("Failed to read Claude skills directory")? {
        let entry = entry?;
        let path = entry.path();
        let name = entry.file_name().to_string_lossy().to_string();

        // Check if this is a symlink
        let metadata = match path.symlink_metadata() {
            Ok(m) => m,
            Err(_) => continue,
        };

        if !metadata.file_type().is_symlink() {
            // Not a symlink, skip
            valid_count += 1;
            continue;
        }

        // Check if the symlink target exists and is a PAIS skill
        let link_target = match fs::read_link(&path) {
            Ok(t) => t,
            Err(_) => {
                // Can't read link, consider orphaned
                if dry_run {
                    println!("Would remove broken symlink: {}", name);
                } else {
                    fs::remove_file(&path)?;
                    println!("Removed broken symlink: {}", name);
                }
                removed_count += 1;
                continue;
            }
        };

        // Check if target exists
        if !link_target.exists() {
            if dry_run {
                println!("Would remove orphaned symlink: {} -> {}", name, link_target.display());
            } else {
                fs::remove_file(&path)?;
                println!("Removed orphaned symlink: {} -> {}", name, link_target.display());
            }
            removed_count += 1;
            continue;
        }

        // Check if target is within our skills or plugins directories
        let is_pais_skill = link_target.starts_with(&skills_dir) || link_target.starts_with(&plugins_dir);

        if is_pais_skill && has_skill_md(&link_target) {
            valid_count += 1;
        } else {
            // Not a PAIS-managed skill, but target exists - leave it alone
            valid_count += 1;
        }
    }

    println!();
    if dry_run {
        println!("Dry run complete:");
        println!("  Would remove: {} orphaned symlink(s)", removed_count);
        println!("  Valid: {} symlink(s)", valid_count);
    } else {
        println!("Clean complete:");
        println!("  Removed: {} orphaned symlink(s)", removed_count);
        println!("  Valid: {} symlink(s)", valid_count);
    }

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs;
    use tempfile::TempDir;

    fn create_skill(dir: &Path, name: &str) {
        let skill_dir = dir.join(name);
        fs::create_dir_all(&skill_dir).unwrap();
        fs::write(
            skill_dir.join("SKILL.md"),
            format!("---\nname: {}\ndescription: Test skill\n---\n# {}\n", name, name),
        )
        .unwrap();
    }

    #[test]
    fn test_get_claude_skills_dir() {
        let dir = get_claude_skills_dir().unwrap();
        assert!(dir.ends_with(".claude/skills"));
    }

    #[test]
    #[cfg(unix)]
    fn test_sync_skill_creates_symlink() {
        let source_temp = TempDir::new().unwrap();
        let target_temp = TempDir::new().unwrap();

        create_skill(source_temp.path(), "test-skill");

        let source = source_temp.path().join("test-skill");
        let result = sync_skill(&source, "test-skill", target_temp.path(), false).unwrap();

        assert!(result);

        let target = target_temp.path().join("test-skill");
        assert!(target.exists());
        assert!(target.symlink_metadata().unwrap().file_type().is_symlink());

        let link_target = fs::read_link(&target).unwrap();
        assert_eq!(link_target, source);
    }

    #[test]
    #[cfg(unix)]
    fn test_sync_skill_skips_existing_correct_link() {
        let source_temp = TempDir::new().unwrap();
        let target_temp = TempDir::new().unwrap();

        create_skill(source_temp.path(), "test-skill");

        let source = source_temp.path().join("test-skill");
        let target = target_temp.path().join("test-skill");

        // Create initial link
        std::os::unix::fs::symlink(&source, &target).unwrap();

        // Try to sync again
        let result = sync_skill(&source, "test-skill", target_temp.path(), false).unwrap();

        // Should skip (return false)
        assert!(!result);
    }

    #[test]
    fn test_sync_skill_dry_run() {
        let source_temp = TempDir::new().unwrap();
        let target_temp = TempDir::new().unwrap();

        create_skill(source_temp.path(), "test-skill");

        let source = source_temp.path().join("test-skill");
        let result = sync_skill(&source, "test-skill", target_temp.path(), true).unwrap();

        // Dry run should return true (would sync)
        assert!(result);

        // But no symlink should exist
        let target = target_temp.path().join("test-skill");
        assert!(!target.exists());
    }
}
