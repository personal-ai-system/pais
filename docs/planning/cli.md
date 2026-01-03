# PAIS CLI Reference

> Command reference for the `pais` CLI tool (Rust).

---

## Overview

The `pais` CLI is the main interface for managing plugins, dispatching hooks, and querying history. It's built in Rust for speed and reliability.

```bash
pais [OPTIONS] <COMMAND>
```

---

## Global Options

| Flag | Short | Description |
|------|-------|-------------|
| `--config <PATH>` | `-c` | Path to pais.yaml (default: `~/.config/pais/pais.yaml`) |
| `--verbose` | `-v` | Increase log verbosity (can be repeated: -vv, -vvv) |
| `--quiet` | `-q` | Suppress non-error output |
| `--help` | `-h` | Print help |
| `--version` | `-V` | Print version |

---

## Commands

### `pais plugin`

Manage plugins.

#### `pais plugin list`

List installed plugins.

```bash
pais plugin list [OPTIONS]

Options:
  --json           Output as JSON
  --verbose        Show contract details
```

**Example output:**

```
Installed plugins:

  hooks (1.0.0)         [rust]    Foundation: hook handling
    provides: HookHandler

  history (1.0.0)       [python]  File-based memory system
    provides: MemoryProvider
    consumes: HookHandler (optional)

  jira (0.5.0)          [python]  Jira integration
    provides: IntegrationProvider[jira]

  incident (1.0.0)      [python]  Incident response workflows
    provides: SkillProvider[incident]
    consumes: MemoryProvider, IntegrationProvider[pagerduty, slack] (optional)
```

#### `pais plugin install`

Install a plugin.

```bash
pais plugin install <SOURCE> [OPTIONS]

Arguments:
  <SOURCE>   Plugin source (name, git URL, or local path)

Options:
  --dev      Symlink for development (don't copy)
  --force    Overwrite existing installation
  --no-deps  Skip dependency installation
```

**Source types:**

| Type | Example | Description |
|------|---------|-------------|
| Name | `history` | Install from core repo |
| Git URL | `github.com/team/pais-plugins/jira` | Clone from git |
| Local | `./my-plugin` | Install from local path |
| Registry | `datadog` | Lookup in registries |

**Examples:**

```bash
# Install from core
pais plugin install hooks
pais plugin install history

# Install from git
pais plugin install github.com/your-company/pais-work-plugins/jira

# Install from local path (development)
pais plugin install ./incident --dev

# Force reinstall
pais plugin install history --force
```

#### `pais plugin remove`

Remove a plugin.

```bash
pais plugin remove <NAME> [OPTIONS]

Arguments:
  <NAME>     Plugin name

Options:
  --force    Remove even if other plugins depend on it
```

#### `pais plugin update`

Update a plugin to the latest version.

```bash
pais plugin update <NAME>

Arguments:
  <NAME>     Plugin name (or "all" to update all)
```

#### `pais plugin info`

Show detailed plugin information.

```bash
pais plugin info <NAME>

Arguments:
  <NAME>     Plugin name
```

**Example output:**

```yaml
name: incident
version: 1.0.0
description: Incident response workflows
language: python
path: ~/.config/pais/plugins/incident

provides:
  - SkillProvider[incident]

consumes:
  memory:
    contract: MemoryProvider
    optional: true
    provider: history (1.0.0)
  pagerduty:
    contract: IntegrationProvider
    service: pagerduty
    optional: true
    provider: pagerduty (0.3.0)
  slack:
    contract: IntegrationProvider
    service: slack
    optional: true
    provider: null  # Not installed

config:
  escalation_threshold_minutes: 30
  default_severity: SEV-2
```

#### `pais plugin new`

Scaffold a new plugin.

```bash
pais plugin new <NAME> [OPTIONS]

Arguments:
  <NAME>     Plugin name

Options:
  --language <LANG>   python or rust (default: python)
  --type <TYPE>       Plugin type: foundation, integration, skill (default: skill)
  --path <PATH>       Output path (default: ./<NAME>)
```

**Example:**

```bash
pais plugin new oncall --type skill --language python
```

Creates:

```
oncall/
├── plugin.yaml
├── pyproject.toml
├── SKILL.md
├── src/
│   └── plugin.py
├── workflows/
│   └── example.md
└── tests/
    └── test_plugin.py
```

#### `pais plugin verify`

Verify a plugin is correctly installed.

```bash
pais plugin verify <NAME>

Arguments:
  <NAME>     Plugin name
```

---

### `pais hook`

Hook event handling (used by Claude Code integration).

#### `pais hook dispatch`

Dispatch a hook event to handlers.

```bash
pais hook dispatch <EVENT> [OPTIONS]

Arguments:
  <EVENT>    Event type: pre-tool-use, post-tool-use, stop, session-start, etc.

Options:
  --payload <JSON>   Event payload (reads from stdin if not provided)
```

**Exit codes:**

| Code | Meaning |
|------|---------|
| 0 | Allow / Success |
| 2 | Block (for PreToolUse) |
| 1 | Error |

**Example (from Claude Code settings.json):**

```json
{
  "hooks": {
    "PreToolUse": [{
      "matcher": "Bash",
      "hooks": [{
        "type": "command",
        "command": "pais hook dispatch pre-tool-use"
      }]
    }]
  }
}
```

#### `pais hook list`

List registered hook handlers.

```bash
pais hook list [OPTIONS]

Options:
  --event <EVENT>   Filter by event type
```

---

### `pais history`

Query and manage history.

#### `pais history query`

Search history.

```bash
pais history query <QUERY> [OPTIONS]

Arguments:
  <QUERY>    Search query (regex)

Options:
  --category <CAT>   Category to search (sessions, learnings, incidents, etc.)
  --limit <N>        Max results (default: 10)
  --since <DATE>     Only entries after this date
  --json             Output as JSON
```

**Examples:**

```bash
# Search all categories
pais history query "authentication"

# Search specific category
pais history query "database" --category incidents

# Search recent only
pais history query "bug" --since 2026-01-01

# JSON output for scripting
pais history query "deploy" --json | jq '.[] | .path'
```

#### `pais history recent`

Show recent history entries.

```bash
pais history recent [OPTIONS]

Options:
  --category <CAT>   Category (default: all)
  --count <N>        Number of entries (default: 5)
```

#### `pais history categories`

List available history categories.

```bash
pais history categories
```

**Example output:**

```
sessions    (1,234 entries)
learnings   (456 entries)
incidents   (89 entries)
decisions   (67 entries)
raw         (5,678 entries)
```

---

### `pais config`

Manage configuration.

#### `pais config show`

Show current configuration.

```bash
pais config show [OPTIONS]

Options:
  --json      Output as JSON
```

#### `pais config get`

Get a configuration value.

```bash
pais config get <KEY>

Arguments:
  <KEY>      Configuration key (dot notation: paths.history)
```

#### `pais config set`

Set a configuration value.

```bash
pais config set <KEY> <VALUE>

Arguments:
  <KEY>      Configuration key
  <VALUE>    New value
```

---

### `pais registry`

Manage plugin registries.

#### `pais registry list`

List configured registries.

```bash
pais registry list
```

#### `pais registry add`

Add a registry.

```bash
pais registry add <NAME> <URL>

Arguments:
  <NAME>     Registry name
  <URL>      Registry URL (git repo or direct URL)
```

#### `pais registry remove`

Remove a registry.

```bash
pais registry remove <NAME>

Arguments:
  <NAME>     Registry name
```

#### `pais registry update`

Update registry plugin listings.

```bash
pais registry update [NAME]

Arguments:
  [NAME]     Registry name (or update all if omitted)
```

---

### `pais run`

Run a plugin action directly.

```bash
pais run <PLUGIN> <ACTION> [ARGS...]

Arguments:
  <PLUGIN>   Plugin name
  <ACTION>   Action to run
  [ARGS]     Action arguments

Options:
  --json     Output as JSON
```

**Examples:**

```bash
# Query Jira
pais run jira get_issue --id PROJ-123

# Acknowledge PagerDuty incident
pais run pagerduty acknowledge --id P123456

# Search history
pais run history query --category incidents --query "database"
```

---

### `pais status`

Show system status.

```bash
pais status [OPTIONS]

Options:
  --json      Output as JSON
```

**Example output:**

```
PAIS Status

  Version:    0.1.0
  Config:     ~/.config/pais/pais.yaml
  Plugins:    ~/.config/pais/plugins/
  History:    ~/.config/pais/history/

Plugins (5 installed):
  ✓ hooks       1.0.0   [rust]
  ✓ history     1.0.0   [python]
  ✓ jira        0.5.0   [python]
  ✓ pagerduty   0.3.0   [python]
  ✓ incident    1.0.0   [python]

Contracts:
  ✓ HookHandler        → hooks
  ✓ MemoryProvider     → history
  ✓ IntegrationProvider[jira] → jira
  ✓ IntegrationProvider[pagerduty] → pagerduty
  ✓ SkillProvider[incident] → incident

History:
  sessions:   1,234 entries (latest: 2 hours ago)
  learnings:  456 entries (latest: 1 day ago)
  incidents:  89 entries (latest: 3 days ago)
```

---

## Environment Variables

| Variable | Description | Default |
|----------|-------------|---------|
| `PAIS_DIR` | Base directory for PAIS | `~/.config/pais` |
| `PAIS_CONFIG` | Path to pais.yaml | `$PAIS_DIR/pais.yaml` |
| `PAIS_LOG_LEVEL` | Log level (trace, debug, info, warn, error) | `info` |
| `PAIS_NO_COLOR` | Disable colored output | unset |

---

## Exit Codes

| Code | Meaning |
|------|---------|
| 0 | Success |
| 1 | General error |
| 2 | Block (for hook dispatch) |
| 10 | Plugin not found |
| 11 | Contract not satisfied |
| 12 | Configuration error |
| 20 | Network error |
| 21 | Authentication error |

---

## Configuration File (`pais.yaml`)

```toml
[pais]
version = "0.1.0"

[paths]
plugins = "~/.config/pais/plugins"
history = "~/.config/pais/history"
registries = "~/.config/pais/registries"

[defaults]
language = "python"
log_level = "info"

[registries]
core = "https://github.com/scottidler/pais/registry/plugins.toml"
# work = "https://github.com/your-company/pais-plugins/registry.toml"

[hooks]
# Global hook configuration
security_enabled = true
history_enabled = true
```

---

## Shell Completions

Generate shell completions:

```bash
# Bash
pais completions bash > ~/.local/share/bash-completion/completions/pais

# Zsh
pais completions zsh > ~/.zfunc/_pais

# Fish
pais completions fish > ~/.config/fish/completions/pais.fish
```

---

## Related Documents

- [architecture.md](architecture.md) — System design
- [plugins.md](plugins.md) — Plugin development guide
- [contracts.md](contracts.md) — Contract specifications

