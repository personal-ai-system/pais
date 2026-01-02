# Design: Personal Skills Separation

> **Status:** Planned  
> **Author:** Scott Idler  
> **Date:** 2026-01-01

## Problem Statement

The PAII core repository currently contains skills specific to Scott's personal tooling:

| Skill | Dependency | Issue |
|-------|------------|-------|
| `rust-coder` | Scott's conventions, `scaffold` | Others have different conventions |
| `python-coder` | `pyr`, `uv`, Scott's conventions | Tool-specific |
| `otto` | `scottidler/otto` | Not everyone uses otto |
| `git-tools` | `scottidler/git-tools` | Personal repo |
| `aka` | `scottidler/aka` | Personal repo |
| `cidr` | `scottidler/cidr` | Personal repo |
| `dashify` | `scottidler/dashify` | Personal repo |
| `rkvr` | `scottidler/rkvr` | Personal repo |
| `whitespace` | `scottidler/whitespace` | Personal repo |

This creates tension with PAII's goals:
1. **Modularity**: Core should be generic, personal stuff should be opt-in
2. **Team sharing**: Teammates shouldn't inherit Scott's personal tools
3. **Discoverability**: Hard to distinguish "core" from "personal" plugins

## Proposed Solution

Split into two repositories:

### 1. `scottidler/paii` (Core Framework)

Contains only:
- Core CLI (`paii`)
- Plugin infrastructure
- Hook system
- Generic examples (`hello-world`, `hello-rust`)
- Documentation

```
paii/
├── src/                    # Core Rust CLI
├── examples/
│   ├── hello-world/        # Example Python plugin
│   └── hello-rust/         # Example Rust plugin
├── registry/
│   └── plugins.toml        # Core registry (examples only)
├── docs/
└── CLAUDE.md
```

### 2. `scottidler/paii-skills` (Personal Skills)

Contains Scott's personal skills:

```
paii-skills/
├── plugins/
│   ├── rust-coder/
│   ├── python-coder/
│   ├── otto/
│   ├── git-tools/
│   ├── aka/
│   ├── cidr/
│   ├── dashify/
│   ├── rkvr/
│   └── whitespace/
├── registry/
│   └── plugins.toml        # Personal registry
└── README.md
```

## Registry Configuration

### Core Registry (in `paii`)

```toml
# registry/plugins.toml
[registry]
name = "paii-core"
description = "Core PAII examples"
version = "0.1.0"

[[plugins]]
name = "hello-world"
# ...

[[plugins]]
name = "hello-rust"
# ...
```

### Personal Registry (in `paii-skills`)

```toml
# registry/plugins.toml
[registry]
name = "scottidler-skills"
description = "Scott's personal PAII skills"
version = "0.1.0"

[[plugins]]
name = "rust-coder"
source = "https://github.com/scottidler/paii-skills"
path = "plugins/rust-coder"
# ...

# ... all personal skills ...
```

## User Workflow

### For Scott (or anyone with personal skills)

```bash
# Core is already configured
paii registry list
# → core: https://...scottidler/paii/main/registry/plugins.toml

# Add personal registry
paii registry add personal https://raw.githubusercontent.com/scottidler/paii-skills/main/registry/plugins.toml

# Install personal skills
paii plugin install rust-coder
paii plugin install otto
paii plugin install git-tools
```

### For Teammates

```bash
# Start with just core
paii init

# Create their own skills repo
# Fork paii-skills or create from scratch

# Add their personal registry
paii registry add personal https://raw.githubusercontent.com/teammate/paii-skills/main/registry/plugins.toml
```

### For Teams (Shared Skills)

```bash
# Team maintains a shared skills repo
paii registry add mycompany https://raw.githubusercontent.com/mycompany/paii-skills/main/registry/plugins.toml

# Team members install shared skills
paii plugin install mycompany-runbooks
paii plugin install mycompany-oncall
```

## Migration Steps

1. **Create `scottidler/paii-skills` repo**
   ```bash
   mkdir paii-skills
   cd paii-skills
   git init
   ```

2. **Move plugins**
   ```bash
   # From paii repo
   mv plugins/rust-coder ../paii-skills/plugins/
   mv plugins/python-coder ../paii-skills/plugins/
   mv plugins/otto ../paii-skills/plugins/
   mv plugins/git-tools ../paii-skills/plugins/
   mv plugins/aka ../paii-skills/plugins/
   mv plugins/cidr ../paii-skills/plugins/
   mv plugins/dashify ../paii-skills/plugins/
   mv plugins/rkvr ../paii-skills/plugins/
   mv plugins/whitespace ../paii-skills/plugins/
   ```

3. **Update registries**
   - Remove personal skills from `paii/registry/plugins.toml`
   - Create `paii-skills/registry/plugins.toml` with moved skills

4. **Update default config**
   - Remove `scottidler-skills` from default `paii.toml`
   - Document how to add personal registries

5. **Update CLAUDE.md**
   - Reference new repo structure
   - Update skill documentation

6. **Push both repos**

## Benefits

| Benefit | Description |
|---------|-------------|
| **Clean core** | Core repo is generic, forkable by anyone |
| **Opt-in personal** | Personal skills require explicit registry add |
| **Team friendly** | Teams can maintain their own skill repos |
| **Clear ownership** | Obvious what's "core" vs "personal" |
| **Independent versioning** | Personal skills can version independently |

## Drawbacks

| Drawback | Mitigation |
|----------|------------|
| Two repos to maintain | Minimal overhead, skills change rarely |
| Extra setup step | Document in README, `paii init` could prompt |
| Harder to discover | Registry search works across all registries |

## Future Considerations

1. **Community registry**: Central registry of community-contributed skills
2. **Private registries**: Support for authenticated/private registries (team use)
3. **Skill templates**: `paii plugin new` could scaffold from templates
4. **Skill marketplace**: Web UI for browsing available skills

## Decision

**Deferred.** Document the plan, implement when:
- Sharing PAII with teammates
- Publishing PAII publicly
- Personal skills become too numerous

For now, keeping everything in one repo is simpler for development.

---

## Appendix: Current State

As of 2026-01-01, `paii` contains:

**Examples (keep in core):**
- `hello-world`
- `hello-rust`

**Personal skills (move to paii-skills):**
- `rust-coder`
- `python-coder`
- `otto`
- `git-tools`
- `aka`
- `cidr`
- `dashify`
- `rkvr`
- `whitespace`

