# PAII CLI Reference

> Command reference for the `paii` CLI tool (Rust).

---

## Overview

The `paii` CLI is the main interface for managing plugins, dispatching hooks, and querying history. It's built in Rust for speed and reliability.

```bash
paii [OPTIONS] <COMMAND>
```

---

## Global Options

| Flag | Short | Description |
|------|-------|-------------|
| `--config <PATH>` | `-c` | Path to paii.toml (default: `~/.config/paii/paii.toml`) |
| `--verbose` | `-v` | Increase log verbosity (can be repeated: -vv, -vvv) |
| `--quiet` | `-q` | Suppress non-error output |
| `--help` | `-h` | Print help |
| `--version` | `-V` | Print version |

---

## Commands

### `paii plugin`

Manage plugins.

#### `paii plugin list`

List installed plugins.

```bash
paii plugin list [OPTIONS]

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

#### `paii plugin install`

Install a plugin.

```bash
paii plugin install <SOURCE> [OPTIONS]

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
| Git URL | `github.com/team/paii-plugins/jira` | Clone from git |
| Local | `./my-plugin` | Install from local path |
| Registry | `datadog` | Lookup in registries |

**Examples:**

```bash
# Install from core
paii plugin install hooks
paii plugin install history

# Install from git
paii plugin install github.com/your-company/paii-work-plugins/jira

# Install from local path (development)
paii plugin install ./incident --dev

# Force reinstall
paii plugin install history --force
```

#### `paii plugin remove`

Remove a plugin.

```bash
paii plugin remove <NAME> [OPTIONS]

Arguments:
  <NAME>     Plugin name

Options:
  --force    Remove even if other plugins depend on it
```

#### `paii plugin update`

Update a plugin to the latest version.

```bash
paii plugin update <NAME>

Arguments:
  <NAME>     Plugin name (or "all" to update all)
```

#### `paii plugin info`

Show detailed plugin information.

```bash
paii plugin info <NAME>

Arguments:
  <NAME>     Plugin name
```

**Example output:**

```yaml
name: incident
version: 1.0.0
description: Incident response workflows
language: python
path: ~/.config/paii/plugins/incident

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

#### `paii plugin new`

Scaffold a new plugin.

```bash
paii plugin new <NAME> [OPTIONS]

Arguments:
  <NAME>     Plugin name

Options:
  --language <LANG>   python or rust (default: python)
  --type <TYPE>       Plugin type: foundation, integration, skill (default: skill)
  --path <PATH>       Output path (default: ./<NAME>)
```

**Example:**

```bash
paii plugin new oncall --type skill --language python
```

Creates:

```
oncall/
├── plugin.toml
├── pyproject.toml
├── SKILL.md
├── src/
│   └── plugin.py
├── workflows/
│   └── example.md
└── tests/
    └── test_plugin.py
```

#### `paii plugin verify`

Verify a plugin is correctly installed.

```bash
paii plugin verify <NAME>

Arguments:
  <NAME>     Plugin name
```

---

### `paii hook`

Hook event handling (used by Claude Code integration).

#### `paii hook dispatch`

Dispatch a hook event to handlers.

```bash
paii hook dispatch <EVENT> [OPTIONS]

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
        "command": "paii hook dispatch pre-tool-use"
      }]
    }]
  }
}
```

#### `paii hook list`

List registered hook handlers.

```bash
paii hook list [OPTIONS]

Options:
  --event <EVENT>   Filter by event type
```

---

### `paii history`

Query and manage history.

#### `paii history query`

Search history.

```bash
paii history query <QUERY> [OPTIONS]

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
paii history query "authentication"

# Search specific category
paii history query "database" --category incidents

# Search recent only
paii history query "bug" --since 2026-01-01

# JSON output for scripting
paii history query "deploy" --json | jq '.[] | .path'
```

#### `paii history recent`

Show recent history entries.

```bash
paii history recent [OPTIONS]

Options:
  --category <CAT>   Category (default: all)
  --count <N>        Number of entries (default: 5)
```

#### `paii history categories`

List available history categories.

```bash
paii history categories
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

### `paii config`

Manage configuration.

#### `paii config show`

Show current configuration.

```bash
paii config show [OPTIONS]

Options:
  --json      Output as JSON
```

#### `paii config get`

Get a configuration value.

```bash
paii config get <KEY>

Arguments:
  <KEY>      Configuration key (dot notation: paths.history)
```

#### `paii config set`

Set a configuration value.

```bash
paii config set <KEY> <VALUE>

Arguments:
  <KEY>      Configuration key
  <VALUE>    New value
```

---

### `paii registry`

Manage plugin registries.

#### `paii registry list`

List configured registries.

```bash
paii registry list
```

#### `paii registry add`

Add a registry.

```bash
paii registry add <NAME> <URL>

Arguments:
  <NAME>     Registry name
  <URL>      Registry URL (git repo or direct URL)
```

#### `paii registry remove`

Remove a registry.

```bash
paii registry remove <NAME>

Arguments:
  <NAME>     Registry name
```

#### `paii registry update`

Update registry plugin listings.

```bash
paii registry update [NAME]

Arguments:
  [NAME]     Registry name (or update all if omitted)
```

---

### `paii run`

Run a plugin action directly.

```bash
paii run <PLUGIN> <ACTION> [ARGS...]

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
paii run jira get_issue --id PROJ-123

# Acknowledge PagerDuty incident
paii run pagerduty acknowledge --id P123456

# Search history
paii run history query --category incidents --query "database"
```

---

### `paii status`

Show system status.

```bash
paii status [OPTIONS]

Options:
  --json      Output as JSON
```

**Example output:**

```
PAII Status

  Version:    0.1.0
  Config:     ~/.config/paii/paii.toml
  Plugins:    ~/.config/paii/plugins/
  History:    ~/.config/paii/history/

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
| `PAII_DIR` | Base directory for PAII | `~/.config/paii` |
| `PAII_CONFIG` | Path to paii.toml | `$PAII_DIR/paii.toml` |
| `PAII_LOG_LEVEL` | Log level (trace, debug, info, warn, error) | `info` |
| `PAII_NO_COLOR` | Disable colored output | unset |

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

## Configuration File (`paii.toml`)

```toml
[paii]
version = "0.1.0"

[paths]
plugins = "~/.config/paii/plugins"
history = "~/.config/paii/history"
registries = "~/.config/paii/registries"

[defaults]
language = "python"
log_level = "info"

[registries]
core = "https://github.com/scottidler/paii/registry/plugins.toml"
# work = "https://github.com/your-company/paii-plugins/registry.toml"

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
paii completions bash > ~/.local/share/bash-completion/completions/paii

# Zsh
paii completions zsh > ~/.zfunc/_paii

# Fish
paii completions fish > ~/.config/fish/completions/paii.fish
```

---

## Related Documents

- [architecture.md](architecture.md) — System design
- [plugins.md](plugins.md) — Plugin development guide
- [contracts.md](contracts.md) — Contract specifications

