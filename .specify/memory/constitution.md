<!--
Sync Impact Report
- Version change: 1.0.0 → 1.0.1
- Modified principles:
  - VI. Reference Implementation Awareness: clarified that both
    reference projects are tracked as git submodules
- Added sections: None
- Removed sections: None
- Templates requiring updates:
  - .specify/templates/plan-template.md ✅ no changes needed
  - .specify/templates/spec-template.md ✅ no changes needed
  - .specify/templates/tasks-template.md ✅ no changes needed
- Follow-up TODOs: None
-->
# LLM Wiki Constitution

## Core Principles

### I. Persistent Wiki Over Transient RAG

The LLM MUST incrementally build and maintain a persistent,
interlinked wiki of markdown files — not re-derive answers from
raw sources on every query. Knowledge is compiled once, kept
current, and compounded over time. The wiki is the primary
artifact; chat history is ephemeral.

**Rationale**: Traditional RAG rediscovers knowledge from scratch
on every question. A persistent wiki accumulates cross-references,
flags contradictions, and synthesizes across sources once — then
keeps the result available and up to date.

### II. Three-Layer Architecture

Every LLM Wiki instance MUST maintain three distinct layers:

1. **Raw sources** (`raw/`) — immutable, user-curated documents.
   The LLM MUST NOT modify files in this layer.
2. **Wiki** (`wiki/`) — LLM-generated and LLM-maintained markdown
   with YAML frontmatter and `[[wikilinks]]`. The LLM owns this
   layer entirely.
3. **Schema** — a configuration document that defines wiki
   structure, conventions, and workflows. The user and LLM
   co-evolve this over time.

**Rationale**: Separating immutable sources from generated wiki
content preserves the source of truth while giving the LLM full
ownership of the knowledge layer. The schema provides governance
without hard-coding conventions.

### III. Incremental Ingest with Traceability

Every wiki page MUST carry YAML frontmatter including `title`,
`type`, `tags`, `sources[]`, and `last_updated`. When a new source
is ingested, the LLM MUST:
- Create or update a source summary page
- Update `wiki/index.md` (content catalog)
- Append to `wiki/log.md` (chronological record)
- Update all affected entity, concept, and synthesis pages
- Preserve `[[wikilink]]` cross-references

SHA256-based caching SHOULD be used to skip unchanged sources.

**Rationale**: Traceability back to raw sources is essential for
trust. The index and log provide navigability at scale. Caching
prevents redundant work.

### IV. Knowledge Graph Integrity

Cross-references via `[[wikilinks]]` MUST be maintained as a
first-class concern. Periodic lint operations MUST check for:
- Orphan pages with no inbound links
- Broken wikilinks pointing to non-existent pages
- Concepts mentioned but lacking dedicated pages
- Contradictions between pages
- Stale claims superseded by newer sources

**Rationale**: The value of a wiki compounds through its
connections. An unmaintained link graph degrades into a flat
document collection.

### V. Human Curates, LLM Maintains

The human's role is to curate sources, direct analysis, ask
questions, and make editorial decisions. The LLM's role is
summarizing, cross-referencing, filing, updating, and all
bookkeeping. The LLM MUST NOT independently add raw sources
or make editorial judgment calls without human direction.

**Rationale**: LLMs excel at the tedious maintenance that causes
humans to abandon wikis. Keeping editorial control with the human
preserves intentionality and trust.

### VI. Reference Implementation Awareness

This project maintains two reference implementations of the LLM
Wiki pattern. Contributors and agents MUST understand both when
making architectural decisions:

1. **`doc/llm-wiki-agent/`** (git submodule →
   `github.com/SamurAIGPT/llm-wiki-agent`) — A coding-agent skill
   (Claude Code, Codex, Gemini CLI). No build step. Slash commands
   drive ingest, query, lint, and graph operations. Python tools
   in `tools/`.
2. **`doc/llm_wiki/`** (git submodule →
   `github.com/nashsu/llm_wiki`) — A cross-platform desktop app
   (Tauri 2 + React 19 + TypeScript + Rust). Features chain-of-
   thought ingest, 4-signal knowledge graph, Louvain community
   detection, vector search via LanceDB, and a Chrome web clipper.

Both descend from Karpathy's original LLM Wiki pattern documented
in `doc/llm-wiki.md`.

**Rationale**: Awareness of both implementations prevents duplicate
work, ensures feature parity where appropriate, and informs
architectural decisions with concrete precedent.

## Reference Implementations

| Aspect | Agent Skill (`doc/llm-wiki-agent/`) | Desktop App (`doc/llm_wiki/`) |
|---|---|---|
| Runtime | LLM coding agent (Claude/Codex/Gemini) | Tauri 2 (Rust) + React 19 (TypeScript) |
| Build | None | `npm install && npm run build` / `cargo build` |
| Ingest | LLM reads source → writes wiki pages | Two-step chain-of-thought with SHA256 cache |
| Search | Index file + optional `qmd` | LanceDB vector store + BM25 |
| Graph | Python script in `tools/` | graphology + sigma.js (4-signal relevance) |
| Schema | `CLAUDE.md` / `AGENTS.md` / `GEMINI.md` | `schema.md` + `purpose.md` |
| Wiki format | YAML frontmatter + `[[wikilinks]]` | YAML frontmatter + `[[wikilinks]]` |
| CI | None | GitHub Actions (macOS/Ubuntu/Windows) |

## Development Workflow

### Source Management

- Raw sources are added to `raw/` and MUST NOT be modified after
  initial placement.
- Sources SHOULD be ingested one at a time with human review of
  generated wiki updates, unless batch mode is explicitly chosen.

### Wiki Maintenance

- `wiki/index.md` MUST be updated on every ingest operation.
- `wiki/log.md` entries MUST use the format:
  `## [YYYY-MM-DD] <operation> | <title>`
- Valuable query answers SHOULD be filed back into the wiki as
  synthesis pages to compound knowledge.
- Periodic lint passes SHOULD be run to maintain wiki health.

### Code Contributions

- Changes to the desktop app (`doc/llm_wiki/`) MUST pass the
  existing CI pipeline (frontend build + Rust build).
- Changes to the agent skill (`doc/llm-wiki-agent/`) MUST preserve
  compatibility with Claude Code, Codex, and Gemini CLI.
- The shared wiki page format (YAML frontmatter + `[[wikilinks]]`)
  MUST remain consistent across both implementations.

## Governance

This constitution is the authoritative reference for all LLM Wiki
development in this repository. It supersedes ad-hoc decisions and
MUST be consulted when making architectural or workflow changes.

- **Amendments** require updating this file, incrementing the
  version, and documenting changes in the Sync Impact Report
  (HTML comment at the top of this file).
- **Version policy**: MAJOR for principle removals or
  redefinitions, MINOR for new principles or sections, PATCH for
  clarifications and wording fixes.
- **Compliance**: All PRs and code reviews SHOULD verify alignment
  with these principles. Deviations MUST be justified in the PR
  description.

**Version**: 1.0.1 | **Ratified**: 2026-04-16 | **Last Amended**: 2026-04-16
