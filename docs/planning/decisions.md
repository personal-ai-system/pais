# Architecture Decision Records

> Documenting the reasoning behind key architectural decisions.

---

## ADR-001: Rust for CLI, Python for Plugins

### Status
Accepted

### Context
We need to choose languages for PAIS components. Options:
1. All TypeScript (like Kai/PAI)
2. All Python
3. All Rust
4. Hybrid: Rust CLI + Python plugins

### Decision
**Hybrid approach: Rust for CLI and hooks, Python for most plugins.**

### Rationale

**Rust for CLI and hooks:**
- Hooks must never crash (Rust's safety guarantees)
- Hooks must be fast (sub-millisecond)
- Single binary distribution (easy for teammates to install)
- User has extensive Rust CLI experience
- Claude Code hook integration is just stdin/stdout

**Python for plugins:**
- Rich SDK ecosystem (Jira, Slack, PagerDuty, etc.)
- Rapid iteration during development
- Familiar to most teams
- AI models work well with Python code

### Consequences
- Need to handle Rust → Python plugin invocation
- Two build systems (Cargo + pip/uv)
- Slightly more complex development setup
- Benefits outweigh complexity for this use case

---

## ADR-002: File-Based Memory over RAG

### Status
Accepted

### Context
We need persistent memory/context storage. Options:
1. Vector database + RAG (semantic search)
2. File-based storage + grep/ripgrep (exact + pattern search)
3. SQLite database
4. Hybrid approach

### Decision
**File-based storage with grep/ripgrep for search.**

### Rationale

**Simplicity:**
- No database to manage
- Backup is just `git commit`
- Standard UNIX tools work

**Performance:**
- Grep is faster than embedding + similarity for most queries
- No embedding model overhead
- Scales well with ripgrep

**Debuggability:**
- Can read/edit files directly with any editor
- `cat`, `head`, `tail` just work
- Git history shows changes

**Developer-friendly:**
- Familiar tools (grep, rg, fd)
- Works offline
- No external dependencies

### Consequences
- Semantic search requires explicit implementation
- Large history may need periodic archival
- No cross-entry relationships (like a graph DB)

### Alternatives Rejected
- **RAG with Chroma/Pinecone**: Overkill, adds infrastructure
- **SQLite**: More structured than needed, less grep-friendly
- **Graph DB**: Unnecessary complexity for our use case

---

## ADR-003: Plugin Architecture over Monorepo Packs

### Status
Accepted

### Context
Daniel's Kai/PAI uses "packs" — coupled components installed in strict order. We need to decide on our distribution model.

### Decision
**True plugin architecture with contracts, not packs.**

### Rationale

**Problems with packs:**
- Strict installation order required
- Removal may break dependents
- Difficult to share subsets
- Global state pollution
- Extraction pain (Kai → PAI struggles)

**Benefits of plugins:**
- Install in any order
- Clean removal, no side effects
- Plugin-local configuration
- Contracts instead of direct dependencies
- Easy for teammates to pick and choose

### Consequences
- More upfront design work for contract system
- Runtime contract resolution complexity
- Must handle graceful degradation for optional contracts
- Plugin authors need to understand contract concept

### Trade-off Acknowledged
Tight pack integration enables "emergent capabilities" — e.g., voice automatically reading history captures. With plugins, such integrations must be explicit. We accept this trade-off for modularity.

---

## ADR-004: Contract-Based Plugin Interfaces

### Status
Accepted

### Context
Plugins need to communicate. Options:
1. Direct dependencies (Plugin A imports Plugin B)
2. Event bus only (pub/sub)
3. Contract-based (plugins provide/consume interfaces)
4. Hybrid

### Decision
**Contract-based interfaces with optional event bus.**

### Rationale

**Contracts provide:**
- Decoupling: Plugin A doesn't know about Plugin B
- Type safety: Interfaces are well-defined
- Graceful degradation: Optional contracts can be missing
- Discoverability: Core knows what's available

**Better than direct dependencies:**
- No import cycles
- Easy to swap implementations
- Testing with mocks is natural

**Better than pure event bus:**
- Request/response patterns are common
- Type safety on interfaces
- Easier to reason about data flow

### Consequences
- Contract definitions must be maintained
- Versioning becomes important
- Slight runtime overhead for resolution

---

## ADR-005: Hybrid Repository Strategy

### Status
Accepted

### Context
How should PAIS and plugins be distributed? Options:
1. Everything in one monorepo
2. Everything in separate repos
3. Core + blessed plugins in monorepo, others in separate repos

### Decision
**Hybrid: Core + foundation plugins in monorepo, external plugins in separate repos.**

### Rationale

**Monorepo for core:**
- Atomic commits across core + foundation
- Easier to refactor interfaces
- Single place for "official" plugins

**Separate repos for others:**
- Team plugins can be private
- Community can contribute without core access
- Independent versioning
- Smaller clone for most users

### Consequences
- Need plugin registry for discovery
- Update coordination between core and external
- Clear documentation of version compatibility

### Repository Structure

```
github.com/scottidler/pais/              # Core + foundation
github.com/your-company/pais-work/       # Team private plugins
github.com/scottidler/pais-personal/     # Personal plugins
github.com/someone/pais-plugin-foo/      # Community plugin
```

---

## ADR-006: Claude Code as Primary Backend

### Status
Accepted

### Context
Which LLM/agent system to build on? Options:
1. Claude Code
2. Cursor
3. Gemini CLI
4. Custom implementation
5. Backend-agnostic design

### Decision
**Claude Code as primary, with backend-agnostic principles.**

### Rationale

**Claude Code advantages:**
- Best scaffolding for AI agents (per Daniel's assessment)
- Native hook system
- Native skill system
- Native subagent support
- Native plugin/marketplace support
- MCP integration

**Backend-agnostic principles:**
- Hook handlers are just executables (any language)
- Skills are markdown files (portable)
- Plugin logic is standard Python/Rust (reusable)

### Consequences
- Tied to Anthropic for primary use case
- Some features may not port (native hooks, skills)
- Core logic remains portable

### Future Consideration
If needed, adapt to other backends by:
- Implementing equivalent hook dispatcher
- Mapping skill format to target system
- Using MCP for tool integration

---

## ADR-007: No TypeScript

### Status
Accepted

### Context
TypeScript is the primary language in Kai/PAI. Should we use it?

### Decision
**No TypeScript. Python and Rust only.**

### Rationale

**User preference:**
- Explicit request to avoid TypeScript
- Extensive Rust CLI experience
- Python is widely used

**Practical reasons:**
- No Bun/Node.js runtime dependency
- Rust produces static binaries
- Python is more widely installed
- CLI tooling is typically Python, Go, or Rust

### Consequences
- Cannot directly reuse Kai/PAI code
- Must reimplement hooks, skills, etc.
- Different ecosystem (pip vs npm)
- Acceptable: Clean-room implementation intended anyway

---

## ADR-008: CLI-First Interface

### Status
Accepted

### Context
How do users interact with PAIS? Options:
1. CLI only
2. GUI/TUI
3. API/SDK
4. All of the above

### Decision
**CLI-first, with SDK as secondary interface.**

### Rationale

**CLI advantages:**
- Scriptable and automatable
- Works in SSH sessions
- Composable with UNIX tools
- Familiar to engineers
- Easy to document and test

**SDK secondary:**
- Needed for Claude Code integration
- Enables programmatic access
- Can be built on top of CLI

### Consequences
- No GUI planned initially
- All features must be CLI-accessible
- Good error messages and help text essential

---

## ADR-009: Configuration Hierarchy

### Status
Accepted

### Context
How is configuration managed across core, plugins, and users?

### Decision
**Three-level configuration hierarchy:**
1. Plugin defaults (in plugin.toml)
2. User overrides (in plugin config.toml)
3. Environment variables (for secrets)

### Rationale

**Separation of concerns:**
- Plugins ship with sensible defaults
- Users customize without editing plugin files
- Secrets never in config files

**No global pollution:**
- Each plugin has its own config namespace
- Changes to one plugin don't affect others

### Consequences
- Config discovery is straightforward
- Clear precedence rules
- Secrets require `.env` management

### Configuration Locations

```
~/.config/pais/
├── pais.toml                    # Global PAIS config
├── .env                         # Secrets (all plugins)
└── plugins/
    └── jira/
        └── config.toml          # User overrides for jira plugin
```

---

## ADR-010: Hook Exit Codes

### Status
Accepted

### Context
How do hooks communicate results to Claude Code?

### Decision
**Match Claude Code's exit code convention:**
- 0 = Allow / Success
- 2 = Block (for PreToolUse)
- 1 = Error (log but don't block)

### Rationale

**Compatibility:**
- Claude Code expects these exit codes
- No need for custom communication protocol
- Stdout used for messages

**Simplicity:**
- Exit codes are universal
- Easy to implement in any language
- Easy to test

### Consequences
- Limited signal vocabulary (only 3 states)
- Complex results require stdout JSON
- Must never exit with unexpected codes

---

## Template for New ADRs

```markdown
## ADR-XXX: [Title]

### Status
[Proposed | Accepted | Deprecated | Superseded by ADR-YYY]

### Context
[What is the issue we're addressing?]

### Decision
[What did we decide?]

### Rationale
[Why did we make this decision?]

### Consequences
[What are the implications, both positive and negative?]

### Alternatives Rejected
[Optional: What other options did we consider and why not?]
```

---

## Related Documents

- [architecture.md](architecture.md) — System design
- [vision.md](vision.md) — Philosophy and goals
- [contracts.md](contracts.md) — Interface specifications

