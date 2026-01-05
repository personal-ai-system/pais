# Image Generation Design

> AI-powered image generation as a core PAIS feature, with text-based diagram support via skill guidance.

## Overview

This document captures the design for PAIS image generation capabilities, inspired by kai/pai's Art skill but adapted to PAIS's Rust-first architecture.

## Goals

1. **AI Image Generation** - Generate polished images (diagrams, illustrations, headers) via `pais image generate`
2. **Text-Based Diagrams** - Guide Claude to output Mermaid/Excalidraw directly (no CLI involvement)
3. **Consistent Aesthetic** - Dark-mode, hand-drawn Excalidraw style with defined color palette
4. **No Runtime Dependencies** - Rust binary, no bun/node required

## Architecture

```
pais/
├── src/commands/
│   └── image.rs                    # pais image generate
├── skills/image/
│   ├── SKILL.md                    # Routing + usage
│   ├── Aesthetic.md                # Style guide
│   └── workflows/
│       ├── technical-diagrams.md   # AI-generated architecture diagrams
│       ├── mermaid.md              # Direct mermaid output (no CLI)
│       ├── excalidraw.md           # Direct JSON output (no CLI)
│       └── editorial.md            # Blog headers, illustrations
```

## Design Decisions

### Why `image` not `art`?

- `image` describes the output (a file), not the activity
- More Unix-like naming convention
- Extensible: `pais image convert`, `pais image optimize` (future)

### Why only `pais image generate`?

Mermaid and Excalidraw output **text**, not images:
- Mermaid → markdown syntax (rendered by GitHub, VS Code, Obsidian)
- Excalidraw → JSON (imported into excalidraw.com)

These don't need a CLI command. Claude outputs the text directly with skill guidance.

| Request | Output | Method |
|---------|--------|--------|
| "quick diagram of auth flow" | Mermaid syntax | Claude outputs directly |
| "excalidraw diagram I can edit" | JSON | Claude outputs directly |
| "polished diagram for presentation" | PNG file | `pais image generate` |
| "blog header image" | PNG file | `pais image generate` |

### Why Rust instead of TypeScript?

kai's Generate.ts requires bun/node runtime. Rust provides:

| Aspect | TypeScript (kai) | Rust (pais) |
|--------|------------------|-------------|
| Runtime | Requires bun/node | None - single binary |
| Distribution | npm ecosystem | `cargo install` |
| Claude integration | Shell to bun | Direct `pais image generate` |
| Error handling | Exceptions | `eyre` with context |

### Future: `pais diagram` command?

Deferred. Could add later for:
- `pais diagram render mermaid input.md --output out.png` (via mermaid-cli)
- `pais diagram validate mermaid input.md`

For now, use existing tools directly if rendering is needed.

## Command Design

### `pais image generate`

```bash
pais image generate \
  --model gemini \
  --prompt "Clean Excalidraw-style technical diagram..." \
  --size 2K \
  --aspect-ratio 16:9 \
  --output ~/Downloads/diagram.png
```

#### Options

| Flag | Description | Values |
|------|-------------|--------|
| `--model` | AI model to use | `gemini`, `flux`, `openai` |
| `--prompt` | Image generation prompt | String |
| `--size` | Output resolution | `1K`, `2K`, `4K` (gemini) or aspect ratios |
| `--aspect-ratio` | Aspect ratio | `16:9`, `1:1`, `3:2`, `21:9`, etc. |
| `--output` | Output file path | Path (default: `~/Downloads/pais-image.png`) |
| `--remove-bg` | Remove background | Flag (requires REMOVEBG_API_KEY) |
| `--thumbnail` | Create thumbnail version | Flag |

#### Models

| Model | Provider | API Key Env Var | Notes |
|-------|----------|-----------------|-------|
| `gemini` | Google | `GOOGLE_API_KEY` | Default, best quality |
| `flux` | Replicate | `REPLICATE_API_TOKEN` | Alternative aesthetic |
| `openai` | OpenAI | `OPENAI_API_KEY` | GPT-image-1 |

### Configuration

API keys stored in `~/.config/pais/.env`:

```bash
GOOGLE_API_KEY=...
REPLICATE_API_TOKEN=...
OPENAI_API_KEY=...
REMOVEBG_API_KEY=...
```

Or in `pais.yaml`:

```yaml
image:
  default-model: gemini
  default-output-dir: ~/Downloads
  api-keys:
    google: ${GOOGLE_API_KEY}
    replicate: ${REPLICATE_API_TOKEN}
```

## Skill Design

### SKILL.md

Routes requests to appropriate workflow:
- Technical/architecture diagram → `workflows/technical-diagrams.md`
- Quick flowchart → `workflows/mermaid.md` (direct output)
- Editable diagram → `workflows/excalidraw.md` (direct output)
- Blog header/illustration → `workflows/editorial.md`

### Aesthetic.md

Defines the visual style (borrowed from kai with attribution):
- Dark background: `#0a0a0f`
- PAI Blue: `#4a90d9` (key elements)
- Electric Cyan: `#22d3ee` (flows, connections)
- Excalidraw hand-drawn aesthetic
- Typography tiers (headers, labels, insights)

### Workflows

Each workflow contains:
1. **Purpose** - When to use this workflow
2. **Style Guidelines** - Visual requirements
3. **Prompt Template** - Proven prompt structure
4. **Validation Checklist** - Must-have / must-not-have

## Implementation Phases

### Phase 1: Skill Structure (No Rust)

Create skill that guides Claude:
- `skills/image/SKILL.md` - routing
- `skills/image/Aesthetic.md` - style guide
- `skills/image/workflows/*.md` - detailed workflows

Claude can output Mermaid/Excalidraw immediately. AI image generation documented but not executable until Phase 2.

### Phase 2: `pais image generate` Command

Implement in Rust:
- `src/commands/image.rs` - command implementation
- Gemini API integration (primary)
- Flux API integration (via Replicate)
- OpenAI API integration
- `--remove-bg` via remove.bg API
- `--thumbnail` via ImageMagick shell-out

### Phase 3: Polish

- Config file support for API keys
- Default model/size preferences
- Error messages with helpful context
- Integration tests

## References

- kai/pai Art skill: `~/repos/danielmiessler/Personal_AI_Infrastructure/Packs/kai-art-skill/`
- Fabric patterns: `create_mermaid_visualization`, `create_excalidraw_visualization`
