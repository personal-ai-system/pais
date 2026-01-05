//! Bundle manager for discovery and installation

use colored::*;
use eyre::{Context, Result};
use std::collections::HashMap;
use std::fs;
use std::path::PathBuf;
use std::process::Command;

use super::manifest::BundleManifest;

/// A discovered bundle
#[derive(Debug)]
pub struct DiscoveredBundle {
    pub manifest: BundleManifest,
    #[allow(dead_code)] // Used for future remote plugin installation
    pub path: PathBuf,
}

/// Bundle manager for discovery and installation
pub struct BundleManager {
    pub bundles: HashMap<String, DiscoveredBundle>,
    pub bundles_dir: PathBuf,
    pub plugins_dir: PathBuf,
}

impl BundleManager {
    pub fn new(bundles_dir: PathBuf, plugins_dir: PathBuf) -> Self {
        Self {
            bundles: HashMap::new(),
            bundles_dir,
            plugins_dir,
        }
    }

    /// Discover all bundles in the bundles directory
    pub fn discover(&mut self) -> Result<usize> {
        self.bundles.clear();

        if !self.bundles_dir.exists() {
            return Ok(0);
        }

        let mut count = 0;

        for entry in fs::read_dir(&self.bundles_dir).context("Failed to read bundles directory")? {
            let entry = entry?;
            let path = entry.path();

            if !path.is_dir() {
                continue;
            }

            let manifest_path = path.join("bundle.yaml");
            if !manifest_path.exists() {
                continue;
            }

            match BundleManifest::load(&manifest_path) {
                Ok(manifest) => {
                    let name = manifest.bundle.name.clone();
                    self.bundles.insert(name, DiscoveredBundle { manifest, path });
                    count += 1;
                }
                Err(e) => {
                    log::warn!("Failed to load bundle manifest {}: {}", manifest_path.display(), e);
                }
            }
        }

        Ok(count)
    }

    /// Get a bundle by name
    pub fn get(&self, name: &str) -> Option<&DiscoveredBundle> {
        self.bundles.get(name)
    }

    /// List all bundles
    pub fn list(&self) -> impl Iterator<Item = &DiscoveredBundle> {
        self.bundles.values()
    }

    /// Install a bundle
    pub fn install(&self, name: &str, required_only: bool, skip_verify: bool) -> Result<InstallResult> {
        let bundle = self
            .get(name)
            .ok_or_else(|| eyre::eyre!("Bundle not found: {}", name))?;

        let mut result = InstallResult {
            bundle_name: name.to_string(),
            installed: Vec::new(),
            skipped: Vec::new(),
            failed: Vec::new(),
        };

        // Get plugins to install
        let plugins_to_install: Vec<_> = if required_only {
            bundle.manifest.required_plugins()
        } else {
            bundle.manifest.all_plugins()
        };

        let total = plugins_to_install.len();

        for (index, (plugin_name, plugin_ref)) in plugins_to_install.iter().enumerate() {
            let step = index + 1;
            let required_str = if plugin_ref.required { "" } else { " (optional)" };

            println!(
                "\n[{}/{}] Installing {}{}...",
                step,
                total,
                plugin_name.cyan(),
                required_str.dimmed()
            );

            // Check if plugin is already installed
            let plugin_path = self.plugins_dir.join(plugin_name);
            if plugin_path.exists() {
                println!("  {} Already installed", "→".blue());
                result.skipped.push(plugin_name.to_string());
                continue;
            }

            // For now, we only support local plugins
            // Check if plugin exists in plugins directory or as a known location
            if plugin_ref.source.is_some() {
                println!("  {} Remote sources not yet supported", "⚠".yellow());
                result.skipped.push(plugin_name.to_string());
                continue;
            }

            // Try to find plugin in common locations
            // For now, just report that it needs manual installation
            println!("  {} Plugin not found. Install manually:", "!".yellow());
            println!("    pais plugin install /path/to/{}", plugin_name);
            result.failed.push(plugin_name.to_string());
        }

        // Run verification if not skipped
        if !skip_verify && !result.installed.is_empty() {
            println!("\n{}", "Running verification...".bold());
            for plugin_name in &result.installed {
                let verify_output = Command::new("pais").args(["plugin", "verify", plugin_name]).output();

                match verify_output {
                    Ok(output) if output.status.success() => {
                        println!("  {} {} verified", "✓".green(), plugin_name);
                    }
                    _ => {
                        println!("  {} {} verification failed", "✗".red(), plugin_name);
                    }
                }
            }
        }

        // Run post-install commands
        if !bundle.manifest.post_install.is_empty() {
            println!("\n{}", "Post-install:".bold());
            for cmd in &bundle.manifest.post_install {
                let desc = cmd.description.as_deref().unwrap_or(&cmd.command);
                print!("  - {}... ", desc);

                let output = Command::new("sh").arg("-c").arg(&cmd.command).output();

                match output {
                    Ok(o) if o.status.success() => {
                        println!("{}", "done".green());
                    }
                    _ => {
                        println!("{}", "failed".red());
                    }
                }
            }
        }

        Ok(result)
    }
}

/// Result of bundle installation
#[derive(Debug)]
pub struct InstallResult {
    pub bundle_name: String,
    pub installed: Vec<String>,
    pub skipped: Vec<String>,
    pub failed: Vec<String>,
}

impl InstallResult {
    pub fn print_summary(&self) {
        println!();
        if self.failed.is_empty() && self.installed.is_empty() && !self.skipped.is_empty() {
            println!(
                "Bundle '{}': All {} plugins already installed",
                self.bundle_name.cyan(),
                self.skipped.len()
            );
        } else if self.failed.is_empty() {
            println!(
                "Bundle '{}' installed successfully ({}/{} plugins)",
                self.bundle_name.green(),
                self.installed.len(),
                self.installed.len() + self.skipped.len()
            );
        } else {
            println!(
                "Bundle '{}' partially installed ({} installed, {} failed, {} skipped)",
                self.bundle_name.yellow(),
                self.installed.len(),
                self.failed.len(),
                self.skipped.len()
            );
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::tempdir;

    fn create_test_bundle(dir: &std::path::Path, name: &str) {
        let bundle_dir = dir.join(name);
        fs::create_dir_all(&bundle_dir).unwrap();

        let manifest = format!(
            r#"
bundle:
  name: {}
  version: 1.0.0
  description: Test bundle

plugins:
  test-plugin:
    required: true
"#,
            name
        );
        fs::write(bundle_dir.join("bundle.yaml"), manifest).unwrap();
    }

    #[test]
    fn test_discover_bundles() {
        let temp = tempdir().unwrap();
        create_test_bundle(temp.path(), "bundle-a");
        create_test_bundle(temp.path(), "bundle-b");

        let mut manager = BundleManager::new(temp.path().to_path_buf(), temp.path().join("plugins"));
        let count = manager.discover().unwrap();

        assert_eq!(count, 2);
        assert!(manager.get("bundle-a").is_some());
        assert!(manager.get("bundle-b").is_some());
    }

    #[test]
    fn test_discover_empty_directory() {
        let temp = tempdir().unwrap();

        let mut manager = BundleManager::new(temp.path().to_path_buf(), temp.path().join("plugins"));
        let count = manager.discover().unwrap();

        assert_eq!(count, 0);
    }
}
