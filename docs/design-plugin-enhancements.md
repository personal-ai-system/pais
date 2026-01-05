# Design: Plugin Enhancements (install.md, verify.md, bundle.yaml)

> Adding installation guides, verification checklists, and plugin bundles to PAIS.

## Overview

This design adds three features to the PAIS plugin system:

1. **install.md** - Human/AI-readable installation instructions for complex plugins
2. **verify.md** - Mandatory verification checklists to confirm successful installation
3. **bundle.yaml** - Curated collections of plugins that work well together

These features are inspired by Daniel Miessler's PAI Pack system while maintaining PAIS's existing architecture.

---

## 1. install.md

### Purpose

Complex plugins may require steps beyond "copy files":
- External dependencies (system packages, API keys)
- Configuration file modifications
- Database migrations
- Service registration

`install.md` provides structured installation instructions that both humans and AI agents can follow.

### Location

```
~/.config/pais/plugins/<plugin-name>/
├── plugin.yaml
├── install.md      # NEW: Optional installation guide
├── skills/
└── ...
```

### Format

```markdown
# Installing <plugin-name>

## Prerequisites

- [ ] PAIS >= 0.2.0
- [ ] API key: `OPENAI_API_KEY` in environment
- [ ] System package: `ffmpeg`

## Installation Steps

### Step 1: Install System Dependencies

```bash
# Ubuntu/Debian
sudo apt install ffmpeg

# macOS
brew install ffmpeg
```

### Step 2: Configure Environment

Add to `~/.config/pais/.env`:

```bash
OPENAI_API_KEY=sk-...
```

### Step 3: Enable Plugin Hooks

The plugin hooks are automatically registered via `plugin.yaml`.
No manual configuration needed.

## Post-Installation

Run verification:

```bash
pais plugin verify <plugin-name>
```
```

### plugin.yaml Reference

```yaml
plugin:
  name: voice-notifications
  version: 1.0.0
  description: Voice notifications via TTS
  install-guide: install.md  # Optional, points to install guide
```

### CLI Support

```bash
# Show installation guide
pais plugin install-guide <plugin-name>

# Or during install
pais plugin install <plugin-name>
# If install.md exists, shows it before proceeding
```

---

## 2. verify.md

### Purpose

Verification ensures plugins are correctly installed. This is critical because:
- AI agents may "simplify" installations
- Dependencies may be missing
- Configuration may be incomplete
- Hooks may fail silently

`verify.md` provides a mandatory checklist that must pass for installation to be considered complete.

### Location

```
~/.config/pais/plugins/<plugin-name>/
├── plugin.yaml
├── install.md
├── verify.md       # NEW: Verification checklist
├── skills/
└── ...
```

### Format

```markdown
# Verification: <plugin-name>

## File Checks

- [ ] `skills/<name>/SKILL.md` exists
- [ ] `hooks/on-stop.py` exists and is executable

## Dependency Checks

- [ ] `ffmpeg --version` returns 0
- [ ] `OPENAI_API_KEY` environment variable is set

## Functional Checks

- [ ] `pais plugin run <name> --test` succeeds
- [ ] Hook fires on test event

## Verification Commands

Run these commands to verify installation:

```bash
# Check skill is discovered
pais skill list | grep <name>

# Check hooks are registered
pais status | grep -A5 "Hooks"

# Run plugin self-test
pais plugin test <name>
```

## Expected Output

```
Plugin <name> v1.0.0: All checks passed
- Files: 3/3 present
- Dependencies: 2/2 satisfied
- Hooks: 1/1 registered
```
```

### plugin.yaml Reference

```yaml
plugin:
  name: voice-notifications
  version: 1.0.0
  description: Voice notifications via TTS
  verification: verify.md  # Optional, points to verification checklist
```

### CLI Support

```bash
# Run verification
pais plugin verify <plugin-name>

# Output
Verifying plugin: voice-notifications

File Checks:
  [x] skills/voice/SKILL.md exists
  [x] hooks/on-stop.py exists

Dependency Checks:
  [x] ffmpeg available
  [x] OPENAI_API_KEY set

Functional Checks:
  [x] Skill discovered
  [x] Hook registered

Result: PASSED (6/6 checks)
```

### Verification Spec in plugin.yaml

For automated verification, plugins can declare checks directly:

```yaml
plugin:
  name: voice-notifications
  version: 1.0.0

verification:
  guide: verify.md  # Human-readable guide

  checks:
    files:
      - skills/voice/SKILL.md
      - hooks/on-stop.py

    commands:
      - name: ffmpeg-available
        command: ffmpeg -version
        expect-exit: 0

      - name: skill-discovered
        command: pais skill list --json
        expect-contains: voice-notifications

    env-vars:
      - OPENAI_API_KEY
```

---

## 3. bundle.yaml

### Purpose

Bundles are curated collections of plugins that:
- Work well together
- Have correct installation order (respecting dependencies)
- Provide a complete capability (e.g., "full development environment")

### Location

```
~/.config/pais/bundles/
├── developer/
│   └── bundle.yaml
├── security/
│   └── bundle.yaml
└── full-stack/
    └── bundle.yaml
```

### Format

```yaml
# ~/.config/pais/bundles/developer/bundle.yaml

bundle:
  name: developer
  version: 1.0.0
  description: Full development environment with Rust, Python, and Git tools
  author: Scott Idler

# Plugin names as keys - order in YAML is preserved
plugins:
  rust-coder:
    required: true

  python-coder:
    required: true

  gx:
    required: false
    description: Multi-repo Git operations

  otto:
    required: false
    description: Task runner for CI/builds

# Optional: environment setup
environment:
  RUST_BACKTRACE: "1"
  PYTHONDONTWRITEBYTECODE: "1"

# Optional: post-install commands
post-install:
  - command: pais skill index --rebuild
    description: Rebuild skill index
```

### CLI Support

```bash
# List available bundles
pais bundle list

# Output
Available bundles:
  developer     Full development environment (4 plugins)
  security      Security analysis tools (3 plugins)
  full-stack    Everything (8 plugins)

# Show bundle contents
pais bundle show developer

# Output
Bundle: developer v1.0.0
Description: Full development environment with Rust, Python, and Git tools

Plugins (4):
  [required] rust-coder      Write Rust code using conventions
  [required] python-coder    Write Python code using conventions
  [optional] gx              Multi-repo Git operations
  [optional] otto            Task runner for CI/builds

# Install a bundle
pais bundle install developer

# Output
Installing bundle: developer

[1/4] Installing rust-coder...
  - Copying skills/rust-coder/
  - No hooks to register
  - Running verification... PASSED

[2/4] Installing python-coder...
  - Copying skills/python-coder/
  - No hooks to register
  - Running verification... PASSED

[3/4] Installing gx (optional)...
  - Copying skills/gx/
  - Copying agents/gx-reviewer.yaml
  - Running verification... PASSED

[4/4] Installing otto (optional)...
  - Copying skills/otto/
  - Running verification... PASSED

Post-install:
  - Rebuilding skill index... done

Bundle 'developer' installed successfully (4/4 plugins)

# Install only required plugins
pais bundle install developer --required-only

# Skip verification (not recommended)
pais bundle install developer --skip-verify
```

### Bundle Sources

Bundles can come from:

1. **Local** - `~/.config/pais/bundles/`
2. **Remote** - Git repositories or URLs (future)

```yaml
# bundle.yaml with remote plugins
bundle:
  name: community-tools

plugins:
  fabric-patterns:
    source: github:danielmiessler/fabric
    path: patterns/

  local-plugin:
    source: local
```

---

## Implementation Plan

### Phase 1: install.md Support

1. Add `install-guide` field to `PluginInfo` struct
2. Add `pais plugin install-guide <name>` command
3. Show install guide during `pais plugin install` if present
4. Update plugin documentation

**Files to modify:**
- `src/plugin/manifest.rs` - Add field
- `src/commands/plugin.rs` - Add command
- `src/cli.rs` - Add subcommand

### Phase 2: verify.md Support

1. Add `verification` field to `PluginManifest`
2. Implement `VerificationRunner` that:
   - Checks file existence
   - Runs verification commands
   - Checks environment variables
3. Add `pais plugin verify <name>` command
4. Run verification after `pais plugin install`

**Files to modify:**
- `src/plugin/manifest.rs` - Add verification spec
- `src/plugin/verify.rs` - NEW: Verification runner
- `src/commands/plugin.rs` - Add verify command

### Phase 3: bundle.yaml Support

1. Create `Bundle` struct and parser
2. Implement `BundleManager` for discovery
3. Add `pais bundle` subcommand group
4. Implement installation with dependency ordering
5. Add post-install hook support

**Files to create:**
- `src/bundle/mod.rs` - Bundle types
- `src/bundle/manifest.rs` - bundle.yaml parsing
- `src/bundle/manager.rs` - Discovery and installation
- `src/commands/bundle.rs` - CLI commands

---

## Schema Definitions

### plugin.yaml (Updated)

```yaml
plugin:
  name: example-plugin
  version: 1.0.0
  description: Example plugin with all features
  authors:
    - Name <email@example.com>
  language: python
  license: MIT
  repository: https://github.com/example/plugin
  keywords:
    - example
    - demo

  # NEW: Installation and verification
  install-guide: install.md
  verification: verify.md

pais:
  core-version: ">=0.2.0"

provides:
  some-service:
    contract: ServiceContract

consumes:
  other-service:
    contract: OtherContract
    optional: true

config:
  api-key:
    type: string
    required: true
    env: EXAMPLE_API_KEY
    secret: true

hooks:
  PreToolUse:
    - script: hooks/validate.py
      matcher: Bash
      timeout: 30

  Stop:
    - script: hooks/capture.py

build:
  type: uv
  requirements: requirements.txt

# NEW: Automated verification checks
verification:
  guide: verify.md
  checks:
    files:
      - skills/example/SKILL.md
      - hooks/validate.py
    commands:
      - name: skill-present
        command: pais skill list --json
        expect-contains: example
    env-vars:
      - EXAMPLE_API_KEY
```

### bundle.yaml

```yaml
bundle:
  name: bundle-name
  version: 1.0.0
  description: What this bundle provides
  author: Author Name
  license: MIT

  # Minimum PAIS version
  pais-version: ">=0.2.0"

# Plugin names as keys - installed in YAML order
plugins:
  plugin-one:
    required: true
    description: Why this plugin is included

  plugin-two:
    required: true

  optional-plugin:
    required: false
    description: Nice to have

  # Remote plugin (future)
  remote-plugin:
    source: github:org/repo
    path: plugins/specific
    required: false

# Environment variables to set
environment:
  SOME_VAR: value

# Commands to run after all plugins installed
post-install:
  - command: pais skill index --rebuild
    description: Rebuild skill index

  - command: pais doctor
    description: Verify system health

# Optional: conflicts with other bundles
conflicts:
  - other-bundle-name
```

---

## Migration Path

### Existing Plugins

Existing plugins continue to work unchanged. The new fields are optional:

```yaml
# Minimal plugin.yaml (unchanged)
plugin:
  name: simple-plugin
  version: 1.0.0
  description: A simple plugin
```

### Adding install.md/verify.md

Plugin authors can add these files at any time:

```bash
cd ~/.config/pais/plugins/my-plugin/

# Add installation guide
cat > install.md << 'EOF'
# Installing my-plugin

## Prerequisites
- PAIS >= 0.2.0

## Steps
1. No additional setup required
EOF

# Add verification
cat > verify.md << 'EOF'
# Verification: my-plugin

- [ ] Skill discovered: `pais skill list | grep my-plugin`
EOF

# Update manifest
# plugin.yaml
plugin:
  name: my-plugin
  install-guide: install.md
  verification: verify.md
```

---

## Open Questions

1. **Remote bundles**: Should we support fetching bundles from Git repos in v1?
   - Recommendation: No, keep v1 local-only. Add remote in v2.

2. **Bundle updates**: How to handle bundle updates when plugins change?
   - Recommendation: `pais bundle update <name>` re-runs installation

3. **Verification strictness**: Should verification failure block installation?
   - Recommendation: Warn but allow `--force` flag

4. **Bundle conflicts**: How to handle plugins that conflict?
   - Recommendation: Warn if same plugin in multiple bundles with different versions

---

## Related Documents

- [CLAUDE.md](/home/saidler/repos/personal-ai-system/pais/CLAUDE.md) - Project conventions
- [design-skills-architecture.md](design-skills-architecture.md) - Skills system design
- [gap-analysis-pais-vs-kai.md](gap-analysis-pais-vs-kai.md) - Feature comparison with PAI
