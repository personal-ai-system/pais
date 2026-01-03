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
| **4** | Skill Scanning | ✅ Complete | 2026-01-02 |
| **5** | Git Repo Init | ✅ Complete | 2026-01-02 |
| **6** | Migration | ✅ Complete | 2026-01-02 |

---

## Overview

This document breaks the skills architecture into implementable phases. Each phase is independently useful and builds toward the complete vision.

---

## Phase 1: Simple Skills Support ✅

**Goal:** Support SKILL.md-only skills (no plugin.yaml required)

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

## Phase 4: Skill Scanning ✅

**Goal:** Discover `.pais/SKILL.md` in repositories you control

**Status:** Complete (2026-01-02)

### What Was Implemented

| File | Purpose |
|------|---------|
| `src/skill/scanner.rs` | Scan directories for `.pais/SKILL.md` files |
| `src/commands/skill.rs` | Added `scan` command handler with `--register` option |
| `src/cli.rs` | Added `Scan` variant to `SkillAction` |
| `Cargo.toml` | Added `walkdir` dependency |

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

## Phase 5: Init as Git Repo ✅

**Goal:** `pais init` creates `~/.config/pais/` as a trackable git repo

**Status:** Complete (2026-01-02)

### What Was Implemented

| File | Purpose |
|------|---------|
| `src/commands/init.rs` | Updated to create skills/, .gitignore, init git repo |
| `src/cli.rs` | Added `--no-git` flag to Init command |

### Updated `pais init`

```bash
pais init                          # Initialize with git repo
pais init --no-git                 # Without git
```

### Key Features

1. **Directory structure**: Creates `plugins/`, `skills/`, `history/`, `registries/`
2. **.gitignore**: Excludes plugins/, history/, registries/, secrets
3. **Git initialization**: Auto-creates repo with initial commit (unless `--no-git`)
4. **Claude Code integration**: Creates `~/.claude/skills/` directory

### Deliverable

```bash
pais init

→ Initializing PAIS in /home/scott/.config/pais
  ✓ Created plugins/
  ✓ Created skills/
  ✓ Created history/
  ✓ Created registries/
  ✓ Created history subdirectories
  ✓ Created pais.yaml
  ✓ Created .gitignore
  ✓ Initialized git repository
  ✓ Created initial commit

✓ PAIS initialized!

Next steps:
  1. Run pais doctor to verify setup
  2. Run pais skill add <name> to create a skill
  3. Run pais sync to sync to Claude Code

Git repository:
  /home/scott/.config/pais is now a git repo
  Your skills and config are version controlled
```

---

## Phase 6: Migration ✅

**Goal:** Move current plugins to their proper locations

**Status:** Complete (2026-01-02)

### What Was Implemented

**Decision:** All skills centralized in `~/.config/pais/skills/` rather than distributed to individual repos.

This simplifies management:
- One repo (`scottidler/pais`) to manage all skills
- Easy backup/sync - push one repo
- No need to commit `.pais/` folders to each tool repo

### Migration Steps Performed

1. **Created `scottidler/pais` repo** on GitHub
2. **Cloned to `~/.config/pais/`**
3. **Ran `pais init`** to create directory structure
4. **Copied all skills** from `personal-ai-system/pais/plugins/` to `~/.config/pais/skills/`

### Skills Migrated

| Skill | Description |
|-------|-------------|
| aka | Shell alias manager |
| cidr | Network CIDR calculator |
| dashify | Filename normalizer |
| git-tools | Git productivity tools |
| otto | Task runner |
| python-coder | Python coding conventions |
| rkvr | Safe file deletion |
| rust-coder | Rust coding conventions |
| whitespace | Trailing whitespace remover |

### Next Steps

```bash
# Commit the migrated skills
cd ~/.config/pais
git add -A
git commit -m "Migrate skills from pais repo"
git push

# Sync to Claude Code
pais sync

# Optionally clean up old plugins from pais repo
cd ~/repos/personal-ai-system/pais
rm -rf plugins/
git add -A
git commit -m "Remove plugins (migrated to scottidler/pais)"
```

### Future Considerations

If distributing skills with tool repos becomes desirable later, `pais skill scan` (Phase 4) already supports discovering `.pais/SKILL.md` files in repositories.

---

## Phase Summary

| Phase | Goal | Key Deliverable | Effort |
|-------|------|-----------------|--------|
| **1** | Simple skills support | Skills without plugin.yaml | Medium |
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

- ✅ Plugin infrastructure (plugin.yaml, contracts, hooks)
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

