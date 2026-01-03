//! Skill scanning - discover .pais/SKILL.md files in repositories
//!
//! Scans directories to find skills defined in repositories you control.

use eyre::Result;
use serde::Serialize;
use std::path::{Path, PathBuf};
use walkdir::{DirEntry, WalkDir};

use super::parser::parse_skill_md;

/// A skill discovered via scanning
#[derive(Debug, Clone, Serialize)]
pub struct DiscoveredSkill {
    /// Skill name (from frontmatter)
    pub name: String,
    /// Skill description
    pub description: String,
    /// Path to the .pais directory containing the skill
    pub pais_path: PathBuf,
    /// Path to the repository root
    pub repo_path: PathBuf,
}

/// Scan a directory for .pais/SKILL.md files
pub fn scan_for_skills(root: &Path, max_depth: usize) -> Result<Vec<DiscoveredSkill>> {
    let mut found = Vec::new();

    if !root.exists() {
        return Ok(found);
    }

    // Use filter_entry to skip ignored directories, but still enter .pais
    let walker = WalkDir::new(root)
        .max_depth(max_depth)
        .into_iter()
        .filter_entry(should_enter);

    for entry in walker {
        let entry = match entry {
            Ok(e) => e,
            Err(e) => {
                log::debug!("Error walking directory: {}", e);
                continue;
            }
        };

        let path = entry.path();

        // Look for .pais directories
        if entry.file_type().is_dir() && path.file_name().map(|n| n == ".pais").unwrap_or(false) {
            let skill_md = path.join("SKILL.md");
            if skill_md.exists() {
                // Found a .pais/SKILL.md
                match parse_discovered_skill(&skill_md, path) {
                    Ok(skill) => {
                        log::info!("Found skill: {} at {}", skill.name, skill_md.display());
                        found.push(skill);
                    }
                    Err(e) => {
                        log::warn!("Failed to parse skill at {}: {}", skill_md.display(), e);
                    }
                }
            }
        }
    }

    Ok(found)
}

/// Parse a discovered SKILL.md file
fn parse_discovered_skill(skill_md_path: &Path, pais_dir: &Path) -> Result<DiscoveredSkill> {
    let metadata = parse_skill_md(skill_md_path)?;

    // Repo root is parent of .pais directory
    let repo_path = pais_dir
        .parent()
        .ok_or_else(|| eyre::eyre!("Cannot determine repo path for {}", pais_dir.display()))?
        .to_path_buf();

    Ok(DiscoveredSkill {
        name: metadata.name,
        description: metadata.description,
        pais_path: pais_dir.to_path_buf(),
        repo_path,
    })
}

/// Check if we should enter a directory during scanning
fn should_enter(entry: &DirEntry) -> bool {
    // Always process the root (depth 0)
    if entry.depth() == 0 {
        return true;
    }

    let name = entry.file_name().to_string_lossy();

    // Always enter .pais directories
    if name == ".pais" {
        return true;
    }

    // Skip other hidden directories
    if name.starts_with('.') {
        return false;
    }

    // Skip common non-repo directories
    !matches!(
        name.as_ref(),
        "node_modules" | "target" | "venv" | "__pycache__" | "dist" | "build"
    )
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs;
    use tempfile::TempDir;

    fn create_pais_skill(repo_dir: &Path, name: &str, description: &str) {
        let pais_dir = repo_dir.join(".pais");
        fs::create_dir_all(&pais_dir).unwrap();
        fs::write(
            pais_dir.join("SKILL.md"),
            format!("---\nname: {}\ndescription: {}\n---\n# {}\n", name, description, name),
        )
        .unwrap();
    }

    #[test]
    fn test_scan_finds_pais_skill() {
        let temp = TempDir::new().unwrap();

        // Create a repo with .pais/SKILL.md
        let repo = temp.path().join("my-tool");
        fs::create_dir_all(&repo).unwrap();
        create_pais_skill(&repo, "my-tool", "A cool tool");

        let skills = scan_for_skills(temp.path(), 4).unwrap();
        assert_eq!(skills.len(), 1);
        assert_eq!(skills[0].name, "my-tool");
        assert_eq!(skills[0].description, "A cool tool");
        assert_eq!(skills[0].repo_path, repo);
    }

    #[test]
    fn test_scan_finds_multiple_skills() {
        let temp = TempDir::new().unwrap();

        // Create multiple repos
        for name in &["tool-a", "tool-b", "tool-c"] {
            let repo = temp.path().join(name);
            fs::create_dir_all(&repo).unwrap();
            create_pais_skill(&repo, name, &format!("{} description", name));
        }

        let skills = scan_for_skills(temp.path(), 4).unwrap();
        assert_eq!(skills.len(), 3);
    }

    #[test]
    fn test_scan_ignores_hidden_dirs() {
        let temp = TempDir::new().unwrap();

        // Create a repo in a hidden directory (should be ignored)
        let hidden = temp.path().join(".hidden-repo");
        fs::create_dir_all(&hidden).unwrap();
        create_pais_skill(&hidden, "hidden", "Should not be found");

        // Create a normal repo
        let normal = temp.path().join("normal-repo");
        fs::create_dir_all(&normal).unwrap();
        create_pais_skill(&normal, "normal", "Should be found");

        let skills = scan_for_skills(temp.path(), 4).unwrap();
        assert_eq!(skills.len(), 1);
        assert_eq!(skills[0].name, "normal");
    }

    #[test]
    fn test_scan_ignores_node_modules() {
        let temp = TempDir::new().unwrap();

        // Create a skill inside node_modules (should be ignored)
        let node_modules = temp.path().join("node_modules").join("some-dep");
        fs::create_dir_all(&node_modules).unwrap();
        create_pais_skill(&node_modules, "dep", "Should not be found");

        let skills = scan_for_skills(temp.path(), 4).unwrap();
        assert!(skills.is_empty());
    }

    #[test]
    fn test_scan_respects_max_depth() {
        let temp = TempDir::new().unwrap();

        // Create a deeply nested repo
        let deep = temp.path().join("a").join("b").join("c").join("d").join("e");
        fs::create_dir_all(&deep).unwrap();
        create_pais_skill(&deep, "deep", "Too deep");

        // With max_depth=4, shouldn't find it (5 levels deep)
        let skills = scan_for_skills(temp.path(), 4).unwrap();
        assert!(skills.is_empty());

        // With max_depth=7, should find it
        let skills = scan_for_skills(temp.path(), 7).unwrap();
        assert_eq!(skills.len(), 1);
    }

    #[test]
    fn test_scan_nonexistent_directory() {
        let skills = scan_for_skills(Path::new("/nonexistent/path"), 4).unwrap();
        assert!(skills.is_empty());
    }

    #[test]
    fn test_should_enter() {
        // Hidden directories (except .pais)
        assert!(!should_enter(&make_nested_entry(".git")));
        assert!(!should_enter(&make_nested_entry(".hidden")));
        assert!(should_enter(&make_nested_entry(".pais")));

        // Common ignored directories
        assert!(!should_enter(&make_nested_entry("node_modules")));
        assert!(!should_enter(&make_nested_entry("target")));
        assert!(!should_enter(&make_nested_entry("venv")));
        assert!(!should_enter(&make_nested_entry("__pycache__")));

        // Normal directories should be entered
        assert!(should_enter(&make_nested_entry("src")));
        assert!(should_enter(&make_nested_entry("lib")));
    }

    // Helper to create a DirEntry at depth > 0 for testing should_enter
    fn make_nested_entry(name: &str) -> DirEntry {
        let temp = TempDir::new().unwrap();
        let parent = temp.path().join("parent");
        let path = parent.join(name);
        fs::create_dir_all(&path).unwrap();

        // Walk from temp to get a depth > 0 entry
        WalkDir::new(temp.path())
            .into_iter()
            .filter_map(|e| e.ok())
            .find(|e| e.file_name().to_string_lossy() == name)
            .unwrap()
    }
}
