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
    minimal: []       # For skills: empty = load all (see "Empty List Semantics")
```

No separate `default-profile` field needed.

### Empty List Semantics

Empty profiles have different meanings for MCPs vs skills due to how they're consumed:

**MCPs:** Empty list = load no MCPs (Claude receives empty MCP config)
```bash
pais session -m minimal    # No MCPs loaded
```

**Skills:** Empty list = load ALL skills (no whitelist = everything passes)
```bash
pais session -s all        # All skills loaded (empty whitelist = no filtering)
```

This is because skills use a whitelist model:
- Non-empty PAIS_SKILLS → only those skills load
- Empty PAIS_SKILLS → no whitelist → all skills load

> **Future consideration:** Could add explicit `"*"` sentinel to mean "all" if empty-list semantics prove confusing.

## Architecture

### MCP Flow

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

```
pais session -s rust-coder -s otto
    │
    ▼
┌─────────────────────────────────────┐
│ session.rs                          │
│ 1. Parse -s flags                   │
│ 2. Expand any profile names         │
│ 3. Set PAIS_SKILLS env var          │
│    PAIS_SKILLS=rust-coder,otto      │
└─────────────────────────────────────┘
    │
    ▼
exec claude (env var inherited)
    │
    ▼
┌─────────────────────────────────────┐
│ Claude Code                         │
│ SessionStart hook fires             │
│ Runs: pais hook dispatch session-start
└─────────────────────────────────────┘
    │
    ▼
┌─────────────────────────────────────┐
│ pais context inject                 │
│ 1. Read PAIS_SKILLS env var         │
│ 2. Filter skills to only those      │
│ 3. Output filtered context          │
└─────────────────────────────────────┘
```

### Why Different Transport?

| Aspect | MCP | Skills |
|--------|-----|--------|
| Flag | `-m` / `--mcp` | `-s` / `--skill` |
| Transport | Temp JSON file | Environment variable |
| Consumer | Claude's `--mcp-config` | Our `pais context inject` |
| Reason | Claude reads the file | We control the injection |

Environment variable survives the `exec()` call, allowing `pais session` to communicate with `pais context inject` across processes.

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
      - rust-coder
      - otto
      - clone
    research:
      - fabric
      - tech-researcher
      - youtube
    writing:
      - writing-researcher
      - fabric
    minimal: []
```

### Implementation Details

#### Config structs (`config.rs`)

```rust
use indexmap::IndexMap;

/// MCP configuration
#[derive(Debug, Clone, Deserialize, Serialize, Default)]
#[serde(default, rename_all = "kebab-case")]
pub struct McpConfig {
    pub sources: Vec<PathBuf>,
    pub profiles: IndexMap<String, Vec<String>>,  // Ordered!
    pub servers: HashMap<String, McpServerConfig>,
}

/// Skills configuration
#[derive(Debug, Clone, Deserialize, Serialize, Default)]
#[serde(default, rename_all = "kebab-case")]
pub struct SkillsConfig {
    pub profiles: IndexMap<String, Vec<String>>,  // Ordered!
}
```

#### CLI flags (`cli.rs`)

```rust
Session {
    /// MCP servers or profiles to load (comma-separated)
    #[arg(short, long, value_delimiter = ',')]
    mcp: Option<Vec<String>>,

    /// Skills or profiles to load (comma-separated)
    #[arg(short, long, value_delimiter = ',')]
    skill: Option<Vec<String>>,

    #[arg(short, long)]
    list: bool,

    #[arg(long)]
    dry_run: bool,

    // ...
}
```

#### Resolution logic (`session.rs`)

```rust
/// Expand a list of names, replacing profile names with their contents
fn expand_names(
    names: &[String],
    profiles: &IndexMap<String, Vec<String>>,
) -> Vec<String> {
    let mut result = Vec::new();
    for name in names {
        if let Some(profile_contents) = profiles.get(name) {
            // It's a profile - expand it
            result.extend(profile_contents.iter().cloned());
        } else {
            // It's a direct name
            result.push(name.clone());
        }
    }
    // Deduplicate while preserving order
    let mut seen = HashSet::new();
    result.retain(|x| seen.insert(x.clone()));
    result
}

/// Get default from first profile (if any)
fn get_default(profiles: &IndexMap<String, Vec<String>>) -> Vec<String> {
    profiles.values().next().cloned().unwrap_or_default()
}
```

#### Context filtering (`context.rs`)

```rust
fn inject_context(raw: bool, config: &Config) -> Result<()> {
    // Read skill filter from environment
    let skill_filter: Option<HashSet<String>> = std::env::var("PAIS_SKILLS")
        .ok()
        .filter(|s| !s.is_empty())
        .map(|s| s.split(',').map(String::from).collect());

    // Filter skills based on PAIS_SKILLS
    let core_skills = if let Some(ref filter) = skill_filter {
        load_core_skills(&skills_dir, &index)
            .into_iter()
            .filter(|(name, _)| filter.contains(name))
            .collect()
    } else {
        load_core_skills(&skills_dir, &index)
    };

    // Similar filtering for deferred skills...
}
```

## Usage Examples

```bash
# Default session (uses first profile for both MCP and skills)
pais session

# Minimal session (nothing loaded)
pais session -m minimal -s minimal

# Development session
pais session -s dev                    # profile
pais session -s rust-coder -s otto     # individuals (same result)

# Research session with MCPs
pais session -m work -s research

# Mix profiles and individuals
pais session -s dev -s fabric          # dev profile + fabric skill

# List everything available
pais session --list

# Preview without launching
pais session -m work -s dev --dry-run
```

## Benefits

1. **Reduced context tokens** — Only load skills relevant to the task
2. **Faster startup** — Less content to inject
3. **Focused sessions** — Avoid skill confusion/overlap
4. **Simple mental model** — Profiles and names are interchangeable
5. **Sensible defaults** — First profile = default, no extra config

## Future Considerations

- Auto-detection based on repo type (detect Cargo.toml → suggest rust-coder)
- Skill dependencies (rust-coder requires core)
- Per-directory skill overrides (.pais/skills.yaml)
