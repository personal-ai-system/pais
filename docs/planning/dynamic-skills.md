# Dynamic Skill Loading

> Load only the skills you need per session, reducing context token usage.

## Overview

Just as `pais session` supports dynamic MCP loading, we can apply the same pattern to skills. This lets you start focused sessions with only relevant skills loaded.

```bash
# Current: dynamic MCPs
pais session -m github -m slack

# Proposed: dynamic skills
pais session -s rust-coder -s otto

# Combined
pais session -m github -s rust-coder -s otto

# With profiles
pais session --mcp-profile work --skill-profile dev
```

## Architecture Comparison

### MCP Flow (Current)

```
pais session -m github -m slack
    │
    ▼
┌─────────────────────────────────────┐
│ session.rs                          │
│ 1. Parse --mcp flags                │
│ 2. Build temp JSON with selections  │
│ 3. Write to /tmp/pais-mcp-xxx.json  │
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

**Key:** Claude has built-in `--mcp-config` flag. We write the file, Claude reads it.

### Skills Flow (Proposed)

```
pais session -s rust-coder -s otto
    │
    ▼
┌─────────────────────────────────────┐
│ session.rs                          │
│ 1. Parse --skill flags              │
│ 2. Set PAIS_SKILLS env var          │
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

**Key:** Claude has no skill flag. We control injection via our hook, so we pass selection through environment variable.

### Why Different Transport Mechanisms?

| Aspect | MCP | Skills |
|--------|-----|--------|
| Selection flags | `-m` / `--mcp` | `-s` / `--skill` |
| Profile flags | `--mcp-profile` | `--skill-profile` |
| Transport | Temp JSON file | Environment variable |
| Consumer | Claude's `--mcp-config` | Our `pais context inject` |
| Why? | Claude reads the file | We control the injection |

The environment variable survives the `exec()` call, allowing `pais session` to communicate with `pais context inject` even though they run in separate processes.

## Implementation Plan

### 1. Add Skill Configuration (`config.rs`)

```yaml
# In pais.yaml
skills:
  profiles:
    minimal: []
    dev:
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
  default-profile: dev
```

### 2. Add CLI Flags (`cli.rs`)

```rust
/// Launch Claude Code with dynamic MCP/skill configuration
Session {
    /// MCP servers to load (repeatable)
    #[arg(short = 'm', long, action = ArgAction::Append)]
    mcp: Option<Vec<String>>,

    /// Use a named MCP profile
    #[arg(long)]
    mcp_profile: Option<String>,

    /// Skills to load (repeatable)
    #[arg(short = 's', long, action = ArgAction::Append)]
    skill: Option<Vec<String>>,

    /// Use a named skill profile
    #[arg(long)]
    skill_profile: Option<String>,

    /// List available MCPs, skills, and profiles
    #[arg(short, long)]
    list: bool,

    // ... rest unchanged
}
```

### 3. Set Environment Variable (`session.rs`)

```rust
fn launch_claude(
    mcp_config_path: Option<PathBuf>,
    skill_list: Vec<String>,
    extra_args: Vec<String>,
) -> Result<()> {
    let mut cmd = Command::new("claude");

    // MCP config (existing)
    cmd.arg("--strict-mcp-config");
    if let Some(ref path) = mcp_config_path {
        cmd.arg("--mcp-config").arg(path);
    }

    // Skill selection (new)
    if !skill_list.is_empty() {
        cmd.env("PAIS_SKILLS", skill_list.join(","));
    }

    cmd.args(&extra_args);
    let err = cmd.exec();
    Err(eyre!("Failed to exec claude: {}", err))
}
```

### 4. Filter Skills in Context Injection (`context.rs`)

```rust
fn inject_context(raw: bool, config: &Config) -> Result<()> {
    // Read skill filter from environment
    let skill_filter: Option<HashSet<String>> = std::env::var("PAIS_SKILLS")
        .ok()
        .filter(|s| !s.is_empty())
        .map(|s| s.split(',').map(String::from).collect());

    // ... existing index generation ...

    // Filter core skills
    let core_skills = if let Some(ref filter) = skill_filter {
        load_core_skills(&skills_dir, &index)
            .into_iter()
            .filter(|(name, _)| filter.contains(name))
            .collect()
    } else {
        load_core_skills(&skills_dir, &index)
    };

    // Filter deferred skills similarly...
}
```

## Usage Examples

```bash
# Minimal session (no skills)
pais session --skill-profile minimal

# Development session
pais session -s rust-coder -s otto -s clone

# Research session with MCPs
pais session -m slack -m github -s tech-researcher -s fabric

# Use profiles for both
pais session --mcp-profile work --skill-profile dev

# List everything available
pais session --list
```

## Benefits

1. **Reduced context tokens** - Only load skills relevant to the task
2. **Faster startup** - Less content to inject
3. **Focused sessions** - Avoid skill confusion/overlap
4. **Profiles** - Save common combinations for reuse

## Future Considerations

- Auto-detection based on repo type (detect Cargo.toml → suggest rust-coder)
- Skill dependencies (rust-coder requires core)
- Per-directory skill overrides (.pais/skills.yaml)
