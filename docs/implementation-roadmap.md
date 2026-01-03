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
| Environment Awareness (Phase 10) | ✅ | `repos-dir`, `tool-preferences`, `tools` in config |

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

## Phase 3: Enhanced Security (Protection) ✅ DONE

**Goal:** Expand security validation to match PAI's 10-tier system.

### 3.1 Expand Pattern Tiers ✅

**Implemented 10 security tiers:**

| Tier | Category | Action |
|------|----------|--------|
| 1 | Catastrophic (rm -rf /, dd) | Block |
| 2 | Reverse shells (bash -i, nc -e, socat) | Block |
| 3 | Remote code execution (curl\|bash) | Block |
| 4 | Prompt injection patterns | Block |
| 5 | Credential theft (.ssh, .aws) | Block |
| 6 | Environment manipulation (API keys) | Block |
| 7 | Git dangerous ops (force push) | Warn |
| 8 | System modification (chmod 777, sudo) | Warn |
| 9 | Network operations (ssh, scp) | Log |
| 10 | Data exfiltration (tar\|curl) | Block |

**Files modified:**
```
src/hook/security.rs    # SecurityTier, SecurityAction, 10 pattern tiers
```

### 3.2 Security Logging ✅

**Tasks:**
- [x] Log all security events to `history/security/YYYY-MM/YYYY-MM-DD.jsonl`
- [x] Include: timestamp, command, tier matched, action taken, session_id

### 3.3 Security CLI Commands ✅

**Implemented commands:**
- `pais security tiers` - Show all security tiers and actions
- `pais security log [--days N]` - View recent security events
- `pais security test <command>` - Test a command against patterns

**Files created:**
```
src/commands/security.rs    # Security CLI commands
src/cli.rs                  # SecurityAction enum
```

---

## Phase 4: Observability (Visibility)

**Goal:** Real-time visibility into AI operations.

### 4.1 Event Streaming ✅

**Tasks:**
- [x] Implement event emitter that can send to multiple sinks
- [x] Support sinks: file (JSONL), stdout, HTTP endpoint
- [x] Configure via `pais.yaml`:
  ```yaml
  observability:
    enabled: true
    sinks:
      - file
      - http
    http_endpoint: http://localhost:4000/events
  ```

**Files created/modified:**
```
src/observability/mod.rs               # New - observability module
src/observability/emitter.rs           # New - event emitter, Event struct, multiple sinks
src/config.rs                          # Modified - ObservabilityConfig, ObservabilitySink
src/commands/hook.rs                   # Modified - uses EventEmitter instead of EventCapture
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

### 4.3 CLI Observability Commands ✅

**Tasks:**
- [x] `pais observe` - live tail of events
- [x] `pais observe --filter PreToolUse` - filtered tail
- [ ] `pais stats` - event statistics (deferred - `pais history stats` already exists)

**Files created:**
```
src/commands/observe.rs                # New - live event tail command
src/cli.rs                             # Modified - Observe command
src/main.rs                            # Modified - route Observe command
```

---

## Phase 5: Agent System (Personalities)

**Goal:** Support for named agent personalities with configurable traits.

### 5.1 Agent Definition ✅

**Implementation:**
- Composable traits system (Expertise + Personality + Approach)
- 28 discrete traits (no numeric scales)
- Agent YAML schema with backstory, traits, communication_style
- Prompt generation from trait composition

**Files created:**
```
src/agent/mod.rs                       # Agent module
src/agent/traits.rs                    # 28 composable traits with prompt fragments
src/agent/loader.rs                    # Agent loading and prompt generation
```

**8 Agent Archetypes (from PAI + additions):**
```
~/.config/pais/agents/
├── intern.yaml       # Eager learner - enthusiastic, research, rapid
├── architect.yaml    # Strategic visionary - technical, analytical, systematic
├── engineer.yaml     # Battle-scarred implementer - technical, pragmatic, meticulous
├── researcher.yaml   # Thorough investigator - research, skeptical, thorough
├── skeptic.yaml      # Red team devil's advocate - security, contrarian, adversarial
├── creative.yaml     # Lateral thinker - creative, enthusiastic, exploratory
├── advisor.yaml      # Executive consultant - business, analytical, consultative
└── reviewer.yaml     # Meticulous quality gate - technical, meticulous, systematic
```

### 5.2 Agent Routing (SubagentStop) ✅

**Implementation:**
- `SubagentStop` event handler added to `HistoryHandler`
- Agent detection from payload: `subagent_type`, `agent_type`, or `agent` fields
- Agent's `history_category` overrides content-based categorization
- History entries tagged with `agent:<name>` when agent detected

**Routing:**
```
researcher → history/research/    (via history_category: research)
architect  → history/decisions/   (via history_category: decisions)
engineer   → history/execution/   (via history_category: execution)
```

**Files modified:**
```
src/hook/history.rs     # Added SubagentStop, agent-aware routing
```

### 5.3 Agent CLI Commands ✅

**Implemented commands:**
- `pais agent list` - list available agents with traits
- `pais agent show <name>` - display full agent config
- `pais agent traits` - list all 28 composable traits
- `pais agent prompt <name>` - generate prompt from traits
- `pais agent create <name>` - create new agent from template

**Files created:**
```
src/commands/agent.rs                  # Agent CLI commands
src/cli.rs                             # AgentAction enum
```

---

## Phase 6: Quality of Life (Polish) ✅

**Goal:** Small features that improve daily usage.

### 6.1 Tab Title Updates ✅

**Implementation:**
- `UserPromptSubmit` hook updates terminal tab title using OSC escape sequences
- Extracts task summary from user prompt (strips common prefixes, truncates)
- Format: `🤖 [task summary]` in terminal tab

**Files created/modified:**
```
src/hook/ui.rs                         # UI hook handler
.claude/settings.json                  # Added UserPromptSubmit hook
src/config.rs                          # Added ui_enabled config
```

### 6.2 Architecture Tracking ✅

**Implementation:**
- Auto-generates `~/.config/pais/architecture.md` on `pais sync`
- Shows: Skills (with tier), Agents (with traits), Hooks, Observability, Paths

**Files created/modified:**
```
src/architecture.rs                    # Architecture doc generation
src/commands/sync.rs                   # Calls architecture generation
```

### 6.3 Upgrade/Migration System ✅

**Implementation:**
- Version tracking via git tags in `~/.config/pais` (e.g., `v1`, `v2`)
- Migration trait for versioned config upgrades
- `pais upgrade` command with `--status` and `--dry-run` flags

**Files created/modified:**
```
src/migrate.rs                         # Migration framework (git tag based)
src/commands/upgrade.rs                # Upgrade CLI command
src/cli.rs                             # Added Upgrade command
```

---

## Phase 7: Plugin System (Extensibility) ✅

**Goal:** Enable third-party plugins with full hook/contract support.

### 7.1 Plugin Manifest (plugin.yaml) ✅

**Implementation:**
- `HooksSpec` now uses script paths instead of booleans
- Each hook can have multiple scripts with optional matchers
- Scripts specify: `script`, `matcher` (optional), `timeout` (default 30s)

**Example:**
```yaml
hooks:
  PreToolUse:
    - script: hooks/validate.py
      matcher: Bash
  Stop:
    - script: hooks/capture.py
```

### 7.2 Plugin Hook Execution ✅

**Implementation:**
- `src/plugin/executor.rs` executes plugin scripts
- Payload passed via stdin as JSON
- Environment variables: `PAIS_EVENT`, `PAIS_PLUGIN`
- Exit codes: 0=allow, 2=block, other=error
- Supports Python (via `uv` or `python3`) and Rust (compiled binaries)

### 7.3 Contract System ✅

**Implementation:**
- `src/contract/mod.rs` defines `ContractType` and `ContractRegistry`
- Plugins declare `provides` and `consumes` in manifest
- Registry validates contract providers exist

### 7.4 Plugin Registry ✅

**Already implemented commands:**
- `pais plugin install <path|name>` - install from local path or registry
- `pais plugin install --dev <path>` - symlink for development
- `pais plugin remove <name>` - uninstall
- `pais plugin update <name>` - update from registry
- `pais plugin list` - show installed plugins
- `pais plugin info <name>` - show plugin details
- `pais plugin new <name>` - scaffold new plugin
- `pais plugin verify <name>` - verify plugin structure

---

## Phase 8: System Status (Utility) ✅

**Goal:** Quick system health check and overview.

### 8.1 `pais status` Command ✅

**Implementation:**
- Shows PAIS version and directory paths
- Lists installed plugins with language/hooks info
- Lists skills with tier (core/deferred) and source (simple/plugin)
- Lists agents with traits and history routing
- Shows hook status (security, history, UI)
- Shows observability status (enabled, sinks)
- Shows history categories with entry counts and latest timestamp
- Shows configured registries with cache status
- Supports `--format text|json|yaml` output

**Files created/modified:**
```
src/commands/status.rs    # Enhanced status command
```

---

## Phase 9: Plugin Execution (Utility)

**Goal:** Direct plugin invocation for testing and scripting.

### 9.1 `pais run` Command (Existing)

**Already implemented:**
- `pais run <plugin> <action> [args...]` - Execute plugin actions
- Supports Python and Rust plugins
- Auto-builds Rust plugins if needed

**Files:**
```
src/commands/run.rs    # Already exists
```

---

## Phase 10: Environment Awareness ✅ DONE

**Goal:** Teach Claude about the user's environment - where code lives, preferred tools, custom CLIs.

### 10.1 Environment Configuration ✅

**Add to `pais.yaml`:**
```yaml
environment:
  repos-dir: ~/repos/
  # All repos cloned as: {repos-dir}/{org}/{repo}

  tool-preferences:
    ls: eza
    tree: eza --tree
    grep: rg
    find: fd
    cat: bat

  tools:
    otto:
      github: otto-rs/otto
      description: "CI runner, task automation"
    leeks:
      github: scottidler/leeks
      description: "Secret detection"
    clone:
      github: scottidler/clone
      description: "Git clone to repos-dir/{org}/{repo}"
```

**Tasks:**
- [x] Add `EnvironmentConfig` to `src/config.rs`
- [x] Parse `tool-preferences` and `tools` maps
- [x] Serde rename attributes for hyphen → underscore

**Files modified:**
```
src/config.rs    # Add EnvironmentConfig, ToolConfig
```

### 10.2 CORE Skill Environment Injection ✅

**Tasks:**
- [x] Update `pais context inject` to include environment section
- [x] Generate environment context from config:
  - Repos location and structure
  - Tool preferences (modern → legacy mappings)
  - Available custom tools with descriptions
- [x] Check tool availability at injection time

**Injected context example:**
```markdown
## Environment

### Repos
All repositories are at `~/repos/{org}/{repo}`.
Use `clone` to checkout new repos (e.g., `clone scottidler/otto`).

### Preferred Tools
Use modern alternatives when available:
- `rg` instead of `grep` (faster, respects .gitignore)
- `fd` instead of `find` (faster, saner defaults)
- `eza` instead of `ls` (better output)
- `bat` instead of `cat` (syntax highlighting)
- `eza --tree` instead of `tree`

Fallback to standard tools if modern ones unavailable.

### Custom Tools
- `otto` - CI/task runner (otto-rs/otto) ✓
- `leeks` - secret scanning (scottidler/leeks) ✓
- `clone` - smart git clone (scottidler/clone) ✓
```

**Files modified:**
```
src/commands/context.rs    # Add environment injection
```

### 10.3 Tool Availability Checking ✅

**Tasks:**
- [x] `pais doctor` checks all declared tools are in PATH
- [x] Shows install hints (github URL) for missing tools
- [x] Warns but doesn't fail for missing tools

**Example output:**
```
Tools:
  ✓ rg (rg 14.1.0)
  ✓ fd (fd 9.0.0)
  ✓ eza (eza 0.18.0)
  ✓ otto (otto 0.1.0)
  ✗ leeks (not found)
    Install: cargo install --git https://github.com/scottidler/leeks
```

**Files modified:**
```
src/commands/doctor.rs    # Add tool checking
```

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
| 8 | System Status | Low |
| 9 | Plugin Execution | Low |
| 10 | Environment Awareness ✅ | Medium |

**Recommended order:** Phase 1 → Phase 2 → Phase 3 → Phase 6 → Phase 4 → Phase 5 → Phase 7 → Phase 8 → Phase 9 → Phase 10

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

