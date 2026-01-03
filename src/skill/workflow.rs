//! Workflow routing support for skills
//!
//! Skills can define workflow routing tables that map intents to specific
//! workflow markdown files in a `workflows/` subdirectory.
//!
//! Example SKILL.md with workflows:
//! ```markdown
//! ---
//! name: rust-coder
//! description: Write Rust code. USE WHEN creating CLIs, libraries.
//! ---
//!
//! ## Workflow Routing
//!
//! | Intent | Workflow |
//! |--------|----------|
//! | new CLI project | workflows/new-cli.md |
//! | add error handling | workflows/error-handling.md |
//! ```

use eyre::{Context, Result};
use regex::Regex;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::fs;
use std::path::Path;

/// A workflow routing entry
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WorkflowRoute {
    /// Intent/trigger phrase
    pub intent: String,
    /// Path to workflow file (relative to skill dir)
    pub workflow: String,
}

/// Parsed workflow information for a skill
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SkillWorkflows {
    /// Skill name
    pub skill: String,
    /// Available workflows
    pub routes: Vec<WorkflowRoute>,
}

impl SkillWorkflows {
    /// Check if this skill has any workflows
    pub fn has_workflows(&self) -> bool {
        !self.routes.is_empty()
    }

    /// Find a workflow by intent (fuzzy match)
    pub fn find_workflow(&self, query: &str) -> Option<&WorkflowRoute> {
        let query_lower = query.to_lowercase();

        // Exact match first
        if let Some(route) = self.routes.iter().find(|r| r.intent.to_lowercase() == query_lower) {
            return Some(route);
        }

        // Partial match
        self.routes
            .iter()
            .find(|r| r.intent.to_lowercase().contains(&query_lower) || query_lower.contains(&r.intent.to_lowercase()))
    }
}

/// Parse workflow routing table from SKILL.md content
pub fn parse_workflows(skill_name: &str, content: &str) -> SkillWorkflows {
    let mut routes = Vec::new();

    // Look for markdown table after "## Workflow Routing" or similar header
    let table_pattern = Regex::new(r"(?i)##\s*workflow\s*(routing)?\s*\n").unwrap();

    if let Some(header_match) = table_pattern.find(content) {
        let after_header = &content[header_match.end()..];

        // Parse markdown table rows: | intent | workflow |
        // Skip header row and separator row
        let row_pattern = Regex::new(r"^\|\s*([^|]+?)\s*\|\s*([^|]+?)\s*\|").unwrap();

        let mut in_table = false;
        let mut skipped_header = false;

        for line in after_header.lines() {
            let trimmed = line.trim();

            // Stop at next section
            if trimmed.starts_with('#') {
                break;
            }

            // Skip empty lines before table
            if !in_table && trimmed.is_empty() {
                continue;
            }

            // Detect table start
            if !in_table && trimmed.starts_with('|') {
                in_table = true;
            }

            if !in_table {
                continue;
            }

            // Skip separator row (|---|---|)
            if trimmed.contains("---") {
                continue;
            }

            // Skip header row
            if !skipped_header
                && (trimmed.to_lowercase().contains("intent") || trimmed.to_lowercase().contains("workflow"))
            {
                skipped_header = true;
                continue;
            }

            // Parse data row
            if let Some(caps) = row_pattern.captures(trimmed) {
                let intent = caps.get(1).map(|m| m.as_str().trim()).unwrap_or("");
                let workflow = caps.get(2).map(|m| m.as_str().trim()).unwrap_or("");

                if !intent.is_empty() && !workflow.is_empty() {
                    routes.push(WorkflowRoute {
                        intent: intent.to_string(),
                        workflow: workflow.to_string(),
                    });
                }
            }
        }
    }

    SkillWorkflows {
        skill: skill_name.to_string(),
        routes,
    }
}

/// Discover workflows for a skill from its directory
pub fn discover_workflows(skill_dir: &Path) -> Result<SkillWorkflows> {
    let skill_name = skill_dir
        .file_name()
        .and_then(|n| n.to_str())
        .unwrap_or("unknown")
        .to_string();

    let skill_md = skill_dir.join("SKILL.md");

    if !skill_md.exists() {
        return Ok(SkillWorkflows {
            skill: skill_name,
            routes: Vec::new(),
        });
    }

    let content = fs::read_to_string(&skill_md).context("Failed to read SKILL.md")?;

    let mut workflows = parse_workflows(&skill_name, &content);

    // Also check for workflows/ directory and add any .md files not in the table
    let workflows_dir = skill_dir.join("workflows");
    if workflows_dir.exists() && workflows_dir.is_dir() {
        let existing: std::collections::HashSet<_> =
            workflows.routes.iter().map(|r| r.workflow.to_lowercase()).collect();

        if let Ok(entries) = fs::read_dir(&workflows_dir) {
            for entry in entries.flatten() {
                let path = entry.path();
                if path.extension().map(|e| e == "md").unwrap_or(false) {
                    let filename = entry.file_name().to_string_lossy().to_string();
                    let workflow_path = format!("workflows/{}", filename);

                    if !existing.contains(&workflow_path.to_lowercase()) {
                        // Add undocumented workflow with filename as intent
                        let intent = filename.trim_end_matches(".md").replace(['-', '_'], " ");

                        workflows.routes.push(WorkflowRoute {
                            intent,
                            workflow: workflow_path,
                        });
                    }
                }
            }
        }
    }

    Ok(workflows)
}

/// Load workflow content from a skill
pub fn load_workflow(skill_dir: &Path, workflow_path: &str) -> Result<String> {
    let full_path = skill_dir.join(workflow_path);

    if !full_path.exists() {
        eyre::bail!("Workflow not found: {}", full_path.display());
    }

    fs::read_to_string(&full_path).context(format!("Failed to read workflow: {}", full_path.display()))
}

/// Get all workflows across all skills in a directory
pub fn get_all_workflows(skills_dir: &Path) -> Result<HashMap<String, SkillWorkflows>> {
    let mut all_workflows = HashMap::new();

    if !skills_dir.exists() {
        return Ok(all_workflows);
    }

    for entry in fs::read_dir(skills_dir)? {
        let entry = entry?;
        let path = entry.path();

        if path.is_dir() {
            let skill_name = entry.file_name().to_string_lossy().to_string();
            let workflows = discover_workflows(&path)?;

            if workflows.has_workflows() {
                all_workflows.insert(skill_name, workflows);
            }
        }
    }

    Ok(all_workflows)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_workflows_basic() {
        let content = r#"---
name: test-skill
description: Test skill
---

# Test Skill

## Workflow Routing

| Intent | Workflow |
|--------|----------|
| new project | workflows/new-project.md |
| add tests | workflows/testing.md |

## Other Section
"#;

        let workflows = parse_workflows("test-skill", content);

        assert_eq!(workflows.skill, "test-skill");
        assert_eq!(workflows.routes.len(), 2);
        assert_eq!(workflows.routes[0].intent, "new project");
        assert_eq!(workflows.routes[0].workflow, "workflows/new-project.md");
        assert_eq!(workflows.routes[1].intent, "add tests");
        assert_eq!(workflows.routes[1].workflow, "workflows/testing.md");
    }

    #[test]
    fn test_parse_workflows_case_insensitive_header() {
        let content = r#"
## WORKFLOW routing

| Intent | Workflow |
|--------|----------|
| deploy | workflows/deploy.md |
"#;

        let workflows = parse_workflows("test", content);
        assert_eq!(workflows.routes.len(), 1);
    }

    #[test]
    fn test_parse_workflows_no_table() {
        let content = r#"---
name: simple-skill
description: No workflows here
---

# Simple Skill

Just some content.
"#;

        let workflows = parse_workflows("simple-skill", content);
        assert!(workflows.routes.is_empty());
    }

    #[test]
    fn test_find_workflow_exact() {
        let workflows = SkillWorkflows {
            skill: "test".to_string(),
            routes: vec![
                WorkflowRoute {
                    intent: "new CLI project".to_string(),
                    workflow: "workflows/new-cli.md".to_string(),
                },
                WorkflowRoute {
                    intent: "add tests".to_string(),
                    workflow: "workflows/testing.md".to_string(),
                },
            ],
        };

        let found = workflows.find_workflow("new CLI project");
        assert!(found.is_some());
        assert_eq!(found.unwrap().workflow, "workflows/new-cli.md");
    }

    #[test]
    fn test_find_workflow_partial() {
        let workflows = SkillWorkflows {
            skill: "test".to_string(),
            routes: vec![WorkflowRoute {
                intent: "new CLI project".to_string(),
                workflow: "workflows/new-cli.md".to_string(),
            }],
        };

        let found = workflows.find_workflow("cli");
        assert!(found.is_some());
    }

    #[test]
    fn test_find_workflow_case_insensitive() {
        let workflows = SkillWorkflows {
            skill: "test".to_string(),
            routes: vec![WorkflowRoute {
                intent: "New CLI Project".to_string(),
                workflow: "workflows/new-cli.md".to_string(),
            }],
        };

        let found = workflows.find_workflow("new cli project");
        assert!(found.is_some());
    }
}
