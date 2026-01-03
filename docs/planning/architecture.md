# PAIS Architecture

> Technical design for Personal AI Infrastructure built on Claude Code.

---

## System Overview

PAIS is a **plugin layer** on top of Claude Code's native primitives. It does not replace Claude Code—it composes its features into a modular, team-shareable system.

```
┌─────────────────────────────────────────────────────────────────────────────┐
│                              USER INTERACTION                                │
│                                                                             │
│   "Help me respond to this PagerDuty incident"                              │
│                                                                             │
└─────────────────────────────────────────────────────────────────────────────┘
                                      │
                                      ▼
┌─────────────────────────────────────────────────────────────────────────────┐
│                              CLAUDE CODE                                     │
│                                                                             │
│   ┌─────────────┐ ┌─────────────┐ ┌─────────────┐ ┌─────────────┐          │
│   │    Hooks    │ │   Skills    │ │  Subagents  │ │     MCP     │          │
│   │  (events)   │ │ (SKILL.md)  │ │ (isolation) │ │ (external)  │          │
│   └──────┬──────┘ └──────┬──────┘ └──────┬──────┘ └──────┬──────┘          │
│          │               │               │               │                  │
└──────────┼───────────────┼───────────────┼───────────────┼──────────────────┘
           │               │               │               │
           ▼               ▼               ▼               ▼
┌─────────────────────────────────────────────────────────────────────────────┐
│                              PAIS LAYER                                      │
│                                                                             │
│   ┌─────────────────────────────────────────────────────────────────────┐   │
│   │                         PAIS CLI (Rust)                             │   │
│   │  • Plugin discovery and loading                                     │   │
│   │  • Contract resolution (provides/consumes)                          │   │
│   │  • Hook dispatcher (routes to plugins)                              │   │
│   │  • Plugin management (install/remove/list)                          │   │
│   └─────────────────────────────────────────────────────────────────────┘   │
│                                      │                                       │
│                    ┌─────────────────┼─────────────────┐                    │
│                    │                 │                 │                    │
│                    ▼                 ▼                 ▼                    │
│   ┌─────────────────────┐ ┌─────────────────┐ ┌─────────────────────┐      │
│   │   Foundation        │ │   Integrations  │ │   Skills            │      │
│   │   Plugins           │ │   Plugins       │ │   Plugins           │      │
│   │                     │ │                 │ │                     │      │
│   │   • hooks           │ │   • jira        │ │   • incident        │      │
│   │   • history         │ │   • slack       │ │   • runbook         │      │
│   │   • security        │ │   • github      │ │   • spanish         │      │
│   │                     │ │   • pagerduty   │ │   • writing         │      │
│   └─────────────────────┘ └─────────────────┘ └─────────────────────┘      │
│                                                                             │
└─────────────────────────────────────────────────────────────────────────────┘
```

---

## Directory Structure

```
~/.config/pais/                      # PAIS_DIR (configurable)
├── pais.yaml                        # Global configuration
├── .env                             # Secrets (API keys)
├── plugins/                         # Installed plugins
│   ├── hooks/                       # Foundation: hook handling
│   │   ├── plugin.yaml              # Plugin manifest
│   │   ├── Cargo.toml               # Rust project
│   │   └── src/
│   │       └── lib.rs
│   ├── history/                     # Foundation: memory system
│   │   ├── plugin.yaml
│   │   ├── pyproject.toml           # Python project
│   │   └── src/
│   │       └── plugin.py
│   ├── jira/                        # Integration: Atlassian
│   │   ├── plugin.yaml
│   │   └── src/
│   └── incident/                    # Skill: incident response
│       ├── plugin.yaml
│       ├── SKILL.md                 # Claude Code skill
│       └── src/
├── history/                         # Memory storage
│   ├── sessions/YYYY-MM/
│   ├── learnings/YYYY-MM/
│   ├── incidents/YYYY-MM/
│   ├── decisions/YYYY-MM/
│   └── raw/YYYY-MM/
└── registries/                      # Plugin registries
    ├── core.toml                    # From PAIS repo
    └── work.toml                    # Team private registry
```

---

## Integration with Claude Code

PAIS integrates with Claude Code through its native extension points:

### Hooks Integration

Claude Code fires hook events. PAIS's hook dispatcher routes them to plugins.

**Claude Code settings.json:**
```json
{
  "hooks": {
    "PreToolUse": [{
      "matcher": "Bash",
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

**PAIS dispatches to plugins:**
```
Claude Code Event → pais hook dispatch → Plugin handlers
```

### Skills Integration

PAIS plugins can provide Claude Code skills by including `SKILL.md` files:

```
plugins/incident/
├── plugin.yaml
├── SKILL.md              # ← Claude Code discovers this
└── workflows/
    ├── declare.md
    └── postmortem.md
```

Claude Code automatically loads skills from `~/.claude/skills/` and `.claude/skills/`. PAIS symlinks or copies plugin skills to these locations.

### Subagents Integration

Plugins can define subagents for isolated, specialized tasks:

```
plugins/incident/
├── plugin.yaml
├── agents/               # ← Claude Code discovers these
│   └── incident-coordinator/
│       └── AGENT.md
└── ...
```

### MCP Integration

Plugins can bundle MCP servers for external integrations:

```
plugins/jira/
├── plugin.yaml
├── .mcp.json             # ← MCP server configuration
└── src/
    └── server.py         # ← MCP server implementation
```

---

## Plugin System

### Plugin Manifest (`plugin.yaml`)

Every plugin has a manifest declaring its identity and contracts:

```toml
[plugin]
name = "incident"
version = "1.0.0"
description = "Incident response workflows"
authors = ["your-team"]
language = "python"  # or "rust" or "mixed"

[pais]
core_version = ">=0.1.0"  # Minimum PAIS version

[provides]
# Contracts this plugin implements
skill = "incident"
integration = false

[consumes]
# Contracts this plugin uses (optional by default)
memory = { contract = "MemoryProvider", optional = true }
pagerduty = { contract = "IntegrationProvider", service = "pagerduty", optional = true }
slack = { contract = "IntegrationProvider", service = "slack", optional = true }

[config]
# Configuration schema
escalation_threshold_minutes = { type = "integer", default = 30 }

[hooks]
# Hook events this plugin handles
pre_tool_use = false
stop = true
session_start = true

[build]
# Build configuration
type = "pip"  # or "cargo"
requirements = "requirements.txt"
```

### Contract System

Plugins communicate through **contracts**, not direct dependencies:

```
┌─────────────────────────────────────────────────────────────────────────────┐
│                            CONTRACT RESOLUTION                               │
│                                                                             │
│   At Load Time:                                                             │
│                                                                             │
│   1. Load all plugin manifests                                              │
│   2. Build provider map: contract → plugin                                  │
│   3. Wire up consumers to providers                                         │
│   4. Skip unavailable optional contracts                                    │
│                                                                             │
│   ┌─────────────┐        provides         ┌─────────────┐                  │
│   │   history   │ ──────────────────────► │MemoryProvider│                  │
│   │   plugin    │                         │  contract   │                  │
│   └─────────────┘                         └──────┬──────┘                  │
│                                                  │                          │
│                                                  │ consumes (optional)      │
│                                                  │                          │
│   ┌─────────────┐        provides         ┌─────▼───────┐                  │
│   │  incident   │ ──────────────────────► │    skill    │                  │
│   │   plugin    │                         │  contract   │                  │
│   └─────────────┘                         └─────────────┘                  │
│                                                                             │
└─────────────────────────────────────────────────────────────────────────────┘
```

### Plugin Loading Sequence

```
1. PAIS CLI starts
2. Scan ~/.config/pais/plugins/ for plugin.yaml files
3. Parse all manifests (no code loaded yet)
4. Build dependency graph from provides/consumes
5. Check for missing required contracts (fail if any)
6. Load plugins in safe order (providers before consumers)
7. For each plugin:
   a. If Rust: check for compiled binary, build if missing
   b. If Python: check for venv, install deps if missing
   c. Initialize plugin with config
   d. Register provided contracts
8. Wire consumers to providers
9. Ready for operation
```

---

## Data Flow

### Hook Event Flow

```
┌──────────────┐     event JSON      ┌──────────────┐
│  Claude Code │ ─────────────────► │  pais hook   │
│    fires     │     via stdin      │   dispatch   │
│  PreToolUse  │                    │              │
└──────────────┘                    └──────┬───────┘
                                           │
                      ┌────────────────────┼────────────────────┐
                      │                    │                    │
                      ▼                    ▼                    ▼
               ┌─────────────┐      ┌─────────────┐      ┌─────────────┐
               │  security   │      │   history   │      │   custom    │
               │   plugin    │      │   plugin    │      │   plugin    │
               │             │      │             │      │             │
               │ validates   │      │ logs event  │      │   ...       │
               │ command     │      │             │      │             │
               └──────┬──────┘      └─────────────┘      └─────────────┘
                      │
                      │ returns block/allow
                      ▼
               ┌─────────────┐
               │  pais hook  │
               │   returns   │
               │  exit code  │ ──────► 0=allow, 2=block
               └─────────────┘
```

### Skill Invocation Flow

```
┌──────────────┐                    ┌──────────────┐
│    User      │  "help with       │  Claude Code │
│   request    │  this incident"   │  routes to   │
│              │ ─────────────────►│  SKILL.md    │
└──────────────┘                    └──────┬───────┘
                                           │
                                           │ skill instructions
                                           ▼
                                    ┌──────────────┐
                                    │  Claude      │
                                    │  follows     │
                                    │  workflow    │
                                    └──────┬───────┘
                                           │
                      ┌────────────────────┼────────────────────┐
                      │                    │                    │
                      ▼                    ▼                    ▼
               ┌─────────────┐      ┌─────────────┐      ┌─────────────┐
               │ pais run    │      │ MCP server  │      │ subagent    │
               │ incident    │      │ (pagerduty) │      │ researcher  │
               │ --action X  │      │             │      │             │
               └─────────────┘      └─────────────┘      └─────────────┘
```

### Memory Capture Flow

```
┌──────────────┐     Stop event     ┌──────────────┐
│  Claude Code │ ─────────────────► │  pais hook   │
│  completes   │                    │   dispatch   │
│    task      │                    │              │
└──────────────┘                    └──────┬───────┘
                                           │
                                           ▼
                                    ┌──────────────┐
                                    │   history    │
                                    │   plugin     │
                                    │              │
                                    │ • categorize │
                                    │ • extract    │
                                    │ • store      │
                                    └──────┬───────┘
                                           │
                      ┌────────────────────┼────────────────────┐
                      │                    │                    │
                      ▼                    ▼                    ▼
               ┌─────────────┐      ┌─────────────┐      ┌─────────────┐
               │  sessions/  │      │ learnings/  │      │ incidents/  │
               │  YYYY-MM/   │      │  YYYY-MM/   │      │  YYYY-MM/   │
               └─────────────┘      └─────────────┘      └─────────────┘
```

---

## Technology Stack

### Core CLI (Rust)

```toml
# Cargo.toml
[dependencies]
clap = { version = "4", features = ["derive"] }
serde = { version = "1", features = ["derive"] }
serde_json = "1"
toml = "0.8"
tokio = { version = "1", features = ["full"] }
dirs = "5"
thiserror = "1"
tracing = "0.1"
```

**Responsibilities:**
- Plugin discovery and loading
- Hook event dispatch
- Contract resolution
- Configuration management
- CLI commands (`pais plugin install`, `pais hook dispatch`, etc.)

### Python Plugins

```toml
# pyproject.toml
[project]
requires-python = ">=3.11"
dependencies = [
    "pydantic>=2.0",
    "rich>=13.0",
]
```

**Responsibilities:**
- Skill implementations
- External integrations (Jira, Slack, etc.)
- Complex business logic
- Rapid iteration

### Rust Plugins

```toml
# Cargo.toml
[dependencies]
serde = { version = "1", features = ["derive"] }
serde_json = "1"
regex = "1"
```

**Responsibilities:**
- Security-critical hooks
- Performance-critical paths
- When Python overhead is unacceptable

---

## Configuration

### Global Configuration (`pais.yaml`)

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
```

### Secrets (`.env`)

```bash
# API Keys (single source of truth)
JIRA_API_TOKEN=xxx
SLACK_BOT_TOKEN=xxx
PAGERDUTY_API_KEY=xxx
GITHUB_TOKEN=xxx
OPENAI_API_KEY=xxx
ANTHROPIC_API_KEY=xxx
```

### Plugin Configuration

Each plugin reads config from its manifest defaults + user overrides:

```toml
# ~/.config/pais/plugins/incident/config.toml (user overrides)
escalation_threshold_minutes = 15
default_severity = "SEV-2"
```

---

## Security Model

### Hook Security

- Hooks run with user privileges
- Hooks must NEVER crash (exit 0 on errors)
- Security validator blocks dangerous commands
- Audit log captures all events

### Plugin Security

- Plugins are code — review before installing
- Plugin-local config prevents global pollution
- Secrets only in `.env`, never in plugin code
- Team plugins should be from trusted repos

### Integration Security

- API keys in `.env` only
- MCP servers run sandboxed where possible
- OAuth tokens stored securely by Claude Code

---

## Related Documents

- [vision.md](vision.md) — Philosophy and goals
- [contracts.md](contracts.md) — Interface specifications
- [cli.md](cli.md) — Command reference
- [plugins.md](plugins.md) — Plugin development guide
- [decisions.md](decisions.md) — Architecture decision records

