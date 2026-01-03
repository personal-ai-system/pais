# PAIS Implementation Roadmap

> A phased plan to achieve feature parity with Daniel Miessler's PAI/Kai system, implemented in Rust.

## Current State (Completed)

| Feature | Status | Implementation |
|---------|--------|----------------|
| Context Injection | ✅ | `pais context inject` outputs `<system-reminder>` |
| Skill Index | ✅ | `pais skill index` generates `skill-index.yaml` + `context-snippet.md` |
| USE WHEN Triggers | ✅ | Extracted from skill descriptions |
| Security Validation | ✅ | `src/hook/security.rs` with 3 pattern categories |
| Skill Sync | ✅ | `pais sync` symlinks to `~/.claude/skills/` |
| CLI Infrastructure | ✅ | Comprehensive command structure |
| Simple Skills | ✅ | SKILL.md with frontmatter |
| CORE Skill (Phase 1.1) | ✅ | `~/.config/pais/skills/core/SKILL.md` - always loaded first |
| YAML Migration | ✅ | All config now YAML (`pais.yaml`, `plugin.yaml`, `skill-index.yaml`) |

---

## Phase 1: Core Skill System (Foundation)

**Goal:** Implement Tier 0 (CORE) skill that's always present, plus workflow routing.

### 1.1 CORE Skill Creation ✅ DONE

Create a special CORE skill that defines your AI identity and is always injected.

**Tasks:**
- [x] Create `~/.config/pais/skills/core/SKILL.md` with:
  - Operating principles (blunt, concise, no estimates)
  - Code preferences (no magic numbers, no obvious comments, >80% coverage)
  - Workflow rules (never commit without permission, ask on ambiguity)
  - Forbidden behaviors (no filler phrases, no unsolicited refactoring)
- [x] Update `pais context inject` to always include CORE content first
- [ ] ~~Add `core_skills` config option~~ (hardcoded to `core/` for now)

**Files created/modified:**
```
~/.config/pais/skills/core/SKILL.md    # Created
src/commands/context.rs                 # Modified - loads core first
```

### 1.2 Workflow Routing ✅ DONE

Enable skills to route to specific workflow files for step-by-step procedures.

**Tasks:**
- [x] Add `workflows/` directory support to skill structure
- [x] Parse workflow routing table from SKILL.md body
- [x] Update context snippet to include workflow hints
- [x] Add `pais skill workflow <skill> <workflow>` command to output specific workflow

**Example SKILL.md with workflows:**
```markdown
---
name: rust-coder
description: Write Rust code. USE WHEN creating CLIs, libraries.
---

## Workflow Routing

| Intent | Workflow |
|--------|----------|
| new CLI project | workflows/new-cli.md |
| add error handling | workflows/error-handling.md |
| write tests | workflows/testing.md |
```

**Files created/modified:**
```
src/skill/workflow.rs                  # Created - workflow parsing/loading
src/commands/skill.rs                  # Modified - added workflow subcommand
src/cli.rs                             # Modified - added workflow action
src/skill/indexer.rs                   # Modified - workflows in index/context
~/.config/pais/skills/rust-coder/workflows/new-cli.md  # Example workflow
```

### 1.3 Tiered Loading Implementation ✅ DONE

Formalize the tier system for skill loading.

**Tasks:**
- [x] Define `SkillTier` enum: `Core`, `Deferred`
- [x] Update skill index to include tier information
- [x] Add tier configuration per skill in frontmatter (`tier: core` or `tier: 0`)
- [x] Update context injection to load all core-tier skills

**Tier Definitions:**
```
Tier 0 (Core):     Always present, full body injected at session start
Tier 1 (Deferred): Name + description + triggers in context, body on invoke
```

**Usage:**
```yaml
---
name: my-skill
description: My custom skill
tier: core  # or tier: 0
---
```

**Files created/modified:**
```
src/skill/parser.rs      # Added SkillTier enum with serde support
src/skill/indexer.rs     # Uses tier from frontmatter
src/commands/context.rs  # Loads all core-tier skills at injection
```

---

## Phase 2: History System (Memory) ✅ DONE

**Goal:** Automatic capture of all AI work, categorized and searchable.

### 2.1 Event Capture Infrastructure ✅

**Tasks:**
- [x] Create `~/.config/pais/history/` directory structure
- [x] Implement `EventCapture` that appends to daily JSONL (`raw-events/YYYY-MM/YYYY-MM-DD.jsonl`)
- [x] Wire into `pais hook dispatch` for all event types

**Files created/modified:**
```
src/history/capture.rs                 # Created - CapturedEvent, EventCapture
src/history/mod.rs                     # Modified - expose capture module
src/commands/hook.rs                   # Modified - capture events on dispatch
```

### 2.2 Stop Hook - Session/Learning Categorization ✅

**Tasks:**
- [x] Implement content analysis for learning detection (2+ indicator matches)
- [x] Generate markdown files with YAML frontmatter
- [x] Auto-extract tags from technical content

**Files created/modified:**
```
src/history/categorize.rs              # Created - categorize_content, extract_tags
src/hook/history.rs                    # Modified - uses categorization on Stop
```

### 2.3 Session Summaries ✅

**Tasks:**
- [x] SessionStart/SessionEnd/Stop handlers capture session lifecycle
- [x] Entries stored with metadata (session_id, tags, timestamps)

### 2.4 History CLI Commands ✅

**Implemented commands:**
- `pais history categories` - list available categories with counts
- `pais history recent [--category X]` - show recent entries
- `pais history query <regex>` - search across history
- `pais history show <id>` - display specific entry
- `pais history stats [--days N]` - event statistics
- `pais history events` - list raw event log dates

---

## Phase 3: Enhanced Security (Protection)

**Goal:** Expand security validation to match PAI's 10-tier system.

### 3.1 Expand Pattern Tiers

**Tasks:**
- [ ] Add missing tiers:
  - Tier 2: Reverse shells (bash -i, nc -e, socat)
  - Tier 4: Prompt injection patterns
  - Tier 5: Environment manipulation (API key access)
  - Tier 6: Git dangerous operations (force push, hard reset)
  - Tier 7: System modification (chmod 777, sudo)
  - Tier 8: Network operations (ssh, scp)
  - Tier 9: Data exfiltration (tar | curl)
  - Tier 10: PAIS-specific protection
- [ ] Implement action types: `block`, `warn`, `confirm`, `log`
- [ ] Add configurable security levels in `pais.yaml`

**Files to modify:**
```
src/hook/security.rs                   # Modify - add all tiers
src/config.rs                          # Modify - security config
```

### 3.2 Security Logging

**Tasks:**
- [ ] Log all security events to `history/security/YYYY-MM/`
- [ ] Include: timestamp, command, tier matched, action taken
- [ ] Add `pais security log` command to view recent events

**Files to create/modify:**
```
src/history/security.rs                # New - security event logging
src/commands/security.rs               # New - security CLI commands
```

---

## Phase 4: Observability (Visibility)

**Goal:** Real-time visibility into AI operations.

### 4.1 Event Streaming

**Tasks:**
- [ ] Implement event emitter that can send to multiple sinks
- [ ] Support sinks: file (JSONL), stdout, HTTP endpoint
- [ ] Configure via `pais.yaml`:
  ```yaml
  observability:
    enabled: true
    sinks:
      - file
      - http
    http_endpoint: http://localhost:4000/events
  ```

**Files to create/modify:**
```
src/observability/mod.rs               # New - observability module
src/observability/emitter.rs           # New - event emitter
src/observability/sinks.rs             # New - sink implementations
src/config.rs                          # Modify - observability config
```

### 4.2 Dashboard Server (Optional)

**Tasks:**
- [ ] Create simple event viewer (could be Python/Flask or Rust/Axum)
- [ ] Real-time event stream via SSE or WebSocket
- [ ] Filter by event type, session, time range
- [ ] Basic metrics: events/minute, tool usage, blocks

**Files to create:**
```
tools/dashboard/                       # New directory
tools/dashboard/server.py              # Python dashboard server
tools/dashboard/templates/index.html   # Dashboard UI
```

### 4.3 CLI Observability Commands

**Tasks:**
- [ ] `pais observe` - live tail of events
- [ ] `pais observe --filter PreToolUse` - filtered tail
- [ ] `pais stats` - event statistics

---

## Phase 5: Agent System (Personalities)

**Goal:** Support for named agent personalities with configurable traits.

### 5.1 Agent Definition

**Tasks:**
- [ ] Define agent trait schema:
  ```yaml
  name: researcher
  traits:
    thoroughness: 0.9
    creativity: 0.7
    verbosity: 0.5
  voice: analytical
  focus: investigation
  ```
- [ ] Create `~/.config/pais/agents/` directory
- [ ] Implement agent loading and context injection

**Files to create:**
```
~/.config/pais/agents/researcher.yaml  # Example agent
~/.config/pais/agents/architect.yaml   # Example agent
src/agent/mod.rs                       # New - agent module
src/agent/traits.rs                    # New - trait definitions
src/agent/loader.rs                    # New - agent loading
```

### 5.2 Agent Routing (SubagentStop)

**Tasks:**
- [ ] Detect agent type from session metadata
- [ ] Route outputs to appropriate history directories:
  ```
  researcher → history/research/
  architect  → history/decisions/
  engineer   → history/execution/features/
  ```
- [ ] Tag history entries with agent name

**Files to modify:**
```
src/hook/history.rs                    # Modify - agent-aware routing
src/history/categorize.rs              # Modify - agent detection
```

### 5.3 Agent CLI Commands

**Tasks:**
- [ ] `pais agent list` - list available agents
- [ ] `pais agent show <name>` - display agent config
- [ ] `pais agent create <name>` - create new agent from template

---

## Phase 6: Quality of Life (Polish)

**Goal:** Small features that improve daily usage.

### 6.1 Tab Title Updates

**Tasks:**
- [ ] Implement `UserPromptSubmit` hook to update terminal tab title
- [ ] Extract task context from user prompt
- [ ] Format: `🤖 [task summary]`

**Files to modify:**
```
src/hook/ui.rs                         # New - UI-related hooks
.claude/settings.json                  # Add UserPromptSubmit hook
```

### 6.2 Architecture Tracking

**Tasks:**
- [ ] Auto-generate `~/.config/pais/ARCHITECTURE.md`:
  - Installed skills
  - Configured agents
  - Hook status
  - Last sync time
- [ ] Update on `pais sync` and `pais skill index`

**Files to create/modify:**
```
src/commands/sync.rs                   # Modify - generate architecture doc
src/architecture.rs                    # New - architecture doc generation
```

### 6.3 Upgrade/Migration System

**Tasks:**
- [ ] Version tracking for PAIS config
- [ ] Migration scripts for config changes
- [ ] `pais upgrade` command to apply migrations

---

## Phase 7: Plugin System (Extensibility)

**Goal:** Enable third-party plugins with full hook/contract support.

### 7.1 Plugin Manifest (plugin.yaml)

**Tasks:**
- [ ] Finalize plugin.yaml schema:
  ```yaml
  plugin:
    name: my-plugin
    version: 1.0.0
    description: Does something cool

  hooks:
    PreToolUse: hooks/validate.py
    Stop: hooks/capture.py

  contract:
    consumes:
      - bash-output
    produces:
      - analysis-report

  build:
    type: python  # or rust
    entrypoint: src/main.py
  ```
- [ ] Implement manifest parsing
- [ ] Validate hook/contract references

**Files to modify:**
```
src/plugin/manifest.rs                 # Modify - full schema support
```

### 7.2 Plugin Hook Execution

**Tasks:**
- [ ] Execute plugin hooks on relevant events
- [ ] Pass payload via stdin, capture stdout/stderr
- [ ] Handle exit codes (0=allow, 2=block)
- [ ] Support Python and Rust plugin languages

**Files to create/modify:**
```
src/plugin/executor.rs                 # New - plugin execution
src/hook/dispatcher.rs                 # Modify - include plugin hooks
```

### 7.3 Contract System

**Tasks:**
- [ ] Define contract schema (consumes/produces)
- [ ] Validate contract compatibility between plugins
- [ ] Enable workflow chaining based on contracts

**Files to create:**
```
src/contract/mod.rs                    # New - contract module
src/contract/validator.rs              # New - contract validation
src/contract/chain.rs                  # New - workflow chaining
```

### 7.4 Plugin Registry

**Tasks:**
- [ ] `pais plugin install <url|path>`
- [ ] `pais plugin remove <name>`
- [ ] `pais plugin update <name>`
- [ ] Support git URLs and local paths

---

## Phase Summary

| Phase | Focus | Priority |
|-------|-------|----------|
| 1 | Core Skill System | High |
| 2 | History System | High |
| 3 | Enhanced Security | Medium |
| 4 | Observability | Medium |
| 5 | Agent System | Low |
| 6 | Quality of Life | Low |
| 7 | Plugin System | Low |

**Recommended order:** Phase 1 → Phase 2 → Phase 3 → Phase 6 → Phase 4 → Phase 5 → Phase 7

---

## Success Metrics

After completing all phases, PAIS should:

1. **Match PAI functionality:** All features from kai-hook-system, kai-history-system, kai-core-install
2. **Be faster:** Rust performance advantage over TypeScript
3. **Be maintainable:** Clean Rust codebase with comprehensive tests
4. **Be extensible:** Plugin system for community contributions
5. **Have institutional memory:** Automatic capture of all AI work

---

## Notes

- Each phase can be implemented incrementally
- Tests should be written alongside features
- Documentation should be updated as features land
- Consider user feedback between phases to reprioritize

