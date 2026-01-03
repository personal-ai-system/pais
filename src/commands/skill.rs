//! Skill management commands

use eyre::{Context, Result};
use serde::Serialize;

use crate::cli::{OutputFormat, SkillAction};
use crate::config::Config;
use crate::skill::loader::{discover_plugin_skills, discover_simple_skills};
use crate::skill::{Skill, SkillSource};

/// Run a skill subcommand
pub fn run(action: SkillAction, config: &Config) -> Result<()> {
    match action {
        SkillAction::List { format, simple, plugin } => {
            list_skills(OutputFormat::resolve(format), simple, plugin, config)
        }
        SkillAction::Info { name } => show_skill_info(&name, config),
    }
}

/// Serializable skill info for JSON/YAML output
#[derive(Serialize)]
struct SkillInfo {
    name: String,
    description: String,
    path: String,
    source: String,
    source_detail: Option<String>,
}

impl From<&Skill> for SkillInfo {
    fn from(skill: &Skill) -> Self {
        let (source, source_detail) = match &skill.source {
            SkillSource::Simple => ("simple".to_string(), None),
            SkillSource::Plugin(name) => ("plugin".to_string(), Some(name.clone())),
            SkillSource::Discovered(path) => ("discovered".to_string(), Some(path.display().to_string())),
        };

        Self {
            name: skill.name.clone(),
            description: skill.description.clone(),
            path: skill.path.display().to_string(),
            source,
            source_detail,
        }
    }
}

/// List all skills
fn list_skills(format: OutputFormat, only_simple: bool, only_plugin: bool, config: &Config) -> Result<()> {
    let skills_dir = Config::expand_path(&config.paths.skills);
    let plugins_dir = Config::expand_path(&config.paths.plugins);

    let mut all_skills = Vec::new();

    // Get simple skills unless filtering to plugins only
    if !only_plugin {
        let simple_skills = discover_simple_skills(&skills_dir).context("Failed to discover simple skills")?;
        all_skills.extend(simple_skills);
    }

    // Get plugin skills unless filtering to simple only
    if !only_simple {
        let plugin_skills = discover_plugin_skills(&plugins_dir).context("Failed to discover plugin skills")?;
        all_skills.extend(plugin_skills);
    }

    match format {
        OutputFormat::Text => {
            if all_skills.is_empty() {
                println!("No skills found.");
                println!();
                println!("To create a skill:");
                println!("  mkdir -p ~/.config/pais/skills/myskill");
                println!("  # Create SKILL.md with frontmatter");
                return Ok(());
            }

            // Group by source type
            let simple: Vec<_> = all_skills.iter().filter(|s| s.is_simple()).collect();
            let plugin: Vec<_> = all_skills.iter().filter(|s| s.is_plugin_skill()).collect();
            let discovered: Vec<_> = all_skills.iter().filter(|s| s.is_discovered()).collect();

            if !simple.is_empty() {
                println!("Simple Skills (SKILL.md only):");
                for skill in &simple {
                    println!("  {} - {}", skill.name, skill.description);
                }
                println!();
            }

            if !plugin.is_empty() {
                println!("Plugin Skills:");
                for skill in &plugin {
                    if let SkillSource::Plugin(plugin_name) = &skill.source {
                        println!("  {} - {} (plugin: {})", skill.name, skill.description, plugin_name);
                    }
                }
                println!();
            }

            if !discovered.is_empty() {
                println!("Discovered Skills:");
                for skill in &discovered {
                    if let SkillSource::Discovered(repo_path) = &skill.source {
                        println!("  {} - {} ({})", skill.name, skill.description, repo_path.display());
                    }
                }
                println!();
            }

            println!("Total: {} skill(s)", all_skills.len());
        }
        OutputFormat::Json => {
            let infos: Vec<SkillInfo> = all_skills.iter().map(SkillInfo::from).collect();
            println!("{}", serde_json::to_string_pretty(&infos)?);
        }
        OutputFormat::Yaml => {
            let infos: Vec<SkillInfo> = all_skills.iter().map(SkillInfo::from).collect();
            println!("{}", serde_yaml::to_string(&infos)?);
        }
    }

    Ok(())
}

/// Show details for a specific skill
fn show_skill_info(name: &str, config: &Config) -> Result<()> {
    let skills_dir = Config::expand_path(&config.paths.skills);
    let plugins_dir = Config::expand_path(&config.paths.plugins);

    // Check simple skills first
    let skill_path = skills_dir.join(name);
    if skill_path.exists() && skill_path.join("SKILL.md").exists() {
        let skill = crate::skill::loader::load_simple_skill(&skill_path)?;
        print_skill_details(&skill)?;
        return Ok(());
    }

    // Check plugins
    let plugin_path = plugins_dir.join(name);
    if plugin_path.exists() && plugin_path.join("SKILL.md").exists() {
        let skill = crate::skill::loader::load_plugin_skill(&plugin_path, name)?;
        print_skill_details(&skill)?;
        return Ok(());
    }

    eyre::bail!("Skill '{}' not found", name);
}

fn print_skill_details(skill: &Skill) -> Result<()> {
    println!("Name: {}", skill.name);
    println!("Description: {}", skill.description);
    println!("Path: {}", skill.path.display());
    println!(
        "Type: {}",
        match &skill.source {
            SkillSource::Simple => "Simple (SKILL.md only)".to_string(),
            SkillSource::Plugin(name) => format!("Plugin ({})", name),
            SkillSource::Discovered(path) => format!("Discovered ({})", path.display()),
        }
    );

    // Show SKILL.md content preview
    let skill_md = skill.path.join("SKILL.md");
    if skill_md.exists() {
        println!();
        println!("SKILL.md preview:");
        println!("─────────────────");
        let content = std::fs::read_to_string(&skill_md)?;
        // Show first 20 lines
        for (i, line) in content.lines().take(20).enumerate() {
            println!("{}", line);
            if i == 19 {
                println!("... (truncated)");
            }
        }
    }

    Ok(())
}
