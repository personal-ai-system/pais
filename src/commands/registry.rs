use colored::*;
use eyre::{Context, Result};
use serde::{Deserialize, Serialize};
use std::fs;

use crate::cli::{OutputFormat, RegistryAction};
use crate::config::Config;

/// Parsed registry file
#[derive(Debug, Deserialize)]
struct RegistryFile {
    #[allow(dead_code)] // Deserialized but not directly accessed
    registry: RegistryMeta,
    #[serde(default)]
    plugins: Vec<PluginEntry>,
}

#[derive(Debug, Deserialize)]
struct RegistryMeta {
    #[allow(dead_code)] // Required for YAML structure but not accessed
    name: String,
    #[allow(dead_code)]
    #[serde(default)]
    description: Option<String>,
    #[allow(dead_code)]
    #[serde(default)]
    version: Option<String>,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
struct PluginEntry {
    name: String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    version: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    description: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    source: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    path: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    language: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    r#type: Option<String>,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    tags: Vec<String>,
}

pub fn run(action: RegistryAction, config: &Config) -> Result<()> {
    match action {
        RegistryAction::List => list(config),
        RegistryAction::Add { name, url } => add(&name, &url, config),
        RegistryAction::Remove { name } => remove(&name, config),
        RegistryAction::Update { name } => update(name.as_deref(), config),
        RegistryAction::Search { query, format } => search(&query, OutputFormat::resolve(format), config),
        RegistryAction::Show { name, format } => show(&name, OutputFormat::resolve(format), config),
    }
}

fn list(config: &Config) -> Result<()> {
    println!("{}", "Configured registries:".bold());
    println!();

    if config.registries.is_empty() {
        println!("  {}", "(none)".dimmed());
        return Ok(());
    }

    let registries_dir = Config::expand_path(&config.paths.registries);

    for (name, url) in &config.registries {
        // Check if we have a cached listing
        let cache_file = registries_dir.join(format!("{}.toml", name));
        let status = if cache_file.exists() {
            "✓".green().to_string()
        } else {
            "○".dimmed().to_string()
        };
        println!("  {} {}: {}", status, name.cyan(), url.dimmed());
    }

    Ok(())
}

fn add(name: &str, url: &str, config: &Config) -> Result<()> {
    println!("{} Adding registry: {} → {}", "→".blue(), name.cyan(), url.dimmed());

    // Check if already exists
    if config.registries.contains_key(name) {
        eyre::bail!("Registry '{}' already exists. Remove it first.", name);
    }

    // Update config
    let mut new_config = config.clone();
    new_config.registries.insert(name.to_string(), url.to_string());

    // Save config
    let config_path = Config::pais_dir().join("pais.yaml");
    if let Some(parent) = config_path.parent() {
        fs::create_dir_all(parent)?;
    }

    let yaml_str = serde_yaml::to_string(&new_config).context("Failed to serialize config")?;
    fs::write(&config_path, yaml_str).context("Failed to write config file")?;

    println!("  {} Added registry: {}", "✓".green(), name);

    Ok(())
}

fn remove(name: &str, config: &Config) -> Result<()> {
    println!("{} Removing registry: {}", "→".blue(), name.cyan());

    // Check if exists
    if !config.registries.contains_key(name) {
        eyre::bail!("Registry '{}' not found", name);
    }

    // Update config
    let mut new_config = config.clone();
    new_config.registries.remove(name);

    // Save config
    let config_path = Config::pais_dir().join("pais.yaml");
    let yaml_str = serde_yaml::to_string(&new_config).context("Failed to serialize config")?;
    fs::write(&config_path, yaml_str).context("Failed to write config file")?;

    // Remove cached listing if exists
    let registries_dir = Config::expand_path(&config.paths.registries);
    let cache_file = registries_dir.join(format!("{}.toml", name));
    if cache_file.exists() {
        fs::remove_file(&cache_file).ok();
    }

    println!("  {} Removed registry: {}", "✓".green(), name);

    Ok(())
}

fn update(name: Option<&str>, config: &Config) -> Result<()> {
    let registries_dir = Config::expand_path(&config.paths.registries);
    fs::create_dir_all(&registries_dir).context("Failed to create registries directory")?;

    let registries_to_update: Vec<(String, String)> = match name {
        Some(n) => {
            if let Some(url) = config.registries.get(n) {
                vec![(n.to_string(), url.clone())]
            } else {
                eyre::bail!("Registry '{}' not found", n);
            }
        }
        None => config.registries.iter().map(|(k, v)| (k.clone(), v.clone())).collect(),
    };

    if registries_to_update.is_empty() {
        println!("  {}", "No registries configured".dimmed());
        return Ok(());
    }

    for (reg_name, url) in &registries_to_update {
        println!("{} Updating registry: {}", "→".blue(), reg_name.cyan());

        // For now, we can't actually fetch URLs without an HTTP client
        // Just create a placeholder or check if it's a local file
        if url.starts_with("file://") || url.starts_with('/') || url.starts_with('.') {
            // Local file registry
            let path = url.trim_start_matches("file://");
            if std::path::Path::new(path).exists() {
                let content = fs::read_to_string(path).context("Failed to read local registry")?;
                let cache_file = registries_dir.join(format!("{}.toml", reg_name));
                fs::write(&cache_file, content).context("Failed to cache registry")?;
                println!("  {} Cached from local file", "✓".green());
            } else {
                println!("  {} Local file not found: {}", "✗".red(), path);
            }
        } else if url.starts_with("http://") || url.starts_with("https://") {
            // Remote URL - fetch with ureq
            match fetch_remote_registry(url) {
                Ok(content) => {
                    let cache_file = registries_dir.join(format!("{}.toml", reg_name));
                    fs::write(&cache_file, &content).context("Failed to cache registry")?;
                    println!("  {} Fetched and cached", "✓".green());
                }
                Err(e) => {
                    println!("  {} Failed to fetch: {}", "✗".red(), e);
                }
            }
        } else {
            println!("  {} Unknown URL scheme: {}", "✗".red(), url);
        }
    }

    Ok(())
}

/// Fetch a registry from a remote URL
fn fetch_remote_registry(url: &str) -> Result<String> {
    log::info!("Fetching registry from: {}", url);

    let response = ureq::get(url)
        .header("User-Agent", "pais/0.1.0")
        .call()
        .context("HTTP request failed")?;

    if response.status() != 200 {
        eyre::bail!("HTTP {} error", response.status());
    }

    let body = response
        .into_body()
        .read_to_string()
        .context("Failed to read response body")?;

    Ok(body)
}

/// Search result containing plugin and its source registry
#[derive(Debug)]
struct SearchResult {
    registry: String,
    plugin: PluginEntry,
}

fn search(query: &str, format: OutputFormat, config: &Config) -> Result<()> {
    let registries_dir = Config::expand_path(&config.paths.registries);
    let query_lower = query.to_lowercase();

    let mut results: Vec<SearchResult> = Vec::new();

    // Iterate through all cached registry files
    if registries_dir.exists() {
        for entry in fs::read_dir(&registries_dir)? {
            let entry = entry?;
            let path = entry.path();

            if path.extension().is_some_and(|e| e == "toml") {
                let registry_name = path
                    .file_stem()
                    .and_then(|s| s.to_str())
                    .unwrap_or("unknown")
                    .to_string();

                let content = fs::read_to_string(&path).context("Failed to read registry file")?;
                let registry: RegistryFile = serde_yaml::from_str(&content).context("Failed to parse registry file")?;

                // Search plugins
                for plugin in registry.plugins {
                    let matches = plugin.name.to_lowercase().contains(&query_lower)
                        || plugin
                            .description
                            .as_ref()
                            .is_some_and(|d| d.to_lowercase().contains(&query_lower))
                        || plugin.tags.iter().any(|t| t.to_lowercase().contains(&query_lower));

                    if matches {
                        results.push(SearchResult {
                            registry: registry_name.clone(),
                            plugin,
                        });
                    }
                }
            }
        }
    }

    match format {
        OutputFormat::Json => {
            let json_results: Vec<serde_json::Value> = results
                .iter()
                .map(|r| {
                    serde_json::json!({
                        "registry": r.registry,
                        "name": r.plugin.name,
                        "version": r.plugin.version,
                        "description": r.plugin.description,
                        "language": r.plugin.language,
                        "type": r.plugin.r#type,
                        "tags": r.plugin.tags,
                        "source": r.plugin.source,
                        "path": r.plugin.path,
                    })
                })
                .collect();
            println!("{}", serde_json::to_string_pretty(&json_results)?);
        }
        OutputFormat::Yaml => {
            let plugins: Vec<&PluginEntry> = results.iter().map(|r| &r.plugin).collect();
            println!("{}", serde_yaml::to_string(&plugins)?);
        }
        OutputFormat::Text => {
            if results.is_empty() {
                println!("{}", "No plugins found matching query.".dimmed());
                println!();
                println!(
                    "Try running {} to update registries first.",
                    "pais registry update".cyan()
                );
                return Ok(());
            }

            println!("{} {} plugin(s) found:\n", "→".blue(), results.len());

            for result in &results {
                let desc = result.plugin.description.as_deref().unwrap_or("No description");
                let version = result.plugin.version.as_deref().unwrap_or("?");
                let lang = result.plugin.language.as_deref().unwrap_or("?");

                println!(
                    "  {} {} ({})",
                    result.plugin.name.cyan().bold(),
                    format!("v{}", version).dimmed(),
                    lang.yellow()
                );
                println!("    {}", desc.dimmed());
                if !result.plugin.tags.is_empty() {
                    println!("    Tags: {}", result.plugin.tags.join(", ").blue());
                }
                println!("    Registry: {}", result.registry.dimmed());
                println!();
            }
        }
    }

    Ok(())
}

fn show(name: &str, format: OutputFormat, config: &Config) -> Result<()> {
    let registries_dir = Config::expand_path(&config.paths.registries);
    let cache_file = registries_dir.join(format!("{}.toml", name));

    if !cache_file.exists() {
        // Check if registry is configured but not cached
        if config.registries.contains_key(name) {
            eyre::bail!(
                "Registry '{}' is configured but not cached.\nRun 'pais registry update {}' first.",
                name,
                name
            );
        } else {
            eyre::bail!("Registry '{}' not found", name);
        }
    }

    let content = fs::read_to_string(&cache_file).context("Failed to read registry file")?;
    let registry: RegistryFile = serde_yaml::from_str(&content).context("Failed to parse registry file")?;

    match format {
        OutputFormat::Json => {
            println!("{}", serde_json::to_string_pretty(&registry.plugins)?);
        }
        OutputFormat::Yaml => {
            println!("{}", serde_yaml::to_string(&registry.plugins)?);
        }
        OutputFormat::Text => {
            println!(
                "{} {} ({} plugins)\n",
                "Registry:".bold(),
                name.cyan(),
                registry.plugins.len()
            );

            if registry.plugins.is_empty() {
                println!("  {}", "(no plugins)".dimmed());
            } else {
                for plugin in &registry.plugins {
                    let version = plugin.version.as_deref().unwrap_or("?");
                    let lang = plugin.language.as_deref().unwrap_or("?");
                    let desc = plugin.description.as_deref().unwrap_or("No description");

                    println!(
                        "  {} {} ({})",
                        plugin.name.cyan().bold(),
                        format!("v{}", version).dimmed(),
                        lang.yellow()
                    );
                    println!("    {}", desc.dimmed());
                    if !plugin.tags.is_empty() {
                        println!("    Tags: {}", plugin.tags.join(", ").blue());
                    }
                    println!();
                }
            }
        }
    }

    Ok(())
}
