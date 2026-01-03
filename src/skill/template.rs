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
tags: []
---

# {title}

## USE WHEN

- [When should Claude activate this skill?]
- [What user requests or contexts trigger this?]

## INSTRUCTIONS

[Detailed instructions for Claude when this skill is active]

1. [Step or guideline 1]
2. [Step or guideline 2]
3. [Step or guideline 3]

## EXAMPLES

### Example 1: [Scenario]

**User:** [Example user request]

**Response approach:** [How Claude should respond]

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
        assert!(template.contains("## INSTRUCTIONS"));
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
