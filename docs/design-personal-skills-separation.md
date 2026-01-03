# Design: Personal Skills Separation

> **Status:** Planned
> **Author:** Scott Idler
> **Date:** 2026-01-01

## Problem Statement

The PAIS core repository currently contains skills specific to Scott's personal tooling:

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

This creates tension with PAIS's goals:
1. **Modularity**: Core should be generic, personal stuff should be opt-in
2. **Team sharing**: Teammates shouldn't inherit Scott's personal tools
3. **Discoverability**: Hard to distinguish "core" from "personal" plugins

## Proposed Solution

Split into two repositories:

### 1. `scottidler/pais` (Core Framework)

Contains only:
- Core CLI (`pais`)
- Plugin infrastructure
- Hook system
- Generic examples (`hello-world`, `hello-rust`)
- Documentation

```
pais/
├── src/                    # Core Rust CLI
├── examples/
│   ├── hello-world/        # Example Python plugin
│   └── hello-rust/         # Example Rust plugin
├── registry/
│   └── plugins.yaml        # Core registry (examples only)
├── docs/
└── CLAUDE.md
```

### 2. `scottidler/pais-skills` (Personal Skills)

Contains Scott's personal skills:

```
pais-skills/
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
│   └── plugins.yaml        # Personal registry
└── README.md
```

## Registry Configuration

### Core Registry (in `pais`)

```toml
# registry/plugins.yaml
[registry]
name = "pais-core"
description = "Core PAIS examples"
version = "0.1.0"

[[plugins]]
name = "hello-world"
# ...

[[plugins]]
name = "hello-rust"
# ...
```

### Personal Registry (in `pais-skills`)

```toml
# registry/plugins.yaml
[registry]
name = "scottidler-skills"
description = "Scott's personal PAIS skills"
version = "0.1.0"

[[plugins]]
name = "rust-coder"
source = "https://github.com/scottidler/pais-skills"
path = "plugins/rust-coder"
# ...

# ... all personal skills ...
```

## User Workflow

### For Scott (or anyone with personal skills)

```bash
# Core is already configured
pais registry list
# → core: https://...scottidler/pais/main/registry/plugins.yaml

# Add personal registry
pais registry add personal https://raw.githubusercontent.com/scottidler/pais-skills/main/registry/plugins.yaml

# Install personal skills
pais plugin install rust-coder
pais plugin install otto
pais plugin install git-tools
```

### For Teammates

```bash
# Start with just core
pais init

# Create their own skills repo
# Fork pais-skills or create from scratch

# Add their personal registry
pais registry add personal https://raw.githubusercontent.com/teammate/pais-skills/main/registry/plugins.yaml
```

### For Teams (Shared Skills)

```bash
# Team maintains a shared skills repo
pais registry add mycompany https://raw.githubusercontent.com/mycompany/pais-skills/main/registry/plugins.yaml

# Team members install shared skills
pais plugin install mycompany-runbooks
pais plugin install mycompany-oncall
```

## Migration Steps

1. **Create `scottidler/pais-skills` repo**
   ```bash
   mkdir pais-skills
   cd pais-skills
   git init
   ```

2. **Move plugins**
   ```bash
   # From pais repo
   mv plugins/rust-coder ../pais-skills/plugins/
   mv plugins/python-coder ../pais-skills/plugins/
   mv plugins/otto ../pais-skills/plugins/
   mv plugins/git-tools ../pais-skills/plugins/
   mv plugins/aka ../pais-skills/plugins/
   mv plugins/cidr ../pais-skills/plugins/
   mv plugins/dashify ../pais-skills/plugins/
   mv plugins/rkvr ../pais-skills/plugins/
   mv plugins/whitespace ../pais-skills/plugins/
   ```

3. **Update registries**
   - Remove personal skills from `pais/registry/plugins.yaml`
   - Create `pais-skills/registry/plugins.yaml` with moved skills

4. **Update default config**
   - Remove `scottidler-skills` from default `pais.yaml`
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
| Extra setup step | Document in README, `pais init` could prompt |
| Harder to discover | Registry search works across all registries |

## Future Considerations

1. **Community registry**: Central registry of community-contributed skills
2. **Private registries**: Support for authenticated/private registries (team use)
3. **Skill templates**: `pais plugin new` could scaffold from templates
4. **Skill marketplace**: Web UI for browsing available skills

## Decision

**Deferred.** Document the plan, implement when:
- Sharing PAIS with teammates
- Publishing PAIS publicly
- Personal skills become too numerous

For now, keeping everything in one repo is simpler for development.

---

## Appendix: Current State

As of 2026-01-01, `pais` contains:

**Examples (keep in core):**
- `hello-world`
- `hello-rust`

**Personal skills (move to pais-skills):**
- `rust-coder`
- `python-coder`
- `otto`
- `git-tools`
- `aka`
- `cidr`
- `dashify`
- `rkvr`
- `whitespace`

