//! Skill loading and discovery
//!
//! Loads skills from:
//! - ~/.config/pais/skills/ (simple skills)
//! - ~/.config/pais/plugins/ (plugin skills)
//! - .pais/ directories in repos (discovered skills)

use eyre::{Context, Result};
use std::fs;
use std::path::Path;

use super::parser::{has_skill_md, is_simple_skill, parse_skill_md};
use super::{Skill, SkillSource};

/// Load a simple skill from a directory containing SKILL.md
pub fn load_simple_skill(path: &Path) -> Result<Skill> {
    let skill_md = path.join("SKILL.md");

    if !skill_md.exists() {
        eyre::bail!("No SKILL.md found in {}", path.display());
    }

    let metadata = parse_skill_md(&skill_md)?;

    Ok(Skill {
        name: metadata.name,
        description: metadata.description,
        path: path.to_path_buf(),
        source: SkillSource::Simple,
    })
}

/// Load a skill that's part of a plugin
pub fn load_plugin_skill(path: &Path, plugin_name: &str) -> Result<Skill> {
    let skill_md = path.join("SKILL.md");

    if !skill_md.exists() {
        eyre::bail!("No SKILL.md found in plugin {}", plugin_name);
    }

    let metadata = parse_skill_md(&skill_md)?;

    Ok(Skill {
        name: metadata.name,
        description: metadata.description,
        path: path.to_path_buf(),
        source: SkillSource::Plugin(plugin_name.to_string()),
    })
}

/// Discover all simple skills in a directory
pub fn discover_simple_skills(skills_dir: &Path) -> Result<Vec<Skill>> {
    let mut skills = Vec::new();

    if !skills_dir.exists() {
        return Ok(skills);
    }

    for entry in fs::read_dir(skills_dir)
        .with_context(|| format!("Failed to read skills directory: {}", skills_dir.display()))?
    {
        let entry = entry?;
        let path = entry.path();

        if path.is_dir() && is_simple_skill(&path) {
            match load_simple_skill(&path) {
                Ok(skill) => skills.push(skill),
                Err(e) => {
                    log::warn!("Failed to load skill from {}: {}", path.display(), e);
                }
            }
        }
    }

    Ok(skills)
}

/// Discover skills from plugins directory
pub fn discover_plugin_skills(plugins_dir: &Path) -> Result<Vec<Skill>> {
    let mut skills = Vec::new();

    if !plugins_dir.exists() {
        return Ok(skills);
    }

    for entry in fs::read_dir(plugins_dir)
        .with_context(|| format!("Failed to read plugins directory: {}", plugins_dir.display()))?
    {
        let entry = entry?;
        let path = entry.path();

        if path.is_dir() && has_skill_md(&path) {
            let plugin_name = path
                .file_name()
                .map(|n| n.to_string_lossy().to_string())
                .unwrap_or_default();

            match load_plugin_skill(&path, &plugin_name) {
                Ok(skill) => skills.push(skill),
                Err(e) => {
                    log::warn!("Failed to load skill from plugin {}: {}", plugin_name, e);
                }
            }
        }
    }

    Ok(skills)
}

/// Discover all skills (both simple and plugin)
pub fn discover_all_skills(skills_dir: &Path, plugins_dir: &Path) -> Result<Vec<Skill>> {
    let mut all_skills = discover_simple_skills(skills_dir)?;
    let plugin_skills = discover_plugin_skills(plugins_dir)?;
    all_skills.extend(plugin_skills);
    Ok(all_skills)
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs;
    use tempfile::TempDir;

    fn create_skill_md(dir: &Path, name: &str, description: &str) {
        let content = format!(
            r#"---
name: {}
description: {}
---

# {}

Content here
"#,
            name, description, name
        );
        fs::write(dir.join("SKILL.md"), content).unwrap();
    }

    #[test]
    fn test_load_simple_skill() {
        let temp = TempDir::new().unwrap();
        let skill_dir = temp.path().join("terraform");
        fs::create_dir_all(&skill_dir).unwrap();
        create_skill_md(&skill_dir, "terraform", "Terraform best practices");

        let skill = load_simple_skill(&skill_dir).unwrap();
        assert_eq!(skill.name, "terraform");
        assert_eq!(skill.description, "Terraform best practices");
        assert!(skill.is_simple());
    }

    #[test]
    fn test_load_plugin_skill() {
        let temp = TempDir::new().unwrap();
        let plugin_dir = temp.path().join("incident");
        fs::create_dir_all(&plugin_dir).unwrap();
        create_skill_md(&plugin_dir, "incident", "Incident management");
        // Add plugin.yaml to make it a plugin
        fs::write(
            plugin_dir.join("plugin.yaml"),
            "plugin:\n  name: incident\n  version: 0.1.0\n  description: test",
        )
        .unwrap();

        let skill = load_plugin_skill(&plugin_dir, "incident").unwrap();
        assert_eq!(skill.name, "incident");
        assert!(skill.is_plugin_skill());
    }

    #[test]
    fn test_discover_simple_skills() {
        let temp = TempDir::new().unwrap();
        let skills_dir = temp.path();

        // Create two simple skills
        let skill1 = skills_dir.join("terraform");
        fs::create_dir_all(&skill1).unwrap();
        create_skill_md(&skill1, "terraform", "Terraform");

        let skill2 = skills_dir.join("kubectl");
        fs::create_dir_all(&skill2).unwrap();
        create_skill_md(&skill2, "kubectl", "Kubernetes CLI");

        let skills = discover_simple_skills(skills_dir).unwrap();
        assert_eq!(skills.len(), 2);
    }

    #[test]
    fn test_discover_empty_directory() {
        let temp = TempDir::new().unwrap();
        let skills = discover_simple_skills(temp.path()).unwrap();
        assert!(skills.is_empty());
    }

    #[test]
    fn test_discover_nonexistent_directory() {
        let skills = discover_simple_skills(Path::new("/nonexistent/path")).unwrap();
        assert!(skills.is_empty());
    }
}
