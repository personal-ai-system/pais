# PAIS Contracts

> Interface specifications for plugin communication.

---

## What is a Contract?

A **contract** is an interface that plugins can **provide** or **consume**. Contracts enable plugins to work together without direct dependencies.

```
Plugin A                          Plugin B
┌─────────────┐                  ┌─────────────┐
│ provides:   │                  │ consumes:   │
│MemoryProvider│ ◄──────────────│MemoryProvider│
│             │    at runtime    │ (optional)  │
└─────────────┘                  └─────────────┘
```

**Key properties:**
- Plugins depend on **contracts**, not other plugins
- Missing optional contracts = graceful degradation
- Missing required contracts = fail at load time
- Core wires providers to consumers at runtime

---

## Contract Categories

| Category | Purpose | Examples |
|----------|---------|----------|
| **Provider** | Offers a capability | MemoryProvider, NotificationProvider |
| **Skill** | Teaches Claude a workflow | IncidentSkill, RunbookSkill |
| **Integration** | Connects to external service | JiraIntegration, SlackIntegration |
| **Hook** | Handles Claude Code events | SecurityHook, HistoryHook |

---

## Core Contracts

### MemoryProvider

A plugin that provides persistent memory/context storage.

**Python interface:**

```python
from typing import Protocol, Any
from dataclasses import dataclass

@dataclass
class MemoryResult:
    path: str
    category: str
    timestamp: str
    content: str
    metadata: dict[str, Any]

class MemoryProvider(Protocol):
    """Contract for plugins that provide memory storage."""

    def capture(
        self,
        category: str,           # e.g., "sessions", "learnings", "incidents"
        content: str,            # The content to store
        metadata: dict[str, Any] # Additional context
    ) -> str:
        """
        Store content in the specified category.

        Returns: Path to the stored file.
        """
        ...

    def query(
        self,
        category: str,           # Category to search
        query: str,              # Search query (grep-style regex)
        limit: int = 10          # Max results
    ) -> list[MemoryResult]:
        """Search stored content."""
        ...

    def list_categories(self) -> list[str]:
        """Return available memory categories."""
        ...

    def get_recent(
        self,
        category: str,
        count: int = 5
    ) -> list[MemoryResult]:
        """Get most recent entries in a category."""
        ...
```

**Rust interface:**

```rust
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MemoryResult {
    pub path: String,
    pub category: String,
    pub timestamp: String,
    pub content: String,
    pub metadata: HashMap<String, serde_json::Value>,
}

pub trait MemoryProvider {
    fn capture(
        &self,
        category: &str,
        content: &str,
        metadata: HashMap<String, serde_json::Value>,
    ) -> Result<String, MemoryError>;

    fn query(
        &self,
        category: &str,
        query: &str,
        limit: usize,
    ) -> Result<Vec<MemoryResult>, MemoryError>;

    fn list_categories(&self) -> Vec<String>;

    fn get_recent(
        &self,
        category: &str,
        count: usize,
    ) -> Result<Vec<MemoryResult>, MemoryError>;
}
```

---

### HookHandler

A plugin that handles Claude Code hook events.

**Python interface:**

```python
from typing import Protocol, Any
from dataclasses import dataclass
from enum import Enum

class HookAction(Enum):
    ALLOW = 0      # Continue execution
    BLOCK = 2      # Block the action
    # Note: exit codes match Claude Code expectations

@dataclass
class HookResult:
    action: HookAction
    message: str | None = None
    metadata: dict[str, Any] | None = None

class HookHandler(Protocol):
    """Contract for plugins that handle Claude Code events."""

    def handles_event(self, event_type: str) -> bool:
        """Return True if this handler processes the given event type."""
        ...

    def handle(
        self,
        event_type: str,         # "PreToolUse", "Stop", "SessionStart", etc.
        payload: dict[str, Any]  # Event payload from Claude Code
    ) -> HookResult:
        """
        Process a hook event.

        Returns: HookResult indicating action to take.
        """
        ...
```

**Rust interface:**

```rust
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

#[derive(Debug, Clone, Copy)]
pub enum HookAction {
    Allow = 0,
    Block = 2,
}

#[derive(Debug, Clone)]
pub struct HookResult {
    pub action: HookAction,
    pub message: Option<String>,
    pub metadata: Option<HashMap<String, serde_json::Value>>,
}

pub trait HookHandler {
    fn handles_event(&self, event_type: &str) -> bool;

    fn handle(
        &self,
        event_type: &str,
        payload: &serde_json::Value,
    ) -> HookResult;
}
```

---

### SkillProvider

A plugin that provides a Claude Code skill.

**Python interface:**

```python
from typing import Protocol, Any
from pathlib import Path

class SkillProvider(Protocol):
    """Contract for plugins that provide Claude Code skills."""

    def skill_name(self) -> str:
        """Return the skill name (matches SKILL.md name field)."""
        ...

    def skill_path(self) -> Path:
        """Return path to SKILL.md file."""
        ...

    def match_intent(self, intent: str) -> float:
        """
        Return confidence (0.0-1.0) that this skill matches the intent.
        Used for routing when multiple skills might apply.
        """
        ...

    def execute(
        self,
        action: str,             # Specific action within the skill
        context: dict[str, Any]  # Execution context
    ) -> dict[str, Any]:
        """
        Execute a skill action programmatically.
        Returns result dict.
        """
        ...
```

---

### IntegrationProvider

A plugin that connects to an external service.

**Python interface:**

```python
from typing import Protocol, Any

class IntegrationProvider(Protocol):
    """Contract for plugins that integrate with external services."""

    def service_name(self) -> str:
        """Return the service name (e.g., 'jira', 'slack', 'pagerduty')."""
        ...

    def is_configured(self) -> bool:
        """Return True if the integration has valid configuration."""
        ...

    def health_check(self) -> bool:
        """Return True if the service is reachable and authenticated."""
        ...

    def execute(
        self,
        action: str,             # e.g., "create_ticket", "send_message"
        params: dict[str, Any]   # Action-specific parameters
    ) -> dict[str, Any]:
        """
        Execute an action on the external service.
        Returns result dict with 'success' and 'data' keys.
        """
        ...

    def list_actions(self) -> list[str]:
        """Return list of supported actions."""
        ...
```

---

### NotificationProvider

A plugin that can notify the user.

**Python interface:**

```python
from typing import Protocol
from enum import Enum

class NotificationLevel(Enum):
    DEBUG = "debug"
    INFO = "info"
    WARNING = "warning"
    ERROR = "error"
    SUCCESS = "success"

class NotificationProvider(Protocol):
    """Contract for plugins that send notifications."""

    def notify(
        self,
        message: str,
        level: NotificationLevel = NotificationLevel.INFO,
        title: str | None = None
    ) -> bool:
        """
        Send a notification to the user.
        Returns True if notification was delivered.
        """
        ...

    def supports_rich(self) -> bool:
        """Return True if rich formatting (markdown, etc.) is supported."""
        ...
```

---

## Plugin Manifest Schema

The `plugin.toml` manifest declares contracts:

```toml
[plugin]
name = "history"
version = "1.0.0"
description = "File-based memory system"
authors = ["pais-team"]
language = "python"

[pais]
core_version = ">=0.1.0"

# Contracts this plugin PROVIDES
[provides]
memory = "MemoryProvider"

# Contracts this plugin CONSUMES
[consumes]
# Format: contract_alias = { contract = "ContractName", optional = bool }
hooks = { contract = "HookHandler", optional = true }
notifications = { contract = "NotificationProvider", optional = true }

# For IntegrationProvider consumers, specify the service:
# pagerduty = { contract = "IntegrationProvider", service = "pagerduty", optional = true }

[config]
# Configuration schema
history_dir = { type = "string", default = "~/.config/pais/history" }
categories = { type = "array", default = ["sessions", "learnings", "incidents", "decisions"] }

[hooks]
# Which hook events this plugin handles (if it provides HookHandler)
pre_tool_use = false
post_tool_use = false
stop = true
session_start = true
session_end = true
```

---

## Contract Resolution

### At Load Time

```python
# Pseudocode for contract resolution

def load_plugins(plugin_dir: Path) -> dict[str, Plugin]:
    # Phase 1: Load all manifests
    manifests = {}
    for plugin_path in plugin_dir.iterdir():
        if (plugin_path / "plugin.toml").exists():
            manifests[plugin_path.name] = load_manifest(plugin_path)

    # Phase 2: Build provider map
    providers = {}  # contract -> plugin_name
    for name, manifest in manifests.items():
        for contract_name in manifest.provides:
            if contract_name in providers:
                raise ContractConflict(f"{contract_name} provided by multiple plugins")
            providers[contract_name] = name

    # Phase 3: Check required contracts
    for name, manifest in manifests.items():
        for alias, spec in manifest.consumes.items():
            if not spec.optional and spec.contract not in providers:
                raise MissingContract(f"{name} requires {spec.contract}")

    # Phase 4: Load plugins in dependency order
    loaded = {}
    for name in topological_sort(manifests, providers):
        plugin = load_plugin(manifests[name])
        loaded[name] = plugin

    # Phase 5: Wire consumers to providers
    for name, plugin in loaded.items():
        for alias, spec in manifests[name].consumes.items():
            if spec.contract in providers:
                provider_plugin = loaded[providers[spec.contract]]
                plugin.wire_contract(alias, provider_plugin)

    return loaded
```

### Runtime Access

Plugins access contracts through a registry:

```python
class PluginContext:
    """Provided to each plugin at initialization."""

    def __init__(self, contracts: dict[str, Any]):
        self._contracts = contracts

    def get_contract(self, alias: str) -> Any | None:
        """Get a consumed contract by alias. Returns None if not available."""
        return self._contracts.get(alias)

    def has_contract(self, alias: str) -> bool:
        """Check if a contract is available."""
        return alias in self._contracts


# In a plugin:
class IncidentPlugin:
    def __init__(self, context: PluginContext, config: dict):
        self.context = context
        self.config = config

        # Check for optional contracts
        self.memory = context.get_contract("memory")
        self.pagerduty = context.get_contract("pagerduty")
        self.slack = context.get_contract("slack")

    def handle_incident(self, incident_id: str):
        # Use contracts if available
        if self.pagerduty:
            incident = self.pagerduty.execute("get_incident", {"id": incident_id})

        # Store in memory if available
        if self.memory:
            self.memory.capture("incidents", str(incident), {"id": incident_id})

        # Notify if available
        if self.slack:
            self.slack.execute("send_message", {
                "channel": "#incidents",
                "text": f"Handling incident {incident_id}"
            })
```

---

## Event Payload Schemas

### PreToolUse

```json
{
  "session_id": "abc123",
  "tool_name": "Bash",
  "tool_input": {
    "command": "rm -rf /tmp/test",
    "description": "Clean up test directory"
  }
}
```

### PostToolUse

```json
{
  "session_id": "abc123",
  "tool_name": "Bash",
  "tool_input": {
    "command": "npm test"
  },
  "tool_output": {
    "stdout": "All tests passed",
    "stderr": "",
    "exit_code": 0
  }
}
```

### Stop

```json
{
  "session_id": "abc123",
  "response": "I've completed the task. Here's a summary...",
  "transcript_path": "/path/to/transcript.jsonl"
}
```

### SessionStart

```json
{
  "session_id": "abc123",
  "is_resume": false,
  "project_dir": "/home/user/project",
  "timestamp": "2026-01-01T12:00:00Z"
}
```

### SessionEnd

```json
{
  "session_id": "abc123",
  "duration_seconds": 3600,
  "tools_used": ["Bash", "Read", "Edit"],
  "timestamp": "2026-01-01T13:00:00Z"
}
```

---

## Error Handling Contract

All plugins should follow consistent error handling:

```python
from dataclasses import dataclass
from enum import Enum

class ErrorSeverity(Enum):
    WARNING = "warning"   # Log and continue
    ERROR = "error"       # Log, notify, continue
    FATAL = "fatal"       # Log, notify, stop

@dataclass
class PluginError:
    plugin: str
    operation: str
    message: str
    severity: ErrorSeverity
    details: dict | None = None

# Plugins should raise PluginError or return error dicts
# Plugins should NEVER raise unhandled exceptions that crash PAIS
```

---

## Versioning and Compatibility

Contracts are versioned with the PAIS core:

```toml
[pais]
core_version = ">=0.1.0,<1.0.0"  # SemVer range
```

**Compatibility rules:**
- Major version bump = breaking contract changes
- Minor version bump = new optional methods/fields
- Patch version bump = bug fixes only

When a contract changes:
1. Old version deprecated but supported for 1 minor version
2. Plugins have 1 minor version cycle to migrate
3. Breaking changes only in major versions

---

## Related Documents

- [architecture.md](architecture.md) — System design
- [plugins.md](plugins.md) — Plugin development guide
- [cli.md](cli.md) — Command reference

