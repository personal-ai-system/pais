# PAIS Vision

> Personal AI System — A modular plugin system for Claude Code, built for team sharing.

---

## The Problem We're Solving

The most powerful AI setups are being built inside companies for efficiency and profits. But technology should serve humans—not the other way around.

**Daniel Miessler's Kai/PAI** showed what's possible: a personal AI system that knows your goals, learns from your history, and gets better at helping you over time. But his implementation has limitations:

1. **Tight coupling** — Packs depend on other packs in strict order
2. **Extraction pain** — He's still struggling to separate Kai from PAI
3. **Single-user focus** — Designed for one person, not teams
4. **TypeScript-only** — No Python or Rust options

**PAIS** takes the best ideas from Kai/PAI and rebuilds them as a **true plugin system** on top of Claude Code's native scaffolding.

---

## Who This Is For

**Primary audience:** engineers who want to:

- Augment their workflows with AI
- Share capabilities with teammates (pick and choose)
- Integrate with work tools (Jira, Slack, PagerDuty, GitHub, etc.)
- Build in Python and Rust (not TypeScript)

**Secondary audience:** Anyone who wants a modular, team-shareable AI augmentation system.

---

## Core Philosophy

### 1. Built ON Claude Code, Not Beside It

PAIS is **not** a replacement for Claude Code. It's a layer on top that uses Claude Code's native primitives:

| Claude Code Primitive | How PAIS Uses It |
|----------------------|------------------|
| **Hooks** | Event-driven automation (security, history capture) |
| **Skills** | Capability routing via SKILL.md files |
| **Subagents** | Specialized agents with isolated context |
| **Plugins** | Packaging and distribution mechanism |
| **MCP Servers** | External integrations (Jira, Slack, etc.) |
| **Marketplaces** | Team and community sharing |

We don't reinvent these. We compose them.

### 2. Plugins Over Packs

Daniel's "packs" are coupled layers that must be installed in order. PAIS uses true plugins:

| Packs (Kai/PAI) | Plugins (PAIS) |
|-----------------|----------------|
| Depend on other packs | Depend on **contracts** (interfaces) |
| Install in strict order | Install in any order |
| Removal may break others | Clean removal, no side effects |
| Global configuration | Plugin-local config |
| Hardcoded paths | Discoverable/configurable |

### 3. Contracts, Not Dependencies

Plugins don't depend on each other. They depend on **contracts**:

```
# Plugin A doesn't know about Plugin B
# Plugin A provides: MemoryProvider contract
# Plugin B consumes: MemoryProvider contract (optional)
# Core wires them together at runtime
```

If a consumed contract isn't available, the plugin gracefully degrades.

### 4. Python + Rust, Not TypeScript

| Component | Language | Rationale |
|-----------|----------|-----------|
| CLI (`pais`) | Rust | Fast, single binary, your expertise |
| Hooks | Rust | Safety-critical, must never crash |
| Skills/Plugins | Python | SDK availability, rapid iteration |
| Performance-critical | Rust | When Python is too slow |

### 5. Team-Shareable by Default

Every design decision asks: "Can Alice install this without affecting Bob?"

- Plugins are independent
- Configuration is plugin-local
- Sharing is via Git repos or marketplaces
- No global state pollution

### 6. Dogfooding from Day One

PAIS should help build PAIS. The `meta/improve` plugin tracks usage, identifies gaps, and generates new plugin scaffolding.

---

## Non-Goals (What We're NOT Building)

1. **NOT a chatbot framework** — Claude Code handles LLM interaction
2. **NOT a RAG system** — File-based memory, not vector search
3. **NOT a monolith** — If it can't be removed cleanly, it's wrong
4. **NOT TypeScript** — Python and Rust only
5. **NOT a fork of Kai/PAI** — Clean-room implementation using the ideas
6. **NOT replacing Claude Code primitives** — We compose them

---

## Success Criteria

PAIS is successful when:

1. **A new teammate can install plugins in < 5 minutes** — No complex setup
2. **Plugins can be added/removed without side effects** — True independence
3. **The system helps improve itself** — Dogfooding works
4. **Workflows are measurably faster** — Whatever you use it for
5. **Team members share plugins freely** — No conflicts or collisions

---

## The 12 Principles

These guide all design and implementation decisions:

### From Kai/PAI (Adopted)

1. **Scaffolding > Model** — Architecture matters more than which model
2. **Code Before Prompts** — Use AI only for what needs intelligence
3. **CLI as Interface** — Command-line is faster and more reliable
4. **File-Based Memory** — Grep over RAG for most use cases
5. **Self-Improvement** — System should help improve itself
6. **Clear Thinking = Clear Prompts** — Good output requires good input

### PAIS-Specific (New)

7. **Plugins Over Packs** — True independence, any install order
8. **Contracts Over Dependencies** — Interfaces, not implementations
9. **Team-Shareable by Default** — No global state pollution
10. **Graceful Degradation** — Missing optional deps = reduced features, not failure
11. **Claude Code Native** — Use hooks, skills, subagents, plugins, MCP
12. **Python + Rust** — No TypeScript

---

## What We're Building

### Core (`pais` CLI - Rust)

Minimal core that:
- Discovers and loads plugins
- Dispatches hooks to Claude Code
- Provides plugin management commands
- Wires contracts between plugins

### Plugin Categories

| Category | Examples | Purpose |
|----------|----------|---------|
| **Foundation** | `hooks`, `history` | Infrastructure others build on |
| **Work** | `incident`, `runbook`, `oncall` | Work-specific workflows |
| **Team** | `jira`, `slack`, `github`, `gmail` | External integrations |
| **Learning** | `spanish`, `writing` | Personal development |
| **Meta** | `improve`, `create-plugin` | Self-improvement |

### Distribution

- **Core plugins** — In the PAIS repo (blessed, maintained)
- **Team plugins** — Private Git repos for work-specific tools
- **Personal plugins** — Local or personal Git repos
- **Community plugins** — Public repos, discovered via registry

---

## The Journey

### Phase 1: Foundation
- Rust CLI with plugin loading
- Hook dispatcher (integrates with Claude Code)
- Contract system (provides/consumes)
- First plugins: `hooks`, `history`

### Phase 2: Work Workflows
- Your domain-specific plugins
- External integrations (Jira, GitHub, Slack, etc.)
- Team-specific tooling

### Phase 3: Team Sharing
- Plugin registry
- Team marketplace
- Documentation and onboarding

### Phase 4: Self-Improvement
- Meta plugin for usage analytics
- Plugin generator
- Feedback loops

---

## Related Documents

- [architecture.md](architecture.md) — Technical system design
- [contracts.md](contracts.md) — Plugin interface specifications
- [cli.md](cli.md) — Command reference
- [plugins.md](plugins.md) — Plugin development guide
- [use-cases.md](use-cases.md) — Real-world scenarios
- [decisions.md](decisions.md) — Architecture decision records
- [comparison.md](comparison.md) — PAIS vs. Kai/PAI

---

*PAIS — Augment yourself, share with your team.*

