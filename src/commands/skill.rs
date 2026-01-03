//! Skill management commands

use eyre::{Context, Result};
use serde::Serialize;
use std::fs;
use std::io::{self, Write};
use std::process::Command;

use std::path::PathBuf;

use crate::cli::{OutputFormat, SkillAction};
use crate::config::Config;
use crate::skill::indexer::{generate_context_snippet, generate_index, write_index};
use crate::skill::loader::{discover_plugin_skills, discover_simple_skills, load_simple_skill};
use crate::skill::parser::{SkillMetadata, parse_skill_md};
use crate::skill::scanner::{DiscoveredSkill, scan_for_skills};
use crate::skill::template::generate_skill_template;
use crate::skill::workflow::{discover_workflows, load_workflow};
use crate::skill::{Skill, SkillSource};

/// Run a skill subcommand
pub fn run(action: SkillAction, config: &Config) -> Result<()> {
    match action {
        SkillAction::List { format, simple, plugin } => {
            list_skills(OutputFormat::resolve(format), simple, plugin, config)
        }
        SkillAction::Add { name, edit } => add_skill(&name, edit, config),
        SkillAction::Info { name } => show_skill_info(&name, config),
        SkillAction::Edit { name } => edit_skill(&name, config),
        SkillAction::Remove { name, force } => remove_skill(&name, force, config),
        SkillAction::Validate { name } => validate_skill(&name, config),
        SkillAction::Scan {
            path,
            depth,
            register,
            format,
        } => scan_skills(path, depth, register, OutputFormat::resolve(format), config),
        SkillAction::Index { format } => generate_skill_index(OutputFormat::resolve(format), config),
        SkillAction::Workflow {
            skill,
            workflow,
            format,
        } => show_workflow(&skill, workflow.as_deref(), OutputFormat::resolve(format), config),
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
                println!("  pais skill add <name>");
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

/// Create a new skill from template
fn add_skill(name: &str, open_editor: bool, config: &Config) -> Result<()> {
    let skills_dir = Config::expand_path(&config.paths.skills);
    let skill_dir = skills_dir.join(name);

    // Check if skill already exists
    if skill_dir.exists() {
        eyre::bail!(
            "Skill '{}' already exists at {}\nUse 'pais skill edit {}' to modify it.",
            name,
            skill_dir.display(),
            name
        );
    }

    // Create directory
    fs::create_dir_all(&skill_dir)
        .with_context(|| format!("Failed to create skill directory: {}", skill_dir.display()))?;

    // Generate and write template
    let template = generate_skill_template(name);
    let skill_md = skill_dir.join("SKILL.md");
    fs::write(&skill_md, &template).with_context(|| format!("Failed to write SKILL.md: {}", skill_md.display()))?;

    println!("Created skill: {}", name);
    println!("  Path: {}", skill_md.display());

    if open_editor {
        println!();
        open_in_editor(&skill_md)?;
    } else {
        println!();
        println!("Edit with: pais skill edit {}", name);
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
        let skill = load_simple_skill(&skill_path)?;
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

/// Edit a skill in $EDITOR
fn edit_skill(name: &str, config: &Config) -> Result<()> {
    let skills_dir = Config::expand_path(&config.paths.skills);
    let plugins_dir = Config::expand_path(&config.paths.plugins);

    // Check simple skills first
    let skill_path = skills_dir.join(name);
    let skill_md = skill_path.join("SKILL.md");
    if skill_md.exists() {
        return open_in_editor(&skill_md);
    }

    // Check plugins
    let plugin_path = plugins_dir.join(name);
    let plugin_skill_md = plugin_path.join("SKILL.md");
    if plugin_skill_md.exists() {
        return open_in_editor(&plugin_skill_md);
    }

    eyre::bail!("Skill '{}' not found.\nCreate it with: pais skill add {}", name, name);
}

/// Remove a skill
fn remove_skill(name: &str, force: bool, config: &Config) -> Result<()> {
    let skills_dir = Config::expand_path(&config.paths.skills);
    let skill_path = skills_dir.join(name);

    if !skill_path.exists() {
        eyre::bail!("Skill '{}' not found in {}", name, skills_dir.display());
    }

    // Check if it's a plugin skill (can't remove those via skill remove)
    let plugins_dir = Config::expand_path(&config.paths.plugins);
    let plugin_path = plugins_dir.join(name);
    if plugin_path.exists() && plugin_path.join("plugin.yaml").exists() {
        eyre::bail!(
            "Skill '{}' is part of a plugin. Use 'pais plugin remove {}' instead.",
            name,
            name
        );
    }

    // Confirm removal unless forced
    if !force {
        print!("Remove skill '{}' at {}? [y/N] ", name, skill_path.display());
        io::stdout().flush()?;

        let mut input = String::new();
        io::stdin().read_line(&mut input)?;

        if !input.trim().eq_ignore_ascii_case("y") {
            println!("Cancelled.");
            return Ok(());
        }
    }

    // Remove the skill directory
    fs::remove_dir_all(&skill_path)
        .with_context(|| format!("Failed to remove skill directory: {}", skill_path.display()))?;

    println!("Removed skill: {}", name);
    Ok(())
}

/// Validate SKILL.md format
fn validate_skill(name: &str, config: &Config) -> Result<()> {
    let skills_dir = Config::expand_path(&config.paths.skills);
    let plugins_dir = Config::expand_path(&config.paths.plugins);

    if name == "all" {
        // Validate all skills
        let mut errors = Vec::new();
        let mut valid_count = 0;

        // Check simple skills
        if skills_dir.exists() {
            for entry in fs::read_dir(&skills_dir)? {
                let entry = entry?;
                let path = entry.path();
                if path.is_dir() {
                    let skill_md = path.join("SKILL.md");
                    if skill_md.exists() {
                        match validate_skill_md(&skill_md) {
                            Ok(_) => {
                                valid_count += 1;
                                println!("✓ {}", entry.file_name().to_string_lossy());
                            }
                            Err(e) => {
                                errors.push((entry.file_name().to_string_lossy().to_string(), e));
                            }
                        }
                    }
                }
            }
        }

        // Check plugin skills
        if plugins_dir.exists() {
            for entry in fs::read_dir(&plugins_dir)? {
                let entry = entry?;
                let path = entry.path();
                if path.is_dir() {
                    let skill_md = path.join("SKILL.md");
                    if skill_md.exists() {
                        match validate_skill_md(&skill_md) {
                            Ok(_) => {
                                valid_count += 1;
                                println!("✓ {} (plugin)", entry.file_name().to_string_lossy());
                            }
                            Err(e) => {
                                errors.push((format!("{} (plugin)", entry.file_name().to_string_lossy()), e));
                            }
                        }
                    }
                }
            }
        }

        println!();
        if errors.is_empty() {
            println!("✓ All {} skill(s) valid", valid_count);
        } else {
            println!("Validation errors:");
            for (name, err) in &errors {
                println!("  ✗ {}: {}", name, err);
            }
            println!();
            println!("{} valid, {} with errors", valid_count, errors.len());
            eyre::bail!("Some skills have validation errors");
        }
    } else {
        // Validate single skill
        let skill_path = skills_dir.join(name);
        let skill_md = skill_path.join("SKILL.md");

        if !skill_md.exists() {
            // Check plugins
            let plugin_path = plugins_dir.join(name);
            let plugin_skill_md = plugin_path.join("SKILL.md");
            if plugin_skill_md.exists() {
                validate_skill_md(&plugin_skill_md)?;
                println!("✓ Skill '{}' (plugin) is valid", name);
                return Ok(());
            }
            eyre::bail!("Skill '{}' not found", name);
        }

        validate_skill_md(&skill_md)?;
        println!("✓ Skill '{}' is valid", name);
    }

    Ok(())
}

/// Validate a single SKILL.md file
fn validate_skill_md(path: &std::path::Path) -> Result<SkillMetadata> {
    parse_skill_md(path)
}

/// Open a file in the user's preferred editor
fn open_in_editor(path: &std::path::Path) -> Result<()> {
    let editor = std::env::var("EDITOR")
        .or_else(|_| std::env::var("VISUAL"))
        .unwrap_or_else(|_| "vi".to_string());

    println!("Opening in {}...", editor);

    let status = Command::new(&editor)
        .arg(path)
        .status()
        .with_context(|| format!("Failed to open editor: {}", editor))?;

    if !status.success() {
        eyre::bail!("Editor exited with non-zero status");
    }

    Ok(())
}

/// Scan directories for .pais/SKILL.md files
fn scan_skills(
    path: Option<PathBuf>,
    depth: usize,
    register: bool,
    format: OutputFormat,
    config: &Config,
) -> Result<()> {
    // Default to ~/repos if no path provided
    let scan_path = path.unwrap_or_else(|| dirs::home_dir().unwrap_or_else(|| PathBuf::from(".")).join("repos"));

    let scan_path = Config::expand_path(&scan_path);

    if !scan_path.exists() {
        eyre::bail!("Path does not exist: {}", scan_path.display());
    }

    println!("Scanning {} (max depth: {})...", scan_path.display(), depth);
    println!();

    let skills = scan_for_skills(&scan_path, depth).context("Failed to scan for skills")?;

    if skills.is_empty() {
        println!("No skills found.");
        println!();
        println!("To create a skill in a repo, add a .pais/SKILL.md file:");
        println!("  mkdir -p /path/to/repo/.pais");
        println!("  # Create .pais/SKILL.md with frontmatter");
        return Ok(());
    }

    match format {
        OutputFormat::Text => {
            println!("Found {} skill(s):", skills.len());
            println!();
            for skill in &skills {
                println!("  {} - {}", skill.name, skill.description);
                println!("    Repo: {}", skill.repo_path.display());
                println!("    Path: {}", skill.pais_path.display());
                println!();
            }
        }
        OutputFormat::Json => {
            println!("{}", serde_json::to_string_pretty(&skills)?);
        }
        OutputFormat::Yaml => {
            println!("{}", serde_yaml::to_string(&skills)?);
        }
    }

    if register {
        register_discovered_skills(&skills, config)?;
    } else if format == OutputFormat::Text {
        println!("To register these skills, run:");
        println!("  pais skill scan {} --register", scan_path.display());
    }

    Ok(())
}

/// Register discovered skills by creating symlinks in the skills directory
fn register_discovered_skills(skills: &[DiscoveredSkill], config: &Config) -> Result<()> {
    let skills_dir = Config::expand_path(&config.paths.skills);
    fs::create_dir_all(&skills_dir)
        .with_context(|| format!("Failed to create skills directory: {}", skills_dir.display()))?;

    let mut registered = 0;
    let mut skipped = 0;

    for skill in skills {
        let target = skills_dir.join(&skill.name);

        if target.exists() || target.symlink_metadata().is_ok() {
            println!("  Skipped: {} (already exists)", skill.name);
            skipped += 1;
            continue;
        }

        #[cfg(unix)]
        {
            std::os::unix::fs::symlink(&skill.pais_path, &target)
                .with_context(|| format!("Failed to create symlink for {}", skill.name))?;
        }

        #[cfg(not(unix))]
        {
            // On non-Unix, copy the files instead
            fs::create_dir_all(&target)?;
            let skill_md = skill.pais_path.join("SKILL.md");
            if skill_md.exists() {
                fs::copy(&skill_md, target.join("SKILL.md"))?;
            }
        }

        println!("  Registered: {} -> {}", skill.name, skill.pais_path.display());
        registered += 1;
    }

    println!();
    println!("Registered: {}, Skipped: {}", registered, skipped);

    if registered > 0 {
        println!();
        println!("Run 'pais sync' to sync registered skills to Claude Code.");
    }

    Ok(())
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
        let content = fs::read_to_string(&skill_md)?;
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

/// Generate skill index for context injection
fn generate_skill_index(format: OutputFormat, config: &Config) -> Result<()> {
    let skills_dir = Config::expand_path(&config.paths.skills);

    println!("Generating skill index from {}...", skills_dir.display());
    println!();

    let index = generate_index(&skills_dir).context("Failed to generate skill index")?;

    // Write the index file
    let index_path = skills_dir.join("skill-index.yaml");
    write_index(&index, &index_path).context("Failed to write skill index")?;

    match format {
        OutputFormat::Text => {
            println!("Skills indexed:");
            println!();

            // Group by tier
            let mut core: Vec<_> = index.skills.values().filter(|s| s.tier.is_core()).collect();
            let mut deferred: Vec<_> = index.skills.values().filter(|s| !s.tier.is_core()).collect();

            core.sort_by(|a, b| a.name.cmp(&b.name));
            deferred.sort_by(|a, b| a.name.cmp(&b.name));

            if !core.is_empty() {
                println!("Core Skills (always loaded):");
                for skill in &core {
                    println!("  🔒 {} - {}", skill.name, truncate_desc(&skill.description, 50));
                    if !skill.triggers.is_empty() {
                        println!("      Triggers: {}", skill.triggers.join(", "));
                    }
                }
                println!();
            }

            println!("Deferred Skills (loaded on match):");
            for skill in &deferred {
                println!("  📦 {} - {}", skill.name, truncate_desc(&skill.description, 50));
                if !skill.triggers.is_empty() {
                    println!("      Triggers: {}", skill.triggers.join(", "));
                }
            }
            println!();

            println!("Index written to: {}", index_path.display());
            println!();
            println!("Summary:");
            println!("  Total: {} skill(s)", index.total_skills);
            println!("  Core: {} skill(s)", index.core_count);
            println!("  Deferred: {} skill(s)", index.deferred_count);
            println!();

            // Also generate context snippet
            let context = generate_context_snippet(&index, &skills_dir);
            let context_path = skills_dir.join("context-snippet.md");
            fs::write(&context_path, &context)
                .with_context(|| format!("Failed to write context snippet: {}", context_path.display()))?;
            println!("Context snippet written to: {}", context_path.display());
            println!();
            println!("Next steps:");
            println!("  1. Run 'pais sync' to update Claude Code");
            println!("  2. The SessionStart hook will inject this context");
        }
        OutputFormat::Json => {
            println!("{}", serde_json::to_string_pretty(&index)?);
        }
        OutputFormat::Yaml => {
            println!("{}", serde_yaml::to_string(&index)?);
        }
    }

    Ok(())
}

fn truncate_desc(desc: &str, max_len: usize) -> String {
    if desc.len() <= max_len {
        desc.to_string()
    } else {
        format!("{}...", &desc[..max_len - 3])
    }
}

/// Show or list workflows for a skill
fn show_workflow(skill_name: &str, workflow: Option<&str>, format: OutputFormat, config: &Config) -> Result<()> {
    let skills_dir = Config::expand_path(&config.paths.skills);
    let plugins_dir = Config::expand_path(&config.paths.plugins);

    // Find the skill directory
    let skill_dir = {
        let simple_path = skills_dir.join(skill_name);
        let plugin_path = plugins_dir.join(skill_name);

        if simple_path.exists() && simple_path.join("SKILL.md").exists() {
            simple_path
        } else if plugin_path.exists() && plugin_path.join("SKILL.md").exists() {
            plugin_path
        } else {
            eyre::bail!("Skill '{}' not found", skill_name);
        }
    };

    // Discover workflows for this skill
    let workflows = discover_workflows(&skill_dir).context("Failed to discover workflows")?;

    match workflow {
        Some(query) => {
            // Load and output specific workflow
            if let Some(route) = workflows.find_workflow(query) {
                let content = load_workflow(&skill_dir, &route.workflow)
                    .with_context(|| format!("Failed to load workflow '{}'", route.workflow))?;

                match format {
                    OutputFormat::Text => {
                        println!("{}", content);
                    }
                    OutputFormat::Json => {
                        #[derive(serde::Serialize)]
                        struct WorkflowOutput {
                            skill: String,
                            intent: String,
                            path: String,
                            content: String,
                        }
                        let output = WorkflowOutput {
                            skill: skill_name.to_string(),
                            intent: route.intent.clone(),
                            path: route.workflow.clone(),
                            content,
                        };
                        println!("{}", serde_json::to_string_pretty(&output)?);
                    }
                    OutputFormat::Yaml => {
                        #[derive(serde::Serialize)]
                        struct WorkflowOutput {
                            skill: String,
                            intent: String,
                            path: String,
                            content: String,
                        }
                        let output = WorkflowOutput {
                            skill: skill_name.to_string(),
                            intent: route.intent.clone(),
                            path: route.workflow.clone(),
                            content,
                        };
                        println!("{}", serde_yaml::to_string(&output)?);
                    }
                }
            } else {
                eyre::bail!(
                    "No workflow matching '{}' found for skill '{}'\nAvailable workflows: {}",
                    query,
                    skill_name,
                    workflows
                        .routes
                        .iter()
                        .map(|r| r.intent.as_str())
                        .collect::<Vec<_>>()
                        .join(", ")
                );
            }
        }
        None => {
            // List all workflows
            match format {
                OutputFormat::Text => {
                    if workflows.routes.is_empty() {
                        println!("No workflows defined for skill '{}'", skill_name);
                        println!();
                        println!("To add workflows, create a 'workflows/' directory and add .md files.");
                        println!("Then add a '## Workflow Routing' section to SKILL.md:");
                        println!();
                        println!("  ## Workflow Routing");
                        println!();
                        println!("  | Intent | Workflow |");
                        println!("  |--------|----------|");
                        println!("  | new project | workflows/new-project.md |");
                        return Ok(());
                    }

                    println!("Workflows for skill '{}':", skill_name);
                    println!();
                    println!("| Intent | Workflow |");
                    println!("|--------|----------|");
                    for route in &workflows.routes {
                        println!("| {} | {} |", route.intent, route.workflow);
                    }
                    println!();
                    println!("Use: pais skill workflow {} \"<intent>\"", skill_name);
                }
                OutputFormat::Json => {
                    println!("{}", serde_json::to_string_pretty(&workflows)?);
                }
                OutputFormat::Yaml => {
                    println!("{}", serde_yaml::to_string(&workflows)?);
                }
            }
        }
    }

    Ok(())
}
