# Implementation Phases: Skills Architecture

> **Related:** [design-skills-architecture.md](design-skills-architecture.md)
> **Date:** 2026-01-02

---

## Progress Tracker

| Phase | Description | Status | Completed |
|-------|-------------|--------|-----------|
| **1** | Simple Skills Support | ✅ Complete | 2026-01-02 |
| **2** | Skill Management Commands | ✅ Complete | 2026-01-02 |
| **3** | Sync to Claude Code | ✅ Complete | 2026-01-02 |
| **4** | Skill Scanning | ⏳ Pending | - |
| **5** | Git Repo Init | ⏳ Pending | - |
| **6** | Migration | ⏳ Pending | - |

---

## Overview

This document breaks the skills architecture into implementable phases. Each phase is independently useful and builds toward the complete vision.

---

## Phase 1: Simple Skills Support ✅

**Goal:** Support SKILL.md-only skills (no plugin.toml required)

**Status:** Complete (2026-01-02)

### What Was Implemented

| File | Purpose |
|------|---------|
| `src/skill/mod.rs` | `Skill` struct, `SkillSource` enum |
| `src/skill/parser.rs` | Parse SKILL.md YAML frontmatter |
| `src/skill/loader.rs` | Discover and load skills from directories |
| `src/commands/skill.rs` | `pais skill list` and `pais skill info` commands |
| `src/config.rs` | Added `skills` path to `PathsConfig` |
| `src/cli.rs` | Added `SkillAction` subcommand |

### Changes

1. **Update skill discovery** to recognize directories with just SKILL.md:
   ```rust
   // In plugin/loader.rs or new skills/loader.rs
   fn discover_skill(path: &Path) -> Option<Skill> {
       let skill_md = path.join("SKILL.md");
       if skill_md.exists() {
           // Parse SKILL.md frontmatter for name, description
           return Some(Skill::from_skill_md(&skill_md)?);
       }
       None
   }
   ```

2. **Add `Skill` struct** (distinct from `Plugin`):
   ```rust
   pub struct Skill {
       pub name: String,
       pub description: String,
       pub path: PathBuf,
       pub source: SkillSource,  // SimpleSkill | PluginSkill
   }

   pub enum SkillSource {
       Simple,              // Just SKILL.md
       Plugin(String),      // Part of a plugin
       Discovered(PathBuf), // Found via scan
   }
   ```

3. **Parse SKILL.md frontmatter**:
   ```rust
   // Parse YAML frontmatter from SKILL.md
   fn parse_skill_md(path: &Path) -> Result<SkillMetadata> {
       let content = fs::read_to_string(path)?;
       // Extract --- delimited YAML
       // Return name, description
   }
   ```

### Files Modified/Created

| File | Action | Status |
|------|--------|--------|
| `src/skill/mod.rs` | Created - Skill struct and types | ✅ |
| `src/skill/loader.rs` | Created - Load skills from SKILL.md | ✅ |
| `src/skill/parser.rs` | Created - Parse SKILL.md frontmatter | ✅ |
| `src/commands/skill.rs` | Created - Skill command handlers | ✅ |
| `src/commands/mod.rs` | Added `skill` module | ✅ |
| `src/cli.rs` | Added `SkillAction` subcommand | ✅ |
| `src/config.rs` | Added `skills` path | ✅ |
| `src/main.rs` | Added `skill` module and command routing | ✅ |

### Deliverable

```bash
pais skill list    # Shows both plugin skills AND simple skills
pais skill info <name>  # Show details for a specific skill
```

---

## Phase 2: Skill Management Commands ✅

**Goal:** CLI commands for managing simple skills

**Status:** Complete (2026-01-02)

### What Was Implemented

| File | Purpose |
|------|---------|
| `src/skill/template.rs` | Generate SKILL.md templates for new skills |
| `src/commands/skill.rs` | Added `add`, `edit`, `remove`, `validate` handlers |
| `src/cli.rs` | Added new `SkillAction` variants |

### New Commands

```bash
pais skill list                    # List all skills
pais skill add <name>              # Create new skill from template
pais skill info <name>             # Show skill details
pais skill edit <name>             # Open in $EDITOR
pais skill remove <name>           # Remove a skill
pais skill validate <name>         # Validate SKILL.md format
```

### Implementation

1. **`pais skill list`**:
   ```rust
   fn list_skills(config: &Config) -> Result<()> {
       let skills_dir = config.paths.skills();  // ~/.config/pais/skills/
       let plugins_dir = config.paths.plugins();

       // Collect simple skills
       let simple_skills = discover_simple_skills(&skills_dir)?;

       // Collect plugin skills
       let plugin_skills = discover_plugin_skills(&plugins_dir)?;

       // Display
       println!("Simple Skills:");
       for skill in simple_skills { ... }

       println!("\nPlugin Skills:");
       for skill in plugin_skills { ... }
   }
   ```

2. **`pais skill add <name>`**:
   ```rust
   fn add_skill(name: &str, config: &Config) -> Result<()> {
       let skill_dir = config.paths.skills().join(name);
       fs::create_dir_all(&skill_dir)?;

       let template = generate_skill_template(name);
       fs::write(skill_dir.join("SKILL.md"), template)?;

       println!("Created: {}/SKILL.md", skill_dir.display());
       println!("Edit with: pais skill edit {}", name);
   }
   ```

3. **Skill template**:
   ```markdown
   ---
   name: {name}
   description: [What this skill does. When should Claude use it?]
   ---

   # {Name}

   ## USE WHEN

   - [Trigger condition 1]
   - [Trigger condition 2]

   ## INSTRUCTIONS

   [What Claude should do when this skill is active]

   ## EXAMPLES

   [Example interactions]

   ## NOTES

   [Additional context]
   ```

### Files Created/Modified

| File | Purpose | Status |
|------|---------|--------|
| `src/skill/template.rs` | Skill template generation | ✅ |
| `src/commands/skill.rs` | Skill subcommand handlers | ✅ |
| `src/cli.rs` | Added SkillAction variants | ✅ |

### CLI Updates

```rust
// In cli.rs
#[derive(Subcommand)]
pub enum Commands {
    // ... existing ...

    /// Manage skills
    Skill {
        #[command(subcommand)]
        action: SkillAction,
    },
}

#[derive(Subcommand)]
pub enum SkillAction {
    /// List all skills
    List { format: Option<String> },
    /// Create a new skill
    Add { name: String },
    /// Show skill details
    Info { name: String },
    /// Edit a skill
    Edit { name: String },
    /// Remove a skill
    Remove { name: String, #[arg(long)] force: bool },
    /// Validate skill format
    Validate { name: String },
}
```

### Deliverable

```bash
pais skill add terraform
# Creates ~/.config/pais/skills/terraform/SKILL.md
# Opens in $EDITOR
```

---

## Phase 3: Sync to Claude Code ✅

**Goal:** Sync PAIS skills to `~/.claude/skills/` for Claude Code discovery

**Status:** Complete (2026-01-02)

### What Was Implemented

| File | Purpose |
|------|---------|
| `src/commands/sync.rs` | Sync skills to `~/.claude/skills/` using symlinks |
| `src/cli.rs` | Added `Sync` command with `--dry-run` and `--clean` flags |
| `src/main.rs` | Added routing for sync command |

### New Command

```bash
pais sync                          # Sync all skills
pais sync --dry-run                # Show what would happen
pais sync --clean                  # Remove orphaned symlinks
```

### Implementation

1. **Sync strategy** - Symlinks (not copies):
   ```rust
   fn sync_skills(config: &Config, dry_run: bool) -> Result<()> {
       let pais_skills = config.paths.skills();      // ~/.config/pais/skills/
       let claude_skills = dirs::home_dir()
           .unwrap()
           .join(".claude/skills");

       fs::create_dir_all(&claude_skills)?;

       for entry in fs::read_dir(&pais_skills)? {
           let entry = entry?;
           let name = entry.file_name();
           let source = entry.path();
           let target = claude_skills.join(&name);

           if dry_run {
               println!("Would link: {} -> {}", target.display(), source.display());
           } else {
               // Remove existing if present
               if target.exists() {
                   fs::remove_file(&target).or_else(|_| fs::remove_dir_all(&target))?;
               }

               #[cfg(unix)]
               std::os::unix::fs::symlink(&source, &target)?;

               println!("Linked: {} -> {}", name.to_string_lossy(), source.display());
           }
       }
   }
   ```

2. **Also sync discovered skills** (from `.pais/` in repos):
   ```rust
   fn sync_discovered_skills(discovered: &[DiscoveredSkill], claude_skills: &Path) -> Result<()> {
       for skill in discovered {
           let target = claude_skills.join(&skill.name);
           // Symlink skill.source_path to target
       }
   }
   ```

### Files Created/Modified

| File | Purpose | Status |
|------|---------|--------|
| `src/commands/sync.rs` | Sync command handler | ✅ |
| `src/commands/mod.rs` | Added sync module | ✅ |
| `src/cli.rs` | Added Sync command | ✅ |
| `src/main.rs` | Added sync routing | ✅ |

### Deliverable

```bash
pais sync
# Linked: terraform -> /home/scott/.config/pais/skills/terraform
# Linked: kubectl -> /home/scott/.config/pais/skills/kubectl

# Verify
ls -la ~/.claude/skills/
# terraform -> /home/scott/.config/pais/skills/terraform
# kubectl -> /home/scott/.config/pais/skills/kubectl
```

---

## Phase 4: Skill Scanning

**Goal:** Discover `.pais/SKILL.md` in repositories you control

### New Command

```bash
pais skill scan <path>             # Scan directory for skills
pais skill scan ~/repos            # Scan all repos
pais skill scan --register         # Also register found skills
```

### Implementation

1. **Scanner**:
   ```rust
   fn scan_for_skills(root: &Path) -> Result<Vec<DiscoveredSkill>> {
       let mut found = Vec::new();

       for entry in WalkDir::new(root)
           .max_depth(4)  // Don't go too deep
           .into_iter()
           .filter_entry(|e| !is_ignored(e))
       {
           let entry = entry?;
           let path = entry.path();

           if path.ends_with(".pais/SKILL.md") {
               let skill = parse_skill_md(path)?;
               let repo_root = path.parent().unwrap().parent().unwrap();

               found.push(DiscoveredSkill {
                   name: skill.name,
                   description: skill.description,
                   source_path: path.parent().unwrap().to_path_buf(),
                   repo_path: repo_root.to_path_buf(),
               });
           }
       }

       Ok(found)
   }

   fn is_ignored(entry: &DirEntry) -> bool {
       let name = entry.file_name().to_string_lossy();
       name.starts_with('.') && name != ".pais"
           || name == "node_modules"
           || name == "target"
           || name == "venv"
           || name == "__pycache__"
   }
   ```

2. **Registration** (optional):
   ```rust
   fn register_discovered_skills(skills: &[DiscoveredSkill], config: &Config) -> Result<()> {
       // Option A: Add to manifest file
       // Option B: Create symlinks in skills/
       // Option C: Just remember for sync
   }
   ```

### Deliverable

```bash
pais skill scan ~/repos/scottidler

Found skills:
  aka        ~/repos/scottidler/aka/.pais/SKILL.md
  otto       ~/repos/scottidler/otto/.pais/SKILL.md
  cidr       ~/repos/scottidler/cidr/.pais/SKILL.md

Register these skills? [y/N] y
Registered 3 skills.

pais sync
# Now they're available in Claude Code
```

---

## Phase 5: Init as Git Repo

**Goal:** `pais init` creates `~/.config/pais/` as a trackable git repo

### Updated `pais init`

```bash
pais init                          # Initialize with git repo
pais init --no-git                 # Without git
```

### Implementation

```rust
fn init(path: Option<PathBuf>, no_git: bool, config: &Config) -> Result<()> {
    let pais_dir = path.unwrap_or_else(|| config.paths.pais_dir());

    // Create directory structure
    fs::create_dir_all(pais_dir.join("skills"))?;
    fs::create_dir_all(pais_dir.join("plugins"))?;
    fs::create_dir_all(pais_dir.join("history"))?;
    fs::create_dir_all(pais_dir.join("registries"))?;

    // Create default config
    if !pais_dir.join("pais.toml").exists() {
        fs::write(pais_dir.join("pais.toml"), DEFAULT_CONFIG)?;
    }

    // Create .gitignore
    let gitignore = r#"# Secrets
.env

# Installed plugins (from registries)
plugins/

# Runtime data
history/
registries/

# Cache
*.cache
"#;
    fs::write(pais_dir.join(".gitignore"), gitignore)?;

    // Initialize git repo
    if !no_git && !pais_dir.join(".git").exists() {
        Command::new("git")
            .args(["init"])
            .current_dir(&pais_dir)
            .status()?;

        Command::new("git")
            .args(["add", "-A"])
            .current_dir(&pais_dir)
            .status()?;

        Command::new("git")
            .args(["commit", "-m", "Initial PAIS configuration"])
            .current_dir(&pais_dir)
            .status()?;

        println!("Initialized git repo at {}", pais_dir.display());
    }

    // Set up Claude Code integration
    setup_claude_hooks(&pais_dir)?;

    println!("PAIS initialized at {}", pais_dir.display());
    println!("\nNext steps:");
    println!("  pais skill add <name>    # Create a skill");
    println!("  pais sync                # Sync to Claude Code");
}
```

### Deliverable

```bash
pais init

Initialized PAIS at /home/scott/.config/pais
Initialized git repo at /home/scott/.config/pais

Next steps:
  pais skill add <name>    # Create a skill
  pais sync                # Sync to Claude Code

# Check it's a git repo
cd ~/.config/pais && git status
# On branch main
# nothing to commit, working tree clean
```

---

## Phase 6: Migration

**Goal:** Move current plugins to their proper locations

### One-Time Script

```bash
#!/bin/bash
# migrate-skills.sh

PAIS_REPO=~/repos/personal-ai-system/pais
CONFIG_DIR=~/.config/pais

# Initialize config
pais init

# Move convention/documentation skills to config
for skill in rust-coder python-coder; do
    if [ -d "$PAIS_REPO/plugins/$skill" ]; then
        cp -r "$PAIS_REPO/plugins/$skill" "$CONFIG_DIR/skills/"
        echo "Moved $skill to config"
    fi
done

# For controlled tools, create .pais/ in their repos
declare -A TOOL_REPOS=(
    ["aka"]="$HOME/repos/scottidler/aka"
    ["otto"]="$HOME/repos/scottidler/otto"
    ["cidr"]="$HOME/repos/scottidler/cidr"
    ["dashify"]="$HOME/repos/scottidler/dashify"
    ["git-tools"]="$HOME/repos/scottidler/git-tools"
    ["rkvr"]="$HOME/repos/scottidler/rkvr"
    ["whitespace"]="$HOME/repos/scottidler/whitespace"
)

for skill in "${!TOOL_REPOS[@]}"; do
    repo="${TOOL_REPOS[$skill]}"
    if [ -d "$repo" ] && [ -d "$PAIS_REPO/plugins/$skill" ]; then
        mkdir -p "$repo/.pais"
        cp "$PAIS_REPO/plugins/$skill/SKILL.md" "$repo/.pais/"
        # Copy plugin.toml only if it has hooks/code
        if grep -q "hooks\|provides\|consumes" "$PAIS_REPO/plugins/$skill/plugin.toml" 2>/dev/null; then
            cp "$PAIS_REPO/plugins/$skill/plugin.toml" "$repo/.pais/"
        fi
        echo "Moved $skill to $repo/.pais/"
    fi
done

# Commit config changes
cd "$CONFIG_DIR"
git add -A
git commit -m "Migrate skills from pais repo"

# Clean up pais repo (keep examples only)
cd "$PAIS_REPO"
rm -rf plugins/aka plugins/otto plugins/cidr plugins/dashify
rm -rf plugins/git-tools plugins/rkvr plugins/whitespace
rm -rf plugins/rust-coder plugins/python-coder
# Keep: examples/hello-world, examples/hello-rust

git add -A
git commit -m "Remove personal skills (moved to proper locations)"

echo "Migration complete!"
echo ""
echo "Don't forget to commit .pais/ in each tool repo"
```

---

## Phase Summary

| Phase | Goal | Key Deliverable | Effort |
|-------|------|-----------------|--------|
| **1** | Simple skills support | Skills without plugin.toml | Medium |
| **2** | Skill commands | `pais skill add/list/edit` | Medium |
| **3** | Sync to Claude | `pais sync` | Small |
| **4** | Skill scanning | `pais skill scan` | Medium |
| **5** | Git repo init | `pais init` creates git repo | Small |
| **6** | Migration | Move current plugins | One-time |

---

## Recommended Order

```
Phase 1 ──► Phase 2 ──► Phase 3 ──► Phase 5 ──► Phase 6
                              │
                              └──► Phase 4 (can be parallel)
```

1. **Phase 1** first — Foundation for everything else
2. **Phase 2** next — Makes skills usable
3. **Phase 3** next — Connects to Claude Code
4. **Phase 5** next — Makes config trackable
5. **Phase 6** — Migrate when ready
6. **Phase 4** can happen anytime after Phase 2

---

## What Doesn't Change

- ✅ Plugin infrastructure (plugin.toml, contracts, hooks)
- ✅ Registry system
- ✅ Hook dispatch
- ✅ `pais plugin` commands
- ✅ All existing tests

---

## Success Criteria

After all phases:

```bash
# Create a skill for a tool I don't control
pais skill add terraform
# Edit it
pais skill edit terraform
# Sync to Claude Code
pais sync
# Verify Claude can use it
claude "How should I structure my terraform project?"
# Claude uses the terraform skill

# Skills in my tool repos are discovered
pais skill scan ~/repos
# Found: aka, otto, cidr...

# Config is tracked in git
cd ~/.config/pais && git log
# Shows history of skill changes

# Full plugins still work
pais plugin install incident
pais plugin list
# Shows incident plugin with its contracts
```

