use colored::*;
use eyre::{Context, Result};
use std::process::Command;

use crate::commands::plugin::find_plugin;
use crate::config::Config;
use crate::plugin::manifest::PluginLanguage;

pub fn run(plugin_name: &str, action: &str, args: &[String], config: &Config) -> Result<()> {
    log::info!("Running plugin: {} action: {}", plugin_name, action);

    // Find the plugin
    let plugin = find_plugin(plugin_name, config)?;

    // Determine how to execute based on language
    let output = match plugin.manifest.plugin.language {
        PluginLanguage::Python => execute_python(&plugin.path, action, args)?,
        PluginLanguage::Rust => execute_rust(&plugin.path, action, args)?,
        PluginLanguage::Mixed => {
            // Try Python first, then Rust
            let python_main = plugin.path.join("src").join("main.py");
            if python_main.exists() {
                execute_python(&plugin.path, action, args)?
            } else {
                execute_rust(&plugin.path, action, args)?
            }
        }
    };

    // Print output
    print!("{}", output);

    Ok(())
}

fn execute_python(plugin_path: &std::path::Path, action: &str, args: &[String]) -> Result<String> {
    let main_py = plugin_path.join("src").join("main.py");

    if !main_py.exists() {
        eyre::bail!("Python main not found: {}", main_py.display());
    }

    // Use plugin's venv Python if it exists, otherwise fall back to system python3
    let venv_python = plugin_path.join(".venv").join("bin").join("python");
    let python_cmd = if venv_python.exists() {
        venv_python
    } else {
        std::path::PathBuf::from("python3")
    };

    let mut cmd = Command::new(&python_cmd);
    cmd.arg(&main_py).arg(action);

    for arg in args {
        cmd.arg(arg);
    }

    cmd.current_dir(plugin_path);

    let output = cmd.output().context("Failed to execute Python plugin")?;

    if !output.status.success() {
        let stderr = String::from_utf8_lossy(&output.stderr);
        let stdout = String::from_utf8_lossy(&output.stdout);
        eyre::bail!(
            "Plugin failed with exit code {:?}:\n{}\n{}",
            output.status.code(),
            stdout,
            stderr
        );
    }

    Ok(String::from_utf8_lossy(&output.stdout).to_string())
}

fn execute_rust(plugin_path: &std::path::Path, action: &str, args: &[String]) -> Result<String> {
    // Look for built binary
    let plugin_name = plugin_path.file_name().and_then(|n| n.to_str()).unwrap_or("plugin");

    // Try release first, then debug
    let binary_paths = [
        plugin_path.join("target").join("release").join(plugin_name),
        plugin_path.join("target").join("debug").join(plugin_name),
    ];

    let binary = binary_paths.iter().find(|p| p.exists());

    let binary = match binary {
        Some(b) => b,
        None => {
            // Try to build it
            eprintln!("{} Building Rust plugin: {}", "→".blue(), plugin_name.cyan());
            let status = Command::new("cargo")
                .arg("build")
                .arg("--release")
                .current_dir(plugin_path)
                .status()
                .context("Failed to build Rust plugin")?;

            if !status.success() {
                eyre::bail!("Failed to build Rust plugin");
            }

            &binary_paths[0]
        }
    };

    let mut cmd = Command::new(binary);
    cmd.arg(action);

    for arg in args {
        cmd.arg(arg);
    }

    cmd.current_dir(plugin_path);

    let output = cmd.output().context("Failed to execute Rust plugin")?;

    if !output.status.success() {
        let stderr = String::from_utf8_lossy(&output.stderr);
        let stdout = String::from_utf8_lossy(&output.stdout);
        eyre::bail!(
            "Plugin failed with exit code {:?}:\n{}\n{}",
            output.status.code(),
            stdout,
            stderr
        );
    }

    Ok(String::from_utf8_lossy(&output.stdout).to_string())
}
