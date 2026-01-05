//! Plugin verification
//!
//! Runs verification checks to ensure plugins are correctly installed.

use colored::*;
use eyre::{Context, Result};
use serde::Serialize;
use std::path::Path;
use std::process::Command;

use super::manifest::{VerificationCommand, VerificationSpec};

/// Result of a single verification check
#[derive(Debug, Clone, Serialize)]
pub struct CheckResult {
    pub name: String,
    pub passed: bool,
    pub message: Option<String>,
}

/// Result of all verification checks for a plugin
#[derive(Debug, Serialize)]
pub struct VerificationResult {
    pub plugin_name: String,
    pub passed: bool,
    pub checks: Vec<CheckResult>,
    pub summary: String,
}

/// Run verification for a plugin
pub fn verify_plugin(plugin_name: &str, plugin_path: &Path, spec: &VerificationSpec) -> Result<VerificationResult> {
    let mut checks = Vec::new();
    let checks_spec = &spec.checks;

    // File checks
    for file in &checks_spec.files {
        let file_path = plugin_path.join(file);
        let passed = file_path.exists();
        checks.push(CheckResult {
            name: format!("file: {}", file),
            passed,
            message: if passed {
                None
            } else {
                Some(format!("File not found: {}", file_path.display()))
            },
        });
    }

    // Environment variable checks
    for var in &checks_spec.env_vars {
        let passed = std::env::var(var).is_ok();
        checks.push(CheckResult {
            name: format!("env: {}", var),
            passed,
            message: if passed {
                None
            } else {
                Some(format!("Environment variable not set: {}", var))
            },
        });
    }

    // Command checks
    for cmd in &checks_spec.commands {
        let result = run_verification_command(cmd)?;
        checks.push(result);
    }

    let passed_count = checks.iter().filter(|c| c.passed).count();
    let total_count = checks.len();
    let all_passed = passed_count == total_count;

    let summary = format!("{}/{} checks passed", passed_count, total_count);

    Ok(VerificationResult {
        plugin_name: plugin_name.to_string(),
        passed: all_passed,
        checks,
        summary,
    })
}

/// Run a single verification command
fn run_verification_command(cmd: &VerificationCommand) -> Result<CheckResult> {
    let output = Command::new("sh")
        .arg("-c")
        .arg(&cmd.command)
        .output()
        .with_context(|| format!("Failed to execute command: {}", cmd.command))?;

    let exit_code = output.status.code().unwrap_or(-1);
    let stdout = String::from_utf8_lossy(&output.stdout).to_string();

    let mut passed = true;
    let mut messages = Vec::new();

    // Check exit code
    let expected_exit = cmd.expect_exit.unwrap_or(0);
    if exit_code != expected_exit {
        passed = false;
        messages.push(format!("Exit code {} (expected {})", exit_code, expected_exit));
    }

    // Check output contains expected string
    if let Some(ref expected) = cmd.expect_contains
        && !stdout.contains(expected)
    {
        passed = false;
        messages.push(format!("Output does not contain: {}", expected));
    }

    Ok(CheckResult {
        name: cmd.name.clone(),
        passed,
        message: if messages.is_empty() { None } else { Some(messages.join("; ")) },
    })
}

/// Print verification result to terminal
pub fn print_verification_result(result: &VerificationResult) {
    println!("Verifying plugin: {}\n", result.plugin_name.cyan().bold());

    // Group checks by type
    let file_checks: Vec<_> = result.checks.iter().filter(|c| c.name.starts_with("file:")).collect();
    let env_checks: Vec<_> = result.checks.iter().filter(|c| c.name.starts_with("env:")).collect();
    let cmd_checks: Vec<_> = result
        .checks
        .iter()
        .filter(|c| !c.name.starts_with("file:") && !c.name.starts_with("env:"))
        .collect();

    if !file_checks.is_empty() {
        println!("{}:", "File Checks".bold());
        for check in file_checks {
            print_check(check);
        }
        println!();
    }

    if !env_checks.is_empty() {
        println!("{}:", "Environment Checks".bold());
        for check in env_checks {
            print_check(check);
        }
        println!();
    }

    if !cmd_checks.is_empty() {
        println!("{}:", "Command Checks".bold());
        for check in cmd_checks {
            print_check(check);
        }
        println!();
    }

    // Summary
    if result.passed {
        println!("Result: {} ({})", "PASSED".green().bold(), result.summary.dimmed());
    } else {
        println!("Result: {} ({})", "FAILED".red().bold(), result.summary.dimmed());
    }
}

fn print_check(check: &CheckResult) {
    let icon = if check.passed { "✓".green() } else { "✗".red() };
    let name = check.name.trim_start_matches("file: ").trim_start_matches("env: ");

    if let Some(ref msg) = check.message {
        println!("  {} {} - {}", icon, name, msg.dimmed());
    } else {
        println!("  {} {}", icon, name);
    }
}

/// Check if verification spec has any checks defined
pub fn has_checks(spec: &VerificationSpec) -> bool {
    !spec.checks.files.is_empty() || !spec.checks.commands.is_empty() || !spec.checks.env_vars.is_empty()
}

/// Read and display the verification guide (verify.md)
pub fn read_verification_guide(plugin_path: &Path, guide_path: &str) -> Result<String> {
    let full_path = plugin_path.join(guide_path);
    std::fs::read_to_string(&full_path)
        .with_context(|| format!("Failed to read verification guide: {}", full_path.display()))
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::plugin::manifest::VerificationChecks;
    use tempfile::tempdir;

    #[test]
    fn test_file_check_passes_when_file_exists() {
        let temp = tempdir().unwrap();
        std::fs::write(temp.path().join("test.txt"), "content").unwrap();

        let spec = VerificationSpec {
            guide: None,
            checks: VerificationChecks {
                files: vec!["test.txt".to_string()],
                commands: vec![],
                env_vars: vec![],
            },
        };

        let result = verify_plugin("test-plugin", temp.path(), &spec).unwrap();
        assert!(result.passed);
        assert_eq!(result.checks.len(), 1);
        assert!(result.checks[0].passed);
    }

    #[test]
    fn test_file_check_fails_when_file_missing() {
        let temp = tempdir().unwrap();

        let spec = VerificationSpec {
            guide: None,
            checks: VerificationChecks {
                files: vec!["missing.txt".to_string()],
                commands: vec![],
                env_vars: vec![],
            },
        };

        let result = verify_plugin("test-plugin", temp.path(), &spec).unwrap();
        assert!(!result.passed);
        assert!(!result.checks[0].passed);
    }

    #[test]
    fn test_command_check_exit_code() {
        let temp = tempdir().unwrap();

        let spec = VerificationSpec {
            guide: None,
            checks: VerificationChecks {
                files: vec![],
                commands: vec![VerificationCommand {
                    name: "true-command".to_string(),
                    command: "true".to_string(),
                    expect_exit: Some(0),
                    expect_contains: None,
                }],
                env_vars: vec![],
            },
        };

        let result = verify_plugin("test-plugin", temp.path(), &spec).unwrap();
        assert!(result.passed);
    }

    #[test]
    fn test_command_check_output_contains() {
        let temp = tempdir().unwrap();

        let spec = VerificationSpec {
            guide: None,
            checks: VerificationChecks {
                files: vec![],
                commands: vec![VerificationCommand {
                    name: "echo-test".to_string(),
                    command: "echo 'hello world'".to_string(),
                    expect_exit: None,
                    expect_contains: Some("hello".to_string()),
                }],
                env_vars: vec![],
            },
        };

        let result = verify_plugin("test-plugin", temp.path(), &spec).unwrap();
        assert!(result.passed);
    }

    #[test]
    fn test_has_checks() {
        let empty_spec = VerificationSpec::default();
        assert!(!has_checks(&empty_spec));

        let spec_with_files = VerificationSpec {
            guide: None,
            checks: VerificationChecks {
                files: vec!["test.txt".to_string()],
                commands: vec![],
                env_vars: vec![],
            },
        };
        assert!(has_checks(&spec_with_files));
    }
}
