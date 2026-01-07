# Dynamic Skill Loading

> Load only the skills you need per session, reducing context token usage.

## Overview

`pais session` supports dynamic loading for both MCP servers and skills. This lets you start focused sessions with only relevant capabilities loaded.

```bash
# Dynamic MCPs
pais session -m github -m slack

# Dynamic skills
pais session -s rust-coder -s otto

# Combined
pais session -m github -s rust-coder -s otto

# Using profiles (profiles and individual names are interchangeable)
pais session -m work -s dev
```

## Design Principles

### Unified Flag Behavior

Both `-m`/`--mcp` and `-s`/`--skill` flags accept **either profile names or individual names**:

```bash
# These are equivalent if 'dev' profile contains [rust-coder, otto]
pais session -s dev
pais session -s rust-coder -s otto

# Mix profiles and individuals
pais session -s dev -s fabric    # expands to: rust-coder, otto, fabric
```

**Resolution order for each name:**
1. Check if name matches a profile → expand to profile's list
2. Otherwise treat as individual MCP/skill name

### First Profile = Default

Profiles use `IndexMap` to preserve YAML ordering. The **first profile defined is the default** when no flags are provided:

```yaml
skills:
  profiles:
    dev:              # ← First = default (used when no -s flag)
      - rust-coder
      - otto
    research:
      - fabric
      - tech-researcher
    all: []           # For skills: empty = load all (see "Empty List Semantics")
```

No separate `default-profile` field needed.

### Empty List Semantics

Empty profiles have different meanings for MCPs vs skills due to how they're consumed:

**MCPs:** Empty list = load no MCPs (Claude receives empty MCP config)
```bash
pais session -m minimal    # No MCPs loaded
```

**Skills:** Empty list = load ALL skills (all symlinks created)
```bash
pais session -s all        # All skills loaded
```

## Architecture

### MCP Flow

MCPs use Claude Code's native `--mcp-config` flag:

```
pais session -m github -m slack
    │
    ▼
┌─────────────────────────────────────┐
│ session.rs                          │
│ 1. Parse -m flags                   │
│ 2. Expand any profile names         │
│ 3. Build temp JSON with selections  │
│ 4. Write to /tmp/pais-mcp-xxx.json  │
└─────────────────────────────────────┘
    │
    ▼
exec claude --strict-mcp-config --mcp-config /tmp/pais-mcp-xxx.json
    │
    ▼
┌─────────────────────────────────────┐
│ Claude Code                         │
│ Reads --mcp-config file             │
│ Loads only specified MCPs           │
└─────────────────────────────────────┘
```

### Skills Flow

Claude Code has NO native skill filtering - it loads all skills from `~/.claude/skills/`.
We work around this by managing symlinks in that directory:

```
pais session -s rust-coder -s otto
    │
    ▼
┌─────────────────────────────────────┐
│ session.rs                          │
│ 1. Parse -s flags                   │
│ 2. Expand any profile names         │
│ 3. Sync symlinks in ~/.claude/skills│
│    - Remove unwanted symlinks       │
│    - Add missing symlinks           │
│    - Leave common ones untouched    │
└─────────────────────────────────────┘
    │
    ▼
exec claude
    │
    ▼
┌─────────────────────────────────────┐
│ Claude Code                         │
│ Loads skills from ~/.claude/skills/ │
│ Only sees symlinks we created       │
└─────────────────────────────────────┘
```

### Smart Symlink Sync

The symlink management uses set comparison to minimize churn:

```
Current symlinks:  {rust-coder, fabric, youtube}
Requested skills:  {rust-coder, otto, clone}

Compute diff:
  - to_remove: {fabric, youtube}     # current - requested
  - to_add:    {otto, clone}         # requested - current
  - unchanged: {rust-coder}          # intersection

Only modify what's necessary:
  - Remove fabric, youtube symlinks
  - Create otto, clone symlinks
  - Leave rust-coder untouched
```

This avoids unnecessary filesystem operations when skills overlap between sessions.

### Why Different Approaches?

| Aspect | MCP | Skills |
|--------|-----|--------|
| Flag | `-m` / `--mcp` | `-s` / `--skill` |
| Mechanism | Temp JSON file | Symlink management |
| Consumer | Claude's `--mcp-config` | Claude's skill loader |
| Reason | Claude has native support | Claude has NO native filtering |

Claude Code natively supports `--mcp-config` + `--strict-mcp-config` for MCPs.
It has no equivalent for skills, so we manage symlinks instead.

## Skill Sources

Skills can come from two locations:

1. **Dedicated skills:** `~/.config/pais/skills/<name>/SKILL.md`
2. **Plugin skills:** `~/.config/pais/plugins/<name>/SKILL.md`

When syncing, we check both locations to find skill source paths.

## Configuration

### pais.yaml Structure

```yaml
# MCP configuration
mcp:
  sources:
    - ~/.mcp.json

  profiles:
    minimal: []           # First = default
    github:
      - multi-account-github
    work:
      - multi-account-github
      - slack
      - atlassian

  servers:
    multi-account-github:
      command: multi-account-github-mcp
      args: [serve]

# Skills configuration
skills:
  profiles:
    dev:                  # First = default
      - core
      - rust-coder
      - otto
      - clone
    research:
      - core
      - fabric
      - tech-researcher
      - youtube
    writing:
      - core
      - writing-researcher
      - fabric
    all: []               # Empty = load all available skills
```

### Implementation Details

#### Symlink sync logic (`session.rs`)

```rust
/// Sync skill symlinks in ~/.claude/skills/ to match the requested skill list
fn sync_skill_symlinks(skill_list: &[String], config: &Config) -> Result<SyncResult> {
    let claude_skills_dir = get_claude_skills_dir()?;

    // Get current state
    let current_symlinks = get_current_symlinks(&claude_skills_dir);
    let current_names: HashSet<String> = current_symlinks.keys().cloned().collect();

    // Determine requested skills (empty list = all available)
    let requested_names: HashSet<String> = if skill_list.is_empty() {
        get_all_skill_names(config)
    } else {
        skill_list.iter().cloned().collect()
    };

    // Compute diff
    let to_remove = current_names.difference(&requested_names);
    let to_add = requested_names.difference(&current_names);
    let unchanged = current_names.intersection(&requested_names);

    // Apply changes
    for name in to_remove {
        fs::remove_file(claude_skills_dir.join(name))?;
    }
    for name in to_add {
        if let Some(source) = find_skill_source(name, config) {
            unix_fs::symlink(&source, claude_skills_dir.join(name))?;
        }
    }

    Ok(SyncResult { added, removed, unchanged, not_found })
}
```

#### Context injection (`context.rs`)

Context injection reads from `~/.claude/skills/` symlinks to know which skills to include:

```rust
fn get_skill_filter() -> Option<HashSet<String>> {
    let claude_skills_dir = home_dir()?.join(".claude").join("skills");

    if !claude_skills_dir.exists() {
        return None;  // No filtering
    }

    let symlinks: HashSet<String> = read_dir(&claude_skills_dir)
        .ok()?
        .flatten()
        .filter(|e| e.path().is_symlink())
        .filter_map(|e| e.file_name().to_str().map(String::from))
        .collect();

    if symlinks.is_empty() { None } else { Some(symlinks) }
}
```

## Usage Examples

```bash
# Default session (uses first profile for both MCP and skills)
pais session

# Development session
pais session -s dev                    # profile
pais session -s rust-coder -s otto     # individuals (same result)

# Research session with MCPs
pais session -m work -s research

# Mix profiles and individuals
pais session -s dev -s fabric          # dev profile + fabric skill

# Load all skills (empty profile)
pais session -s all

# List everything available
pais session --list

# Preview without launching
pais session -m work -s dev --dry-run
```

### Dry Run Output

```
Dry run - would launch Claude with:
  MCPs: multi-account-github, slack
  MCP servers found: 2
  Skills: core, rust-coder, otto, clone

Skill symlink changes:
  - fabric
  - youtube
  + clone
  Unchanged: 3
  Extra args: []
```

## Benefits

1. **Reduced context tokens** — Only load skills relevant to the task
2. **Faster startup** — Less content to inject
3. **Focused sessions** — Avoid skill confusion/overlap
4. **Simple mental model** — Profiles and names are interchangeable
5. **Sensible defaults** — First profile = default, no extra config
6. **Minimal churn** — Smart sync only changes what's needed

## Future Considerations

- Auto-detection based on repo type (detect Cargo.toml → suggest rust-coder)
- Skill dependencies (rust-coder requires core)
- Per-directory skill overrides (.pais/skills.yaml)
