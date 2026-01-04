# PAIS Planning Documentation

> Personal AI System — Built on Claude Code

---

## Overview

PAIS is a modular plugin system for Claude Code, designed for team sharing. It takes inspiration from Daniel Miessler's Kai/PAI system but reimplements the ideas with a focus on:

- **True plugin architecture** (vs. coupled packs)
- **Python + Rust** (vs. TypeScript)
- **Team sharing** (vs. single-user optimization)

---

## Documents

### Core

| Document | Description |
|----------|-------------|
| [vision.md](vision.md) | Philosophy, goals, and principles |
| [architecture.md](architecture.md) | Technical system design |
| [contracts.md](contracts.md) | Plugin interface specifications |

### Development

| Document | Description |
|----------|-------------|
| [cli.md](cli.md) | `pais` command reference |
| [plugins.md](plugins.md) | Plugin development guide |

### Context

| Document | Description |
|----------|-------------|
| [use-cases.md](use-cases.md) | Real-world scenarios |
| [decisions.md](decisions.md) | Architecture decision records |
| [comparison.md](comparison.md) | PAIS vs. Kai/PAI |

---

## Quick Links

### Claude Code Reference

The Claude Code documentation is in `docs/claude-code/`:

- [hooks-guide.md](../claude-code/hooks-guide.md) — Event-driven automation
- [skills.md](../claude-code/skills.md) — SKILL.md format
- [sub-agents.md](../claude-code/sub-agents.md) — Specialized agents
- [plugins.md](../claude-code/plugins.md) — Claude Code plugin system
- [mcp.md](../claude-code/mcp.md) — External integrations
- [discover-plugins.md](../claude-code/discover-plugins.md) — Plugin marketplace

### Source Material

- [Daniel's Kai/PAI Video](https://www.youtube.com/watch?v=Le0DLrn7ta0)
- [PAI GitHub Repository](https://github.com/danielmiessler/Personal_AI_Infrastructure)

---

## Key Principles

1. **Built ON Claude Code** — Use hooks, skills, subagents, plugins, MCP
2. **Plugins Over Packs** — True independence, any install order
3. **Contracts Over Dependencies** — Interfaces, not implementations
4. **Python + Rust** — No TypeScript
5. **Team-Shareable** — No global state pollution
6. **Dogfooding** — System helps improve itself

---

## Technology Stack

| Component | Language | Purpose |
|-----------|----------|---------|
| `pais` CLI | Rust | Plugin management, hook dispatch |
| Foundation plugins | Rust | Security, performance-critical |
| Integration plugins | Python | SDK access (Jira, Slack, etc.) |
| Skill plugins | Python | Workflow logic |

---

## Getting Started

*Implementation pending. Planning phase complete.*

Next steps:
1. Create Rust CLI skeleton
2. Implement plugin discovery
3. Create first foundation plugin (hooks)
4. Create first integration plugin (Jira or Slack)
5. Create first skill plugin (incident)

