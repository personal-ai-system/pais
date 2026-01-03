//! Diagnose PAIS setup issues

use colored::*;
use eyre::Result;
use std::fs;
use std::process::Command;

use crate::config::Config;

pub fn run(config: &Config) -> Result<()> {
    println!("{}", "PAIS Doctor".bold());
    println!("{}", "═".repeat(50));
    println!();

    let mut issues = 0;

    // Check PAIS directory
    let pais_dir = Config::pais_dir();
    if pais_dir.exists() {
        println!("{} PAIS directory: {}", "✓".green(), pais_dir.display());
    } else {
        println!("{} PAIS directory missing: {}", "✗".red(), pais_dir.display());
        println!("  Run {} to create it", "pais init".cyan());
        issues += 1;
    }

    // Check config file
    let config_file = pais_dir.join("pais.yaml");
    if config_file.exists() {
        println!("{} Config file: {}", "✓".green(), config_file.display());
    } else {
        println!("{} Config file missing: {}", "✗".red(), config_file.display());
        issues += 1;
    }

    // Check plugins directory
    let plugins_dir = Config::expand_path(&config.paths.plugins);
    if plugins_dir.exists() {
        let count = count_plugins(&plugins_dir);
        println!(
            "{} Plugins directory: {} ({} plugins)",
            "✓".green(),
            plugins_dir.display(),
            count
        );
    } else {
        println!("{} Plugins directory missing: {}", "⚠".yellow(), plugins_dir.display());
    }

    // Check history directory
    let history_dir = Config::expand_path(&config.paths.history);
    if history_dir.exists() {
        println!("{} History directory: {}", "✓".green(), history_dir.display());
    } else {
        println!("{} History directory missing: {}", "⚠".yellow(), history_dir.display());
    }

    // Check registries directory
    let registries_dir = Config::expand_path(&config.paths.registries);
    if registries_dir.exists() {
        let count = count_registries(&registries_dir);
        println!(
            "{} Registries directory: {} ({} cached)",
            "✓".green(),
            registries_dir.display(),
            count
        );
    } else {
        println!(
            "{} Registries directory missing: {}",
            "⚠".yellow(),
            registries_dir.display()
        );
    }

    println!();

    // Check configured registries
    println!("{}", "Registries:".bold());
    if config.registries.is_empty() {
        println!("  {} No registries configured", "⚠".yellow());
    } else {
        for (name, url) in &config.registries {
            let cache_file = registries_dir.join(format!("{}.yaml", name));
            if cache_file.exists() {
                println!("  {} {} (cached)", "✓".green(), name);
            } else {
                println!("  {} {} (not cached)", "⚠".yellow(), name);
                println!("    URL: {}", url.dimmed());
                println!("    Run {} to fetch", "pais registry update".cyan());
            }
        }
    }

    println!();

    // Check dependencies
    println!("{}", "Dependencies:".bold());

    // Check git
    if check_command("git", &["--version"]) {
        println!("  {} git", "✓".green());
    } else {
        println!("  {} git (required for plugin install)", "✗".red());
        issues += 1;
    }

    // Check Python/uv for Python plugins
    if check_command("uv", &["--version"]) {
        println!("  {} uv (Python package manager)", "✓".green());
    } else if check_command("python3", &["--version"]) {
        println!("  {} python3 (uv recommended for faster installs)", "⚠".yellow());
    } else {
        println!("  {} python3/uv (needed for Python plugins)", "⚠".yellow());
    }

    // Check cargo for Rust plugins
    if check_command("cargo", &["--version"]) {
        println!("  {} cargo (Rust build tool)", "✓".green());
    } else {
        println!("  {} cargo (needed for Rust plugins)", "⚠".yellow());
    }

    println!();

    // Check environment tools
    let env = &config.environment;
    let has_tool_config = !env.tool_preferences.is_empty() || !env.tools.is_empty();

    if has_tool_config {
        println!("{}", "Environment Tools:".bold());

        // Check tool preferences (modern replacements)
        if !env.tool_preferences.is_empty() {
            let mut prefs: Vec<_> = env.tool_preferences.iter().collect();
            prefs.sort_by_key(|(k, _)| *k);

            for (legacy, modern) in prefs {
                let binary = modern.split_whitespace().next().unwrap_or(modern);
                if let Some(version) = get_command_version(binary) {
                    println!("  {} {} → {} ({})", "✓".green(), legacy, modern, version.dimmed());
                } else {
                    println!("  {} {} → {} (not found)", "✗".red(), legacy, modern);
                    println!("    Fallback: {} is available", legacy);
                }
            }
        }

        // Check custom tools
        if !env.tools.is_empty() {
            let mut tools: Vec<_> = env.tools.iter().collect();
            tools.sort_by_key(|(k, _)| *k);

            for (name, tool_config) in tools {
                if let Some(version) = get_command_version(name) {
                    let desc = tool_config.description.as_deref().unwrap_or("");
                    println!("  {} {} - {} ({})", "✓".green(), name, desc, version.dimmed());
                } else {
                    let desc = tool_config.description.as_deref().unwrap_or("custom tool");
                    println!("  {} {} - {} (not found)", "✗".red(), name, desc);

                    // Show install hint
                    if let Some(ref github) = tool_config.github {
                        if let Some(ref install) = tool_config.install {
                            println!("    Install: {}", install.cyan());
                        } else {
                            println!(
                                "    Install: {}",
                                format!("cargo install --git https://github.com/{}", github).cyan()
                            );
                        }
                    }
                }
            }
        }

        println!();
    }

    // Check repos-dir
    if let Some(ref repos_dir) = env.repos_dir {
        let expanded = Config::expand_path(repos_dir);
        println!("{}", "Repos Directory:".bold());
        if expanded.exists() {
            let count = count_repos(&expanded);
            println!("  {} {} ({} repos)", "✓".green(), expanded.display(), count);
        } else {
            println!("  {} {} (does not exist)", "✗".red(), expanded.display());
            issues += 1;
        }
        println!();
    }

    // Check hooks configuration
    println!("{}", "Hooks:".bold());
    println!(
        "  Security: {}",
        if config.hooks.security_enabled {
            "enabled".green()
        } else {
            "disabled".yellow()
        }
    );
    println!(
        "  History:  {}",
        if config.hooks.history_enabled {
            "enabled".green()
        } else {
            "disabled".yellow()
        }
    );

    // Check Claude Code hooks file
    let claude_hooks = std::env::current_dir().ok().map(|d| d.join(".claude/settings.json"));
    if let Some(hooks_file) = claude_hooks {
        if hooks_file.exists() {
            println!("  {} Claude Code hooks configured", "✓".green());
        } else {
            println!("  {} Claude Code hooks not configured", "⚠".yellow());
            println!("    Create {} to enable hooks", ".claude/settings.json".cyan());
        }
    }

    println!();

    // Summary
    println!("{}", "═".repeat(50));
    if issues == 0 {
        println!("{} All checks passed!", "✓".green().bold());
    } else {
        println!("{} {} issue(s) found", "⚠".yellow().bold(), issues);
    }

    Ok(())
}

fn count_plugins(dir: &std::path::Path) -> usize {
    fs::read_dir(dir)
        .map(|entries| {
            entries
                .filter_map(|e| e.ok())
                .filter(|e| e.path().join("plugin.yaml").exists())
                .count()
        })
        .unwrap_or(0)
}

fn count_registries(dir: &std::path::Path) -> usize {
    fs::read_dir(dir)
        .map(|entries| {
            entries
                .filter_map(|e| e.ok())
                .filter(|e| e.path().extension().is_some_and(|ext| ext == "yaml"))
                .count()
        })
        .unwrap_or(0)
}

fn check_command(cmd: &str, args: &[&str]) -> bool {
    Command::new(cmd)
        .args(args)
        .stdout(std::process::Stdio::null())
        .stderr(std::process::Stdio::null())
        .status()
        .map(|s| s.success())
        .unwrap_or(false)
}

fn get_command_version(cmd: &str) -> Option<String> {
    // First check if command exists
    let which_output = Command::new("which")
        .arg(cmd)
        .stdout(std::process::Stdio::null())
        .stderr(std::process::Stdio::null())
        .status()
        .ok()?;

    if !which_output.success() {
        return None;
    }

    // Try to get version
    let version_output = Command::new(cmd).arg("--version").output().ok()?;

    if version_output.status.success() {
        let version_str = String::from_utf8_lossy(&version_output.stdout);
        // Extract first line, limit to reasonable length
        let version = version_str
            .lines()
            .next()
            .unwrap_or("")
            .chars()
            .take(40)
            .collect::<String>();
        Some(version.trim().to_string())
    } else {
        // Command exists but --version failed, just say it's available
        Some("available".to_string())
    }
}

fn count_repos(dir: &std::path::Path) -> usize {
    // Count directories that look like org/repo structure
    let mut count = 0;
    if let Ok(orgs) = fs::read_dir(dir) {
        for org_entry in orgs.filter_map(|e| e.ok()) {
            if org_entry.path().is_dir()
                && let Ok(repos) = fs::read_dir(org_entry.path())
            {
                count += repos.filter_map(|e| e.ok()).filter(|e| e.path().is_dir()).count();
            }
        }
    }
    count
}
