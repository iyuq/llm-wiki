# Implementation Plan: LLM Wiki Agent — Rust CLI Tool

**Branch**: `001-llm-wiki-agent-rust` | **Date**: 2026-04-16 | **Spec**: [spec.md](spec.md)
**Input**: Feature specification from `/specs/001-llm-wiki-agent-rust/spec.md`

## Summary

Build a high-performance Rust CLI tool (`wiki-tool`) that implements the
LLM Wiki pattern — a utility for incrementally building and maintaining
a personal knowledge base from source documents. The tool operates in
two modes:

1. **Agent-companion mode** (default, no API key): The coding agent
   (Claude Code, Copilot CLI, Codex, Gemini CLI) IS the LLM. It reads
   the schema file, does all reasoning/generation, and shells out to
   `wiki-tool` for deterministic compute-heavy operations: search,
   graph building, lint, cache checks, document extraction, and index
   management.

2. **Standalone mode** (with API key, for CI/automation): `wiki-tool`
   makes its own LLM API calls for ingest and query. Useful in CI
   pipelines, batch scripts, or environments without a coding agent.

The tool combines the best ideas from both reference implementations
and targets all major coding agents.

## Technical Context

**Language/Version**: Rust 1.75+ (2021 edition)
**Primary Dependencies**:
- `clap` — CLI argument parsing
- `comrak` — Markdown parsing (CommonMark + extensions)
- `tantivy` — Full-text search engine (BM25)
- `petgraph` — Knowledge graph data structure
- `reqwest` — HTTP client for LLM API calls
- `serde` / `serde_yaml` / `serde_json` — Serialization
- `sha2` — SHA256 hashing for ingest cache
- `tokio` — Async runtime (streaming LLM responses)
- `pdf-extract` — PDF text extraction
- `encoding_rs` — Character encoding detection
**Storage**: File-based (JSON cache files, markdown wiki pages)
**Testing**: `cargo test` (unit + integration)
**Target Platform**: Linux, macOS, Windows (cross-compiled)
**Project Type**: CLI tool
**Performance Goals**: Search 500 pages <200ms, graph build 500 nodes <1s, binary <20MB
**Constraints**: Single binary, no runtime dependencies, offline-capable (with local LLM)
**Scale/Scope**: Personal wikis up to ~1000 pages, ~500 sources

## Constitution Check

*GATE: Must pass before Phase 0 research. Re-check after Phase 1 design.*

| Principle | Status | Notes |
|---|---|---|
| I. Persistent Wiki Over Transient RAG | ✅ PASS | Core design: wiki is the artifact |
| II. Three-Layer Architecture | ✅ PASS | raw/ + wiki/ + schema files |
| III. Incremental Ingest with Traceability | ✅ PASS | Two-pass pipeline, YAML frontmatter, SHA256 cache |
| IV. Knowledge Graph Integrity | ✅ PASS | Wikilink graph + lint + community detection |
| V. Human Curates, LLM Maintains | ✅ PASS | Agent invokes tool; human reviews output |
| VI. Reference Implementation Awareness | ✅ PASS | Design synthesizes both implementations |

All gates pass. No violations.

## Project Structure

### Documentation (this feature)

```text
specs/001-llm-wiki-agent-rust/
├── plan.md              # This file
├── spec.md              # Feature specification
├── research.md          # Phase 0: technology research
├── data-model.md        # Phase 1: data model design
├── quickstart.md        # Phase 1: getting started guide
├── contracts/           # Phase 1: CLI interface contracts
└── tasks.md             # Phase 2: task breakdown
```

### Source Code (repository root)

```text
wiki-tool/
├── Cargo.toml
├── Cargo.lock
├── src/
│   ├── main.rs              # CLI entry point (clap)
│   ├── lib.rs               # Library root
│   ├── commands/
│   │   ├── mod.rs
│   │   ├── ingest.rs        # Standalone: two-pass ingest pipeline
│   │   ├── query.rs         # Standalone: wiki query with citations
│   │   ├── search.rs        # Both: full-text search (no LLM)
│   │   ├── lint.rs          # Both: wiki health check (no LLM)
│   │   ├── graph.rs         # Both: knowledge graph builder (no LLM)
│   │   ├── cache.rs         # Both: SHA256 cache check (no LLM)
│   │   ├── extract.rs       # Both: document text extraction (no LLM)
│   │   └── index.rs         # Both: rebuild index.md (no LLM)
│   ├── llm/                 # Only used in standalone mode
│   │   ├── mod.rs
│   │   ├── client.rs        # Streaming LLM client
│   │   ├── providers.rs     # Multi-provider abstraction
│   │   └── prompts.rs       # Ingest/query prompt templates
│   ├── wiki/
│   │   ├── mod.rs
│   │   ├── page.rs          # WikiPage model + frontmatter
│   │   ├── index.rs         # index.md management
│   │   ├── log.rs           # log.md management
│   │   └── wikilinks.rs     # [[wikilink]] parser/resolver
│   ├── graph/
│   │   ├── mod.rs
│   │   ├── builder.rs       # Graph construction from pages
│   │   ├── relevance.rs     # 4-signal relevance scoring
│   │   └── community.rs     # Louvain community detection
│   ├── search/
│   │   ├── mod.rs
│   │   ├── engine.rs        # Tantivy-based search
│   │   └── tokenizer.rs     # CJK-aware tokenizer
│   ├── cache/
│   │   ├── mod.rs
│   │   ├── ingest_cache.rs  # SHA256-based skip cache
│   │   └── queue.rs         # Persistent ingest queue
│   ├── extract/
│   │   ├── mod.rs
│   │   ├── markdown.rs      # Markdown reader
│   │   ├── pdf.rs           # PDF text extraction
│   │   └── text.rs          # Plain text reader
│   └── config.rs            # Config file management
├── tests/
│   ├── integration/
│   │   ├── ingest_test.rs
│   │   ├── search_test.rs
│   │   ├── lint_test.rs
│   │   └── graph_test.rs
│   └── fixtures/
│       ├── raw/             # Test source documents
│       └── wiki/            # Test wiki state
├── schema/
│   ├── CLAUDE.md            # Claude Code schema
│   ├── AGENTS.md            # Codex/OpenCode schema
│   ├── GEMINI.md            # Gemini CLI schema
│   └── COPILOT.md           # Copilot CLI schema
└── README.md
```

**Structure Decision**: Single Rust project (CLI tool) with modular crate
layout. Schema files live alongside the binary for distribution. The tool
is self-contained — no external services required.

## Complexity Tracking

> No Constitution Check violations. No complexity justifications needed.
