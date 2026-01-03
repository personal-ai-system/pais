//! Initialize PAIS configuration

use colored::*;
use eyre::{Context, Result};
use std::fs;
use std::path::{Path, PathBuf};
use std::process::Command;

use crate::config::Config;

/// Default .gitignore content for PAIS configuration directory
const DEFAULT_GITIGNORE: &str = r#"# Secrets
.env
*.secret

# Installed plugins (from registries)
plugins/

# Runtime data
history/
registries/

# Cache
*.cache
"#;

/// Set up Claude Code integration by ensuring ~/.claude/skills/ exists
fn setup_claude_hooks(pais_dir: &Path) -> Result<()> {
    let claude_skills = dirs::home_dir()
        .ok_or_else(|| eyre::eyre!("Could not determine home directory"))?
        .join(".claude")
        .join("skills");

    if !claude_skills.exists() {
        fs::create_dir_all(&claude_skills).context("Failed to create ~/.claude/skills")?;
        println!("  {} Created ~/.claude/skills/", "✓".green());
    }

    // Create a README in Claude skills directory pointing to PAIS
    let readme_path = claude_skills.join("README.md");
    if !readme_path.exists() {
        let readme_content = format!(
            r#"# Claude Code Skills

This directory contains skills synced from PAIS.

**Source:** {}
**Command:** `pais sync`

Do not edit files here directly - they are symlinks managed by PAIS.
Edit the source files instead and run `pais sync` to update.
"#,
            pais_dir.display()
        );
        fs::write(&readme_path, readme_content).context("Failed to write README")?;
    }

    Ok(())
}

/// Initialize a git repository in the PAIS directory
fn init_git_repo(pais_dir: &PathBuf) -> Result<bool> {
    let git_dir = pais_dir.join(".git");

    if git_dir.exists() {
        println!("  {} Git repository already exists", "✓".green());
        return Ok(false);
    }

    // Check if git is available
    let git_check = Command::new("git").arg("--version").output();
    if git_check.is_err() {
        println!("  {} Git not found, skipping repository initialization", "⚠".yellow());
        return Ok(false);
    }

    // Initialize git repo
    let init_result = Command::new("git")
        .args(["init"])
        .current_dir(pais_dir)
        .output()
        .context("Failed to run git init")?;

    if !init_result.status.success() {
        let stderr = String::from_utf8_lossy(&init_result.stderr);
        println!("  {} Failed to initialize git: {}", "✗".red(), stderr);
        return Ok(false);
    }

    println!("  {} Initialized git repository", "✓".green());

    // Stage all files
    let add_result = Command::new("git")
        .args(["add", "-A"])
        .current_dir(pais_dir)
        .output()
        .context("Failed to run git add")?;

    if !add_result.status.success() {
        let stderr = String::from_utf8_lossy(&add_result.stderr);
        println!("  {} Failed to stage files: {}", "⚠".yellow(), stderr);
    }

    // Create initial commit
    let commit_result = Command::new("git")
        .args(["commit", "-m", "Initial PAIS configuration"])
        .current_dir(pais_dir)
        .output()
        .context("Failed to run git commit")?;

    if commit_result.status.success() {
        println!("  {} Created initial commit", "✓".green());
    } else {
        // This might fail if there's nothing to commit, which is fine
        let stderr = String::from_utf8_lossy(&commit_result.stderr);
        if !stderr.contains("nothing to commit") {
            println!("  {} Could not create initial commit: {}", "⚠".yellow(), stderr.trim());
        }
    }

    Ok(true)
}

pub fn run(path: Option<PathBuf>, force: bool, no_git: bool) -> Result<()> {
    let pais_dir = path.unwrap_or_else(Config::pais_dir);

    println!("{} Initializing PAIS in {}", "→".blue(), pais_dir.display());

    // Check if already initialized
    let config_file = pais_dir.join("pais.yaml");
    if config_file.exists() && !force {
        println!("  {} PAIS already initialized at {}", "✓".green(), pais_dir.display());
        println!("  Use {} to reinitialize", "--force".cyan());
        return Ok(());
    }

    // Create directory structure
    let dirs = ["plugins", "skills", "history", "registries"];
    for dir in &dirs {
        let dir_path = pais_dir.join(dir);
        fs::create_dir_all(&dir_path).context(format!("Failed to create {}", dir))?;
        println!("  {} Created {}/", "✓".green(), dir);
    }

    // Create history subdirectories
    let history_dirs = ["sessions", "decisions", "learnings", "errors"];
    for dir in &history_dirs {
        let dir_path = pais_dir.join("history").join(dir);
        fs::create_dir_all(&dir_path).context(format!("Failed to create history/{}", dir))?;
    }
    println!("  {} Created history subdirectories", "✓".green());

    // Generate default config
    let config = Config::default();
    let yaml_str = serde_yaml::to_string(&config).context("Failed to serialize config")?;
    fs::write(&config_file, yaml_str).context("Failed to write pais.yaml")?;
    println!("  {} Created pais.yaml", "✓".green());

    // Create .gitignore
    let gitignore_path = pais_dir.join(".gitignore");
    fs::write(&gitignore_path, DEFAULT_GITIGNORE).context("Failed to write .gitignore")?;
    println!("  {} Created .gitignore", "✓".green());

    // Initialize git repository (unless --no-git)
    if !no_git {
        init_git_repo(&pais_dir)?;
    }

    // Set up Claude Code integration
    setup_claude_hooks(&pais_dir)?;

    println!();
    println!("{} PAIS initialized!", "✓".green().bold());
    println!();
    println!("Next steps:");
    println!("  1. Run {} to verify setup", "pais doctor".cyan());
    println!("  2. Run {} to create a skill", "pais skill add <name>".cyan());
    println!("  3. Run {} to sync to Claude Code", "pais sync".cyan());

    if !no_git {
        println!();
        println!("Git repository:");
        println!("  {} is now a git repo", pais_dir.display().to_string().cyan());
        println!("  Your skills and config are version controlled");
    }

    Ok(())
}
