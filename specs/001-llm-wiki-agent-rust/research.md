# Research: LLM Wiki Agent — Rust CLI Tool

## Architecture Decision: Dual-Mode Operation

**Decision**: Support both agent-companion and standalone modes.

**Agent-companion mode** (default, no API key):
The coding agent (Claude Code, Copilot CLI, etc.) IS the LLM. It reads
schema files, does all reasoning/generation, and calls wiki-tool for
deterministic operations only: search, graph, lint, cache, extract, index.

**Standalone mode** (requires API key):
wiki-tool makes its own LLM API calls for ingest and query. For CI
pipelines, batch scripts, or environments without a coding agent.

**Rationale**: In local development, the user already has a coding agent
running — having wiki-tool make separate LLM API calls doubles token
usage for no benefit. But CI pipelines and automation scripts don't have
an interactive coding agent, so standalone mode is needed there.

**Alternatives considered**:
- Agent-only: Would block CI/automation use cases.
- Standalone-only: Wastes tokens when a coding agent is already present.

## Technology Decisions

### 1. Markdown Parser: comrak

**Decision**: Use `comrak` for CommonMark + GitHub Flavored Markdown parsing.
**Rationale**: Most mature Rust markdown parser; supports YAML frontmatter
extraction via raw AST access; handles wikilink-style syntax with extensions.
**Alternatives considered**:
- `pulldown-cmark`: Faster but lacks GFM extensions and frontmatter support.
- `markdown-rs`: Newer, less battle-tested.

### 2. Full-Text Search: tantivy

**Decision**: Use `tantivy` for BM25-based full-text search.
**Rationale**: Pure Rust, Lucene-level performance, supports custom tokenizers
(needed for CJK bigram tokenization), schema-based indexing.
**Alternatives considered**:
- Custom grep-based search: Too slow at scale (>100 pages).
- `meilisearch` embedded: Not embeddable, requires separate server.

### 3. Knowledge Graph: petgraph

**Decision**: Use `petgraph` for in-memory graph operations.
**Rationale**: Mature Rust graph library, supports directed/undirected graphs,
has algorithms for connected components (basis for community detection).
**Alternatives considered**:
- Custom adjacency list: Reinventing the wheel.
- `graph` crate: Less maintained.

### 4. LLM Client: reqwest + tokio + SSE parsing

**Decision**: Custom streaming client using `reqwest` + `tokio` with
manual SSE line parsing.
**Rationale**: All major LLM APIs use Server-Sent Events (SSE) for streaming.
A thin abstraction over reqwest avoids heavy dependencies while supporting
OpenAI, Anthropic, Google, and Ollama APIs.
**Alternatives considered**:
- `async-openai`: OpenAI-only; doesn't support Anthropic/Google natively.
- `litellm` (Python): Wrong language.
- `eventsource-client`: Adds dependency for simple line parsing.

### 5. PDF Extraction: pdf-extract

**Decision**: Use `pdf-extract` crate for basic PDF text extraction.
**Rationale**: Pure Rust, no system dependencies (unlike poppler bindings).
Handles most text-based PDFs. For scanned PDFs, users can pre-process
with OCR tools.
**Alternatives considered**:
- `lopdf` + manual text extraction: More control but much more code.
- `pdfium-render`: Requires pdfium binary (platform-specific).

### 6. Community Detection: Custom Louvain

**Decision**: Implement Louvain algorithm on top of `petgraph`.
**Rationale**: No existing Rust crate for Louvain on petgraph. The algorithm
is straightforward (~200 lines) and well-documented. The desktop app's
JavaScript implementation (via graphology) provides a reference.
**Alternatives considered**:
- Skip community detection for MVP: Loses valuable feature.
- Use label propagation instead: Less stable results.

### 7. Configuration: TOML config file

**Decision**: Use a `.wiki-tool.toml` config file for LLM provider settings,
cache paths, and tool preferences.
**Rationale**: TOML is the Rust ecosystem standard (Cargo.toml), human-readable,
and well-supported via `toml` crate.
**Alternatives considered**:
- YAML: More common in agent ecosystems but Rust TOML support is better.
- Environment variables only: Insufficient for multi-provider config.
- JSON: No comments, poor human editing experience.

### 8. Relevance Scoring: 4-Signal Model

**Decision**: Adopt the desktop app's 4-signal relevance model:
- Direct links (weight 3.0)
- Source overlap (weight 4.0)
- Common neighbors / Adamic-Adar (weight 1.5)
- Type affinity (weight 1.0)

**Rationale**: Well-tested in the desktop app; captures implicit
relationships beyond wikilinks.
**Alternatives considered**:
- Wikilinks-only (agent skill approach): Misses implicit connections.
- LLM-inferred edges: Too expensive for CLI tool.

## Architecture Overview

```
wiki-tool ingest <file>
    │
    ├─ Cache check (SHA256)
    │
    ├─ Pass 1: Analysis (streaming)
    │   └─ LLM extracts entities, concepts, contradictions
    │
    ├─ Pass 2: Generation (streaming)
    │   └─ LLM produces wiki page content in ---FILE: blocks
    │
    ├─ Write pages + update index + append log
    │
    ├─ Update search index (tantivy)
    │
    └─ Report: pages created/updated, review items

wiki-tool search <query>
    │
    ├─ Tantivy BM25 search
    ├─ CJK bigram tokenization
    └─ Return ranked results with snippets

wiki-tool query <question>
    │
    ├─ Search for relevant pages
    ├─ Build context from top results + graph neighbors
    ├─ LLM synthesizes answer with citations
    └─ Optionally save as synthesis page

wiki-tool lint
    │
    ├─ Parse all wiki pages
    ├─ Check: orphans, broken links, missing pages
    ├─ Check: contradictions (frontmatter-based)
    └─ Report issues with file:line references

wiki-tool graph
    │
    ├─ Parse wikilinks from all pages
    ├─ Build petgraph
    ├─ Compute relevance scores
    ├─ Run Louvain community detection
    └─ Output graph.json
```
