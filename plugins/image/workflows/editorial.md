# Editorial Illustration Workflow

**AI-generated blog headers, illustrations, and editorial images**

## When to Use

- Blog post header images
- Newsletter illustrations
- Presentation title slides
- Social media graphics
- Any editorial/creative visual content

## Prerequisites

Requires API key in `~/.config/pais/.env`:

```bash
GOOGLE_API_KEY=...  # For gemini model (recommended)
```

## Execution Steps

1. **Understand** - Read the content/topic to illustrate
2. **Concept** - Identify the core visual metaphor
3. **Compose** - Design the illustration with subject and mood
4. **Prompt** - Construct using the template below
5. **Generate** - Execute with `pais image generate`
6. **Validate** - Check against criteria

## Prompt Template

```
Hand-drawn Excalidraw-style editorial illustration on dark background.

BACKGROUND: Pure dark #0a0a0f - clean, no texture.

STYLE: Sketch-like editorial illustration - think New Yorker meets tech whiteboard.

SUBJECT: [MAIN VISUAL ELEMENT - e.g., "a robot hand reaching toward a human hand"]

MOOD: [EMOTIONAL REGISTER - e.g., "discovery", "technical", "whimsical"]

COMPOSITION:
- Central focus on subject
- Breathing space around edges
- Title area preserved at [top/bottom] for text overlay

COLOR USAGE:
- Primary structure in white #e5e7eb
- Key elements in Primary Blue #4a90d9
- Accent details in Cyan #22d3ee
- Sparse use of Accent Purple #8b5cf6 for highlights

EXCALIDRAW CHARACTERISTICS:
- Hand-drawn, slightly imperfect lines
- Organic shapes, not perfect vectors
- Variable line weight for depth
- Professional but approachable
```

## Generate Command

```bash
pais image generate \
  --model gemini \
  --prompt "[YOUR PROMPT]" \
  --size 2K \
  --aspect-ratio 16:9 \
  --output ~/Downloads/header.png
```

### With Thumbnail

For blog headers that need both full-size and thumbnail:

```bash
pais image generate \
  --model gemini \
  --prompt "[YOUR PROMPT]" \
  --size 2K \
  --aspect-ratio 1:1 \
  --thumbnail \
  --output ~/Downloads/header.png
```

Creates both:
- `header.png` (transparent background)
- `header-thumb.png` (dark background `#0a0a0f`)

## Aspect Ratios by Use Case

| Use Case | Aspect Ratio |
|----------|--------------|
| Blog header (wide) | `16:9` |
| Blog header (standard) | `3:2` |
| Social media (square) | `1:1` |
| Twitter/LinkedIn header | `3:1` or `21:9` |
| Newsletter | `16:9` or `3:2` |

## Visual Metaphors by Topic

| Topic | Visual Metaphor |
|-------|-----------------|
| AI/ML | Neural networks, robot figures, data streams |
| Security | Locks, shields, walls, keys |
| Performance | Speedometers, rockets, lightning |
| Data | Charts, databases, flowing streams |
| Collaboration | Puzzle pieces, hands, bridges |
| Growth | Plants, stairs, upward arrows |
| Automation | Gears, conveyor belts, robots |
| Integration | Puzzle pieces, bridges, connectors |

## Example: AI Blog Header

```bash
pais image generate \
  --model gemini \
  --size 2K \
  --aspect-ratio 16:9 \
  --output ~/Downloads/ai-agents-header.png \
  --prompt "Hand-drawn Excalidraw-style editorial illustration on dark background #0a0a0f.

SUBJECT: A friendly robot figure (sketched, not photorealistic) sitting at a whiteboard, drawing flowcharts. Small floating icons around it representing tasks being completed.

MOOD: Technical but approachable, sense of capability and helpfulness.

COMPOSITION:
- Robot centered, slightly left of center
- Whiteboard to the right with simple diagrams
- Floating task icons scattered in upper right
- Clear space at bottom for title text overlay

COLOR:
- Robot outline in white #e5e7eb with Primary Blue #4a90d9 accents
- Whiteboard content in Cyan #22d3ee
- Floating icons in mixed Primary Blue and Purple #8b5cf6
- Keep overall 70% white/gray, 30% color

STYLE: Hand-drawn, wobbly lines, Excalidraw aesthetic. Professional editorial illustration feel, not cartoonish."
```

## Validation

### Must Have

- [ ] Dark background #0a0a0f
- [ ] Hand-drawn Excalidraw aesthetic
- [ ] Clear focal point
- [ ] Space for text overlay (if needed)
- [ ] On-brand color usage
- [ ] Professional quality

### Must NOT Have

- [ ] Light backgrounds
- [ ] Photorealistic elements
- [ ] Generic stock illustration style
- [ ] Cluttered composition
- [ ] Off-brand colors (neon, pastels)
- [ ] Perfect vector shapes

## Output Location

```
ALL GENERATED IMAGES GO TO ~/Downloads/ FIRST
Preview before placing in project
Rename to descriptive filename before use
```
