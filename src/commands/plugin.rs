use colored::*;
use eyre::{Context, Result};
use serde::Deserialize;
use std::fs;
use std::path::Path;
use std::process::Command;

use crate::cli::PluginAction;
use crate::config::Config;
use crate::plugin::loader::load_plugin;

/// Registry plugin entry for lookup
#[allow(dead_code)]
#[derive(Debug, Clone, Deserialize)]
struct RegistryPlugin {
    name: String,
    #[serde(default)]
    version: Option<String>,
    #[serde(default)]
    description: Option<String>,
    #[serde(default)]
    source: Option<String>,
    #[serde(default)]
    path: Option<String>,
}

/// Parsed registry file
#[derive(Debug, Deserialize)]
struct RegistryFile {
    #[allow(dead_code)]
    registry: RegistryMeta,
    #[serde(default)]
    plugins: Vec<RegistryPlugin>,
}

#[allow(dead_code)]
#[derive(Debug, Deserialize)]
struct RegistryMeta {
    name: String,
}

pub fn run(action: PluginAction, config: &Config) -> Result<()> {
    match action {
        PluginAction::List { json } => list(json, config),
        PluginAction::Install { source, dev, force } => install(&source, dev, force, config),
        PluginAction::Remove { name, force } => remove(&name, force, config),
        PluginAction::Update { name } => update(&name, config),
        PluginAction::Info { name } => info(&name, config),
        PluginAction::New {
            name,
            language,
            r#type,
            path,
        } => new(&name, &language, &r#type, path.as_ref(), config),
        PluginAction::Verify { name } => verify(&name, config),
    }
}

fn list(json: bool, config: &Config) -> Result<()> {
    let plugins_dir = Config::expand_path(&config.paths.plugins);

    if !plugins_dir.exists() {
        if json {
            println!("[]");
        } else {
            println!("{}", "Installed plugins:".bold());
            println!();
            println!("  {}", "(none)".dimmed());
        }
        return Ok(());
    }

    let mut plugins = Vec::new();

    // Scan for plugin directories
    for entry in fs::read_dir(&plugins_dir).context("Failed to read plugins directory")? {
        let entry = entry?;
        let path = entry.path();

        if path.is_dir() {
            let manifest_path = path.join("plugin.toml");
            if manifest_path.exists() {
                match load_plugin(&path) {
                    Ok(plugin) => plugins.push(plugin),
                    Err(e) => {
                        log::warn!("Failed to load plugin at {}: {}", path.display(), e);
                    }
                }
            }
        }
    }

    if json {
        let output: Vec<serde_json::Value> = plugins
            .iter()
            .map(|p| {
                serde_json::json!({
                    "name": p.manifest.plugin.name,
                    "version": p.manifest.plugin.version,
                    "description": p.manifest.plugin.description,
                    "language": format!("{:?}", p.manifest.plugin.language),
                    "path": p.path.display().to_string(),
                })
            })
            .collect();
        println!("{}", serde_json::to_string_pretty(&output)?);
    } else {
        println!("{}", "Installed plugins:".bold());
        println!();

        if plugins.is_empty() {
            println!("  {}", "(none)".dimmed());
        } else {
            for plugin in &plugins {
                println!(
                    "  {} {} {}",
                    plugin.manifest.plugin.name.green(),
                    format!("v{}", plugin.manifest.plugin.version).dimmed(),
                    format!("- {}", plugin.manifest.plugin.description).dimmed(),
                );
            }
        }
    }

    Ok(())
}

fn install(source: &str, dev: bool, force: bool, config: &Config) -> Result<()> {
    println!(
        "{} Installing plugin: {} {}{}",
        "→".blue(),
        source.cyan(),
        if dev { "(dev mode) ".dimmed().to_string() } else { String::new() },
        if force { "(force) ".dimmed().to_string() } else { String::new() },
    );

    let source_path = Path::new(source);

    // Check if source is a local path or a plugin name from registry
    if source_path.exists() {
        // Local path installation
        install_from_path(source_path, dev, force, config)
    } else if !source.contains('/') && !source.contains('\\') {
        // Looks like a plugin name - search registry
        install_from_registry(source, force, config)
    } else {
        eyre::bail!(
            "Source not found: {}. Use a local path or a plugin name from registry.\n\
             Hint: Run 'paii registry search {}' to find available plugins.",
            source,
            source
        );
    }
}

/// Install a plugin from a local path
fn install_from_path(source_path: &Path, dev: bool, force: bool, config: &Config) -> Result<()> {
    // Load and validate the plugin
    let plugin = load_plugin(source_path).context("Failed to load plugin from source")?;
    let plugin_name = &plugin.manifest.plugin.name;

    // Determine destination
    let plugins_dir = Config::expand_path(&config.paths.plugins);
    let dest_path = plugins_dir.join(plugin_name);

    // Check if already installed
    if dest_path.exists() {
        if force {
            if dev {
                // Remove symlink
                fs::remove_file(&dest_path).ok();
            } else {
                // Remove directory
                fs::remove_dir_all(&dest_path).context("Failed to remove existing installation")?;
            }
        } else {
            eyre::bail!("Plugin '{}' already installed. Use --force to overwrite.", plugin_name);
        }
    }

    // Create plugins directory if needed
    fs::create_dir_all(&plugins_dir).context("Failed to create plugins directory")?;

    if dev {
        // Create symlink for development
        #[cfg(unix)]
        {
            let source_abs = fs::canonicalize(source_path)?;
            std::os::unix::fs::symlink(&source_abs, &dest_path).context("Failed to create symlink")?;
        }
        #[cfg(not(unix))]
        {
            eyre::bail!("Dev mode (symlinks) not supported on this platform");
        }
        println!(
            "  {} Linked {} → {}",
            "✓".green(),
            dest_path.display(),
            source_path.display()
        );
    } else {
        // Copy the plugin directory
        copy_dir_recursive(source_path, &dest_path)?;
        println!("  {} Installed to {}", "✓".green(), dest_path.display());
    }

    println!(
        "  {} {} v{}",
        "✓".green(),
        plugin_name.green(),
        plugin.manifest.plugin.version
    );

    Ok(())
}

/// Install a plugin from the registry by name
fn install_from_registry(name: &str, force: bool, config: &Config) -> Result<()> {
    println!("  {} Searching registries for '{}'...", "→".blue(), name);

    // Search cached registries for the plugin
    let registries_dir = Config::expand_path(&config.paths.registries);

    if !registries_dir.exists() {
        eyre::bail!("No cached registries. Run 'paii registry update' first.");
    }

    let mut found_plugin: Option<(String, RegistryPlugin)> = None;

    for entry in fs::read_dir(&registries_dir)? {
        let entry = entry?;
        let path = entry.path();

        if path.extension().is_some_and(|e| e == "toml") {
            let registry_name = path
                .file_stem()
                .and_then(|s| s.to_str())
                .unwrap_or("unknown")
                .to_string();

            let content = fs::read_to_string(&path)?;
            let registry: RegistryFile =
                toml::from_str(&content).context(format!("Failed to parse registry: {}", registry_name))?;

            for plugin in registry.plugins {
                if plugin.name == name {
                    found_plugin = Some((registry_name, plugin));
                    break;
                }
            }

            if found_plugin.is_some() {
                break;
            }
        }
    }

    let (registry_name, plugin) = found_plugin.ok_or_else(|| {
        eyre::eyre!(
            "Plugin '{}' not found in any registry.\n\
             Hint: Run 'paii registry search {}' to find similar plugins.",
            name,
            name
        )
    })?;

    let version = plugin.version.as_deref().unwrap_or("unknown");
    println!(
        "  {} Found {} v{} in registry '{}'",
        "✓".green(),
        plugin.name.cyan(),
        version,
        registry_name.dimmed()
    );

    // Get source and path
    let source_url = plugin
        .source
        .as_ref()
        .ok_or_else(|| eyre::eyre!("Plugin '{}' has no source URL in registry", name))?;
    let plugin_path = plugin.path.as_deref().unwrap_or("");

    // Clone the repository to a temp directory
    let temp_dir = tempfile::tempdir().context("Failed to create temp directory")?;
    let clone_path = temp_dir.path().join("repo");

    println!("  {} Cloning {}...", "→".blue(), source_url.dimmed());

    let status = Command::new("git")
        .args(["clone", "--depth", "1", source_url, clone_path.to_str().unwrap()])
        .stdout(std::process::Stdio::null())
        .stderr(std::process::Stdio::piped())
        .status()
        .context("Failed to run git clone")?;

    if !status.success() {
        eyre::bail!("Git clone failed for: {}", source_url);
    }

    // Find the plugin within the cloned repo
    let plugin_source_path = if plugin_path.is_empty() { clone_path } else { clone_path.join(plugin_path) };

    if !plugin_source_path.exists() {
        eyre::bail!("Plugin path '{}' not found in repository {}", plugin_path, source_url);
    }

    // Now install from the cloned path (not dev mode for registry installs)
    install_from_path(&plugin_source_path, false, force, config)?;

    // Temp dir is automatically cleaned up when it goes out of scope

    Ok(())
}

fn copy_dir_recursive(src: &Path, dst: &Path) -> Result<()> {
    fs::create_dir_all(dst)?;

    for entry in fs::read_dir(src)? {
        let entry = entry?;
        let src_path = entry.path();
        let dst_path = dst.join(entry.file_name());

        if src_path.is_dir() {
            // Skip target directories and hidden directories
            let name = entry.file_name();
            let name_str = name.to_string_lossy();
            if name_str == "target" || name_str.starts_with('.') {
                continue;
            }
            copy_dir_recursive(&src_path, &dst_path)?;
        } else {
            fs::copy(&src_path, &dst_path)?;
        }
    }

    Ok(())
}

fn remove(name: &str, force: bool, config: &Config) -> Result<()> {
    println!(
        "{} Removing plugin: {} {}",
        "→".blue(),
        name.cyan(),
        if force { "(force) ".dimmed().to_string() } else { String::new() },
    );

    let plugins_dir = Config::expand_path(&config.paths.plugins);
    let plugin_path = plugins_dir.join(name);

    if !plugin_path.exists() {
        eyre::bail!("Plugin not found: {}", name);
    }

    // Check if it's a symlink
    let is_symlink = plugin_path.symlink_metadata()?.file_type().is_symlink();

    if is_symlink {
        fs::remove_file(&plugin_path).context("Failed to remove plugin symlink")?;
    } else {
        fs::remove_dir_all(&plugin_path).context("Failed to remove plugin directory")?;
    }

    println!("  {} Removed plugin: {}", "✓".green(), name);

    Ok(())
}

fn update(name: &str, _config: &Config) -> Result<()> {
    println!("{} Updating plugin: {}", "→".blue(), name.cyan());

    // TODO: Implement plugin update
    println!("  {} Plugin update not yet implemented", "⚠".yellow());

    Ok(())
}

fn info(name: &str, config: &Config) -> Result<()> {
    let plugin = find_plugin(name, config)?;

    println!("{}", plugin.manifest.plugin.name.bold());
    println!();
    println!("  {} {}", "Version:".dimmed(), plugin.manifest.plugin.version);
    println!("  {} {}", "Description:".dimmed(), plugin.manifest.plugin.description);
    println!("  {} {:?}", "Language:".dimmed(), plugin.manifest.plugin.language);
    println!("  {} {}", "Path:".dimmed(), plugin.path.display());

    if !plugin.manifest.plugin.authors.is_empty() {
        println!(
            "  {} {}",
            "Authors:".dimmed(),
            plugin.manifest.plugin.authors.join(", ")
        );
    }

    if let Some(ref license) = plugin.manifest.plugin.license {
        println!("  {} {}", "License:".dimmed(), license);
    }

    if let Some(ref repo) = plugin.manifest.plugin.repository {
        println!("  {} {}", "Repository:".dimmed(), repo);
    }

    // Show provides/consumes
    if !plugin.manifest.provides.is_empty() {
        println!();
        println!("  {}:", "Provides".cyan());
        for contract_name in plugin.manifest.provides.keys() {
            println!("    - {}", contract_name.green());
        }
    }

    if !plugin.manifest.consumes.is_empty() {
        println!();
        println!("  {}:", "Consumes".cyan());
        for (contract_name, spec) in &plugin.manifest.consumes {
            let optional = if spec.optional { " (optional)" } else { "" };
            println!("    - {}{}", contract_name.yellow(), optional.dimmed());
        }
    }

    Ok(())
}

/// Find a plugin by name in the plugins directory
pub fn find_plugin(name: &str, config: &Config) -> Result<crate::plugin::Plugin> {
    let plugins_dir = Config::expand_path(&config.paths.plugins);

    // Try exact path first
    let plugin_path = plugins_dir.join(name);
    if plugin_path.exists() {
        return load_plugin(&plugin_path).context(format!("Failed to load plugin '{}'", name));
    }

    // Scan for matching plugin
    if plugins_dir.exists() {
        for entry in fs::read_dir(&plugins_dir)? {
            let entry = entry?;
            let path = entry.path();

            if !path.is_dir() {
                continue;
            }

            let Ok(plugin) = load_plugin(&path) else {
                continue;
            };

            if plugin.manifest.plugin.name == name {
                return Ok(plugin);
            }
        }
    }

    eyre::bail!("Plugin not found: {}", name)
}

fn new(
    name: &str,
    language: &str,
    plugin_type: &str,
    path: Option<&std::path::PathBuf>,
    _config: &Config,
) -> Result<()> {
    let output_path = path
        .cloned()
        .unwrap_or_else(|| std::path::PathBuf::from(format!("./{}", name)));

    println!(
        "{} Creating new {} plugin: {} ({})",
        "→".blue(),
        plugin_type.cyan(),
        name.green(),
        language.dimmed(),
    );
    println!("  Output: {}", output_path.display());

    // Check if directory already exists
    if output_path.exists() {
        eyre::bail!("Directory already exists: {}", output_path.display());
    }

    // Create plugin directory structure
    fs::create_dir_all(&output_path).context("Failed to create plugin directory")?;
    fs::create_dir_all(output_path.join("src")).context("Failed to create src directory")?;

    // Generate plugin.toml
    let manifest_content = generate_manifest(name, language, plugin_type);
    fs::write(output_path.join("plugin.toml"), manifest_content).context("Failed to write plugin.toml")?;

    // Generate main entry point based on language
    match language.to_lowercase().as_str() {
        "python" => {
            let main_py = generate_python_main(name);
            fs::write(output_path.join("src").join("main.py"), main_py).context("Failed to write main.py")?;

            let pyproject = generate_pyproject_toml(name);
            fs::write(output_path.join("pyproject.toml"), pyproject).context("Failed to write pyproject.toml")?;
        }
        "rust" => {
            let main_rs = generate_rust_main(name);
            fs::write(output_path.join("src").join("main.rs"), main_rs).context("Failed to write main.rs")?;

            let cargo_toml = generate_cargo_toml(name);
            fs::write(output_path.join("Cargo.toml"), cargo_toml).context("Failed to write Cargo.toml")?;
        }
        _ => {
            eyre::bail!("Unsupported language: {}. Use 'python' or 'rust'", language);
        }
    }

    // Generate SKILL.md if it's a skill type
    if plugin_type == "skill" {
        let skill_md = generate_skill_md(name);
        fs::write(output_path.join("SKILL.md"), skill_md).context("Failed to write SKILL.md")?;
    }

    // Generate README
    let readme = generate_readme(name, plugin_type, language);
    fs::write(output_path.join("README.md"), readme).context("Failed to write README.md")?;

    println!("  {} Created plugin scaffold", "✓".green());
    println!();
    println!("  Next steps:");
    println!("    1. cd {}", output_path.display());
    println!("    2. Edit plugin.toml to configure contracts");
    if language == "python" {
        println!("    3. Implement your plugin in src/main.py");
    } else {
        println!("    3. Implement your plugin in src/main.rs");
    }
    println!("    4. paii plugin install --dev {}", output_path.display());

    Ok(())
}

fn generate_manifest(name: &str, language: &str, plugin_type: &str) -> String {
    format!(
        r#"[plugin]
name = "{name}"
version = "0.1.0"
description = "A PAII {plugin_type} plugin"
authors = []
language = "{language}"
license = "MIT"

[paii]
core_version = ">=0.1.0"

# Contracts this plugin provides
[provides]

# Contracts this plugin consumes (optional dependencies)
[consumes]

# Plugin configuration schema
[config]

# Hook subscriptions
[hooks]
pre_tool_use = false
post_tool_use = false
stop = false
session_start = false
session_end = false

# Build configuration
[build]
type = "{build_type}"
"#,
        name = name,
        plugin_type = plugin_type,
        language = language,
        build_type = if language == "rust" { "cargo" } else { "uv" },
    )
}

fn generate_python_main(name: &str) -> String {
    format!(
        r#"#!/usr/bin/env python3
"""
{name} - A PAII plugin
"""
import json
import sys


def main():
    """Main entry point."""
    if len(sys.argv) < 2:
        print(json.dumps({{"error": "No action specified"}}))
        sys.exit(1)

    action = sys.argv[1]
    args = sys.argv[2:]

    if action == "greet":
        name = args[0] if args else "World"
        print(json.dumps({{"message": f"Hello, {{name}}!"}}))
    elif action == "version":
        print(json.dumps({{"version": "0.1.0"}}))
    else:
        print(json.dumps({{"error": f"Unknown action: {{action}}"}}))
        sys.exit(1)


if __name__ == "__main__":
    main()
"#,
        name = name,
    )
}

fn generate_rust_main(name: &str) -> String {
    format!(
        r##"//! {name} - A PAII plugin

use std::env;

fn main() {{
    let args: Vec<String> = env::args().collect();

    if args.len() < 2 {{
        eprintln!(r#"{{{{"error": "No action specified"}}}}"#);
        std::process::exit(1);
    }}

    let action = &args[1];
    let action_args = &args[2..];

    match action.as_str() {{
        "greet" => {{
            let name = action_args.first().map(|s| s.as_str()).unwrap_or("World");
            println!(r#"{{{{"message": "Hello, {{}}!"}}}}"#, name);
        }}
        "version" => {{
            println!(r#"{{{{"version": "0.1.0"}}}}"#);
        }}
        _ => {{
            eprintln!(r#"{{{{"error": "Unknown action: {{}}"}}}}"#, action);
            std::process::exit(1);
        }}
    }}
}}
"##,
        name = name,
    )
}

fn generate_cargo_toml(name: &str) -> String {
    format!(
        r#"[package]
name = "{name}"
version = "0.1.0"
edition = "2021"

[dependencies]
serde = {{ version = "1.0", features = ["derive"] }}
serde_json = "1.0"
"#,
        name = name,
    )
}

fn generate_pyproject_toml(name: &str) -> String {
    format!(
        r#"[project]
name = "{name}"
version = "0.1.0"
description = "A PAII plugin"
requires-python = ">=3.10"
dependencies = []

[build-system]
requires = ["hatchling"]
build-backend = "hatchling.build"

[tool.uv]
dev-dependencies = []
"#,
        name = name,
    )
}

fn generate_skill_md(name: &str) -> String {
    format!(
        r#"# {name}

## USE WHEN

- User asks about {name}
- User wants to perform actions related to {name}

## ACTIONS

### greet
Greet someone by name.

**Arguments:**
- `name` (optional): Name to greet. Defaults to "World".

**Example:**
```
paii run {name} greet Alice
```

### version
Show the plugin version.

## NOTES

- This is a sample skill template
- Customize the USE WHEN triggers for your use case
"#,
        name = name,
    )
}

fn generate_readme(name: &str, plugin_type: &str, language: &str) -> String {
    format!(
        r#"# {name}

A PAII {plugin_type} plugin written in {language}.

## Installation

```bash
paii plugin install --dev .
```

## Usage

```bash
paii run {name} greet
paii run {name} greet Alice
paii run {name} version
```

## Development

{dev_instructions}

## License

MIT
"#,
        name = name,
        plugin_type = plugin_type,
        language = language,
        dev_instructions = if language == "rust" {
            "```bash\ncargo build --release\n```"
        } else {
            "```bash\nuv sync\n```"
        },
    )
}

fn verify(name: &str, config: &Config) -> Result<()> {
    println!("{} Verifying plugin: {}", "→".blue(), name.cyan());

    let plugin = find_plugin(name, config)?;

    // Check manifest is valid
    println!("  {} Manifest valid", "✓".green());

    // Check entry point exists
    match plugin.manifest.plugin.language {
        crate::plugin::manifest::PluginLanguage::Python => {
            let main_py = plugin.path.join("src").join("main.py");
            if main_py.exists() {
                println!("  {} Python entry point found", "✓".green());
            } else {
                println!("  {} Python entry point missing: src/main.py", "✗".red());
            }
        }
        crate::plugin::manifest::PluginLanguage::Rust => {
            let cargo_toml = plugin.path.join("Cargo.toml");
            if cargo_toml.exists() {
                println!("  {} Rust project found", "✓".green());
            } else {
                println!("  {} Cargo.toml missing", "✗".red());
            }
        }
        crate::plugin::manifest::PluginLanguage::Mixed => {
            println!("  {} Mixed language plugin", "ℹ".blue());
        }
    }

    // Check for SKILL.md if it's a skill
    let skill_md = plugin.path.join("SKILL.md");
    if skill_md.exists() {
        println!("  {} SKILL.md found", "✓".green());
    }

    println!();
    println!("  {} Plugin verification complete", "✓".green());

    Ok(())
}
