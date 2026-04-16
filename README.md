# LLM Wiki

Build and maintain a personal knowledge base using LLMs.

Instead of traditional RAG — where the LLM re-derives answers from raw documents on every query — the LLM **incrementally builds and maintains a persistent wiki**: a structured, interlinked collection of markdown files that compounds knowledge over time. You curate sources and ask questions; the LLM does all the summarizing, cross-referencing, and bookkeeping.

> Based on [Andrej Karpathy's LLM Wiki pattern](https://gist.github.com/karpathy/442a6bf555914893e9891c11519de94f).

## What's in this repo

### [`wiki-tool/`](wiki-tool/) — Rust CLI Tool

A high-performance CLI tool for wiki operations. Works in two modes:

- **Agent-companion** (default, no API key) — your coding agent handles the LLM reasoning; `wiki-tool` provides fast search, graph, lint, cache, and extraction
- **Standalone** (with API key) — `wiki-tool` makes its own LLM calls for CI pipelines and automation

```bash
cd wiki-tool && cargo build --release

wiki-tool init                          # create a new wiki project
wiki-tool search "transformer"          # BM25 search with CJK support
wiki-tool lint                          # check wiki health
wiki-tool graph --communities           # build knowledge graph
wiki-tool ingest raw/paper.md           # two-pass LLM ingest (standalone)
wiki-tool query "What is attention?"    # LLM-synthesized answer (standalone)
```

**9 commands**: init, ingest, query, search, lint, graph, cache, extract, index

### [`doc/llm-wiki.md`](doc/llm-wiki.md) — The Pattern

Karpathy's original design document describing the LLM Wiki concept — the abstract pattern that everything else implements.

### [`doc/llm-wiki-agent/`](doc/llm-wiki-agent/) — Reference: Agent Skill

A coding-agent skill (Claude Code, Codex, Gemini CLI) by [SamurAIGPT](https://github.com/SamurAIGPT/llm-wiki-agent). No build step — slash commands drive ingest, query, lint, and graph operations. Included as a git submodule for reference.

### [`doc/llm_wiki/`](doc/llm_wiki/) — Reference: Desktop App

A cross-platform desktop app (Tauri + React + Rust) by [nashsu](https://github.com/nashsu/llm_wiki). Features chain-of-thought ingest, 4-signal knowledge graph, vector search, and Chrome web clipper. Included as a git submodule for reference.

## Architecture

Three layers, as defined by the pattern:

```
raw/     Immutable source documents — the LLM reads but never modifies these
wiki/    LLM-generated markdown with YAML frontmatter and [[wikilinks]]
schema   Configuration that tells the LLM how the wiki is structured
```

### Wiki page format

```yaml
---
title: "Page Title"
type: source | entity | concept | synthesis
tags: [tag1, tag2]
sources: [source-slug]
last_updated: 2026-04-16
---

Content with [[wikilinks]] to other pages.
```

### Key operations

| Operation | What happens |
|-----------|-------------|
| **Ingest** | LLM reads a source → creates/updates wiki pages → updates index + log |
| **Query** | Search relevant pages → LLM synthesizes answer with citations |
| **Lint** | Check for orphan pages, broken links, contradictions, stale content |
| **Graph** | Build knowledge graph with relevance scoring + community detection |

## Quick start

```bash
# Clone with submodules
git clone --recurse-submodules https://github.com/iyuq/llm-wiki.git
cd llm-wiki

# Build the CLI tool
cd wiki-tool
cargo build --release

# Initialize a wiki project
./target/release/wiki-tool init

# Drop sources into raw/ and start building your knowledge base
cp ~/articles/*.md raw/
```

### Using with a coding agent

Open the project in your preferred coding agent (Claude Code, Copilot CLI, Codex, Gemini CLI). The agent reads the schema file and uses `wiki-tool` for search, graph, lint, and extraction while handling all LLM reasoning itself.

```
You: "Ingest raw/my-paper.pdf"
Agent: *reads schema* → runs wiki-tool extract → analyzes content →
       generates wiki pages → runs wiki-tool index
```

## Project structure

```
llm-wiki/
├── wiki-tool/              # Rust CLI tool (this project)
│   ├── src/
│   │   ├── commands/       # CLI command handlers
│   │   ├── wiki/           # Page model, index, log, wikilinks
│   │   ├── search/         # Tantivy BM25 + CJK tokenizer
│   │   ├── graph/          # Petgraph + relevance + Louvain
│   │   ├── llm/            # Multi-provider streaming client
│   │   ├── cache/          # SHA256 ingest cache + queue
│   │   └── extract/        # Markdown, text, PDF extraction
│   └── schema/             # Agent schema files
├── doc/
│   ├── llm-wiki.md         # Karpathy's original pattern
│   ├── llm-wiki-agent/     # Reference: agent skill (submodule)
│   └── llm_wiki/           # Reference: desktop app (submodule)
├── specs/                  # Feature specifications (Spec Kit)
└── .specify/               # Spec Kit configuration
```

## License

MIT
