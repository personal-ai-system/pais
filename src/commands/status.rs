use colored::*;
use eyre::Result;
use serde::Serialize;

use crate::cli::OutputFormat;
use crate::config::Config;

#[derive(Serialize)]
struct Status {
    version: String,
    config_path: Option<String>,
    plugins_dir: String,
    history_dir: String,
    plugins_count: usize,
    registries_count: usize,
}

pub fn run(format: OutputFormat, config: &Config) -> Result<()> {
    let plugins_dir = Config::expand_path(&config.paths.plugins);
    let history_dir = Config::expand_path(&config.paths.history);

    // Count plugins (follow symlinks)
    let plugins_count = if plugins_dir.exists() {
        std::fs::read_dir(&plugins_dir)?
            .filter_map(|e| e.ok())
            .filter(|e| e.path().is_dir()) // is_dir() follows symlinks
            .filter(|e| e.path().join("plugin.yaml").exists())
            .count()
    } else {
        0
    };

    match format {
        OutputFormat::Json | OutputFormat::Yaml => {
            let status = Status {
                version: env!("CARGO_PKG_VERSION").to_string(),
                config_path: None, // TODO: Track where config was loaded from
                plugins_dir: plugins_dir.display().to_string(),
                history_dir: history_dir.display().to_string(),
                plugins_count,
                registries_count: config.registries.len(),
            };
            match format {
                OutputFormat::Json => println!("{}", serde_json::to_string_pretty(&status)?),
                OutputFormat::Yaml => println!("{}", serde_yaml::to_string(&status)?),
                _ => unreachable!(),
            }
        }
        OutputFormat::Text => {
            println!("{}", "PAIS Status".bold());
            println!();

            println!("  {:12} {}", "Version:".dimmed(), env!("CARGO_PKG_VERSION"));
            println!("  {:12} {}", "Plugins:".dimmed(), plugins_dir.display());
            println!("  {:12} {}", "History:".dimmed(), history_dir.display());
            println!();

            // Plugins
            println!("{} ({} installed):", "Plugins".cyan(), plugins_count);
            if plugins_count == 0 {
                println!("  {}", "(none)".dimmed());
            } else {
                for entry in std::fs::read_dir(&plugins_dir)? {
                    let entry = entry?;
                    let path = entry.path();
                    if path.is_dir() {
                        // is_dir() follows symlinks
                        let manifest = path.join("plugin.yaml");
                        if manifest.exists() {
                            let name = entry.file_name();
                            println!("  {} {}", "✓".green(), name.to_string_lossy());
                        }
                    }
                }
            }
            println!();

            // Registries
            println!("{} ({} configured):", "Registries".cyan(), config.registries.len());
            for (name, url) in &config.registries {
                println!("  {} {} → {}", "✓".green(), name, url.dimmed());
            }
            println!();

            // History
            println!("{}:", "History".cyan());
            if history_dir.exists() {
                for entry in std::fs::read_dir(&history_dir)? {
                    let entry = entry?;
                    if entry.file_type()?.is_dir() {
                        let name = entry.file_name();
                        let count = std::fs::read_dir(entry.path())?.filter_map(|e| e.ok()).count();
                        println!("  {:15} {} entries", name.to_string_lossy(), count);
                    }
                }
            } else {
                println!("  {}", "(no history yet)".dimmed());
            }
        }
    }

    Ok(())
}
