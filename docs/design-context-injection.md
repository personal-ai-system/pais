# Context Injection Architecture

> How PAIS achieves instant skill routing without Claude Code's permission prompts

**Date:** 2026-01-02
**Status:** Design Complete, Implementation Pending

---

## The Problem

Claude Code's built-in skill system has significant latency:

1. **Discovery** — Claude loads only skill names/descriptions at startup
2. **Matching** — When a request matches, Claude asks for permission
3. **Confirmation** — User must approve before skill content loads
4. **Loading** — Full SKILL.md is read and processed
5. **Response** — Claude finally responds using the skill

This creates:
- **2+ second latency** on every skill invocation
- **Interruption** with permission prompts
- **Poor UX** compared to instant responses

Daniel Miessler's Kai/PAI system achieves **~95-98% routing accuracy** with **instant responses**. How?

---

## The Solution: Hook-Based Context Injection

Daniel bypasses Claude Code's skill system entirely. Instead:

### At Session Start (Once)

A `SessionStart` hook injects context directly into Claude's system prompt:

```
┌─────────────────────────────────────────────────────────────────┐
│                     SESSION START                                │
├─────────────────────────────────────────────────────────────────┤
│                                                                 │
│  1. Claude Code fires SessionStart event                        │
│                    │                                            │
│                    ▼                                            │
│  2. load-context hook runs                                      │
│                    │                                            │
│                    ▼                                            │
│  3. Hook reads skill index + core context                       │
│                    │                                            │
│                    ▼                                            │
│  4. Hook outputs <system-reminder> with context                 │
│                    │                                            │
│                    ▼                                            │
│  5. Claude Code injects into system prompt                      │
│                    │                                            │
│                    ▼                                            │
│  6. Context is NOW LOADED for entire session                    │
│                                                                 │
└─────────────────────────────────────────────────────────────────┘
```

### At Request Time (Instant)

No skill invocation needed — context is already present:

```
┌─────────────────────────────────────────────────────────────────┐
│                     USER REQUEST                                 │
├─────────────────────────────────────────────────────────────────┤
│                                                                 │
│  User: "What error handling crate should I use in Rust?"        │
│                    │                                            │
│                    ▼                                            │
│  Claude sees pre-loaded context:                                │
│    - Skill index with USE WHEN triggers                         │
│    - rust-coder: "USE WHEN Rust, cargo, CLI tools"              │
│                    │                                            │
│                    ▼                                            │
│  Claude matches intent → rust-coder skill                       │
│                    │                                            │
│                    ▼                                            │
│  Claude reads full skill file (if needed)                       │
│                    │                                            │
│                    ▼                                            │
│  Response: "eyre" (following skill conventions)                 │
│                                                                 │
│  TOTAL TIME: Instant (no permission prompt)                     │
│                                                                 │
└─────────────────────────────────────────────────────────────────┘
```

---

## The Key Components

### 1. Skill Index

A JSON file containing all skill metadata and triggers:

```json
{
  "generated": "2026-01-02T12:00:00Z",
  "skills": {
    "rust-coder": {
      "name": "rust-coder",
      "path": "rust-coder/SKILL.md",
      "description": "Write Rust code using Scott's conventions...",
      "triggers": ["rust", "cargo", "cli", "eyre", "clap"],
      "tier": "deferred"
    },
    "python-coder": {
      "name": "python-coder",
      "path": "python-coder/SKILL.md",
      "description": "Write Python code using Scott's conventions...",
      "triggers": ["python", "uv", "ruff", "pytest"],
      "tier": "deferred"
    }
  }
}
```

### 2. USE WHEN Triggers

Every skill's description contains routing triggers:

```yaml
---
name: rust-coder
description: Write Rust code using Scott's conventions. USE WHEN creating Rust CLIs, libraries, reviewing Rust code, OR when the user mentions Rust, cargo, or CLI tools.
---
```

The `USE WHEN` clause is parsed to extract trigger words: `["rust", "cli", "cargo", "libraries"]`

### 3. SessionStart Hook

A hook that runs at session start and injects context:

```typescript
// hooks/load-context.ts
const output = `<system-reminder>
PAIS CONTEXT (Auto-loaded at Session Start)

## Available Skills

${skillIndex}

## Routing Instructions

When a user request matches a skill's triggers:
1. Read the full SKILL.md file for that skill
2. Follow the skill's instructions and conventions
3. No need to ask for permission - just use the context

</system-reminder>`;

console.log(output);
```

### 4. Tiered Loading

Not all skills need full content in context:

| Tier | What's Loaded | When |
|------|---------------|------|
| **0: Core** | Full SKILL.md content | Session start (always) |
| **1: Index** | Name + description + triggers | Session start (always) |
| **2: Deferred** | Full SKILL.md content | On first match (read file) |

This keeps the system prompt small while enabling instant routing.

---

## Implementation Plan

### Phase 1: Skill Index Generator

**Command:** `pais skill index`

Generates `~/.config/pais/skills/skill-index.json`:
- Parses all SKILL.md frontmatter
- Extracts USE WHEN triggers
- Assigns tiers (core vs deferred)

### Phase 2: Context Template

**File:** `~/.config/pais/context/session-context.md`

A template that gets populated with:
- Current date/time
- User identity (from config)
- Skill index (names, descriptions, triggers)
- Core skill content (if any)

### Phase 3: SessionStart Hook

**File:** `~/.config/pais/hooks/load-context.ts`

A Bun/TypeScript hook that:
1. Reads the context template
2. Reads the skill index
3. Outputs as `<system-reminder>` for Claude

**Registration:** Add to `.claude/settings.json`:
```json
{
  "hooks": {
    "SessionStart": [{
      "matcher": "",
      "hooks": [{
        "type": "command",
        "command": "bun run ~/.config/pais/hooks/load-context.ts"
      }]
    }]
  }
}
```

### Phase 4: Sync Update

Update `pais sync` to also:
- Copy/symlink the hook to appropriate location
- Update Claude Code's settings.json with hook registration

---

## Comparison: Before and After

### Before (Claude Code Skills Only)

```
User: "What crate for errors in Rust?"
       │
       ▼
Claude: "I'll check your Rust conventions..."
       │
       ▼
[Permission prompt: Use rust-coder skill?]
       │
       ▼
User: [clicks approve]
       │
       ▼
[Loads SKILL.md - 1-2 seconds]
       │
       ▼
Claude: "Based on your conventions, use eyre..."

TOTAL: 3-5 seconds + interruption
```

### After (Context Injection)

```
User: "What crate for errors in Rust?"
       │
       ▼
Claude: (sees pre-loaded triggers, matches rust-coder)
       │
       ▼
Claude: (reads skill file directly)
       │
       ▼
Claude: "eyre"

TOTAL: Instant, no interruption
```

---

## Why This Works

### 1. Hooks Don't Need Permission

Claude Code hooks run automatically. No confirmation prompt.

### 2. `<system-reminder>` Is Injected

Content output by hooks in this format becomes part of Claude's context.

### 3. File Reading Is Fast

Claude can read files directly without the skill invocation overhead.

### 4. Triggers Enable Routing

The USE WHEN triggers let Claude match intent without the skill system.

---

## Security Considerations

- Hooks run at session start — they must be fast and never crash
- Context injection is automatic — be careful what you inject
- Skill content is read from disk — ensure skills come from trusted sources

---

## Related Documents

- [design-skills-architecture-phases.md](design-skills-architecture-phases.md) — Skill management implementation
- [docs/claude-code/hooks-guide.md](claude-code/hooks-guide.md) — Claude Code hooks reference
- [docs/kai/summary.md](kai/summary.md) — Original Kai/PAI inspiration

---

## Next Steps

1. Implement `pais skill index` command
2. Create context template
3. Write SessionStart hook in TypeScript
4. Update `pais sync` to register hook
5. Test end-to-end with Claude Code

