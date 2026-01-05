# Mermaid Diagram Workflow

**Quick visualization using Mermaid syntax - outputs directly, no CLI required.**

## When to Use

- User wants a quick flowchart, sequence diagram, or state diagram
- Output will be embedded in markdown documentation
- Standard diagram types (flowchart, sequence, ER, class, state, gantt)
- No need for custom styling beyond what Mermaid supports

## Output Format

Output raw Mermaid syntax in a fenced code block:

````markdown
```mermaid
flowchart LR
    A[Start] --> B[Process]
    B --> C{Decision}
    C -->|Yes| D[Action]
    C -->|No| E[End]
```
````

This renders in GitHub, VS Code, Obsidian, and most modern markdown viewers.

## Diagram Types

### Flowchart (Most Common)

```mermaid
flowchart TD
    A[User Request] --> B{Authenticated?}
    B -->|Yes| C[Process Request]
    B -->|No| D[Return 401]
    C --> E[Return Response]
```

**Direction options:** `TD` (top-down), `LR` (left-right), `BT` (bottom-top), `RL` (right-left)

### Sequence Diagram

```mermaid
sequenceDiagram
    participant U as User
    participant A as Auth Service
    participant API as API Gateway

    U->>A: Login Request
    A->>A: Validate Credentials
    A-->>U: JWT Token
    U->>API: Request + JWT
    API->>API: Verify Token
    API-->>U: Response
```

### State Diagram

```mermaid
stateDiagram-v2
    [*] --> Idle
    Idle --> Processing: Start
    Processing --> Success: Complete
    Processing --> Error: Fail
    Success --> [*]
    Error --> Idle: Retry
```

### Entity Relationship

```mermaid
erDiagram
    USER ||--o{ ORDER : places
    ORDER ||--|{ LINE_ITEM : contains
    PRODUCT ||--o{ LINE_ITEM : "ordered in"
```

### Class Diagram

```mermaid
classDiagram
    class Animal {
        +String name
        +int age
        +makeSound()
    }
    class Dog {
        +fetch()
    }
    Animal <|-- Dog
```

## Styling Guidelines

Mermaid has limited styling, but follow these conventions:

1. **Use clear, short labels** - Avoid long text in nodes
2. **Group related nodes** - Use subgraphs for logical grouping
3. **Direction matters** - `LR` for flows, `TD` for hierarchies
4. **Consistent shapes** - `[]` for process, `{}` for decision, `()` for start/end

### Subgraphs for Grouping

```mermaid
flowchart LR
    subgraph Frontend
        A[React App]
        B[Next.js]
    end

    subgraph Backend
        C[API Server]
        D[Database]
    end

    A --> C
    B --> C
    C --> D
```

## Best Practices

1. **Keep it simple** - Mermaid is for quick diagrams, not complex visualizations
2. **Use participant aliases** - `participant U as User` for cleaner diagrams
3. **Add notes when needed** - `Note over A,B: Important info`
4. **Test rendering** - Preview in VS Code or GitHub before sharing

## When NOT to Use Mermaid

Switch to Excalidraw or AI generation when:
- User needs custom styling or colors
- Diagram is too complex for Mermaid's layout
- Hand-drawn aesthetic is important
- Diagram will be used in a presentation (use AI generation)

## Example: Architecture Diagram

```mermaid
flowchart TD
    subgraph Client
        A[Web App]
        B[Mobile App]
    end

    subgraph "API Layer"
        C[API Gateway]
        D[Auth Service]
    end

    subgraph Services
        E[User Service]
        F[Order Service]
        G[Inventory Service]
    end

    subgraph Data
        H[(PostgreSQL)]
        I[(Redis Cache)]
    end

    A --> C
    B --> C
    C --> D
    C --> E
    C --> F
    C --> G
    E --> H
    F --> H
    G --> H
    E --> I
```
