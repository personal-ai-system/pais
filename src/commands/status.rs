//! System status command
//!
//! Shows comprehensive PAIS system health and configuration.

use chrono::{DateTime, Local};
use colored::*;
use eyre::Result;
use serde::Serialize;
use std::collections::HashMap;
use std::fs;
use std::path::PathBuf;

use crate::agent::loader::AgentLoader;
use crate::cli::OutputFormat;
use crate::config::Config;
use crate::plugin::PluginManager;
use crate::skill::loader::{discover_plugin_skills, discover_simple_skills};

#[derive(Serialize)]
struct Status {
    version: String,
    pais_dir: String,
    plugins_dir: String,
    skills_dir: String,
    agents_dir: String,
    history_dir: String,
    plugins: Vec<PluginStatus>,
    skills: Vec<SkillStatus>,
    agents: Vec<AgentStatus>,
    hooks: HookStatus,
    observability: ObservabilityStatus,
    history: HistoryStatus,
    registries: Vec<RegistryStatus>,
}

#[derive(Serialize)]
struct PluginStatus {
    name: String,
    version: String,
    language: String,
    has_hooks: bool,
}

#[derive(Serialize)]
struct SkillStatus {
    name: String,
    tier: String,
    source: String, // "simple" or "plugin"
}

#[derive(Serialize)]
struct AgentStatus {
    name: String,
    traits: Vec<String>,
    history_category: String,
}

#[derive(Serialize)]
struct HookStatus {
    security_enabled: bool,
    history_enabled: bool,
    ui_enabled: bool,
}

#[derive(Serialize)]
struct ObservabilityStatus {
    enabled: bool,
    sinks: Vec<String>,
}

#[derive(Serialize)]
struct HistoryStatus {
    categories: HashMap<String, CategoryStats>,
    total_entries: usize,
}

#[derive(Serialize)]
struct CategoryStats {
    count: usize,
    latest: Option<String>,
}

#[derive(Serialize)]
struct RegistryStatus {
    name: String,
    url: String,
    cached: bool,
}

pub fn run(format: OutputFormat, config: &Config) -> Result<()> {
    let pais_dir = Config::pais_dir();
    let plugins_dir = Config::expand_path(&config.paths.plugins);
    let skills_dir = Config::expand_path(&config.paths.skills);
    let agents_dir = pais_dir.join("agents"); // Not in PathsConfig, hardcoded to standard location
    let history_dir = Config::expand_path(&config.paths.history);
    let registries_dir = Config::expand_path(&config.paths.registries);

    // Gather plugin info
    let mut plugin_manager = PluginManager::new(plugins_dir.clone());
    let _ = plugin_manager.discover();
    let plugins: Vec<PluginStatus> = plugin_manager
        .plugins
        .values()
        .map(|p| PluginStatus {
            name: p.manifest.plugin.name.clone(),
            version: p.manifest.plugin.version.clone(),
            language: format!("{:?}", p.manifest.plugin.language).to_lowercase(),
            has_hooks: p.manifest.hooks.has_hooks(),
        })
        .collect();

    // Gather skill info
    let mut skills = Vec::new();
    for skill in discover_simple_skills(&skills_dir).unwrap_or_default() {
        if let Ok(meta) = crate::skill::parser::parse_skill_md(&skill.path.join("SKILL.md")) {
            skills.push(SkillStatus {
                name: meta.name,
                tier: meta.tier.to_string(),
                source: "simple".to_string(),
            });
        }
    }
    for skill in discover_plugin_skills(&plugins_dir).unwrap_or_default() {
        if let Ok(meta) = crate::skill::parser::parse_skill_md(&skill.path.join("SKILL.md")) {
            skills.push(SkillStatus {
                name: meta.name,
                tier: meta.tier.to_string(),
                source: "plugin".to_string(),
            });
        }
    }

    // Gather agent info
    let mut agent_loader = AgentLoader::new(agents_dir.clone());
    let agents_list = agent_loader.load_all().unwrap_or_default();
    let agents: Vec<AgentStatus> = agents_list
        .iter()
        .map(|a| AgentStatus {
            name: a.name.clone(),
            traits: a
                .traits
                .iter()
                .map(|t: &crate::agent::traits::Trait| t.to_string())
                .collect(),
            history_category: a.history_category.clone().unwrap_or_else(|| "sessions".to_string()),
        })
        .collect();

    // Hook status
    let hooks = HookStatus {
        security_enabled: config.hooks.security_enabled,
        history_enabled: config.hooks.history_enabled,
        ui_enabled: config.hooks.ui_enabled,
    };

    // Observability status
    let observability = ObservabilityStatus {
        enabled: config.observability.enabled,
        sinks: config
            .observability
            .sinks
            .iter()
            .map(|s| format!("{:?}", s).to_lowercase())
            .collect(),
    };

    // History stats
    let history = gather_history_stats(&history_dir);

    // Registry status
    let registries: Vec<RegistryStatus> = config
        .registries
        .iter()
        .map(|(name, url)| {
            let cache_file = registries_dir.join(format!("{}.yaml", name));
            RegistryStatus {
                name: name.clone(),
                url: url.clone(),
                cached: cache_file.exists(),
            }
        })
        .collect();

    let status = Status {
        version: env!("CARGO_PKG_VERSION").to_string(),
        pais_dir: pais_dir.display().to_string(),
        plugins_dir: plugins_dir.display().to_string(),
        skills_dir: skills_dir.display().to_string(),
        agents_dir: agents_dir.display().to_string(),
        history_dir: history_dir.display().to_string(),
        plugins,
        skills,
        agents,
        hooks,
        observability,
        history,
        registries,
    };

    match format {
        OutputFormat::Json => println!("{}", serde_json::to_string_pretty(&status)?),
        OutputFormat::Yaml => println!("{}", serde_yaml::to_string(&status)?),
        OutputFormat::Text => print_text_status(&status),
    }

    Ok(())
}

fn gather_history_stats(history_dir: &PathBuf) -> HistoryStatus {
    let mut categories = HashMap::new();
    let mut total_entries = 0;

    if history_dir.exists()
        && let Ok(entries) = fs::read_dir(history_dir)
    {
        for entry in entries.filter_map(|e| e.ok()) {
            let path = entry.path();
            if path.is_dir() {
                let name = entry.file_name().to_string_lossy().to_string();

                // Skip raw-events and security (they're JSONL, not markdown)
                if name == "raw-events" || name == "security" {
                    continue;
                }

                let (count, latest) = count_history_entries(&path);
                total_entries += count;
                categories.insert(
                    name,
                    CategoryStats {
                        count,
                        latest: latest.map(|dt| dt.format("%Y-%m-%d %H:%M").to_string()),
                    },
                );
            }
        }
    }

    HistoryStatus {
        categories,
        total_entries,
    }
}

fn count_history_entries(category_dir: &PathBuf) -> (usize, Option<DateTime<Local>>) {
    let mut count = 0;
    let mut latest: Option<DateTime<Local>> = None;

    // Walk through year-month directories
    let Ok(entries) = fs::read_dir(category_dir) else {
        return (count, latest);
    };

    for entry in entries.filter_map(|e| e.ok()) {
        let path = entry.path();
        if path.is_dir() {
            // This is a YYYY-MM directory
            let Ok(files) = fs::read_dir(&path) else {
                continue;
            };
            for file in files.filter_map(|f| f.ok()) {
                let file_path = file.path();
                if file_path.extension().is_some_and(|e| e == "md") {
                    count += 1;

                    // Get modification time for latest
                    if let Ok(metadata) = fs::metadata(&file_path)
                        && let Ok(modified) = metadata.modified()
                    {
                        let dt: DateTime<Local> = modified.into();
                        if latest.is_none() || Some(dt) > latest {
                            latest = Some(dt);
                        }
                    }
                }
            }
        }
    }

    (count, latest)
}

fn print_text_status(status: &Status) {
    println!("{}", "PAIS Status".bold());
    println!();

    // Version and paths
    println!("  {:14} {}", "Version:".dimmed(), status.version);
    println!("  {:14} {}", "PAIS Dir:".dimmed(), status.pais_dir);
    println!();

    // Plugins
    println!(
        "{} ({}):",
        "Plugins".cyan(),
        format!("{} installed", status.plugins.len()).dimmed()
    );
    if status.plugins.is_empty() {
        println!("  {}", "(none)".dimmed());
    } else {
        for plugin in &status.plugins {
            let hooks_badge = if plugin.has_hooks { " [hooks]".yellow().to_string() } else { String::new() };
            println!(
                "  {} {} {} {}{}",
                "✓".green(),
                plugin.name.green(),
                format!("v{}", plugin.version).dimmed(),
                format!("[{}]", plugin.language).dimmed(),
                hooks_badge
            );
        }
    }
    println!();

    // Skills
    println!(
        "{} ({}):",
        "Skills".cyan(),
        format!("{} total", status.skills.len()).dimmed()
    );
    if status.skills.is_empty() {
        println!("  {}", "(none)".dimmed());
    } else {
        let core_skills: Vec<_> = status.skills.iter().filter(|s| s.tier == "core").collect();
        let deferred_skills: Vec<_> = status.skills.iter().filter(|s| s.tier != "core").collect();

        if !core_skills.is_empty() {
            println!("  {} Core tier (always loaded):", "●".yellow());
            for skill in core_skills {
                let source_badge = format!("[{}]", skill.source).dimmed();
                println!("    {} {} {}", "✓".green(), skill.name, source_badge);
            }
        }
        if !deferred_skills.is_empty() {
            println!("  {} Deferred tier:", "○".dimmed());
            for skill in deferred_skills {
                let source_badge = format!("[{}]", skill.source).dimmed();
                println!("    {} {} {}", "✓".green(), skill.name, source_badge);
            }
        }
    }
    println!();

    // Agents
    println!(
        "{} ({}):",
        "Agents".cyan(),
        format!("{} configured", status.agents.len()).dimmed()
    );
    if status.agents.is_empty() {
        println!("  {}", "(none)".dimmed());
    } else {
        for agent in &status.agents {
            let traits_str = if agent.traits.is_empty() {
                "(no traits)".dimmed().to_string()
            } else {
                agent.traits.join(", ").dimmed().to_string()
            };
            println!(
                "  {} {} → {} ({})",
                "✓".green(),
                agent.name.green(),
                agent.history_category.cyan(),
                traits_str
            );
        }
    }
    println!();

    // Hooks
    println!("{}:", "Hooks".cyan());
    print_hook_status("Security", status.hooks.security_enabled);
    print_hook_status("History", status.hooks.history_enabled);
    print_hook_status("UI (Tab Titles)", status.hooks.ui_enabled);
    println!();

    // Observability
    println!("{}:", "Observability".cyan());
    if status.observability.enabled {
        println!(
            "  {} Enabled: {}",
            "✓".green(),
            status.observability.sinks.join(", ").cyan()
        );
    } else {
        println!("  {} Disabled", "○".dimmed());
    }
    println!();

    // History
    println!(
        "{} ({} entries total):",
        "History".cyan(),
        status.history.total_entries.to_string().yellow()
    );
    if status.history.categories.is_empty() {
        println!("  {}", "(no history yet)".dimmed());
    } else {
        let mut categories: Vec<_> = status.history.categories.iter().collect();
        categories.sort_by(|a, b| b.1.count.cmp(&a.1.count));

        for (name, stats) in categories {
            let latest = stats
                .latest
                .as_ref()
                .map(|l| format!("(latest: {})", l).dimmed().to_string())
                .unwrap_or_default();
            println!(
                "  {:15} {:>5} entries {}",
                name,
                stats.count.to_string().yellow(),
                latest
            );
        }
    }
    println!();

    // Registries
    println!(
        "{} ({}):",
        "Registries".cyan(),
        format!("{} configured", status.registries.len()).dimmed()
    );
    if status.registries.is_empty() {
        println!("  {}", "(none)".dimmed());
    } else {
        for registry in &status.registries {
            let cached_badge = if registry.cached {
                "[cached]".green().to_string()
            } else {
                "[not cached]".yellow().to_string()
            };
            println!(
                "  {} {} {} {}",
                "✓".green(),
                registry.name,
                cached_badge,
                registry.url.dimmed()
            );
        }
    }
}

fn print_hook_status(name: &str, enabled: bool) {
    if enabled {
        println!("  {} {}", "✓".green(), name);
    } else {
        println!("  {} {} {}", "○".dimmed(), name, "(disabled)".dimmed());
    }
}
