# Data Model: LLM Wiki Agent — Rust CLI Tool

## Core Entities

### WikiPage

Represents a single wiki page (markdown file with YAML frontmatter).

```rust
struct WikiPage {
    /// File path relative to wiki/ directory
    path: String,
    /// Page title from frontmatter
    title: String,
    /// Page type: source, entity, concept, synthesis
    page_type: PageType,
    /// Searchable tags
    tags: Vec<String>,
    /// Source slugs that inform this page
    sources: Vec<String>,
    /// ISO date of last update
    last_updated: String,
    /// Raw markdown content (without frontmatter)
    content: String,
    /// Extracted [[wikilinks]] from content
    wikilinks: Vec<String>,
}

enum PageType {
    Source,
    Entity,
    Concept,
    Synthesis,
}
```

### Source

Tracks an ingested source document.

```rust
struct Source {
    /// Path relative to raw/ directory
    path: String,
    /// SHA256 hash of file content
    sha256: String,
    /// When this source was ingested
    ingested_at: String,
    /// Wiki pages generated from this source
    pages_generated: Vec<String>,
}
```

### IngestCache

Persistent cache mapping source hashes to ingest results.

```rust
struct IngestCache {
    /// Map of source path → cache entry
    entries: HashMap<String, CacheEntry>,
}

struct CacheEntry {
    /// SHA256 of source content at time of ingest
    hash: String,
    /// When ingested
    timestamp: u64,
    /// Wiki files written during this ingest
    files_written: Vec<String>,
}
```

### IngestQueue

Persistent queue for batch ingest with crash recovery.

```rust
struct IngestQueue {
    tasks: Vec<IngestTask>,
}

struct IngestTask {
    /// Unique task identifier
    id: String,
    /// Path to source file
    source_path: String,
    /// Current status
    status: TaskStatus,
    /// Number of retry attempts
    retry_count: u32,
    /// Error message if failed
    error: Option<String>,
}

enum TaskStatus {
    Pending,
    Processing,
    Done,
    Failed,
}
```

### GraphNode / GraphEdge

Knowledge graph structure.

```rust
struct GraphNode {
    /// Wiki page identifier (filename without extension)
    id: String,
    /// Display title
    title: String,
    /// Page type
    node_type: PageType,
    /// Louvain community assignment
    community_id: u32,
    /// Number of inbound + outbound links
    link_count: u32,
}

struct GraphEdge {
    /// Source node id
    source: String,
    /// Target node id
    target: String,
    /// Combined relevance weight
    weight: f64,
    /// Individual signal scores
    signals: RelevanceSignals,
}

struct RelevanceSignals {
    direct_link: f64,     // weight 3.0
    source_overlap: f64,  // weight 4.0
    common_neighbors: f64, // weight 1.5 (Adamic-Adar)
    type_affinity: f64,   // weight 1.0
}
```

### LlmConfig

LLM provider configuration.

```rust
struct LlmConfig {
    /// Active provider name
    provider: String,
    /// Provider-specific settings
    providers: HashMap<String, ProviderConfig>,
}

struct ProviderConfig {
    /// API base URL
    api_url: String,
    /// API key (from env var or config)
    api_key: Option<String>,
    /// Model name
    model: String,
    /// Max tokens for response
    max_tokens: u32,
    /// Request timeout in seconds
    timeout_secs: u64,
}
```

## File Layout (Runtime)

```
project-root/
├── raw/                      # Immutable source documents
│   ├── article.md
│   └── paper.pdf
├── wiki/                     # LLM-maintained wiki pages
│   ├── index.md              # Content catalog
│   ├── log.md                # Chronological operation log
│   ├── overview.md           # Living synthesis
│   ├── sources/              # One summary per source
│   ├── entities/             # People, orgs, projects
│   ├── concepts/             # Ideas, methods, theories
│   └── syntheses/            # Saved query answers
├── .wiki-tool/               # Tool state (gitignored)
│   ├── ingest-cache.json     # SHA256 → ingest mapping
│   ├── ingest-queue.json     # Pending tasks
│   └── search-index/         # Tantivy index files
├── .wiki-tool.toml           # Configuration
├── CLAUDE.md                 # Agent schema (Claude Code)
├── AGENTS.md                 # Agent schema (Codex)
├── GEMINI.md                 # Agent schema (Gemini CLI)
└── COPILOT.md                # Agent schema (Copilot CLI)
```

## Relationships

```
Source (raw/) ──ingests──→ WikiPage (wiki/)
WikiPage ──[[wikilinks]]──→ WikiPage
WikiPage ──sources[]──→ Source
GraphNode ←──maps──→ WikiPage
GraphEdge ←──connects──→ GraphNode × GraphNode
IngestTask ──references──→ Source
CacheEntry ──tracks──→ Source × WikiPage[]
```

## YAML Frontmatter Format

```yaml
---
title: "Page Title"
type: source | entity | concept | synthesis
tags: [tag1, tag2]
sources: [source-slug-1, source-slug-2]
last_updated: 2026-04-16
---
```
