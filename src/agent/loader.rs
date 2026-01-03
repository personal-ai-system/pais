//! Agent loading and management

#![allow(dead_code)] // get/agents_dir methods - for future CLI commands

use eyre::{Context, Result};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::fs;
use std::path::Path;

use super::traits::Trait;

/// A named agent with traits and configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Agent {
    /// Agent name (e.g., "intern", "architect")
    pub name: String,

    /// Human-readable description
    pub description: String,

    /// Backstory for personality context
    #[serde(default)]
    pub backstory: Option<String>,

    /// Composable traits
    #[serde(default)]
    pub traits: Vec<Trait>,

    /// Custom prompt prefix (overrides trait-generated prefix)
    #[serde(default)]
    pub prompt_prefix: Option<String>,

    /// History category for routing outputs
    #[serde(default)]
    pub history_category: Option<String>,

    /// Communication style examples
    #[serde(default)]
    pub communication_style: Vec<String>,
}

impl Agent {
    /// Generate the full prompt prefix from traits and backstory
    pub fn generate_prompt(&self) -> String {
        // If custom prefix provided, use it
        if let Some(ref prefix) = self.prompt_prefix {
            return prefix.clone();
        }

        let mut parts = Vec::new();

        // Add backstory if present
        if let Some(ref backstory) = self.backstory {
            parts.push(backstory.clone());
        }

        // Add trait fragments
        for trait_ in &self.traits {
            parts.push(trait_.prompt_fragment().to_string());
        }

        // Add communication style
        if !self.communication_style.is_empty() {
            parts.push(format!("Communication style: {}", self.communication_style.join(" | ")));
        }

        parts.join("\n\n")
    }
}

/// Agent loader for discovering and loading agents
pub struct AgentLoader {
    agents_dir: std::path::PathBuf,
    cache: HashMap<String, Agent>,
}

impl AgentLoader {
    /// Create a new agent loader
    pub fn new(agents_dir: std::path::PathBuf) -> Self {
        Self {
            agents_dir,
            cache: HashMap::new(),
        }
    }

    /// Load all agents from the agents directory
    pub fn load_all(&mut self) -> Result<Vec<Agent>> {
        let mut agents = Vec::new();

        if !self.agents_dir.exists() {
            return Ok(agents);
        }

        let entries = fs::read_dir(&self.agents_dir)
            .with_context(|| format!("Failed to read agents directory: {}", self.agents_dir.display()))?;

        for entry in entries.flatten() {
            let path = entry.path();
            if path.extension().map(|e| e == "yaml" || e == "yml").unwrap_or(false) {
                match self.load_agent(&path) {
                    Ok(agent) => {
                        self.cache.insert(agent.name.clone(), agent.clone());
                        agents.push(agent);
                    }
                    Err(e) => {
                        log::warn!("Failed to load agent from {}: {}", path.display(), e);
                    }
                }
            }
        }

        // Sort by name for consistent ordering
        agents.sort_by(|a, b| a.name.cmp(&b.name));

        Ok(agents)
    }

    /// Load a single agent from a file
    pub fn load_agent(&self, path: &Path) -> Result<Agent> {
        let content =
            fs::read_to_string(path).with_context(|| format!("Failed to read agent file: {}", path.display()))?;

        let agent: Agent = serde_yaml::from_str(&content)
            .with_context(|| format!("Failed to parse agent file: {}", path.display()))?;

        Ok(agent)
    }

    /// Get an agent by name
    pub fn get(&self, name: &str) -> Option<&Agent> {
        self.cache.get(name)
    }

    /// Get the agents directory
    pub fn agents_dir(&self) -> &Path {
        &self.agents_dir
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_agent_generate_prompt_from_traits() {
        let agent = Agent {
            name: "test".to_string(),
            description: "Test agent".to_string(),
            backstory: Some("A test backstory.".to_string()),
            traits: vec![Trait::Security, Trait::Skeptical, Trait::Thorough],
            prompt_prefix: None,
            history_category: Some("research".to_string()),
            communication_style: vec!["Direct".to_string(), "Questioning".to_string()],
        };

        let prompt = agent.generate_prompt();

        assert!(prompt.contains("test backstory"));
        assert!(prompt.contains("vulnerabilities"));
        assert!(prompt.contains("Question assumptions"));
        assert!(prompt.contains("exhaustive"));
        assert!(prompt.contains("Direct | Questioning"));
    }

    #[test]
    fn test_agent_generate_prompt_custom_prefix() {
        let agent = Agent {
            name: "test".to_string(),
            description: "Test agent".to_string(),
            backstory: None,
            traits: vec![Trait::Security],
            prompt_prefix: Some("Custom prefix override".to_string()),
            history_category: None,
            communication_style: vec![],
        };

        let prompt = agent.generate_prompt();

        assert_eq!(prompt, "Custom prefix override");
    }

    #[test]
    fn test_agent_deserialize() {
        let yaml = r#"
name: intern
description: Eager learner
backstory: Young and eager
traits:
  - enthusiastic
  - research
  - rapid
history_category: learnings
communication_style:
  - "I can do that!"
  - "Wait, but why?"
"#;

        let agent: Agent = serde_yaml::from_str(yaml).unwrap();

        assert_eq!(agent.name, "intern");
        assert_eq!(agent.traits.len(), 3);
        assert!(agent.traits.contains(&Trait::Enthusiastic));
    }
}
