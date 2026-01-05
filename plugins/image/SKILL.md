---
name: image
description: Generate images and diagrams with consistent dark-mode aesthetic. Use when creating technical diagrams, visualizations, blog headers, or any visual content.
---

# Image Skill

Visual content generation with dark-mode Excalidraw aesthetic. Routes to the appropriate output method based on request.

## Output Methods

| Request Type | Output | Method |
|--------------|--------|--------|
| Quick flowchart, sequence diagram | Mermaid syntax | Direct output |
| Editable diagram | Excalidraw JSON | Direct output |
| Polished presentation diagram | PNG file | `pais image generate` |
| Blog header, illustration | PNG file | `pais image generate` |

## Workflow Routing

Route to the appropriate workflow based on request:

| User Intent | Workflow |
|-------------|----------|
| "mermaid diagram", "flowchart", "sequence diagram" | `workflows/mermaid.md` |
| "excalidraw", "editable diagram", "hand-drawn" | `workflows/excalidraw.md` |
| "architecture diagram", "system diagram", "technical diagram" | `workflows/technical-diagrams.md` |
| "blog header", "illustration", "editorial image" | `workflows/editorial.md` |

## Quick Reference

### Mermaid (Direct Output)

Output mermaid syntax directly - renders in GitHub, VS Code, Obsidian:

```mermaid
flowchart LR
    A[User] --> B[Auth Service]
    B --> C[API Gateway]
    C --> D[Backend]
```

### Excalidraw (Direct Output)

Output Excalidraw JSON - import at excalidraw.com:

```json
{
  "type": "excalidraw",
  "version": 2,
  "elements": [...]
}
```

### AI-Generated Images

Use `pais image generate` for polished PNG output:

```bash
pais image generate \
  --model gemini \
  --prompt "Clean Excalidraw-style technical diagram..." \
  --size 2K \
  --aspect-ratio 16:9 \
  --output ~/Downloads/diagram.png
```

## Aesthetic Summary

All generated images follow the dark-mode Excalidraw aesthetic:

| Element | Value | Usage |
|---------|-------|-------|
| Background | `#0a0a0f` | Dark background (mandatory) |
| Primary Blue | `#4a90d9` | Key elements, structures |
| Cyan | `#22d3ee` | Flows, connections |
| Text | `#e5e7eb` | Labels, headers |
| Lines | `#94a3b8` | Hand-drawn borders |

**Full aesthetic documentation:** `Aesthetic.md`

## When to Use Each Method

### Use Mermaid When:
- User needs quick visualization
- Diagram will be embedded in markdown/docs
- Interactivity not required
- Standard diagram types (flowchart, sequence, ER, class)

### Use Excalidraw When:
- User wants to edit/modify the diagram
- Hand-drawn aesthetic preferred
- Complex layouts that need manual adjustment
- Collaboration/sharing needed

### Use AI Generation When:
- Polished, presentation-ready output needed
- Custom illustration style required
- Blog headers or editorial images
- Complex visual that can't be expressed in Mermaid

## Command Reference

```bash
# AI-generated image (requires API keys)
pais image generate --model gemini --prompt "..." --output ~/Downloads/out.png

# Options
--model <model>       # gemini (default), flux, openai
--size <size>         # 1K, 2K, 4K
--aspect-ratio <ar>   # 16:9, 1:1, 3:2, 21:9, etc.
--output <path>       # Output file path
--remove-bg           # Remove background (requires REMOVEBG_API_KEY)
--thumbnail           # Create thumbnail version with dark background
```

## API Keys

Configure in `~/.config/pais/.env`:

```bash
GOOGLE_API_KEY=...        # For gemini model
REPLICATE_API_TOKEN=...   # For flux model
OPENAI_API_KEY=...        # For openai model
REMOVEBG_API_KEY=...      # For --remove-bg
```
