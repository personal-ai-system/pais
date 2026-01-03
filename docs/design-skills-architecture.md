# Design: Skills Architecture

> **Status:** Proposed
> **Author:** Scott Idler
> **Date:** 2026-01-02
> **Supersedes:** `design-personal-skills-separation.md`

---

## Executive Summary

PAIS supports two distinct mechanisms for extending Claude's capabilities:

1. **Simple Skills** — SKILL.md files that teach Claude about tools/workflows (no code)
2. **Full Plugins** — Complete packages with code, contracts, and hook handlers

This document defines where each type lives, when to use each, and how they integrate with Claude Code's native systems.

---

## Core Principle

**Skills should live with the code they describe, when possible.**

| Scenario | Location | Format |
|----------|----------|--------|
| Tool you **control** | `.pais/` in tool's repo | SKILL.md (+ plugin.toml if needed) |
| Tool you **don't control** | `~/.config/pais/skills/` | SKILL.md only |
| Complex plugin with code | `~/.config/pais/plugins/` | plugin.toml + SKILL.md + code |

---

## Directory Structure

### User's Config Directory (`~/.config/pais/`)

This directory **is a git repo** — trackable like dotfiles.

```
~/.config/pais/                    # Git repository
├── .git/
├── .gitignore
├── pais.toml                      # Main configuration
│
├── skills/                        # Simple skills (TRACKED)
│   ├── terraform/
│   │   └── SKILL.md               # Teaches Claude about terraform
│   ├── kubectl/
│   │   └── SKILL.md               # Teaches Claude about kubectl
│   ├── aws-cli/
│   │   └── SKILL.md
│   └── rust-conventions/
│       └── SKILL.md               # Personal coding conventions
│
├── plugins/                       # Installed plugins (GITIGNORED)
│   ├── incident/                  # Full plugin with code
│   │   ├── plugin.toml
│   │   ├── SKILL.md
│   │   └── src/
│   └── history/
│       ├── plugin.toml
│       └── src/
│
├── history/                       # Memory storage (GITIGNORED)
│   ├── sessions/
│   ├── learnings/
│   └── decisions/
│
├── registries/                    # Cached registries (GITIGNORED)
│   └── core.toml
│
└── .env                           # Secrets (GITIGNORED)
```

### `.gitignore` for `~/.config/pais/`

```gitignore
# Secrets
.env

# Installed plugins (come from registries/repos)
plugins/

# Runtime data
history/
registries/

# Cache
*.cache
```

### Controlled Tool Repository (e.g., `scottidler/aka`)

For tools you control, the skill lives WITH the code:

```
scottidler/aka/
├── src/
│   └── main.rs
├── Cargo.toml
├── README.md
└── .pais/
    ├── SKILL.md                   # Required: teaches Claude about aka
    └── plugin.toml                # Optional: only if hooks/code needed
```

### External Skills Repository (e.g., `scottidler/pais-skills`)

For sharing skills about tools you don't control:

```
scottidler/pais-skills/
├── README.md
├── registry.toml                  # Optional: for discovery
└── skills/
    ├── terraform/
    │   └── SKILL.md
    ├── kubectl/
    │   └── SKILL.md
    └── datadog/
        └── SKILL.md
```

---

## When to Use What

### Simple Skills (SKILL.md only)

**Use when:** Teaching Claude about a tool, workflow, or convention — no executable code needed.

**Examples:**
- terraform CLI conventions
- kubectl common patterns
- Personal coding style guide
- Company runbook patterns

**What you need:**
```
skill-name/
└── SKILL.md
```

**SKILL.md format:**
```markdown
---
name: terraform
description: Infrastructure as Code with Terraform. Use when working with .tf files, terraform commands, or cloud infrastructure provisioning.
---

# Terraform

## USE WHEN

- User mentions Terraform, .tf files, or IaC
- Working with cloud infrastructure (AWS, GCP, Azure)
- User asks about state management or resource provisioning

## CONVENTIONS

- Always run `terraform fmt` before committing
- Use workspaces for environment separation
- Store state in remote backend (S3, GCS)
- Use modules for reusable infrastructure

## COMMON COMMANDS

```bash
terraform init      # Initialize working directory
terraform plan      # Preview changes
terraform apply     # Apply changes
terraform destroy   # Destroy infrastructure
```

## NOTES

- Never commit .tfstate files
- Use variables for environment-specific values
- Prefer `count` over `for_each` for simple cases
```

### Full Plugins (plugin.toml + code)

**Use when:** You need executable code, hook handlers, contracts, or integrations.

**Examples:**
- Incident response workflow (calls PagerDuty, Slack, Jira)
- History capture system (hook handler)
- Security validator (PreToolUse hook)
- Custom integrations

**What you need:**
```
plugin-name/
├── plugin.toml                    # Required: manifest
├── SKILL.md                       # Optional: if it's also a skill
└── src/
    └── main.py                    # Or main.rs for Rust
```

**plugin.toml format:**
```toml
[plugin]
name = "incident"
version = "1.0.0"
description = "Incident response workflows"
authors = ["your-team"]
language = "python"

[pais]
core_version = ">=0.1.0"

[provides]
skill = "incident"
# Or for integrations:
# integration = { contract = "IntegrationProvider", service = "pagerduty" }

[consumes]
# Optional dependencies
memory = { contract = "MemoryProvider", optional = true }
pagerduty = { contract = "IntegrationProvider", service = "pagerduty", optional = true }

[hooks]
pre_tool_use = false
post_tool_use = false
stop = true              # Capture session summaries
session_start = true     # Initialize context

[config]
escalation_threshold_minutes = { type = "integer", default = 30 }
```

---

## Decision Matrix

| Question | Answer → Use |
|----------|--------------|
| Does it need to run code? | Yes → Full Plugin |
| Does it handle hook events? | Yes → Full Plugin |
| Does it consume/provide contracts? | Yes → Full Plugin |
| Does it integrate with external services? | Yes → Full Plugin |
| Is it just teaching Claude about a tool? | Yes → Simple Skill |
| Is it documenting conventions/patterns? | Yes → Simple Skill |

---

## Integration with Claude Code

### How Skills Get to Claude Code

Claude Code discovers skills from:
1. `~/.claude/skills/` (personal)
2. `.claude/skills/` (project)
3. Plugin `skills/` directories

PAIS skills need to be made visible to Claude Code:

**Option A: Symlink (recommended)**
```bash
# PAIS syncs skills to Claude Code's location
ln -s ~/.config/pais/skills/* ~/.claude/skills/
```

**Option B: Copy**
```bash
# PAIS copies skills on change
cp -r ~/.config/pais/skills/* ~/.claude/skills/
```

**CLI Command:**
```bash
pais sync                          # Sync skills to ~/.claude/skills/
pais sync --watch                  # Watch for changes and sync
```

### How Hooks Integrate

Claude Code fires events → PAIS dispatches to plugin handlers:

```json
// ~/.claude/settings.json (managed by pais init)
{
  "hooks": {
    "PreToolUse": [{
      "matcher": "*",
      "hooks": [{
        "type": "command",
        "command": "pais hook dispatch pre-tool-use"
      }]
    }],
    "Stop": [{
      "matcher": "",
      "hooks": [{
        "type": "command",
        "command": "pais hook dispatch stop"
      }]
    }],
    "SessionStart": [{
      "matcher": "",
      "hooks": [{
        "type": "command",
        "command": "pais hook dispatch session-start"
      }]
    }]
  }
}
```

---

## Skill Discovery

### For Tools You Control

When PAIS scans repositories, it looks for `.pais/SKILL.md`:

```bash
pais skill scan ~/repos            # Find skills in local repos
pais skill scan ~/repos/scottidler # Scan specific directory
```

Output:
```
Found skills in controlled repos:
  ~/repos/scottidler/aka/.pais/SKILL.md
  ~/repos/scottidler/otto/.pais/SKILL.md
  ~/repos/scottidler/cidr/.pais/SKILL.md

Register these skills? [y/N]
```

### For External Skills

Skills in `~/.config/pais/skills/` are automatically available after sync.

```bash
pais skill list                    # List all known skills
pais skill add terraform           # Create new skill from template
pais sync                          # Make available to Claude Code
```

---

## The Contract System (for Full Plugins)

Full plugins can provide and consume contracts:

```
┌─────────────────────────────────────────────────────────────────────┐
│                        CONTRACT RESOLUTION                           │
│                                                                     │
│   history plugin                incident plugin                     │
│   ┌─────────────┐              ┌─────────────┐                     │
│   │ provides:   │              │ provides:   │                     │
│   │MemoryProvider│◄────────────│   skill     │                     │
│   │             │  consumes    │             │                     │
│   └─────────────┘  (optional)  │ consumes:   │                     │
│                                │MemoryProvider│                     │
│                                │ (optional)  │                     │
│                                └─────────────┘                     │
│                                                                     │
│   If history plugin is installed → incident uses it                 │
│   If history plugin is NOT installed → incident works without it    │
│                                                                     │
└─────────────────────────────────────────────────────────────────────┘
```

**Simple skills don't participate in contracts.** They're just documentation.

---

## Workflows

### Setting Up a New Machine

```bash
# 1. Clone your config repo
git clone git@github.com:scottidler/pais-config.git ~/.config/pais

# 2. Initialize PAIS (sets up Claude Code integration)
pais init

# 3. Install plugins from registries
pais plugin install incident
pais plugin install history

# 4. Sync skills to Claude Code
pais sync

# 5. Scan local repos for skills
pais skill scan ~/repos
```

### Adding a Skill for a Tool You Control

```bash
# In your tool's repo
cd ~/repos/scottidler/aka

# Create .pais directory with skill
mkdir -p .pais
pais skill new aka --output .pais/

# Edit the skill
$EDITOR .pais/SKILL.md

# Commit with your code
git add .pais/
git commit -m "Add PAIS skill"
```

### Adding a Skill for a Tool You Don't Control

```bash
# Create skill in your config
pais skill add terraform

# Edit the skill
$EDITOR ~/.config/pais/skills/terraform/SKILL.md

# Sync to Claude Code
pais sync

# Commit to your config repo
cd ~/.config/pais
git add skills/terraform/
git commit -m "Add terraform skill"
git push
```

### Sharing Skills with Teammates

**Option A: Share your config repo**
```bash
# Teammate clones your config
git clone git@github.com:scottidler/pais-config.git ~/.config/pais
```

**Option B: Create a skills repo**
```bash
# Create dedicated skills repo
mkdir pais-skills && cd pais-skills
git init

# Add skills
mkdir -p skills/terraform
cp ~/.config/pais/skills/terraform/SKILL.md skills/terraform/

# Create registry for discovery
cat > registry.toml << 'EOF'
[registry]
name = "scottidler-skills"
description = "Scott's PAIS skills"
version = "0.1.0"

[[skills]]
name = "terraform"
path = "skills/terraform"
description = "Terraform IaC conventions"
EOF

# Share
git add -A
git commit -m "Initial skills"
git push origin main
```

**Teammate installs:**
```bash
pais registry add scottidler https://raw.githubusercontent.com/scottidler/pais-skills/main/registry.toml
pais skill install terraform
```

---

## CLI Commands

### Skill Management

```bash
pais skill list                    # List all skills (local + discovered)
pais skill add <name>              # Create new skill from template
pais skill scan <path>             # Find .pais/SKILL.md in repos
pais skill info <name>             # Show skill details
pais skill edit <name>             # Open skill in $EDITOR
pais skill remove <name>           # Remove a skill
```

### Sync

```bash
pais sync                          # Sync skills to ~/.claude/skills/
pais sync --dry-run                # Show what would be synced
pais sync --watch                  # Watch and sync on changes
```

### Plugin Management (unchanged)

```bash
pais plugin list                   # List installed plugins
pais plugin install <name>         # Install from registry
pais plugin remove <name>          # Remove plugin
pais plugin info <name>            # Show plugin details
```

---

## Migration from Current State

### Current plugins/ in PAIS Repo

The current `plugins/` directory in the pais repo contains skills for Scott's personal tools:

| Current Location | New Location | Reason |
|------------------|--------------|--------|
| `plugins/aka/` | `scottidler/aka/.pais/` | Skill lives with code |
| `plugins/otto/` | `scottidler/otto/.pais/` | Skill lives with code |
| `plugins/rust-coder/` | `~/.config/pais/skills/rust-coder/` | Convention, not tool |
| `plugins/python-coder/` | `~/.config/pais/skills/python-coder/` | Convention, not tool |
| `plugins/terraform/` | `~/.config/pais/skills/terraform/` | Tool I don't control |

### Migration Steps

1. **Move tool skills to their repos:**
   ```bash
   # For each controlled tool
   cp -r pais/plugins/aka/ ~/repos/scottidler/aka/.pais/
   cd ~/repos/scottidler/aka && git add .pais/ && git commit
   ```

2. **Move convention skills to config:**
   ```bash
   mkdir -p ~/.config/pais/skills/
   cp -r pais/plugins/rust-coder/ ~/.config/pais/skills/
   cp -r pais/plugins/python-coder/ ~/.config/pais/skills/
   ```

3. **Initialize config as git repo:**
   ```bash
   cd ~/.config/pais
   git init
   echo -e ".env\nplugins/\nhistory/\nregistries/" > .gitignore
   git add -A
   git commit -m "Initial PAIS config"
   ```

4. **Remove from PAIS core repo:**
   ```bash
   # Keep only examples
   rm -rf pais/plugins/aka pais/plugins/otto pais/plugins/rust-coder ...
   # Keep: plugins/hello-world, plugins/hello-rust (examples)
   ```

---

## What Stays in PAIS Core Repo

After separation:

```
pais/
├── src/                           # Core Rust CLI
├── examples/
│   ├── hello-world/               # Example Python plugin
│   │   ├── plugin.toml
│   │   └── SKILL.md
│   └── hello-rust/                # Example Rust plugin
│       ├── plugin.toml
│       └── SKILL.md
├── registry/
│   └── plugins.toml               # Points to examples only
├── docs/
│   ├── design-skills-architecture.md  # This document
│   └── ...
├── completions/
└── README.md
```

---

## Summary

| Type | Location | Format | Use Case |
|------|----------|--------|----------|
| **Simple Skill** | `~/.config/pais/skills/` | SKILL.md only | Tools you don't control |
| **Skill with Code** | `.pais/` in tool repo | SKILL.md + plugin.toml | Tools you control |
| **Full Plugin** | Installed to `~/.config/pais/plugins/` | plugin.toml + code | Integrations, hooks |
| **Examples** | `pais/examples/` | plugin.toml + code | Learning/reference |

**Key insights:**
1. Skills live with code when possible
2. `~/.config/pais/` is a git repo (like dotfiles)
3. Simple skills don't need plugin.toml
4. Full plugins use the contract system for complex integrations
5. PAIS extends Claude Code, doesn't replace it

---

## Open Questions

1. **Auto-discovery:** Should `pais sync` automatically find `.pais/` directories in `~/repos/`?

2. **Skill templates:** Should `pais skill add` have templates for common tools?

3. **Validation:** Should PAIS validate SKILL.md frontmatter?

4. **Versioning:** How do we handle skill version compatibility?

---

## Related Documents

- [architecture.md](planning/architecture.md) — System design
- [vision.md](planning/vision.md) — Philosophy and goals
- [contracts.md](planning/contracts.md) — Plugin interface specifications
- [comparison.md](planning/comparison.md) — PAIS vs. Kai/PAI

