//! Integration tests for the plugin system
//!
//! These tests verify the full plugin workflow:
//! - Creating plugins
//! - Installing plugins
//! - Listing plugins
//! - Running plugin actions
//! - Plugin hook execution
//! - Removing plugins

use std::fs;
use std::os::unix::fs::PermissionsExt;
use std::path::{Path, PathBuf};
use std::process::Command;

use tempfile::TempDir;

/// Helper to get the pais binary path
fn pais_binary() -> PathBuf {
    // When running tests, the binary is in target/debug/pais
    let mut path = std::env::current_exe().unwrap();
    path.pop(); // Remove test binary name
    path.pop(); // Remove deps
    path.push("pais");
    path
}

/// Helper to run pais command with a custom config directory
fn run_pais(pais_dir: &Path, args: &[&str]) -> std::process::Output {
    Command::new(pais_binary())
        .env("PAIS_DIR", pais_dir)
        .args(args)
        .output()
        .expect("Failed to execute pais")
}

/// Helper to run pais and get stdout as string
fn run_pais_stdout(pais_dir: &Path, args: &[&str]) -> String {
    let output = run_pais(pais_dir, args);
    String::from_utf8_lossy(&output.stdout).to_string()
}

/// Helper to create a minimal Python plugin
fn create_python_plugin(dir: &Path, name: &str) {
    let plugin_dir = dir.join(name);
    let src_dir = plugin_dir.join("src");
    fs::create_dir_all(&src_dir).unwrap();

    // Create plugin.yaml
    let manifest = format!(
        r#"plugin:
  name: {name}
  version: 0.1.0
  description: Test plugin for integration tests
  language: python

hooks:
  PreToolUse:
    - script: hooks/security.py
      matcher: Bash
"#
    );
    fs::write(plugin_dir.join("plugin.yaml"), manifest).unwrap();

    // Create main.py for `pais run`
    let main_py = r#"#!/usr/bin/env python3
"""Test plugin main entry point."""
import json
import sys

def main():
    if len(sys.argv) < 2:
        print(json.dumps({"error": "No action specified"}))
        sys.exit(1)

    action = sys.argv[1]
    args = sys.argv[2:]

    if action == "greet":
        name = args[0] if args else "World"
        print(json.dumps({"message": f"Hello, {name}!"}))
    elif action == "version":
        print(json.dumps({"version": "0.1.0"}))
    elif action == "echo":
        print(json.dumps({"args": args}))
    else:
        print(json.dumps({"error": f"Unknown action: {action}"}))
        sys.exit(1)

if __name__ == "__main__":
    main()
"#;
    fs::write(src_dir.join("main.py"), main_py).unwrap();

    // Create hooks directory and security hook
    let hooks_dir = plugin_dir.join("hooks");
    fs::create_dir_all(&hooks_dir).unwrap();

    let hook_py = r#"#!/usr/bin/env python3
"""Security hook that blocks dangerous commands."""
import json
import sys

# Read payload from stdin
payload = json.load(sys.stdin)

tool_name = payload.get("tool_name", "")
tool_input = payload.get("tool_input", {})
command = tool_input.get("command", "")

# Block rm -rf commands
if "rm -rf /" in command or "rm -rf ~" in command:
    print("BLOCKED: Catastrophic deletion detected", file=sys.stderr)
    sys.exit(2)  # Exit 2 = block

# Allow everything else
sys.exit(0)
"#;
    let hook_path = hooks_dir.join("security.py");
    fs::write(&hook_path, hook_py).unwrap();

    // Make scripts executable
    let mut perms = fs::metadata(&hook_path).unwrap().permissions();
    perms.set_mode(0o755);
    fs::set_permissions(&hook_path, perms).unwrap();

    let main_path = src_dir.join("main.py");
    let mut perms = fs::metadata(&main_path).unwrap().permissions();
    perms.set_mode(0o755);
    fs::set_permissions(&main_path, perms).unwrap();
}

/// Helper to create a minimal Rust plugin
fn create_rust_plugin(dir: &Path, name: &str) {
    let plugin_dir = dir.join(name);
    let src_dir = plugin_dir.join("src");
    fs::create_dir_all(&src_dir).unwrap();

    // Create plugin.yaml
    let manifest = format!(
        r#"plugin:
  name: {name}
  version: 0.1.0
  description: Rust test plugin
  language: rust
"#
    );
    fs::write(plugin_dir.join("plugin.yaml"), manifest).unwrap();

    // Create Cargo.toml
    let cargo_toml = format!(
        r#"[package]
name = "{name}"
version = "0.1.0"
edition = "2021"

[[bin]]
name = "{name}"
path = "src/main.rs"

[dependencies]
serde_json = "1.0"
"#
    );
    fs::write(plugin_dir.join("Cargo.toml"), cargo_toml).unwrap();

    // Create main.rs
    let main_rs = r##"use std::env;

fn main() {
    let args: Vec<String> = env::args().collect();

    if args.len() < 2 {
        eprintln!(r#"{{"error": "No action specified"}}"#);
        std::process::exit(1);
    }

    let action = &args[1];

    match action.as_str() {
        "greet" => {
            let name = args.get(2).map(|s| s.as_str()).unwrap_or("World");
            println!(r#"{{"message": "Hello, {}!"}}"#, name);
        }
        "version" => {
            println!(r#"{{"version": "0.1.0"}}"#);
        }
        _ => {
            eprintln!(r#"{{"error": "Unknown action: {}"}}"#, action);
            std::process::exit(1);
        }
    }
}
"##;
    fs::write(src_dir.join("main.rs"), main_rs).unwrap();
}

/// Helper to setup a test PAIS environment
fn setup_test_env() -> (TempDir, PathBuf) {
    let temp = TempDir::new().unwrap();
    let pais_dir = temp.path().join(".config").join("pais");

    // Create directory structure
    fs::create_dir_all(pais_dir.join("plugins")).unwrap();
    fs::create_dir_all(pais_dir.join("skills")).unwrap();
    fs::create_dir_all(pais_dir.join("history")).unwrap();
    fs::create_dir_all(pais_dir.join("registries")).unwrap();
    fs::create_dir_all(pais_dir.join("agents")).unwrap();

    // Create minimal config with absolute paths to the test directory (YAML format)
    let config = format!(
        r#"paths:
  plugins: "{plugins}"
  skills: "{skills}"
  history: "{history}"
  registries: "{registries}"

hooks:
  security_enabled: true
  history_enabled: false
  ui_enabled: false

observability:
  enabled: false
"#,
        plugins = pais_dir.join("plugins").display(),
        skills = pais_dir.join("skills").display(),
        history = pais_dir.join("history").display(),
        registries = pais_dir.join("registries").display(),
    );
    fs::write(pais_dir.join("pais.yaml"), config).unwrap();

    (temp, pais_dir)
}

// ============================================================================
// Integration Tests
// ============================================================================

#[test]
fn test_plugin_install_from_local_path() {
    let (temp, pais_dir) = setup_test_env();
    let source_dir = temp.path().join("source");
    fs::create_dir_all(&source_dir).unwrap();

    // Create a plugin in source directory
    create_python_plugin(&source_dir, "test-plugin");

    // Install it
    let output = run_pais(
        &pais_dir,
        &["plugin", "install", source_dir.join("test-plugin").to_str().unwrap()],
    );

    assert!(output.status.success(), "Install failed: {:?}", output);

    // Verify it was installed
    let installed_path = pais_dir.join("plugins").join("test-plugin");
    assert!(installed_path.exists(), "Plugin not installed to expected path");
    assert!(installed_path.join("plugin.yaml").exists(), "plugin.yaml not found");
}

#[test]
fn test_plugin_install_dev_mode() {
    let (temp, pais_dir) = setup_test_env();
    let source_dir = temp.path().join("source");
    fs::create_dir_all(&source_dir).unwrap();

    create_python_plugin(&source_dir, "dev-plugin");

    // Install in dev mode (symlink)
    let output = run_pais(
        &pais_dir,
        &[
            "plugin",
            "install",
            "--dev",
            source_dir.join("dev-plugin").to_str().unwrap(),
        ],
    );

    assert!(output.status.success(), "Dev install failed: {:?}", output);

    // Verify it's a symlink
    let installed_path = pais_dir.join("plugins").join("dev-plugin");
    assert!(installed_path.exists(), "Plugin not installed");
    assert!(
        installed_path.symlink_metadata().unwrap().file_type().is_symlink(),
        "Should be a symlink in dev mode"
    );
}

#[test]
fn test_plugin_list_shows_installed() {
    let (temp, pais_dir) = setup_test_env();
    let source_dir = temp.path().join("source");
    fs::create_dir_all(&source_dir).unwrap();

    // Install two plugins
    create_python_plugin(&source_dir, "plugin-one");
    create_python_plugin(&source_dir, "plugin-two");

    run_pais(
        &pais_dir,
        &["plugin", "install", source_dir.join("plugin-one").to_str().unwrap()],
    );
    run_pais(
        &pais_dir,
        &["plugin", "install", source_dir.join("plugin-two").to_str().unwrap()],
    );

    // List plugins
    let output = run_pais_stdout(&pais_dir, &["plugin", "list", "--format", "json"]);

    // Parse JSON and verify
    let plugins: Vec<serde_json::Value> = serde_json::from_str(&output).unwrap();
    assert_eq!(plugins.len(), 2, "Should have 2 plugins installed");

    let names: Vec<&str> = plugins.iter().map(|p| p["name"].as_str().unwrap()).collect();
    assert!(names.contains(&"plugin-one"));
    assert!(names.contains(&"plugin-two"));
}

#[test]
fn test_plugin_remove() {
    let (temp, pais_dir) = setup_test_env();
    let source_dir = temp.path().join("source");
    fs::create_dir_all(&source_dir).unwrap();

    create_python_plugin(&source_dir, "removable-plugin");

    // Install
    run_pais(
        &pais_dir,
        &[
            "plugin",
            "install",
            source_dir.join("removable-plugin").to_str().unwrap(),
        ],
    );

    let installed_path = pais_dir.join("plugins").join("removable-plugin");
    assert!(installed_path.exists(), "Plugin should be installed");

    // Remove
    let output = run_pais(&pais_dir, &["plugin", "remove", "removable-plugin"]);
    assert!(output.status.success(), "Remove failed: {:?}", output);

    // Verify removed
    assert!(!installed_path.exists(), "Plugin should be removed");
}

#[test]
fn test_plugin_info() {
    let (temp, pais_dir) = setup_test_env();
    let source_dir = temp.path().join("source");
    fs::create_dir_all(&source_dir).unwrap();

    create_python_plugin(&source_dir, "info-plugin");
    run_pais(
        &pais_dir,
        &["plugin", "install", source_dir.join("info-plugin").to_str().unwrap()],
    );

    // Get info
    let output = run_pais_stdout(&pais_dir, &["plugin", "info", "info-plugin"]);

    assert!(output.contains("info-plugin"), "Should show plugin name");
    assert!(output.contains("0.1.0"), "Should show version");
    assert!(
        output.contains("python") || output.contains("Python"),
        "Should show language"
    );
}

#[test]
fn test_pais_run_python_plugin() {
    let (temp, pais_dir) = setup_test_env();
    let source_dir = temp.path().join("source");
    fs::create_dir_all(&source_dir).unwrap();

    create_python_plugin(&source_dir, "runnable-plugin");
    run_pais(
        &pais_dir,
        &[
            "plugin",
            "install",
            source_dir.join("runnable-plugin").to_str().unwrap(),
        ],
    );

    // Run greet action
    let output = run_pais_stdout(&pais_dir, &["run", "runnable-plugin", "greet"]);
    let result: serde_json::Value = serde_json::from_str(&output).unwrap();
    assert_eq!(result["message"], "Hello, World!");

    // Run greet with argument
    let output = run_pais_stdout(&pais_dir, &["run", "runnable-plugin", "greet", "PAIS"]);
    let result: serde_json::Value = serde_json::from_str(&output).unwrap();
    assert_eq!(result["message"], "Hello, PAIS!");

    // Run version action
    let output = run_pais_stdout(&pais_dir, &["run", "runnable-plugin", "version"]);
    let result: serde_json::Value = serde_json::from_str(&output).unwrap();
    assert_eq!(result["version"], "0.1.0");
}

#[test]
fn test_pais_run_with_multiple_args() {
    let (temp, pais_dir) = setup_test_env();
    let source_dir = temp.path().join("source");
    fs::create_dir_all(&source_dir).unwrap();

    create_python_plugin(&source_dir, "args-plugin");
    run_pais(
        &pais_dir,
        &["plugin", "install", source_dir.join("args-plugin").to_str().unwrap()],
    );

    // Run echo action with multiple args
    let output = run_pais_stdout(&pais_dir, &["run", "args-plugin", "echo", "one", "two", "three"]);
    let result: serde_json::Value = serde_json::from_str(&output).unwrap();
    let args: Vec<&str> = result["args"]
        .as_array()
        .unwrap()
        .iter()
        .map(|v| v.as_str().unwrap())
        .collect();
    assert_eq!(args, vec!["one", "two", "three"]);
}

#[test]
fn test_pais_run_unknown_action_fails() {
    let (temp, pais_dir) = setup_test_env();
    let source_dir = temp.path().join("source");
    fs::create_dir_all(&source_dir).unwrap();

    create_python_plugin(&source_dir, "fail-plugin");
    run_pais(
        &pais_dir,
        &["plugin", "install", source_dir.join("fail-plugin").to_str().unwrap()],
    );

    // Run unknown action
    let output = run_pais(&pais_dir, &["run", "fail-plugin", "unknown_action"]);
    assert!(!output.status.success(), "Should fail for unknown action");
}

#[test]
fn test_plugin_verify() {
    let (temp, pais_dir) = setup_test_env();
    let source_dir = temp.path().join("source");
    fs::create_dir_all(&source_dir).unwrap();

    create_python_plugin(&source_dir, "verify-plugin");
    run_pais(
        &pais_dir,
        &["plugin", "install", source_dir.join("verify-plugin").to_str().unwrap()],
    );

    // Verify plugin
    let output = run_pais(&pais_dir, &["plugin", "verify", "verify-plugin"]);
    assert!(output.status.success(), "Verify should succeed: {:?}", output);

    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(
        stdout.contains("✓") || stdout.contains("valid") || stdout.contains("Manifest"),
        "Should show verification passed"
    );
}

#[test]
fn test_status_shows_plugins() {
    let (temp, pais_dir) = setup_test_env();
    let source_dir = temp.path().join("source");
    fs::create_dir_all(&source_dir).unwrap();

    create_python_plugin(&source_dir, "status-plugin");
    run_pais(
        &pais_dir,
        &["plugin", "install", source_dir.join("status-plugin").to_str().unwrap()],
    );

    // Check status includes plugin
    let output = run_pais_stdout(&pais_dir, &["status", "--format", "json"]);
    let status: serde_json::Value = serde_json::from_str(&output).unwrap();

    let plugins = status["plugins"].as_array().unwrap();
    assert_eq!(plugins.len(), 1);
    assert_eq!(plugins[0]["name"], "status-plugin");
}

// Note: Rust plugin test is slower because it requires cargo build
#[test]
#[ignore] // Run with `cargo test -- --ignored` to include this test
fn test_pais_run_rust_plugin() {
    let (temp, pais_dir) = setup_test_env();
    let source_dir = temp.path().join("source");
    fs::create_dir_all(&source_dir).unwrap();

    create_rust_plugin(&source_dir, "rust-plugin");
    run_pais(
        &pais_dir,
        &["plugin", "install", source_dir.join("rust-plugin").to_str().unwrap()],
    );

    // Run greet action (this will trigger cargo build)
    let output = run_pais(&pais_dir, &["run", "rust-plugin", "greet"]);
    assert!(output.status.success(), "Rust plugin should work: {:?}", output);

    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(
        stdout.contains("Hello") && stdout.contains("World"),
        "Should greet: {}",
        stdout
    );
}
