# Excalidraw Diagram Workflow

**Editable hand-drawn diagrams - outputs JSON that imports into excalidraw.com**

## When to Use

- User wants an editable, hand-drawn style diagram
- Diagram needs manual adjustment after generation
- Collaboration or sharing is needed
- Complex layouts that Mermaid can't express well
- User specifically requests "excalidraw" or "hand-drawn"

## Output Format

Output Excalidraw JSON schema that can be imported at excalidraw.com:

```json
{
  "type": "excalidraw",
  "version": 2,
  "source": "pais",
  "elements": [...],
  "appState": {
    "viewBackgroundColor": "#0a0a0f"
  }
}
```

Save to a `.excalidraw` file or copy JSON directly into excalidraw.com.

## Element Types

### Rectangle

```json
{
  "id": "rect1",
  "type": "rectangle",
  "x": 100,
  "y": 100,
  "width": 200,
  "height": 100,
  "strokeColor": "#4a90d9",
  "backgroundColor": "#1a1a2e",
  "fillStyle": "solid",
  "strokeWidth": 2,
  "roughness": 1,
  "roundness": { "type": 3 }
}
```

### Text

```json
{
  "id": "text1",
  "type": "text",
  "x": 150,
  "y": 140,
  "text": "API Gateway",
  "fontSize": 20,
  "fontFamily": 1,
  "textAlign": "center",
  "strokeColor": "#e5e7eb"
}
```

### Arrow

```json
{
  "id": "arrow1",
  "type": "arrow",
  "x": 300,
  "y": 150,
  "width": 100,
  "height": 0,
  "strokeColor": "#22d3ee",
  "strokeWidth": 2,
  "roughness": 1,
  "startBinding": { "elementId": "rect1", "focus": 0, "gap": 5 },
  "endBinding": { "elementId": "rect2", "focus": 0, "gap": 5 }
}
```

### Ellipse

```json
{
  "id": "ellipse1",
  "type": "ellipse",
  "x": 100,
  "y": 100,
  "width": 100,
  "height": 100,
  "strokeColor": "#4a90d9",
  "backgroundColor": "transparent",
  "strokeWidth": 2,
  "roughness": 1
}
```

## Color Palette (Dark Mode)

Use colors from `Aesthetic.md`:

| Element | Color |
|---------|-------|
| Background | `#0a0a0f` |
| Primary stroke | `#4a90d9` |
| Flow arrows | `#22d3ee` |
| Text | `#e5e7eb` |
| Surface fill | `#1a1a2e` |
| Borders | `#94a3b8` |

## Roughness Values

| Value | Effect |
|-------|--------|
| 0 | Clean lines (less hand-drawn) |
| 1 | Slight wobble (recommended) |
| 2 | More hand-drawn feel |

## Complete Example

Simple architecture diagram:

```json
{
  "type": "excalidraw",
  "version": 2,
  "source": "pais",
  "elements": [
    {
      "id": "client",
      "type": "rectangle",
      "x": 50,
      "y": 100,
      "width": 120,
      "height": 60,
      "strokeColor": "#4a90d9",
      "backgroundColor": "#1a1a2e",
      "fillStyle": "solid",
      "strokeWidth": 2,
      "roughness": 1,
      "roundness": { "type": 3 }
    },
    {
      "id": "client-label",
      "type": "text",
      "x": 75,
      "y": 120,
      "text": "Client",
      "fontSize": 18,
      "fontFamily": 1,
      "strokeColor": "#e5e7eb"
    },
    {
      "id": "api",
      "type": "rectangle",
      "x": 250,
      "y": 100,
      "width": 120,
      "height": 60,
      "strokeColor": "#4a90d9",
      "backgroundColor": "#1a1a2e",
      "fillStyle": "solid",
      "strokeWidth": 2,
      "roughness": 1,
      "roundness": { "type": 3 }
    },
    {
      "id": "api-label",
      "type": "text",
      "x": 265,
      "y": 120,
      "text": "API Server",
      "fontSize": 18,
      "fontFamily": 1,
      "strokeColor": "#e5e7eb"
    },
    {
      "id": "arrow1",
      "type": "arrow",
      "x": 170,
      "y": 130,
      "width": 80,
      "height": 0,
      "strokeColor": "#22d3ee",
      "strokeWidth": 2,
      "roughness": 1,
      "points": [[0, 0], [80, 0]]
    }
  ],
  "appState": {
    "viewBackgroundColor": "#0a0a0f",
    "gridSize": null
  }
}
```

## Usage Instructions

1. Generate the Excalidraw JSON
2. Save to a `.excalidraw` file, OR
3. Go to excalidraw.com
4. Click menu (hamburger) > Open
5. Paste or upload the JSON
6. Edit as needed
7. Export as PNG when done

## When NOT to Use Excalidraw

Switch to other methods when:
- **Quick diagram needed** - Use Mermaid instead
- **Polished, presentation-ready image** - Use `pais image generate`
- **User doesn't want to edit** - Use Mermaid or AI generation
- **Very complex diagram** - AI generation may be faster

## Tips

1. **Grid alignment** - Keep x/y coordinates on multiples of 20 for alignment
2. **Consistent spacing** - Use 100-150px between connected elements
3. **Text positioning** - Center text by offsetting x by ~25% of box width
4. **Arrow binding** - Use `startBinding`/`endBinding` for connected arrows
5. **Group related elements** - Use similar y-coordinates for horizontal flow
