# PAII - Personal AI Infrastructure

> A modular plugin system for Claude Code, built in Rust.

## Project Overview

PAII extends Claude Code with:
- **Security hooks** — Block dangerous commands before execution
- **History system** — Track sessions, learnings, decisions
- **Plugin system** — Python and Rust plugins with manifest-based discovery
- **Registry system** — Plugin discovery and sharing

## Quick Reference

```bash
# Build
cargo build --release

# Run tests
cargo test

# Full CI (lint + check + test)
otto ci

# Install to PATH
cargo install --path . --locked
```

## Architecture

```
src/
├── main.rs          # Entry point, CLI dispatch
├── cli.rs           # Clap command definitions
├── config.rs        # Configuration loading (paii.toml)
├── commands/        # Subcommand implementations
│   ├── plugin.rs    # paii plugin *
│   ├── hook.rs      # paii hook dispatch
│   ├── history.rs   # paii history *
│   ├── config.rs    # paii config *
│   ├── registry.rs  # paii registry *
│   ├── run.rs       # paii run <plugin> <action>
│   └── status.rs    # paii status
├── hook/
│   ├── mod.rs       # HookEvent, HookResult, HookHandler trait
│   ├── security.rs  # SecurityValidator (blocks dangerous commands)
│   └── history.rs   # HistoryHandler (captures sessions)
├── history/
│   └── mod.rs       # HistoryEntry, HistoryStore
├── plugin/
│   ├── mod.rs       # Plugin struct
│   ├── manifest.rs  # plugin.toml parsing
│   └── loader.rs    # Plugin loading
└── contract/        # Interface definitions (future)
```

## Key Files

- `paii.toml` — User configuration (paths, registries, hooks)
- `plugin.toml` — Plugin manifest (in each plugin directory)
- `.claude/settings.json` — Claude Code hook registration

## Coding Conventions

1. **Rust style** — Use `cargo fmt`, follow clippy warnings
2. **Error handling** — Use `eyre` with `.context()` for errors
3. **Logging** — Use `log::info!`, `log::debug!`, etc.
4. **Colors** — Use `colored` crate for terminal output
5. **Tests** — Add `#[cfg(test)]` module in same file

## Plugin System

Plugins live in `~/.config/paii/plugins/<name>/` with:
- `plugin.toml` — Manifest (name, version, language, hooks)
- `src/main.py` or `src/main.rs` — Entry point
- `SKILL.md` — Optional skill definition

**Python plugins** use `uv` (not pip/poetry).
**Rust plugins** use `cargo build --release`.

## Hooks

Claude Code calls `paii hook dispatch <event>` with JSON on stdin.

| Event | Purpose |
|-------|---------|
| PreToolUse | Security check before tool runs |
| SessionStart | Log session start |
| Stop | Capture session summary |
| SessionEnd | Log session end |

Exit codes: `0` = allow, `2` = block.

## History

Stored in `~/.config/paii/history/<category>/<date>/<id>.md`

Categories: `sessions`, `events`, `learnings`, `decisions`

## Data Formats

| Format | Use For | Why |
|--------|---------|-----|
| **Markdown** | History, skills, docs | Human + LLM readable, natural structure |
| **TOML/YAML** | Config, structured data | Less noise than JSON, LLM-friendly |
| **JSON** | API responses, hook payloads | Required by external systems |

- History entries use **Markdown with YAML frontmatter**
- Config files use **TOML** (paii.toml, plugin.toml)
- Hook payloads are **JSON** (Claude Code's protocol)
- Plugin CLI output is **JSON** (for programmatic parsing)

Prefer YAML/TOML over JSON for any data we control. JSON only when required by external APIs.

## DO NOT

- Use pip/poetry for Python — use `uv`
- Hardcode dependency versions in Cargo.toml — use `cargo add`
- Add features without tests
- Ignore clippy warnings

## Build Tools

This project uses `otto` as the task runner (not Make):

```bash
otto ci       # Full CI pipeline
otto check    # Compile + clippy + format
otto test     # Run tests
otto build    # Release build
```

