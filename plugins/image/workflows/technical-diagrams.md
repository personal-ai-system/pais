# Technical Diagram Workflow

**AI-generated polished diagrams using `pais image generate`**

## When to Use

- Architecture diagrams for documentation or presentations
- System design visualizations
- Process flows that need professional polish
- Any diagram where hand-drawn Excalidraw aesthetic is desired as a PNG

## Prerequisites

Requires API key configured in `~/.config/pais/.env`:

```bash
GOOGLE_API_KEY=...  # For gemini model (recommended)
```

## Execution Steps

1. **Understand** - Analyze the system/concept to visualize
2. **Structure** - Plan the diagram layout (components, relationships, hierarchy)
3. **Compose** - Design with title, subtitle, and 1-3 key insights
4. **Prompt** - Construct using the template below
5. **Generate** - Execute with `pais image generate`
6. **Validate** - Check against validation criteria

## Prompt Template

```
Clean Excalidraw-style technical diagram on dark background.

BACKGROUND: Pure dark #0a0a0f - NO grid lines, NO texture, completely clean.

STYLE: Hand-drawn Excalidraw aesthetic - like a skilled architect's whiteboard sketch.

TYPOGRAPHY:
- HEADER: Elegant serif italic, large, white color, top-left position
- SUBTITLE: Same serif but regular weight, smaller, gray color, below header
- LABELS: Geometric sans-serif, white, clean and technical
- INSIGHTS: Condensed italic, Primary Blue #4a90d9, used for callouts with asterisks

DIAGRAM CONTENT:
Title: '[TITLE]' (Top left)
Subtitle: '[SUBTITLE]' (Below title)
Components: [LIST THE MAIN COMPONENTS]
Connections: [DESCRIBE THE FLOW/RELATIONSHIPS]

Include 1-3 insight callouts like "*key insight here*" in Primary Blue.

COLOR USAGE:
- White #e5e7eb for all text and primary structure
- Primary Blue #4a90d9 for key components and insights
- Cyan #22d3ee for flow arrows and connections
- Keep 70% of image in white/gray tones, color as accent

EXCALIDRAW CHARACTERISTICS:
- Slightly wobbly hand-drawn lines
- Imperfect rectangles with rounded corners
- Organic arrow curves
- Variable line weight
- Professional but approachable feel
```

## Generate Command

```bash
pais image generate \
  --model gemini \
  --prompt "[YOUR PROMPT]" \
  --size 2K \
  --aspect-ratio 16:9 \
  --output ~/Downloads/diagram.png
```

## Intent-to-Flag Mapping

### Model Selection

| User Says | Flag | When to Use |
|-----------|------|-------------|
| "fast", "quick", "draft" | `--model flux` | Faster iteration |
| (default), "best", "high quality" | `--model gemini` | Best quality |
| "openai", "dall-e" | `--model openai` | Alternative |

### Size Selection

| User Says | Flag | Resolution |
|-----------|------|------------|
| "draft", "preview" | `--size 1K` | Quick iterations |
| (default), "standard" | `--size 2K` | Standard output |
| "high res", "print" | `--size 4K` | Maximum resolution |

### Aspect Ratio

| User Says | Flag | Use Case |
|-----------|------|----------|
| "wide", "slide", "presentation" | `--aspect-ratio 16:9` | Default for diagrams |
| "square" | `--aspect-ratio 1:1` | Social media |
| "ultrawide" | `--aspect-ratio 21:9` | Wide system diagrams |

## Validation

### Must Have

- [ ] Dark background #0a0a0f (NO light backgrounds)
- [ ] Hand-drawn Excalidraw aesthetic
- [ ] Title and subtitle in top-left
- [ ] 1-3 insight callouts in Primary Blue
- [ ] Strategic color usage (70% white/gray, 30% color accents)
- [ ] Readable labels and text

### Must NOT Have

- [ ] Light/white backgrounds
- [ ] Grid lines or textures
- [ ] Perfect vector shapes
- [ ] Cartoony or clip-art style
- [ ] Over-coloring (everything blue)
- [ ] Generic AI illustration look

### If Validation Fails

| Problem | Fix |
|---------|-----|
| Light background | Add "dark background #0a0a0f" more explicitly |
| Too perfect/clean | Add "hand-drawn, slightly wobbly, Excalidraw style" |
| Wrong colors | Specify exact hex codes in prompt |
| No insights | Add "include 1-3 callouts in Primary Blue #4a90d9" |

## Example: Auth Flow Diagram

```bash
pais image generate \
  --model gemini \
  --size 2K \
  --aspect-ratio 16:9 \
  --output ~/Downloads/auth-flow.png \
  --prompt "Clean Excalidraw-style technical diagram on dark background #0a0a0f.

Title: 'OAuth2 Authentication Flow' (top-left, elegant serif, white)
Subtitle: 'Token-based API authorization' (below title, gray)

Components (hand-drawn boxes with Primary Blue #4a90d9 borders):
- Client Application (left)
- Authorization Server (center-top)
- Resource Server (center-bottom)
- User (far left, as a simple figure)

Flow (cyan #22d3ee arrows):
1. User -> Client: Login request
2. Client -> Auth Server: Authorization request
3. Auth Server -> User: Consent prompt
4. User -> Auth Server: Grant consent
5. Auth Server -> Client: Authorization code
6. Client -> Auth Server: Exchange code for token
7. Auth Server -> Client: Access token
8. Client -> Resource Server: API request with token

Insights (condensed italic, Primary Blue):
- '*Tokens expire after 1 hour*' near the token flow
- '*PKCE required for public clients*' near the authorization step

Style: Hand-drawn Excalidraw aesthetic with wobbly lines, imperfect shapes, professional but approachable."
```

## Output Location

```
ALL GENERATED IMAGES GO TO ~/Downloads/ FIRST
Preview before final placement
Only copy to project directories after review
```
