//! Skill template generation
//!
//! Generates SKILL.md templates for new skills.

/// Generate a SKILL.md template for a new skill
pub fn generate_skill_template(name: &str) -> String {
    // Convert name to title case for display
    let title = name
        .split(&['-', '_'][..])
        .map(|word| {
            let mut chars = word.chars();
            match chars.next() {
                None => String::new(),
                Some(first) => first.to_uppercase().collect::<String>() + chars.as_str(),
            }
        })
        .collect::<Vec<_>>()
        .join(" ");

    format!(
        r#"---
name: {name}
description: "Brief description of what this skill does"
allowed-tools: Bash({name}:*)
tags: []
---

# {title}

## USE WHEN

- [When should Claude activate this skill?]
- [What user requests or contexts trigger this?]

## SYNTAX DISCOVERY

For CLI tools, ALWAYS discover exact syntax via --help:

```bash
{name} --help                    # List all commands
{name} <command> --help          # Command-specific options
```

Do NOT memorize examples from this file - they may drift out of sync.
Run --help to get current, accurate syntax.

## CORE CONCEPTS

[Describe WHAT the tool does and WHY, not detailed HOW]

- [Key concept 1]
- [Key concept 2]

## WORKFLOW

[Describe the typical workflow/stages, not exact commands]

1. [Discovery/planning phase]
2. [Execution phase]
3. [Verification/cleanup phase]

## NOTES

- [Additional context or caveats]
- [Related skills or tools]
"#,
        name = name,
        title = title
    )
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_generate_template_simple_name() {
        let template = generate_skill_template("terraform");
        assert!(template.contains("name: terraform"));
        assert!(template.contains("# Terraform"));
        assert!(template.contains("## USE WHEN"));
        assert!(template.contains("## SYNTAX DISCOVERY"));
        assert!(template.contains("## CORE CONCEPTS"));
        assert!(template.contains("## WORKFLOW"));
        assert!(template.contains("terraform --help"));
        assert!(template.contains("allowed-tools: Bash(terraform:*)"));
    }

    #[test]
    fn test_generate_template_hyphenated_name() {
        let template = generate_skill_template("git-tools");
        assert!(template.contains("name: git-tools"));
        assert!(template.contains("# Git Tools"));
    }

    #[test]
    fn test_generate_template_underscored_name() {
        let template = generate_skill_template("rust_coder");
        assert!(template.contains("name: rust_coder"));
        assert!(template.contains("# Rust Coder"));
    }

    #[test]
    fn test_template_has_frontmatter() {
        let template = generate_skill_template("test");
        assert!(template.starts_with("---\n"));
        assert!(template.contains("\n---\n"));
    }
}
