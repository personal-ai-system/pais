# PAIS Plugin Development Guide

> How to create, test, and distribute PAIS plugins.

---

## Quick Start

### 1. Scaffold a New Plugin

```bash
pais plugin new my-skill --type skill --language python
cd my-skill
```

This creates:

```
my-skill/
├── plugin.toml           # Plugin manifest
├── pyproject.toml        # Python dependencies
├── SKILL.md              # Claude Code skill definition
├── src/
│   └── plugin.py         # Plugin implementation
├── workflows/
│   └── example.md        # Example workflow
└── tests/
    └── test_plugin.py    # Tests
```

### 2. Define Your Plugin

Edit `plugin.toml`:

```toml
[plugin]
name = "my-skill"
version = "0.1.0"
description = "My custom skill for doing X"
authors = ["your-name"]
language = "python"

[pais]
core_version = ">=0.1.0"

[provides]
skill = "my-skill"

[consumes]
memory = { contract = "MemoryProvider", optional = true }

[config]
some_setting = { type = "string", default = "default_value" }
```

### 3. Implement the Plugin

Edit `src/plugin.py`:

```python
from pais.plugin import Plugin, PluginContext

class MySkillPlugin(Plugin):
    def __init__(self, context: PluginContext, config: dict):
        self.context = context
        self.config = config

        # Access optional contracts
        self.memory = context.get_contract("memory")

    def skill_name(self) -> str:
        return "my-skill"

    def match_intent(self, intent: str) -> float:
        keywords = ["my-skill", "do x", "thing"]
        return 0.9 if any(k in intent.lower() for k in keywords) else 0.0

    def execute(self, action: str, context: dict) -> dict:
        # Your skill logic here
        result = f"Executed {action} with {context}"

        # Store in memory if available
        if self.memory:
            self.memory.capture("sessions", result, {"action": action})

        return {"success": True, "result": result}

# Required: export the plugin class
Plugin = MySkillPlugin
```

### 4. Write the Skill Definition

Edit `SKILL.md`:

```markdown
---
name: my-skill
description: Does X when you ask about Y. USE WHEN user wants to do something specific.
---

# My Skill

## When to Use

Use this skill when the user wants to:
- Do X
- Handle Y
- Process Z

## Workflow Routing

- Basic action -> `workflows/basic.md`
- Advanced action -> `workflows/advanced.md`

## Examples

**Example 1:**
```
User: "Help me do X"
-> Invokes basic workflow
```
```

### 5. Install and Test

```bash
# Install in development mode (symlink)
pais plugin install ./my-skill --dev

# Verify installation
pais plugin verify my-skill

# Test the plugin
pais run my-skill execute --action test
```

---

## Plugin Types

### Foundation Plugins

Provide infrastructure that other plugins build on.

**Examples:** `hooks`, `history`, `security`

**Characteristics:**
- Written in Rust for performance/reliability
- Provide core contracts (MemoryProvider, HookHandler)
- Loaded first in dependency order
- No external dependencies

### Integration Plugins

Connect to external services.

**Examples:** `jira`, `slack`, `pagerduty`, `github`

**Characteristics:**
- Written in Python (SDK availability)
- Provide IntegrationProvider contract
- Require API credentials
- May bundle MCP servers

### Skill Plugins

Teach Claude workflows.

**Examples:** `incident`, `runbook`, `writing`, `spanish`

**Characteristics:**
- Written in Python
- Provide SkillProvider contract
- Include SKILL.md for Claude Code
- May have workflow files

---

## Plugin Structure

### Minimal Structure

```
my-plugin/
├── plugin.toml      # Required: manifest
└── src/
    └── plugin.py    # Required: implementation
```

### Full Structure

```
my-plugin/
├── plugin.toml           # Plugin manifest
├── pyproject.toml        # Python dependencies
├── Cargo.toml            # Rust dependencies (if Rust)
├── SKILL.md              # Claude Code skill (if skill plugin)
├── .mcp.json             # MCP server config (if integration)
├── src/
│   ├── plugin.py         # Main plugin
│   ├── handlers.py       # Hook handlers
│   └── utils.py          # Utilities
├── agents/               # Subagent definitions
│   └── researcher/
│       └── AGENT.md
├── workflows/            # Skill workflows
│   ├── basic.md
│   └── advanced.md
├── config/
│   └── defaults.toml     # Default configuration
└── tests/
    ├── test_plugin.py
    └── fixtures/
```

---

## Plugin Manifest (`plugin.toml`)

### Required Fields

```toml
[plugin]
name = "my-plugin"           # Unique identifier (lowercase, hyphens)
version = "1.0.0"            # SemVer version
description = "What this plugin does"
language = "python"          # python, rust, or mixed
```

### Optional Fields

```toml
[plugin]
authors = ["Your Name <email@example.com>"]
license = "MIT"
repository = "https://github.com/you/my-plugin"
homepage = "https://my-plugin.example.com"
keywords = ["sre", "incident", "automation"]

[pais]
core_version = ">=0.1.0"     # Required PAIS version
```

### Contracts

```toml
[provides]
# Contracts this plugin implements
memory = "MemoryProvider"
hook = "HookHandler"
skill = "my-skill"                              # For SkillProvider
integration = { contract = "IntegrationProvider", service = "jira" }

[consumes]
# Contracts this plugin uses
memory = { contract = "MemoryProvider", optional = true }
notifications = { contract = "NotificationProvider", optional = true }
jira = { contract = "IntegrationProvider", service = "jira", optional = true }
```

### Configuration Schema

```toml
[config]
# Define configuration with type and defaults
api_url = { type = "string", required = true }
timeout_seconds = { type = "integer", default = 30 }
enabled_features = { type = "array", default = ["feature1", "feature2"] }
debug_mode = { type = "boolean", default = false }

# Environment variable mapping
api_token = { type = "string", required = true, env = "MY_PLUGIN_TOKEN", secret = true }
```

### Hook Registration

```toml
[hooks]
# Which events this plugin handles
pre_tool_use = true
post_tool_use = false
stop = true
session_start = true
session_end = false
subagent_stop = false
```

### Build Configuration

```toml
[build]
type = "pip"                 # pip, cargo, or custom
requirements = "requirements.txt"
install_command = "pip install -e ."
build_command = "python -m build"
```

---

## Python Plugins

### Base Class

```python
from abc import ABC, abstractmethod
from typing import Any
from pathlib import Path

class PluginContext:
    """Provided to plugins at initialization."""

    def get_contract(self, alias: str) -> Any | None:
        """Get a consumed contract by alias."""
        ...

    def has_contract(self, alias: str) -> bool:
        """Check if a contract is available."""
        ...

    def get_config(self, key: str, default: Any = None) -> Any:
        """Get a configuration value."""
        ...

    def plugin_dir(self) -> Path:
        """Get the plugin's installation directory."""
        ...


class Plugin(ABC):
    """Base class for all plugins."""

    @abstractmethod
    def __init__(self, context: PluginContext, config: dict):
        """Initialize with context and configuration."""
        ...
```

### Implementing Contracts

#### MemoryProvider

```python
from pathlib import Path
from datetime import datetime
import yaml

class HistoryPlugin(Plugin):
    def __init__(self, context: PluginContext, config: dict):
        self.context = context
        self.history_dir = Path(config.get("history_dir", "~/.config/pais/history")).expanduser()
        self.categories = config.get("categories", ["sessions", "learnings"])
        self._ensure_dirs()

    def capture(self, category: str, content: str, metadata: dict) -> str:
        now = datetime.now()
        year_month = now.strftime("%Y-%m")
        timestamp = now.strftime("%Y%m%dT%H%M%S")

        output_dir = self.history_dir / category / year_month
        output_dir.mkdir(parents=True, exist_ok=True)

        filename = f"{timestamp}_{metadata.get('type', 'entry')}.md"
        filepath = output_dir / filename

        doc = f"---\n{yaml.dump(metadata)}---\n\n{content}"
        filepath.write_text(doc)

        return str(filepath)

    def query(self, category: str, query: str, limit: int = 10) -> list:
        import subprocess
        category_dir = self.history_dir / category
        if not category_dir.exists():
            return []

        result = subprocess.run(
            ["rg", "-l", query, str(category_dir)],
            capture_output=True, text=True
        )
        paths = result.stdout.strip().split("\n")[:limit]
        return [self._load_entry(p) for p in paths if p]
```

#### HookHandler

```python
from dataclasses import dataclass
from enum import Enum
import re

class HookAction(Enum):
    ALLOW = 0
    BLOCK = 2

@dataclass
class HookResult:
    action: HookAction
    message: str | None = None

class SecurityPlugin(Plugin):
    DANGEROUS_PATTERNS = [
        (r"rm\s+(-rf?|--recursive)\s+[/~]", "Catastrophic deletion"),
        (r"curl.*\|\s*(ba)?sh", "Remote code execution"),
    ]

    def handles_event(self, event_type: str) -> bool:
        return event_type == "PreToolUse"

    def handle(self, event_type: str, payload: dict) -> HookResult:
        if payload.get("tool_name") != "Bash":
            return HookResult(HookAction.ALLOW)

        command = payload.get("tool_input", {}).get("command", "")

        for pattern, description in self.DANGEROUS_PATTERNS:
            if re.search(pattern, command, re.IGNORECASE):
                return HookResult(
                    HookAction.BLOCK,
                    f"🚨 BLOCKED: {description}"
                )

        return HookResult(HookAction.ALLOW)
```

#### IntegrationProvider

```python
from jira import JIRA

class JiraPlugin(Plugin):
    def __init__(self, context: PluginContext, config: dict):
        self.context = context
        self.client = JIRA(
            server=config["url"],
            basic_auth=(config["user"], config["token"])
        )

    def service_name(self) -> str:
        return "jira"

    def is_configured(self) -> bool:
        return self.client is not None

    def health_check(self) -> bool:
        try:
            self.client.myself()
            return True
        except:
            return False

    def execute(self, action: str, params: dict) -> dict:
        if action == "get_issue":
            issue = self.client.issue(params["id"])
            return {"success": True, "data": {"key": issue.key, "summary": issue.fields.summary}}

        if action == "create_issue":
            issue = self.client.create_issue(
                project=params["project"],
                summary=params["summary"],
                issuetype={"name": params.get("type", "Task")}
            )
            return {"success": True, "data": {"key": issue.key}}

        return {"success": False, "error": f"Unknown action: {action}"}

    def list_actions(self) -> list[str]:
        return ["get_issue", "create_issue", "update_issue", "search"]
```

---

## Rust Plugins

### Structure

```
my-rust-plugin/
├── plugin.toml
├── Cargo.toml
└── src/
    └── lib.rs
```

### Cargo.toml

```toml
[package]
name = "my-rust-plugin"
version = "1.0.0"
edition = "2021"

[lib]
crate-type = ["cdylib"]

[dependencies]
serde = { version = "1", features = ["derive"] }
serde_json = "1"
regex = "1"
```

### Implementation

```rust
// src/lib.rs

use serde::{Deserialize, Serialize};
use std::collections::HashMap;

#[derive(Debug, Clone, Copy)]
#[repr(u8)]
pub enum HookAction {
    Allow = 0,
    Block = 2,
}

#[derive(Debug, Serialize)]
pub struct HookResult {
    pub action: u8,
    pub message: Option<String>,
}

#[derive(Deserialize)]
pub struct PreToolUsePayload {
    pub session_id: String,
    pub tool_name: String,
    pub tool_input: HashMap<String, serde_json::Value>,
}

pub fn handle_pre_tool_use(payload: &str) -> HookResult {
    let payload: PreToolUsePayload = match serde_json::from_str(payload) {
        Ok(p) => p,
        Err(_) => return HookResult { action: 0, message: None },
    };

    if payload.tool_name != "Bash" {
        return HookResult { action: 0, message: None };
    }

    let command = payload.tool_input
        .get("command")
        .and_then(|v| v.as_str())
        .unwrap_or("");

    // Check dangerous patterns
    if command.contains("rm -rf /") || command.contains("rm -rf ~") {
        return HookResult {
            action: 2,
            message: Some("🚨 BLOCKED: Catastrophic deletion".to_string()),
        };
    }

    HookResult { action: 0, message: None }
}

// Entry point called by PAIS
#[no_mangle]
pub extern "C" fn pais_handle_hook(
    event_type: *const std::os::raw::c_char,
    payload: *const std::os::raw::c_char,
) -> *mut std::os::raw::c_char {
    // FFI implementation
    unimplemented!()
}
```

---

## Including Claude Code Components

### Skills (SKILL.md)

```markdown
---
name: incident
description: Incident response workflows. USE WHEN user mentions incident, outage, alert, SEV-1, SEV-2, or pages.
allowed-tools: Read, Bash, Write
---

# Incident Response

## Workflow Routing

- Declare new incident -> `workflows/declare.md`
- Update existing incident -> `workflows/update.md`
- Write post-mortem -> `workflows/postmortem.md`

## Available Tools

- `pais run incident declare --severity SEV-2`
- `pais run incident update --id INC-123`
- `pais run pagerduty acknowledge --id P123`

## Integration

This skill works with:
- PagerDuty (if installed)
- Slack (if installed)
- Jira (if installed)
```

### Subagents

```
my-plugin/
└── agents/
    └── researcher/
        └── AGENT.md
```

```markdown
---
name: incident-researcher
description: Researches similar past incidents. Use when investigating root cause.
tools: Read, Grep, Glob, Bash
model: haiku
skills: incident
---

You are an incident researcher specializing in finding patterns.

When invoked:
1. Search history for similar incidents
2. Identify common root causes
3. Report findings with confidence levels

Focus on:
- Error messages
- Affected services
- Time patterns
- Previous fixes
```

### MCP Servers

```json
// .mcp.json
{
  "jira": {
    "command": "${PAIS_PLUGIN_DIR}/servers/jira-mcp",
    "args": ["--config", "${PAIS_PLUGIN_DIR}/config.json"],
    "env": {
      "JIRA_URL": "${JIRA_URL}",
      "JIRA_TOKEN": "${JIRA_TOKEN}"
    }
  }
}
```

---

## Testing

### Unit Tests

```python
# tests/test_plugin.py

import pytest
from src.plugin import MyPlugin, PluginContext

class MockContext:
    def __init__(self):
        self._contracts = {}

    def get_contract(self, alias):
        return self._contracts.get(alias)

def test_plugin_initialization():
    context = MockContext()
    config = {"setting": "value"}

    plugin = MyPlugin(context, config)

    assert plugin.skill_name() == "my-skill"

def test_match_intent():
    context = MockContext()
    plugin = MyPlugin(context, {})

    assert plugin.match_intent("help me do x") > 0.5
    assert plugin.match_intent("random text") < 0.3
```

### Integration Tests

```python
# tests/test_integration.py

import subprocess

def test_plugin_runs():
    result = subprocess.run(
        ["pais", "run", "my-skill", "execute", "--action", "test"],
        capture_output=True, text=True
    )
    assert result.returncode == 0
    assert "success" in result.stdout.lower()
```

---

## Distribution

### Via Git Repository

```bash
# Team can install directly
pais plugin install github.com/your-company/pais-plugins/my-skill

# Or clone and install locally
git clone github.com/your-company/pais-plugins
pais plugin install ./pais-plugins/my-skill
```

### Via Registry

Add to a registry:

```toml
# registry/plugins.toml

[plugins.my-skill]
source = "github.com/your-company/pais-plugins/my-skill"
version = ">=1.0.0"
description = "My custom skill"
provides = ["skill"]
```

Install from registry:

```bash
pais plugin install my-skill
```

### Via Claude Code Marketplace

Convert to a Claude Code plugin for marketplace distribution:

```
.claude-plugin/
├── plugin.json
└── ...
```

See [Claude Code plugin docs](../claude-code/plugins.md).

---

## Best Practices

### Do

- **One responsibility per plugin** — Keep plugins focused
- **Graceful degradation** — Handle missing optional contracts
- **Fail safely** — Never crash; log and continue
- **Document well** — Clear SKILL.md descriptions
- **Test thoroughly** — Unit and integration tests
- **Version correctly** — Follow SemVer

### Don't

- **Hardcode paths** — Use `context.plugin_dir()` and config
- **Store secrets in code** — Use `.env` and config
- **Depend on other plugins directly** — Use contracts
- **Block forever** — Use timeouts for external calls
- **Assume contracts exist** — Check `has_contract()` first

---

## Related Documents

- [contracts.md](contracts.md) — Contract specifications
- [cli.md](cli.md) — CLI reference
- [architecture.md](architecture.md) — System design

