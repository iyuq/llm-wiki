# CLI Interface Contract: wiki-tool

## Dual-Mode Architecture

```
┌─────────────────────────────────────────────────────────┐
│  Agent-Companion Mode (default, no API key)             │
│                                                         │
│  Coding Agent (IS the LLM)                              │
│    ├─ Reads schema (CLAUDE.md / AGENTS.md / etc.)       │
│    ├─ Does reasoning, analysis, wiki page generation    │
│    └─ Shells out to wiki-tool for:                      │
│         search, graph, lint, cache, extract, index      │
│                                                         │
│  Commands available: search, graph, lint, cache,        │
│                      extract, index, init               │
└─────────────────────────────────────────────────────────┘

┌─────────────────────────────────────────────────────────┐
│  Standalone Mode (requires API key, for CI/automation)  │
│                                                         │
│  wiki-tool makes its own LLM calls:                     │
│    ├─ ingest: two-pass analysis → generation            │
│    └─ query: search → context → synthesize answer       │
│                                                         │
│  Additional commands: ingest, query                     │
│  Requires: .wiki-tool.toml with LLM provider config     │
└─────────────────────────────────────────────────────────┘
```

## Global Options

```
wiki-tool [OPTIONS] <COMMAND>

Options:
  -c, --config <PATH>    Config file path [default: .wiki-tool.toml]
  -p, --project <PATH>   Project root directory [default: .]
  -v, --verbose          Enable verbose output
  -q, --quiet            Suppress non-essential output
  --json                 Output as JSON (machine-readable)
  -h, --help             Print help
  -V, --version          Print version
```

## Agent-Companion Commands (no LLM, no API key)

### search

Search wiki content by keyword. Agents use this to find relevant
pages before reading them.

```
wiki-tool search [OPTIONS] <QUERY>

Arguments:
  <QUERY>          Search terms

Options:
  -n, --limit <N>  Max results [default: 10]
  --snippet        Include content snippets
```

**Stdout**: Ranked list of matching pages with scores

---

### lint

Health-check the wiki for quality issues. Agents use this to
identify problems, then fix them directly.

```
wiki-tool lint [OPTIONS]

Options:
  --fix            Auto-fix simple issues (e.g., rebuild index)
  --category <C>   Filter: orphans, broken-links, missing-pages,
                   contradictions, stale
```

**Stdout**: Issue list with file:line references
**Exit codes**: 0 = no issues, 1 = issues found

---

### graph

Build or update the knowledge graph. Agents use this for
context discovery and relationship visualization.

```
wiki-tool graph [OPTIONS]

Options:
  -o, --output <PATH>  Output file [default: graph.json]
  --format <FMT>       Output format: json, dot [default: json]
  --communities        Include Louvain community detection
  --related <PAGE>     Show pages related to a specific page
```

**Stdout**: Graph statistics (nodes, edges, communities)

---

### cache

Check or manage the ingest cache. Agents use this to decide
whether a source needs re-ingesting.

```
wiki-tool cache check <SOURCE_PATH>    # check if cached
wiki-tool cache list                   # list all cached sources
wiki-tool cache clear [SOURCE_PATH]    # clear cache entry
```

**Stdout** (check): `cached` or `not-cached` + hash info
**Stdout** (--json): `{ "cached": true, "hash": "abc...", "files": [...] }`

---

### extract

Extract text content from a document. Agents use this to read
PDFs and other binary formats before processing.

```
wiki-tool extract <FILE_PATH>

Supported formats: .md, .txt, .pdf, .docx, .html
```

**Stdout**: Extracted plain text content

---

### index

Rebuild wiki/index.md from current wiki pages. Agents call
this after making wiki changes.

```
wiki-tool index [OPTIONS]

Options:
  --check          Verify index is up-to-date (exit 1 if stale)
```

---

### init

Initialize a new wiki project.

```
wiki-tool init [OPTIONS]

Options:
  --schema <AGENT>   Generate schema for: claude, codex, gemini,
                     copilot, all [default: all]
```

Creates: raw/, wiki/, .wiki-tool.toml, schema files

---

## Standalone Commands (requires LLM config + API key)

### ingest

Ingest a source document into the wiki using two-pass LLM pipeline.

```
wiki-tool ingest [OPTIONS] <SOURCE_PATH>

Arguments:
  <SOURCE_PATH>    Path to source file (relative to raw/)

Options:
  --force          Re-ingest even if cached
  --dry-run        Show what would be created without writing
  --no-stream      Disable streaming output
```

**Exit codes**: 0 = success, 1 = LLM error (no API key or
provider misconfigured), 2 = file not found, 3 = parse error

---

### query

Ask a question and get a synthesized answer from the wiki.

```
wiki-tool query [OPTIONS] <QUESTION>

Arguments:
  <QUESTION>       Natural language question

Options:
  --save           Save answer as a synthesis page
  --context <N>    Number of context pages to use [default: 5]
  --no-stream      Disable streaming output
```

**Stdout**: Answer with `[[wikilink]]` citations

---

## JSON Output Schema (--json)

### Ingest Result

```json
{
  "source": "raw/article.md",
  "cached": false,
  "pages_created": ["wiki/sources/article.md", "wiki/entities/GPT.md"],
  "pages_updated": ["wiki/index.md", "wiki/concepts/Transformers.md"],
  "review_items": [
    {
      "type": "contradiction",
      "page": "wiki/entities/GPT.md",
      "message": "Release date conflicts with existing entry",
      "suggestion": "Verify against primary source"
    }
  ]
}
```

### Search Result

```json
{
  "results": [
    {
      "page": "wiki/concepts/Transformers.md",
      "title": "Transformers",
      "score": 15.2,
      "snippet": "...attention mechanism that..."
    }
  ],
  "total": 1,
  "query_ms": 12
}
```

### Lint Result

```json
{
  "issues": [
    {
      "category": "broken-link",
      "file": "wiki/sources/article.md",
      "line": 42,
      "message": "[[NonExistent]] links to missing page",
      "suggestion": "Create wiki/concepts/NonExistent.md or fix link"
    }
  ],
  "total": 1
}
```

### Graph Result

```json
{
  "nodes": [
    { "id": "Transformers", "type": "concept", "community": 0, "links": 5 }
  ],
  "edges": [
    { "source": "Transformers", "target": "GPT", "weight": 7.5 }
  ],
  "communities": [
    { "id": 0, "size": 8, "cohesion": 0.72, "top_nodes": ["Transformers", "Attention"] }
  ],
  "stats": { "nodes": 42, "edges": 98, "communities": 5 }
}
```
