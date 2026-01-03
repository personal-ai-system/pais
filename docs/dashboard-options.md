# PAIS Dashboard Options

> Deferred from Phase 4.2. Two Rust-native approaches for event visualization.

## Option A: Web Dashboard (Axum)

A browser-based dashboard served from the `pais` binary.

### Command

```bash
pais dashboard              # http://localhost:4000
pais dashboard --port 8080  # custom port
```

### Architecture

```
┌─────────────────────────────────────────────────────┐
│  Browser (localhost:4000)                           │
│  ┌───────────────────────────────────────────────┐  │
│  │  Live Event Stream (SSE)                      │  │
│  │  ┌─────────────────────────────────────────┐  │  │
│  │  │ 12:00:01 SessionStart [abc123]          │  │  │
│  │  │ 12:00:02 PreToolUse   [abc123] Bash     │  │  │
│  │  │ 12:00:03 PostToolUse  [abc123] Bash     │  │  │
│  │  └─────────────────────────────────────────┘  │  │
│  │                                               │  │
│  │  Stats: 42 events today | 3 blocked          │  │
│  └───────────────────────────────────────────────┘  │
└─────────────────────────────────────────────────────┘
         │
         │ SSE: /events/stream
         │ API: /api/events, /api/stats
         ▼
┌─────────────────────────────────────────────────────┐
│  pais dashboard (Axum server)                       │
│                                                     │
│  - Serves embedded HTML/CSS/JS                      │
│  - Tails JSONL files from history/raw-events/       │
│  - Pushes events via Server-Sent Events (SSE)       │
└─────────────────────────────────────────────────────┘
```

### Dependencies

```toml
[dependencies]
axum = "0.8"
tokio = { version = "1", features = ["full"] }
tower-http = { version = "0.6", features = ["fs", "cors"] }
```

### Implementation Sketch

```rust
// src/commands/dashboard.rs

use axum::{
    Router,
    routing::get,
    response::sse::{Event, Sse},
};
use std::convert::Infallible;
use tokio_stream::StreamExt;

async fn events_stream() -> Sse<impl Stream<Item = Result<Event, Infallible>>> {
    // Tail JSONL files and emit SSE events
    let stream = tail_events().map(|event| {
        Ok(Event::default().json_data(event).unwrap())
    });
    Sse::new(stream)
}

pub async fn serve(port: u16) -> Result<()> {
    let app = Router::new()
        .route("/", get(|| async { include_str!("../../assets/index.html") }))
        .route("/events/stream", get(events_stream))
        .route("/api/events", get(list_events))
        .route("/api/stats", get(get_stats));

    let listener = tokio::net::TcpListener::bind(format!("0.0.0.0:{}", port)).await?;
    axum::serve(listener, app).await?;
    Ok(())
}
```

### Pros

- Accessible from any device on network
- Rich UI possibilities (charts, tables, filters)
- Can be left running in a browser tab
- Shareable URL for remote debugging

### Cons

- Requires browser
- Adds ~500KB to binary (axum + tokio)
- More complex async code
- Need to design HTML/CSS UI

---

## Option B: Terminal UI (Ratatui)

A full-screen terminal dashboard.

### Command

```bash
pais tui                    # full-screen dashboard
pais tui --tab events       # start on events tab
```

### Architecture

```
┌─────────────────────────────────────────────────────┐
│  PAIS Dashboard                      [q]uit [?]help │
├─────────────────────────────────────────────────────┤
│  Events │ Stats │ Security │ History               │
├─────────────────────────────────────────────────────┤
│                                                     │
│  12:00:01 SessionStart [abc123]                     │
│  12:00:02 PreToolUse   [abc123] Bash               │
│  12:00:03 PostToolUse  [abc123] Bash               │
│  12:00:05 PreToolUse   [abc123] Read               │
│  ...                                                │
│                                                     │
├─────────────────────────────────────────────────────┤
│  Today: 42 events | Blocked: 3 | Sessions: 2       │
└─────────────────────────────────────────────────────┘
```

### Dependencies

```toml
[dependencies]
ratatui = "0.29"
crossterm = "0.28"
```

### Implementation Sketch

```rust
// src/commands/tui.rs

use ratatui::{
    prelude::*,
    widgets::{Block, Borders, List, ListItem, Tabs},
};
use crossterm::event::{self, Event, KeyCode};

struct App {
    events: Vec<ObservabilityEvent>,
    selected_tab: usize,
}

fn ui(frame: &mut Frame, app: &App) {
    let tabs = Tabs::new(vec!["Events", "Stats", "Security", "History"])
        .select(app.selected_tab);

    let events: Vec<ListItem> = app.events.iter()
        .map(|e| ListItem::new(e.format_display()))
        .collect();

    let list = List::new(events)
        .block(Block::default().borders(Borders::ALL));

    frame.render_widget(tabs, chunks[0]);
    frame.render_widget(list, chunks[1]);
}

pub fn run() -> Result<()> {
    let mut terminal = ratatui::init();
    let mut app = App::new();

    loop {
        terminal.draw(|f| ui(f, &app))?;

        if event::poll(Duration::from_millis(100))? {
            if let Event::Key(key) = event::read()? {
                match key.code {
                    KeyCode::Char('q') => break,
                    KeyCode::Tab => app.next_tab(),
                    _ => {}
                }
            }
        }

        app.poll_new_events();
    }

    ratatui::restore();
    Ok(())
}
```

### Pros

- Native terminal experience (matches `pais observe`)
- Lighter weight (~200KB vs ~500KB)
- Works over SSH
- Keyboard-driven, fast navigation
- No context switch to browser

### Cons

- Limited to terminal capabilities
- No charts/graphs (text-based only)
- Can't share with non-terminal users
- Blocks the terminal while running

---

## Comparison

| Aspect | Web (Axum) | TUI (Ratatui) |
|--------|------------|---------------|
| Binary size impact | ~500KB | ~200KB |
| Requires browser | Yes | No |
| Works over SSH | Partially (port forward) | Yes |
| Rich visualizations | Yes (charts, CSS) | Limited (text art) |
| Complexity | Higher (async, HTML) | Lower (sync, widgets) |
| Multi-device access | Yes | No |
| Keyboard-driven | Partial | Full |

## Recommendation

**For personal use:** TUI (Ratatui)
- Stays in the terminal where you already work
- Simpler implementation
- Lighter binary

**For team/remote use:** Web (Axum)
- Can be accessed from anywhere
- Richer UI for dashboards
- Better for long-running monitoring

**Current stance:** Neither is urgent. The existing CLI tools (`pais observe`, `pais history stats`) cover the core functionality. Build one of these when the need becomes clear.

---

## Decision

**Status:** Deferred

When ready to implement, choose based on primary use case:
- Solo developer, terminal-centric → TUI
- Team visibility, remote access → Web

