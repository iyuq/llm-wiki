# Feature Specification: LLM Wiki Agent — Rust CLI Tool

**Feature Branch**: `001-llm-wiki-agent-rust`
**Created**: 2026-04-16
**Status**: Draft
**Input**: Build a Rust-based LLM Wiki agent skill that combines the best
ideas from both reference implementations (agent skill + desktop app) and
community best practices, targeting all major coding agents.

## User Scenarios & Testing *(mandatory)*

### User Story 1 - Ingest a Source Document (Priority: P1)

A user drops a source file (markdown, text, PDF) into `raw/` and tells
their coding agent to ingest it. The agent reads the source, produces a
structured analysis, generates wiki pages, updates the index and log,
and reports what was created or changed.

**Why this priority**: Ingest is the core value proposition — without it,
nothing else works. Every other feature depends on having wiki content.

**Independent Test**: Run `wiki-tool ingest raw/test-article.md` and
verify wiki pages are created with correct frontmatter, index.md is
updated, log.md has a new entry, and wikilinks resolve.

**Acceptance Scenarios**:

1. **Given** an empty wiki and a markdown file in `raw/`, **When** the
   user runs `wiki-tool ingest raw/article.md`, **Then** a source summary
   page is created in `wiki/sources/`, index.md lists the new page,
   log.md has a timestamped entry, and entity/concept pages are created.
2. **Given** a wiki with existing pages, **When** a new source overlaps
   with existing entities, **Then** existing entity pages are updated
   (not duplicated), new cross-references are added, and contradictions
   are flagged as review items.
3. **Given** a source that was already ingested (same SHA256), **When**
   the user runs ingest again, **Then** it is skipped with a cache-hit
   message.

---

### User Story 2 - Query the Wiki (Priority: P2)

A user asks a question and the agent searches the wiki, reads relevant
pages, and synthesizes an answer with citations. The answer can optionally
be filed back into the wiki as a synthesis page.

**Why this priority**: Querying is how users extract value from the wiki.
Without query, the wiki is write-only.

**Independent Test**: Run `wiki-tool query "What are the main themes?"`
against a wiki with ingested content and verify a cited answer is returned.

**Acceptance Scenarios**:

1. **Given** a wiki with multiple pages, **When** the user queries a
   topic, **Then** relevant pages are found via keyword + graph relevance,
   and the answer cites specific wiki pages.
2. **Given** a query answer the user wants to keep, **When** they run
   `wiki-tool query --save "question"`, **Then** a synthesis page is
   created in `wiki/syntheses/` with proper frontmatter.

---

### User Story 3 - Lint the Wiki (Priority: P3)

A user asks the agent to health-check the wiki. The tool scans for
orphan pages, broken wikilinks, missing pages for mentioned concepts,
contradictions, and stale content.

**Why this priority**: Lint maintains wiki quality as it grows, but the
wiki must exist first (depends on ingest).

**Independent Test**: Run `wiki-tool lint` against a wiki with known
issues (orphan page, broken link) and verify all issues are reported.

**Acceptance Scenarios**:

1. **Given** a wiki with an orphan page, **When** lint runs, **Then**
   the orphan is reported with a suggestion to link it.
2. **Given** a broken `[[wikilink]]`, **When** lint runs, **Then** the
   broken link is reported with the source file and line number.

---

### User Story 4 - Build Knowledge Graph (Priority: P3)

A user asks the agent to build or update the knowledge graph. The tool
parses all wikilinks, computes relevance scores, detects communities,
and outputs a graph data file.

**Why this priority**: Graph visualization and community detection add
discovery value but are not required for basic wiki operation.

**Independent Test**: Run `wiki-tool graph` and verify `graph.json` is
created with nodes, edges, and community assignments.

**Acceptance Scenarios**:

1. **Given** a wiki with cross-referenced pages, **When** graph is built,
   **Then** output contains nodes (with type, community), edges (with
   weight), and community metrics.

---

### User Story 5 - Search Wiki Content (Priority: P2)

A user searches for content across the wiki using keywords. The tool
returns ranked results with snippets.

**Why this priority**: Search is needed for both human browsing and
LLM-driven query workflows.

**Independent Test**: Run `wiki-tool search "transformer"` and verify
matching pages are returned ranked by relevance.

**Acceptance Scenarios**:

1. **Given** wiki pages containing "transformer", **When** the user
   searches, **Then** results are ranked by title match > content match,
   with context snippets.
2. **Given** CJK content, **When** the user searches with CJK terms,
   **Then** bigram tokenization finds partial matches.

---

### Edge Cases

- What happens when the LLM API is unavailable? → Retry 3 times, then
  fail with clear error and queue the task for later.
- What happens with very large source files (>100KB)? → Chunk into
  sections, ingest sequentially with context carry-forward.
- What happens with non-UTF8 files? → Detect encoding, attempt
  conversion, fail gracefully with error message.
- What happens with circular wikilinks? → Graph builder handles cycles
  naturally; lint warns if self-referencing.

## Requirements *(mandatory)*

### Functional Requirements

- **FR-001**: System MUST implement a two-pass ingest pipeline
  (analysis → generation) with streaming output.
- **FR-002**: System MUST maintain SHA256-based ingest cache to skip
  unchanged sources.
- **FR-003**: System MUST parse and generate YAML frontmatter on all
  wiki pages (title, type, tags, sources[], last_updated).
- **FR-004**: System MUST update wiki/index.md and wiki/log.md on
  every ingest operation.
- **FR-005**: System MUST support multiple LLM providers (OpenAI,
  Anthropic, Google, Ollama, custom endpoints).
- **FR-006**: System MUST extract and maintain `[[wikilinks]]` as
  cross-references.
- **FR-007**: System MUST provide keyword-based search with CJK
  bigram tokenization support.
- **FR-008**: System MUST build a knowledge graph from wikilinks with
  multi-signal relevance scoring.
- **FR-009**: System MUST detect and report wiki quality issues (lint).
- **FR-010**: System MUST support PDF and plain text document extraction.
- **FR-011**: System MUST provide agent-compatible schema files
  (CLAUDE.md, AGENTS.md, GEMINI.md, COPILOT.md).
- **FR-012**: System MUST implement a persistent ingest queue with
  crash recovery and retry logic (max 3 retries).

### Key Entities

- **WikiPage**: title, type (source|entity|concept|synthesis), tags[],
  sources[], last_updated, content, wikilinks[]
- **Source**: path, sha256_hash, ingested_at, pages_generated[]
- **IngestTask**: id, source_path, status, retry_count, error
- **GraphNode**: id, title, type, community_id, link_count
- **GraphEdge**: source, target, weight, signals{}

## Success Criteria *(mandatory)*

### Measurable Outcomes

- **SC-001**: Ingest a 10-page markdown file in under 60 seconds
  (excluding LLM latency).
- **SC-002**: Search 500 wiki pages and return results in under 200ms.
- **SC-003**: Build graph for 500 nodes in under 1 second.
- **SC-004**: Binary size under 20MB (statically linked).
- **SC-005**: Zero data loss on crash during ingest (queue recovery).

## Assumptions

- Users have access to at least one LLM API (or local Ollama instance).
- The primary interaction model is through a coding agent (Claude Code,
  Codex, Copilot CLI, Gemini CLI) that reads schema files and invokes
  the CLI tool.
- Source documents are primarily markdown and text; PDF support is
  secondary.
- The tool runs on Linux, macOS, and Windows.
