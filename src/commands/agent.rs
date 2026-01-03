//! Agent management commands

use colored::*;
use eyre::Result;
use serde::Serialize;
use std::fs;

use crate::agent::loader::AgentLoader;
use crate::agent::traits::{Trait, TraitCategory};
use crate::cli::{AgentAction, OutputFormat};
use crate::config::Config;

pub fn run(action: AgentAction, config: &Config) -> Result<()> {
    match action {
        AgentAction::List { format } => list_agents(OutputFormat::resolve(format), config),
        AgentAction::Show { name, format } => show_agent(&name, OutputFormat::resolve(format), config),
        AgentAction::Traits { format } => list_traits(OutputFormat::resolve(format)),
        AgentAction::Prompt { name } => show_prompt(&name, config),
        AgentAction::Create { name } => create_agent(&name, config),
    }
}

fn list_agents(format: OutputFormat, config: &Config) -> Result<()> {
    let agents_dir = Config::expand_path(&config.paths.skills)
        .parent()
        .unwrap_or(&config.paths.skills)
        .join("agents");
    let mut loader = AgentLoader::new(agents_dir.clone());
    let agents = loader.load_all()?;

    #[derive(Serialize)]
    struct AgentSummary {
        name: String,
        description: String,
        traits: Vec<String>,
        history_category: Option<String>,
    }

    let summaries: Vec<AgentSummary> = agents
        .iter()
        .map(|a| AgentSummary {
            name: a.name.clone(),
            description: a.description.clone(),
            traits: a.traits.iter().map(|t| t.to_string()).collect(),
            history_category: a.history_category.clone(),
        })
        .collect();

    match format {
        OutputFormat::Json => println!("{}", serde_json::to_string_pretty(&summaries)?),
        OutputFormat::Yaml => println!("{}", serde_yaml::to_string(&summaries)?),
        OutputFormat::Text => {
            println!("{}", "Available Agents:".bold());
            println!();

            if agents.is_empty() {
                println!("  {} No agents found in {}", "(none)".dimmed(), agents_dir.display());
                println!();
                println!("  Create agents with: {}", "pais agent create <name>".cyan());
            } else {
                for agent in &agents {
                    println!("  {} {}", "●".green(), agent.name.bold());
                    println!("    {}", agent.description.dimmed());
                    if !agent.traits.is_empty() {
                        let traits: Vec<String> = agent.traits.iter().map(|t| t.to_string()).collect();
                        println!("    Traits: {}", traits.join(", ").cyan());
                    }
                    if let Some(ref cat) = agent.history_category {
                        println!("    History: {}", cat.magenta());
                    }
                    println!();
                }
            }
        }
    }

    Ok(())
}

fn show_agent(name: &str, format: OutputFormat, config: &Config) -> Result<()> {
    let agents_dir = Config::expand_path(&config.paths.skills)
        .parent()
        .unwrap_or(&config.paths.skills)
        .join("agents");
    let agent_path = agents_dir.join(format!("{}.yaml", name));

    if !agent_path.exists() {
        eprintln!("{} Agent '{}' not found at {}", "✗".red(), name, agent_path.display());
        return Ok(());
    }

    let loader = AgentLoader::new(agents_dir);
    let agent = loader.load_agent(&agent_path)?;

    match format {
        OutputFormat::Json => println!("{}", serde_json::to_string_pretty(&agent)?),
        OutputFormat::Yaml => println!("{}", serde_yaml::to_string(&agent)?),
        OutputFormat::Text => {
            println!("{} {}", "Agent:".bold(), agent.name.green().bold());
            println!();
            println!("{} {}", "Description:".bold(), agent.description);

            if let Some(ref backstory) = agent.backstory {
                println!();
                println!("{}", "Backstory:".bold());
                for line in backstory.lines() {
                    println!("  {}", line);
                }
            }

            if !agent.traits.is_empty() {
                println!();
                println!("{}", "Traits:".bold());
                for trait_ in &agent.traits {
                    println!(
                        "  {} {} - {}",
                        "•".cyan(),
                        trait_.to_string().bold(),
                        trait_.prompt_fragment()
                    );
                }
            }

            if !agent.communication_style.is_empty() {
                println!();
                println!("{}", "Communication Style:".bold());
                for style in &agent.communication_style {
                    println!("  {}", style.italic());
                }
            }

            if let Some(ref cat) = agent.history_category {
                println!();
                println!("{} {}", "History Category:".bold(), cat.magenta());
            }
        }
    }

    Ok(())
}

fn list_traits(format: OutputFormat) -> Result<()> {
    // Build trait list
    let all_traits = vec![
        // Expertise
        Trait::Security,
        Trait::Legal,
        Trait::Finance,
        Trait::Medical,
        Trait::Technical,
        Trait::Research,
        Trait::Creative,
        Trait::Business,
        Trait::Data,
        Trait::Communications,
        // Personality
        Trait::Skeptical,
        Trait::Enthusiastic,
        Trait::Cautious,
        Trait::Bold,
        Trait::Analytical,
        Trait::Empathetic,
        Trait::Contrarian,
        Trait::Pragmatic,
        Trait::Meticulous,
        // Approach
        Trait::Thorough,
        Trait::Rapid,
        Trait::Systematic,
        Trait::Exploratory,
        Trait::Comparative,
        Trait::Synthesizing,
        Trait::Adversarial,
        Trait::Consultative,
    ];

    #[derive(Serialize)]
    struct TraitInfo {
        name: String,
        category: String,
        description: String,
    }

    let trait_infos: Vec<TraitInfo> = all_traits
        .iter()
        .map(|t| TraitInfo {
            name: t.to_string(),
            category: format!("{:?}", t.category()).to_lowercase(),
            description: t.prompt_fragment().to_string(),
        })
        .collect();

    match format {
        OutputFormat::Json => println!("{}", serde_json::to_string_pretty(&trait_infos)?),
        OutputFormat::Yaml => println!("{}", serde_yaml::to_string(&trait_infos)?),
        OutputFormat::Text => {
            println!("{}", "Available Traits:".bold());
            println!();

            // Group by category
            for category in [
                TraitCategory::Expertise,
                TraitCategory::Personality,
                TraitCategory::Approach,
            ] {
                let label = match category {
                    TraitCategory::Expertise => "Expertise (WHAT they know)",
                    TraitCategory::Personality => "Personality (HOW they think)",
                    TraitCategory::Approach => "Approach (HOW they work)",
                };

                println!("{}", label.bold().underline());
                println!();

                for trait_ in all_traits.iter().filter(|t| t.category() == category) {
                    println!("  {} {}", trait_.to_string().cyan().bold(), "-".dimmed());
                    // Truncate description for display
                    let desc = trait_.prompt_fragment();
                    let short_desc = if desc.len() > 70 { format!("{}...", &desc[..67]) } else { desc.to_string() };
                    println!("    {}", short_desc.dimmed());
                }
                println!();
            }
        }
    }

    Ok(())
}

fn show_prompt(name: &str, config: &Config) -> Result<()> {
    let agents_dir = Config::expand_path(&config.paths.skills)
        .parent()
        .unwrap_or(&config.paths.skills)
        .join("agents");
    let agent_path = agents_dir.join(format!("{}.yaml", name));

    if !agent_path.exists() {
        eprintln!("{} Agent '{}' not found", "✗".red(), name);
        return Ok(());
    }

    let loader = AgentLoader::new(agents_dir);
    let agent = loader.load_agent(&agent_path)?;
    let prompt = agent.generate_prompt();

    println!("{}", prompt);

    Ok(())
}

fn create_agent(name: &str, config: &Config) -> Result<()> {
    let agents_dir = Config::expand_path(&config.paths.skills)
        .parent()
        .unwrap_or(&config.paths.skills)
        .join("agents");
    fs::create_dir_all(&agents_dir)?;

    let agent_path = agents_dir.join(format!("{}.yaml", name));

    if agent_path.exists() {
        eprintln!(
            "{} Agent '{}' already exists at {}",
            "✗".red(),
            name,
            agent_path.display()
        );
        return Ok(());
    }

    let template = format!(
        r#"# Agent: {name}
# Created by: pais agent create {name}

name: {name}
description: TODO - describe this agent's purpose

# Optional backstory for personality depth
backstory: |
  TODO - add backstory for personality context

# Composable traits (expertise + personality + approach)
# Run 'pais agent traits' to see all available options
traits:
  - analytical
  - thorough

# Where this agent's outputs are categorized in history
history_category: research

# Example communication style phrases
communication_style:
  - "Let me analyze this systematically..."
  - "The evidence suggests..."
"#,
        name = name
    );

    fs::write(&agent_path, template)?;

    println!("{} Created agent template: {}", "✓".green(), agent_path.display());
    println!();
    println!("Next steps:");
    println!("  1. Edit {} to customize the agent", agent_path.display());
    println!("  2. Run {} to verify", format!("pais agent show {}", name).cyan());

    Ok(())
}
