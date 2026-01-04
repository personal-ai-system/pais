use colored::*;
use eyre::{Context, Result};
use serde::Serialize;
use std::fs;
use std::path::Path;

use crate::cli::{OutputFormat, PluginAction};
use crate::config::Config;
use crate::plugin::loader::load_plugin;

pub fn run(action: PluginAction, config: &Config) -> Result<()> {
    match action {
        PluginAction::List { format } => list(OutputFormat::resolve(format), config),
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

/// Plugin info for serialization
#[derive(Debug, Serialize)]
struct PluginInfo {
    name: String,
    version: String,
    description: String,
    language: String,
    path: String,
}

fn list(format: OutputFormat, config: &Config) -> Result<()> {
    let plugins_dir = Config::expand_path(&config.paths.plugins);

    if !plugins_dir.exists() {
        match format {
            OutputFormat::Json => println!("[]"),
            OutputFormat::Yaml => println!("[]"),
            OutputFormat::Text => {
                println!("{}", "Installed plugins:".bold());
                println!();
                println!("  {}", "(none)".dimmed());
            }
        }
        return Ok(());
    }

    let mut plugins = Vec::new();

    // Scan for plugin directories
    for entry in fs::read_dir(&plugins_dir).context("Failed to read plugins directory")? {
        let entry = entry?;
        let path = entry.path();

        if path.is_dir() {
            let manifest_path = path.join("plugin.yaml");
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

    match format {
        OutputFormat::Json | OutputFormat::Yaml => {
            let output: Vec<PluginInfo> = plugins
                .iter()
                .map(|p| PluginInfo {
                    name: p.manifest.plugin.name.clone(),
                    version: p.manifest.plugin.version.clone(),
                    description: p.manifest.plugin.description.clone(),
                    language: format!("{:?}", p.manifest.plugin.language),
                    path: p.path.display().to_string(),
                })
                .collect();
            match format {
                OutputFormat::Json => println!("{}", serde_json::to_string_pretty(&output)?),
                OutputFormat::Yaml => println!("{}", serde_yaml::to_string(&output)?),
                _ => unreachable!(),
            }
        }
        OutputFormat::Text => {
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

    // Install from local path only
    if source_path.exists() {
        install_from_path(source_path, dev, force, config)
    } else {
        eyre::bail!(
            "Source not found: {}\n\
             Install plugins from local paths or git repos.\n\
             Examples:\n\
               pais plugin install ./my-plugin\n\
               pais plugin install ~/repos/scottidler/my-plugin",
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

fn update(name: &str, config: &Config) -> Result<()> {
    println!("{} Updating plugin: {}", "→".blue(), name.cyan());

    // Check if plugin is installed
    let plugin = match find_plugin(name, config) {
        Ok(p) => p,
        Err(_) => {
            eyre::bail!("Plugin '{}' is not installed", name);
        }
    };

    let current_version = &plugin.manifest.plugin.version;
    println!("  Current version: {}", current_version.dimmed());

    // Check if it's a dev install (symlink)
    let plugins_dir = Config::expand_path(&config.paths.plugins);
    let plugin_path = plugins_dir.join(name);
    if plugin_path.symlink_metadata()?.file_type().is_symlink() {
        println!("  {} Plugin is installed in dev mode (symlink)", "⚠".yellow());
        println!("    Update the source directory directly (git pull, etc.)");
        return Ok(());
    }

    // For non-dev plugins, suggest reinstallation from source
    println!("  {} To update, reinstall from source:", "→".blue());
    println!("    pais plugin remove {}", name);
    println!("    pais plugin install /path/to/source");

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

    // Generate plugin.yaml
    let manifest_content = generate_manifest(name, language, plugin_type);
    fs::write(output_path.join("plugin.yaml"), manifest_content).context("Failed to write plugin.yaml")?;

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
    println!("    2. Edit plugin.yaml to configure contracts");
    if language == "python" {
        println!("    3. Implement your plugin in src/main.py");
    } else {
        println!("    3. Implement your plugin in src/main.rs");
    }
    println!("    4. pais plugin install --dev {}", output_path.display());

    Ok(())
}

fn generate_manifest(name: &str, language: &str, plugin_type: &str) -> String {
    format!(
        r#"plugin:
  name: {name}
  version: 0.1.0
  description: A PAIS {plugin_type} plugin
  authors: []
  language: {language}
  license: MIT

pais:
  core_version: ">=0.1.0"

# Contracts this plugin provides
provides: {{}}

# Contracts this plugin consumes (optional dependencies)
consumes: {{}}

# Plugin configuration schema
config: {{}}

# Hook subscriptions (scripts to run on events)
# hooks:
#   PreToolUse:
#     - script: hooks/validate.py
#       matcher: Bash  # optional - only run for specific tool
#   Stop:
#     - script: hooks/capture.py

# Build configuration
build:
  type: {build_type}
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
{name} - A PAIS plugin
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
        r##"//! {name} - A PAIS plugin

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
description = "A PAIS plugin"
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
pais run {name} greet Alice
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

A PAIS {plugin_type} plugin written in {language}.

## Installation

```bash
pais plugin install --dev .
```

## Usage

```bash
pais run {name} greet
pais run {name} greet Alice
pais run {name} version
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
