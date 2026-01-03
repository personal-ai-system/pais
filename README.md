# PAIS — Personal AI Infrastructure

A modular plugin system for Claude Code, designed for extensibility and team sharing.

Inspired by [Daniel Miessler's Kai/PAI system](https://github.com/danielmiessler/pais), reimplemented with a focus on true modularity, clean interfaces, and Rust performance.

## Features

- **Plugin System** — Install, manage, and create plugins in Python or Rust
- **Registry** — Discover and install plugins from remote registries
- **Hooks** — Intercept Claude Code events (security validation, history capture)
- **History** — File-based session tracking with YAML frontmatter
- **Config** — TOML-based configuration with environment variable support

## Installation

### From Source

```bash
git clone https://github.com/scottidler/pais.git
cd pais
cargo install --path .
```

### Shell Completions

```bash
# Zsh
mkdir -p ~/.zsh/completions
cp completions/_pais ~/.zsh/completions/
# Add to ~/.zshrc: fpath=(~/.zsh/completions $fpath)

# Bash
sudo cp completions/pais.bash /etc/bash_completion.d/pais

# Fish
cp completions/pais.fish ~/.config/fish/completions/
```

## Quick Start

```bash
# Initialize PAIS in your home directory
pais init

# Check your setup
pais doctor

# Update registries
pais registry update

# Search for plugins
pais registry search hello

# Install a plugin from registry
pais plugin install hello-world

# List installed plugins
pais plugin list

# Run a plugin action
pais run hello-world greet World
```

## Commands

| Command | Description |
|---------|-------------|
| `pais init` | Initialize PAIS configuration |
| `pais doctor` | Diagnose setup issues |
| `pais status` | Show system status |
| `pais plugin list` | List installed plugins |
| `pais plugin install <source>` | Install a plugin (path or registry name) |
| `pais plugin remove <name>` | Remove a plugin |
| `pais plugin new <name>` | Create a new plugin scaffold |
| `pais plugin info <name>` | Show plugin details |
| `pais registry list` | List configured registries |
| `pais registry update` | Update registry cache |
| `pais registry search <query>` | Search for plugins |
| `pais run <plugin> <action>` | Run a plugin action |
| `pais config show` | Show current configuration |
| `pais history recent` | Show recent history entries |

## Creating Plugins

```bash
# Create a Python plugin
pais plugin new my-skill --language python

# Create a Rust plugin
pais plugin new my-hook --language rust --type hook

# Install in dev mode (symlink)
pais plugin install --dev ./my-skill
```

### Plugin Structure

```
my-plugin/
├── plugin.yaml      # Plugin manifest
├── SKILL.md         # Skill documentation (for skill plugins)
├── README.md        # Plugin README
├── pyproject.toml   # Python dependencies (or Cargo.toml for Rust)
└── src/
    └── main.py      # Entry point (or main.rs for Rust)
```

### plugin.yaml

```toml
[plugin]
name = "my-plugin"
version = "0.1.0"
description = "My awesome plugin"
language = "python"

[hooks]
pre_tool_use = false
stop = true

[build]
type = "uv"
```

## Configuration

PAIS looks for configuration in this order:
1. `--config` flag
2. `$PAIS_CONFIG` environment variable
3. `$PAIS_DIR/pais.yaml`
4. `~/.config/pais/pais.yaml`
5. `./pais.yaml` (for development)

### Example pais.yaml

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
core = "https://raw.githubusercontent.com/scottidler/pais/main/registry/plugins.toml"

[hooks]
security_enabled = true
history_enabled = true
```

## Claude Code Integration

PAIS integrates with Claude Code via hooks. Add to `.claude/settings.json`:

```json
{
  "hooks": {
    "PreToolUse": [
      {
        "matcher": "Bash",
        "hooks": [{"type": "command", "command": "pais hook dispatch PreToolUse"}]
      }
    ],
    "Stop": [
      {
        "matcher": "*",
        "hooks": [{"type": "command", "command": "pais hook dispatch Stop"}]
      }
    ]
  }
}
```

## Architecture

```
┌─────────────────────────────────────────────────────────┐
│                     Claude Code                          │
│                         │                                │
│    ┌────────────────────▼────────────────────┐          │
│    │              Hooks System               │          │
│    │  (PreToolUse, Stop, SessionStart, etc.) │          │
│    └────────────────────┬────────────────────┘          │
└─────────────────────────┼───────────────────────────────┘
                          │
                          ▼
┌─────────────────────────────────────────────────────────┐
│                      pais CLI                            │
│  ┌─────────┐ ┌─────────┐ ┌─────────┐ ┌─────────┐       │
│  │ plugin  │ │registry │ │ history │ │  hook   │       │
│  └─────────┘ └─────────┘ └─────────┘ └─────────┘       │
└─────────────────────────────────────────────────────────┘
                          │
                          ▼
┌─────────────────────────────────────────────────────────┐
│                      Plugins                             │
│  ┌─────────────┐ ┌─────────────┐ ┌─────────────┐       │
│  │ Python/Rust │ │ Python/Rust │ │ Python/Rust │       │
│  │   Plugin    │ │   Plugin    │ │   Plugin    │       │
│  └─────────────┘ └─────────────┘ └─────────────┘       │
└─────────────────────────────────────────────────────────┘
```

## Development

```bash
# Run tests
cargo test

# Run with coverage
cargo llvm-cov --html

# Check formatting and lints
cargo fmt --check
cargo clippy -- -D warnings

# Build release binary
cargo build --release
```

## License

MIT

## Credits

- Inspired by [Daniel Miessler's Kai/PAI](https://github.com/danielmiessler/pais)
- Built on [Claude Code](https://claude.ai/code)

