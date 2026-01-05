//! Bundle management commands

use colored::*;
use eyre::{Context, Result};
use serde::Serialize;
use std::fs;
use std::path::PathBuf;

use crate::bundle::manager::BundleManager;
use crate::cli::{BundleAction, OutputFormat};
use crate::config::Config;

pub fn run(action: BundleAction, config: &Config) -> Result<()> {
    match action {
        BundleAction::List { format } => list(OutputFormat::resolve(format), config),
        BundleAction::Show { name, format } => show(&name, OutputFormat::resolve(format), config),
        BundleAction::Install {
            name,
            required_only,
            skip_verify,
        } => install(&name, required_only, skip_verify, config),
        BundleAction::New { name, path } => new(&name, path, config),
    }
}

#[derive(Serialize)]
struct BundleInfo {
    name: String,
    version: String,
    description: String,
    plugin_count: usize,
    required_count: usize,
    optional_count: usize,
}

fn list(format: OutputFormat, config: &Config) -> Result<()> {
    let bundles_dir = Config::pais_dir().join("bundles");

    let mut manager = BundleManager::new(bundles_dir.clone(), Config::expand_path(&config.paths.plugins));
    manager.discover()?;

    let bundles: Vec<BundleInfo> = manager
        .list()
        .map(|b| {
            let required = b.manifest.required_plugins().len();
            let optional = b.manifest.optional_plugins().len();
            BundleInfo {
                name: b.manifest.bundle.name.clone(),
                version: b.manifest.bundle.version.clone(),
                description: b.manifest.bundle.description.clone(),
                plugin_count: required + optional,
                required_count: required,
                optional_count: optional,
            }
        })
        .collect();

    match format {
        OutputFormat::Json => {
            println!("{}", serde_json::to_string_pretty(&bundles)?);
        }
        OutputFormat::Yaml => {
            println!("{}", serde_yaml::to_string(&bundles)?);
        }
        OutputFormat::Text => {
            println!("{}", "Available bundles:".bold());
            println!();

            if bundles.is_empty() {
                println!("  {}", "(none)".dimmed());
                println!();
                println!("  Create a bundle:");
                println!("    pais bundle new my-bundle");
                println!();
                println!("  Bundles directory: {}", bundles_dir.display());
            } else {
                for bundle in &bundles {
                    println!(
                        "  {} {} - {} ({} plugins)",
                        bundle.name.green(),
                        format!("v{}", bundle.version).dimmed(),
                        bundle.description,
                        bundle.plugin_count
                    );
                }
            }
        }
    }

    Ok(())
}

#[derive(Serialize)]
struct BundleDetail {
    name: String,
    version: String,
    description: String,
    author: Option<String>,
    license: Option<String>,
    pais_version: Option<String>,
    plugins: Vec<PluginDetail>,
    environment: Vec<EnvVar>,
    post_install: Vec<String>,
    conflicts: Vec<String>,
}

#[derive(Serialize)]
struct PluginDetail {
    name: String,
    required: bool,
    description: Option<String>,
}

#[derive(Serialize)]
struct EnvVar {
    name: String,
    value: String,
}

fn show(name: &str, format: OutputFormat, config: &Config) -> Result<()> {
    let bundles_dir = Config::pais_dir().join("bundles");

    let mut manager = BundleManager::new(bundles_dir, Config::expand_path(&config.paths.plugins));
    manager.discover()?;

    let bundle = manager
        .get(name)
        .ok_or_else(|| eyre::eyre!("Bundle not found: {}", name))?;

    let detail = BundleDetail {
        name: bundle.manifest.bundle.name.clone(),
        version: bundle.manifest.bundle.version.clone(),
        description: bundle.manifest.bundle.description.clone(),
        author: bundle.manifest.bundle.author.clone(),
        license: bundle.manifest.bundle.license.clone(),
        pais_version: bundle.manifest.bundle.pais_version.clone(),
        plugins: bundle
            .manifest
            .all_plugins()
            .iter()
            .map(|(name, p)| PluginDetail {
                name: (*name).clone(),
                required: p.required,
                description: p.description.clone(),
            })
            .collect(),
        environment: bundle
            .manifest
            .environment
            .iter()
            .map(|(k, v): (&String, &String)| EnvVar {
                name: k.clone(),
                value: v.clone(),
            })
            .collect(),
        post_install: bundle.manifest.post_install.iter().map(|c| c.command.clone()).collect(),
        conflicts: bundle.manifest.conflicts.clone(),
    };

    match format {
        OutputFormat::Json => {
            println!("{}", serde_json::to_string_pretty(&detail)?);
        }
        OutputFormat::Yaml => {
            println!("{}", serde_yaml::to_string(&detail)?);
        }
        OutputFormat::Text => {
            println!("{} {}", detail.name.bold(), format!("v{}", detail.version).dimmed());
            println!("{}", detail.description);
            println!();

            if let Some(ref author) = detail.author {
                println!("  {} {}", "Author:".dimmed(), author);
            }
            if let Some(ref license) = detail.license {
                println!("  {} {}", "License:".dimmed(), license);
            }
            if let Some(ref pais_ver) = detail.pais_version {
                println!("  {} {}", "PAIS version:".dimmed(), pais_ver);
            }
            println!();

            println!("{} ({}):", "Plugins".cyan(), detail.plugins.len());
            for plugin in &detail.plugins {
                let req_badge = if plugin.required { "[required]".green() } else { "[optional]".yellow() };
                let desc = plugin
                    .description
                    .as_ref()
                    .map(|d| format!(" - {}", d))
                    .unwrap_or_default();
                println!("  {} {}{}", req_badge, plugin.name, desc.dimmed());
            }

            if !detail.environment.is_empty() {
                println!();
                println!("{}:", "Environment".cyan());
                for env in &detail.environment {
                    println!("  {}={}", env.name, env.value);
                }
            }

            if !detail.post_install.is_empty() {
                println!();
                println!("{}:", "Post-install".cyan());
                for cmd in &detail.post_install {
                    println!("  {}", cmd.dimmed());
                }
            }

            if !detail.conflicts.is_empty() {
                println!();
                println!("{}: {}", "Conflicts with".red(), detail.conflicts.join(", "));
            }
        }
    }

    Ok(())
}

fn install(name: &str, required_only: bool, skip_verify: bool, config: &Config) -> Result<()> {
    let bundles_dir = Config::pais_dir().join("bundles");

    let manager = BundleManager::new(bundles_dir, Config::expand_path(&config.paths.plugins));

    // Need to re-discover since we moved manager
    let mut manager = manager;
    manager.discover()?;

    println!(
        "{} Installing bundle: {}{}",
        "→".blue(),
        name.cyan(),
        if required_only {
            " (required only)".dimmed().to_string()
        } else {
            String::new()
        }
    );

    let result = manager.install(name, required_only, skip_verify)?;
    result.print_summary();

    Ok(())
}

fn new(name: &str, path: Option<PathBuf>, _config: &Config) -> Result<()> {
    let bundles_dir = Config::pais_dir().join("bundles");
    let output_path = path.unwrap_or_else(|| bundles_dir.join(name));

    println!("{} Creating new bundle: {}", "→".blue(), name.cyan());
    println!("  Output: {}", output_path.display());

    if output_path.exists() {
        eyre::bail!("Directory already exists: {}", output_path.display());
    }

    fs::create_dir_all(&output_path).context("Failed to create bundle directory")?;

    // Generate bundle.yaml
    let manifest_content = generate_bundle_manifest(name);
    fs::write(output_path.join("bundle.yaml"), manifest_content).context("Failed to write bundle.yaml")?;

    println!("  {} Created bundle scaffold", "✓".green());
    println!();
    println!("  Next steps:");
    println!("    1. Edit {}/bundle.yaml", output_path.display());
    println!("    2. Add plugins to the 'plugins:' section");
    println!("    3. pais bundle install {}", name);

    Ok(())
}

fn generate_bundle_manifest(name: &str) -> String {
    format!(
        r#"bundle:
  name: {name}
  version: 1.0.0
  description: A collection of related plugins
  author: Your Name
  license: MIT
  pais-version: ">=0.2.0"

# Plugins in this bundle (installed in order)
plugins:
  # example-plugin:
  #   required: true
  #   description: Why this plugin is included

# Environment variables to set
environment: {{}}

# Commands to run after all plugins installed
post-install: []
  # - command: pais skill index --rebuild
  #   description: Rebuild skill index

# Bundles that conflict with this one
conflicts: []
"#,
        name = name
    )
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_generate_bundle_manifest() {
        let manifest = generate_bundle_manifest("test-bundle");
        assert!(manifest.contains("name: test-bundle"));
        assert!(manifest.contains("version: 1.0.0"));
        assert!(manifest.contains("plugins:"));
    }
}
