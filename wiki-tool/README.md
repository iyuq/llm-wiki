# wiki-tool — LLM Wiki Agent (Rust CLI)

A high-performance Rust CLI tool for building and maintaining a personal knowledge base from source documents using the LLM Wiki pattern.

## Features

- **Two-pass LLM ingest**: Extract entities, concepts, and relationships from source documents
- **Full-text search**: BM25 search with CJK bigram tokenization via tantivy
- **Knowledge graph**: Build graphs with 4-signal relevance scoring and Louvain community detection
- **Wiki health check**: Detect orphan pages, broken wikilinks, missing pages, and stale content
- **Dual-mode operation**: Works as agent companion (no API key) or standalone (with API key)
- **Multi-provider LLM**: Supports OpenAI, Anthropic, Google, Ollama, and custom endpoints

## Installation

```bash
cd wiki-tool
cargo build --release
# Binary at target/release/wiki-tool
```

## Quick Start

```bash
# Initialize a wiki project
wiki-tool init

# Configure LLM provider (edit .wiki-tool.toml)
# Set API key: export ANTHROPIC_API_KEY="sk-ant-..."

# Ingest a source document
wiki-tool ingest raw/article.md

# Search the wiki
wiki-tool search "transformer architecture"

# Ask a question (requires LLM config)
wiki-tool query "What are the key concepts?"

# Check wiki health
wiki-tool lint

# Build knowledge graph
wiki-tool graph --communities
```

## Architecture

```
wiki-tool/src/
├── main.rs          # CLI entry point (clap)
├── lib.rs           # Library root
├── config.rs        # TOML config management
├── commands/        # Subcommand handlers
├── wiki/            # Wiki page model, frontmatter, wikilinks
├── llm/             # LLM client, providers, prompts
├── search/          # Tantivy search engine, CJK tokenizer
├── graph/           # Knowledge graph, relevance, community detection
├── cache/           # SHA256 ingest cache, persistent queue
└── extract/         # Document extractors (markdown, text, PDF)
```

## Dual-Mode Operation

### Agent-Companion Mode (default, no API key)

The coding agent (Claude Code, Copilot CLI, etc.) IS the LLM. It reads schema files and calls wiki-tool for deterministic operations:

```bash
wiki-tool search <QUERY>     # Find relevant pages
wiki-tool lint [--fix]        # Check wiki quality
wiki-tool graph               # Build knowledge graph
wiki-tool cache check <FILE>  # Check ingest cache
wiki-tool extract <FILE>      # Extract text from documents
wiki-tool index               # Rebuild index.md
```

### Standalone Mode (requires API key)

wiki-tool makes its own LLM API calls:

```bash
wiki-tool ingest <SOURCE>    # Two-pass LLM ingest
wiki-tool query <QUESTION>   # Search + synthesize answer
```

## Commands

| Command | Purpose | Needs LLM? |
|---------|---------|-------------|
| `init` | Initialize wiki project | No |
| `ingest` | Ingest source document | Yes |
| `query` | Ask question, get answer | Yes |
| `search` | Full-text search | No |
| `lint` | Wiki health check | No |
| `graph` | Build knowledge graph | No |
| `cache` | Manage ingest cache | No |
| `extract` | Extract text from files | No |
| `index` | Rebuild index.md | No |

## Configuration

Edit `.wiki-tool.toml`:

```toml
[llm]
provider = "anthropic"

[llm.providers.anthropic]
api_url = "https://api.anthropic.com/v1/messages"
model = "claude-sonnet-4-20250514"
max_tokens = 8192
timeout_secs = 900
```

Set your API key via environment variable:
```bash
export ANTHROPIC_API_KEY="sk-ant-..."
export OPENAI_API_KEY="sk-..."
```

## JSON Output

All commands support `--json` for machine-readable output:

```bash
wiki-tool search "query" --json
wiki-tool lint --json
wiki-tool graph --json
```

## Performance Targets

- Search 500 pages: <200ms
- Graph build 500 nodes: <1s
- Binary size: <20MB

## License

MIT
