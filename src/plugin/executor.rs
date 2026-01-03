//! Plugin hook executor
//!
//! Executes plugin scripts when hook events fire.

use eyre::{Context, Result};
use std::io::Write;
use std::path::Path;
use std::process::{Command, Stdio};

use crate::hook::{HookEvent, HookResult};
use crate::plugin::manifest::{HookScript, PluginLanguage, PluginManifest};

/// Result of executing a plugin hook
#[derive(Debug)]
pub struct PluginHookResult {
    pub plugin_name: String,
    pub script: String,
    pub exit_code: i32,
    pub stdout: String,
    pub stderr: String,
}

impl PluginHookResult {
    /// Convert to HookResult based on exit code
    pub fn to_hook_result(&self) -> HookResult {
        match self.exit_code {
            0 => HookResult::Allow,
            2 => HookResult::Block {
                message: if self.stderr.is_empty() {
                    format!("Blocked by plugin '{}' ({})", self.plugin_name, self.script)
                } else {
                    self.stderr.clone()
                },
            },
            _ => HookResult::Error {
                message: format!(
                    "Plugin '{}' ({}) exited with code {}: {}",
                    self.plugin_name, self.script, self.exit_code, self.stderr
                ),
            },
        }
    }
}

/// Execute a plugin hook script
pub fn execute_hook(
    plugin_path: &Path,
    manifest: &PluginManifest,
    hook_script: &HookScript,
    event: HookEvent,
    payload: &serde_json::Value,
) -> Result<PluginHookResult> {
    let script_path = plugin_path.join(&hook_script.script);

    if !script_path.exists() {
        return Ok(PluginHookResult {
            plugin_name: manifest.plugin.name.clone(),
            script: hook_script.script.clone(),
            exit_code: 1,
            stdout: String::new(),
            stderr: format!("Script not found: {}", script_path.display()),
        });
    }

    // Check matcher if specified
    if let Some(ref matcher) = hook_script.matcher {
        let tool_name = payload.get("tool_name").and_then(|v| v.as_str()).unwrap_or("");
        if tool_name != matcher {
            // Matcher doesn't match, skip this hook
            return Ok(PluginHookResult {
                plugin_name: manifest.plugin.name.clone(),
                script: hook_script.script.clone(),
                exit_code: 0,
                stdout: String::new(),
                stderr: String::new(),
            });
        }
    }

    // Determine how to run the script based on plugin language
    let (program, args) = match manifest.plugin.language {
        PluginLanguage::Python => {
            // Try uv first, fall back to python
            if which::which("uv").is_ok() {
                ("uv", vec!["run", "python", script_path.to_str().unwrap_or("")])
            } else {
                ("python3", vec![script_path.to_str().unwrap_or("")])
            }
        }
        PluginLanguage::Rust => {
            // Rust plugins should be compiled binaries
            (script_path.to_str().unwrap_or(""), vec![])
        }
        PluginLanguage::Mixed => {
            // Determine by file extension
            let ext = script_path.extension().and_then(|e| e.to_str()).unwrap_or("");
            match ext {
                "py" => {
                    if which::which("uv").is_ok() {
                        ("uv", vec!["run", "python", script_path.to_str().unwrap_or("")])
                    } else {
                        ("python3", vec![script_path.to_str().unwrap_or("")])
                    }
                }
                _ => (script_path.to_str().unwrap_or(""), vec![]),
            }
        }
    };

    // Serialize payload
    let payload_json = serde_json::to_string(payload).context("Failed to serialize payload")?;

    // Spawn process
    let mut child = Command::new(program)
        .args(&args)
        .current_dir(plugin_path)
        .stdin(Stdio::piped())
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .env("PAIS_EVENT", event.to_string())
        .env("PAIS_PLUGIN", &manifest.plugin.name)
        .spawn()
        .with_context(|| format!("Failed to spawn plugin script: {}", script_path.display()))?;

    // Write payload to stdin
    if let Some(mut stdin) = child.stdin.take() {
        stdin
            .write_all(payload_json.as_bytes())
            .context("Failed to write payload to plugin stdin")?;
    }

    // Wait for completion with timeout
    let output = child.wait_with_output().context("Failed to wait for plugin script")?;

    let exit_code = output.status.code().unwrap_or(1);
    let stdout = String::from_utf8_lossy(&output.stdout).to_string();
    let stderr = String::from_utf8_lossy(&output.stderr).to_string();

    Ok(PluginHookResult {
        plugin_name: manifest.plugin.name.clone(),
        script: hook_script.script.clone(),
        exit_code,
        stdout,
        stderr,
    })
}

/// Execute all hooks for a plugin on a given event
pub fn execute_plugin_hooks(
    plugin_path: &Path,
    manifest: &PluginManifest,
    event: HookEvent,
    payload: &serde_json::Value,
) -> Vec<PluginHookResult> {
    let scripts = manifest.hooks.scripts_for_event(&event.to_string());

    scripts
        .iter()
        .filter_map(
            |script| match execute_hook(plugin_path, manifest, script, event, payload) {
                Ok(result) => Some(result),
                Err(e) => {
                    log::error!("Failed to execute plugin hook: {}", e);
                    None
                }
            },
        )
        .collect()
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs;
    use tempfile::tempdir;

    fn create_test_plugin(dir: &Path, script_content: &str) -> PluginManifest {
        // Create hooks directory
        let hooks_dir = dir.join("hooks");
        fs::create_dir_all(&hooks_dir).unwrap();

        // Write test script
        let script_path = hooks_dir.join("test.py");
        fs::write(&script_path, script_content).unwrap();

        // Make executable on Unix
        #[cfg(unix)]
        {
            use std::os::unix::fs::PermissionsExt;
            let mut perms = fs::metadata(&script_path).unwrap().permissions();
            perms.set_mode(0o755);
            fs::set_permissions(&script_path, perms).unwrap();
        }

        // Create manifest
        let manifest_content = r#"
plugin:
  name: test-plugin
  version: 0.1.0
  description: Test plugin
  language: python

hooks:
  PreToolUse:
    - script: hooks/test.py
"#;
        PluginManifest::from_str(manifest_content).unwrap()
    }

    #[test]
    fn test_execute_hook_allow() {
        let temp = tempdir().unwrap();
        let manifest = create_test_plugin(
            temp.path(),
            r#"#!/usr/bin/env python3
import sys
sys.exit(0)  # Allow
"#,
        );

        let payload = serde_json::json!({"tool_name": "Bash"});
        let result = execute_hook(
            temp.path(),
            &manifest,
            &manifest.hooks.pre_tool_use[0],
            HookEvent::PreToolUse,
            &payload,
        )
        .unwrap();

        assert_eq!(result.exit_code, 0);
        assert!(matches!(result.to_hook_result(), HookResult::Allow));
    }

    #[test]
    fn test_execute_hook_block() {
        let temp = tempdir().unwrap();
        let manifest = create_test_plugin(
            temp.path(),
            r#"#!/usr/bin/env python3
import sys
print("Blocked!", file=sys.stderr)
sys.exit(2)  # Block
"#,
        );

        let payload = serde_json::json!({"tool_name": "Bash"});
        let result = execute_hook(
            temp.path(),
            &manifest,
            &manifest.hooks.pre_tool_use[0],
            HookEvent::PreToolUse,
            &payload,
        )
        .unwrap();

        assert_eq!(result.exit_code, 2);
        assert!(matches!(result.to_hook_result(), HookResult::Block { .. }));
    }

    #[test]
    fn test_hook_result_conversion() {
        let allow = PluginHookResult {
            plugin_name: "test".to_string(),
            script: "test.py".to_string(),
            exit_code: 0,
            stdout: String::new(),
            stderr: String::new(),
        };
        assert!(matches!(allow.to_hook_result(), HookResult::Allow));

        let block = PluginHookResult {
            plugin_name: "test".to_string(),
            script: "test.py".to_string(),
            exit_code: 2,
            stdout: String::new(),
            stderr: "Blocked!".to_string(),
        };
        assert!(matches!(block.to_hook_result(), HookResult::Block { .. }));

        let error = PluginHookResult {
            plugin_name: "test".to_string(),
            script: "test.py".to_string(),
            exit_code: 1,
            stdout: String::new(),
            stderr: "Error".to_string(),
        };
        assert!(matches!(error.to_hook_result(), HookResult::Error { .. }));
    }
}
