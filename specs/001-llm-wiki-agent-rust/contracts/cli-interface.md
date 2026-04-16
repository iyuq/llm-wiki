# CLI Interface Contract: wiki-tool

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

## Commands

### ingest

Ingest a source document into the wiki.

```
wiki-tool ingest [OPTIONS] <SOURCE_PATH>

Arguments:
  <SOURCE_PATH>    Path to source file (relative to raw/)

Options:
  --force          Re-ingest even if cached
  --dry-run        Show what would be created without writing
  --no-stream      Disable streaming output
```

**Exit codes**: 0 = success, 1 = LLM error, 2 = file not found,
3 = parse error

**Stdout** (default): Streaming analysis + generated page summaries
**Stdout** (--json): `{ "pages_created": [...], "pages_updated": [...],
  "review_items": [...], "cached": false }`

---

### search

Search wiki content by keyword.

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

### lint

Health-check the wiki for quality issues.

```
wiki-tool lint [OPTIONS]

Options:
  --fix            Auto-fix simple issues (e.g., update index)
  --category <C>   Filter by category: orphans, broken-links,
                   missing-pages, contradictions, stale
```

**Stdout**: Issue list with file:line references
**Exit codes**: 0 = no issues, 1 = issues found

---

### graph

Build or update the knowledge graph.

```
wiki-tool graph [OPTIONS]

Options:
  -o, --output <PATH>  Output file [default: graph.json]
  --format <FMT>       Output format: json, dot [default: json]
  --communities        Include Louvain community detection
```

**Stdout**: Graph statistics (nodes, edges, communities)

---

### init

Initialize a new wiki project in the current directory.

```
wiki-tool init [OPTIONS]

Options:
  --schema <AGENT>   Generate schema for: claude, codex, gemini,
                     copilot, all [default: all]
```

Creates: raw/, wiki/, .wiki-tool.toml, schema files

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
