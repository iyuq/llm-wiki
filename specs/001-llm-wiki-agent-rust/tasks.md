# Tasks: LLM Wiki Agent — Rust CLI Tool

**Input**: Design documents from `/specs/001-llm-wiki-agent-rust/`
**Prerequisites**: plan.md ✅, spec.md ✅, research.md ✅, data-model.md ✅, contracts/cli-interface.md ✅, quickstart.md ✅

**Tests**: Not explicitly requested in the feature specification. Test tasks are omitted. Test fixtures are created in Phase 8 for future use.

**Organization**: Tasks are grouped by user story to enable independent implementation and testing of each story.

## Format: `[ID] [P?] [Story] Description`

- **[P]**: Can run in parallel (different files, no dependencies)
- **[Story]**: Which user story this task belongs to (e.g., US1, US2, US3)
- Include exact file paths in descriptions

## Path Conventions

- **Single project**: `wiki-tool/src/` at repository root (per plan.md project structure)

---

## Phase 1: Setup (Shared Infrastructure)

**Purpose**: Create the Rust project skeleton and configure build tooling

- [X] T001 Create project directory structure with all subdirectories per plan.md: wiki-tool/src/{commands,llm,wiki,graph,search,cache,extract}/, wiki-tool/tests/{integration,fixtures/raw,fixtures/wiki}/, wiki-tool/schema/
- [X] T002 Initialize wiki-tool/Cargo.toml with all dependencies: clap (derive), comrak, tantivy, petgraph, reqwest (rustls-tls, stream), serde (derive), serde_yaml, serde_json, sha2, tokio (full), pdf-extract, encoding_rs, toml, chrono
- [X] T003 [P] Configure Rust tooling: wiki-tool/rustfmt.toml (edition 2024) and clippy configuration in wiki-tool/Cargo.toml

---

## Phase 2: Foundational (Blocking Prerequisites)

**Purpose**: Core models, config, parsing infrastructure, and CLI skeleton that ALL user stories depend on

**⚠️ CRITICAL**: No user story work can begin until this phase is complete

- [X] T004 Create library root with module declarations (wiki, graph, search, cache, extract, config, commands, llm) and shared error enum (WikiToolError) with Display/Error derives in wiki-tool/src/lib.rs
- [X] T005 [P] Implement PageType enum (Source, Entity, Concept, Synthesis) and WikiPage struct with YAML frontmatter serialization/deserialization using serde_yaml and comrak in wiki-tool/src/wiki/mod.rs and wiki-tool/src/wiki/page.rs
- [X] T006 [P] Implement `[[wikilink]]` extraction regex parser and slug resolution (title → filename mapping) in wiki-tool/src/wiki/wikilinks.rs
- [X] T007 [P] Implement TOML config file parsing with LlmConfig and ProviderConfig structs (api_url, api_key, model, max_tokens, timeout_secs) and defaults in wiki-tool/src/config.rs
- [X] T008 [P] Implement document extractors: markdown reader via comrak (strips frontmatter, returns plain text) in wiki-tool/src/extract/markdown.rs, plain text reader with encoding_rs auto-detection in wiki-tool/src/extract/text.rs, PDF text extraction via pdf-extract in wiki-tool/src/extract/pdf.rs, and format-dispatch module in wiki-tool/src/extract/mod.rs
- [X] T009 Build CLI entry point with clap derive API: global options (--config, --project, --verbose, --quiet, --json, --version) and subcommand enum routing in wiki-tool/src/main.rs
- [X] T010 Implement init subcommand that creates raw/, wiki/{sources,entities,concepts,syntheses}/, .wiki-tool/ directories, default .wiki-tool.toml config, and placeholder schema files in wiki-tool/src/commands/mod.rs

**Checkpoint**: Foundation ready — all core types, config parsing, extraction, and CLI framework are in place. User story implementation can now begin.

---

## Phase 3: User Story 1 — Ingest a Source Document (Priority: P1) 🎯 MVP

**Goal**: A user drops a source file into raw/ and the tool ingests it via two-pass LLM pipeline, creating wiki pages, updating index.md and log.md, with SHA256 cache skip for unchanged sources.

**Independent Test**: Run `wiki-tool ingest raw/test-article.md` and verify wiki pages are created with correct YAML frontmatter, index.md is updated, log.md has a new entry, wikilinks resolve, and re-running skips with cache-hit message.

### Implementation for User Story 1

- [X] T011 [P] [US1] Implement SHA256-based ingest cache with JSON file persistence (.wiki-tool/ingest-cache.json), CacheEntry struct (hash, timestamp, files_written), and lookup/insert/clear operations in wiki-tool/src/cache/mod.rs and wiki-tool/src/cache/ingest_cache.rs
- [X] T012 [P] [US1] Implement persistent ingest queue with crash recovery: IngestQueue and IngestTask structs (id, source_path, status, retry_count, error), JSON persistence (.wiki-tool/ingest-queue.json), max 3 retries, and resume-on-start logic in wiki-tool/src/cache/queue.rs
- [X] T013 [P] [US1] Implement index.md catalog generation: scan wiki/ for all pages, read frontmatter, generate categorized listing (by PageType), and atomic file write in wiki-tool/src/wiki/index.rs
- [X] T014 [P] [US1] Implement log.md chronological entry appending: timestamped entry format, append-only writes, and operation type recording (ingest, update, delete) in wiki-tool/src/wiki/log.rs
- [X] T015 [US1] Implement async streaming HTTP client with SSE line parsing (data:, event:, [DONE] handling) using reqwest + tokio for LLM API responses in wiki-tool/src/llm/mod.rs and wiki-tool/src/llm/client.rs
- [X] T016 [US1] Implement multi-provider abstraction supporting OpenAI, Anthropic, Google, Ollama, and custom endpoints: provider-specific request/response mapping, API key resolution (env var or config), and unified streaming interface in wiki-tool/src/llm/providers.rs
- [X] T017 [P] [US1] Create ingest prompt templates (Pass 1: entity/concept extraction with contradiction detection; Pass 2: wiki page generation in ---FILE: block format) and query prompt templates (synthesis with citations) in wiki-tool/src/llm/prompts.rs
- [X] T018 [US1] Implement cache subcommand with three sub-commands: `check <SOURCE_PATH>` (cached/not-cached + hash), `list` (all cached sources), `clear [SOURCE_PATH]` (remove cache entries) in wiki-tool/src/commands/cache.rs
- [X] T019 [P] [US1] Implement extract subcommand that dispatches to format-specific extractors based on file extension (.md, .txt, .pdf) and outputs plain text to stdout in wiki-tool/src/commands/extract.rs
- [X] T020 [US1] Implement two-pass ingest pipeline command: SHA256 cache check → queue task → Pass 1 (analysis via LLM) → Pass 2 (generation via LLM) → parse ---FILE: blocks → write wiki pages → update index.md → append log.md → update cache → report created/updated pages and review items in wiki-tool/src/commands/ingest.rs
- [X] T021 [US1] Implement index rebuild subcommand with full rebuild and --check flag (verify index is up-to-date, exit 1 if stale) in wiki-tool/src/commands/index.rs

**Checkpoint**: At this point, User Story 1 should be fully functional and testable independently. Users can ingest documents, skip cached sources, check cache status, extract text, and rebuild the index.

---

## Phase 4: User Story 5 — Search Wiki Content (Priority: P2)

**Goal**: A user searches wiki content by keywords and gets ranked results with snippets. CJK content is searchable via bigram tokenization.

**Independent Test**: Run `wiki-tool search "transformer"` against a wiki with ingested content and verify matching pages are returned ranked by relevance with context snippets.

### Implementation for User Story 5

- [X] T022 [P] [US5] Implement CJK-aware bigram tokenizer as a custom tantivy TokenFilter: detect CJK Unicode ranges, split into bigrams, pass Latin text through standard tokenizer in wiki-tool/src/search/tokenizer.rs
- [X] T023 [US5] Implement tantivy-based BM25 search engine: define schema (title, content, path, page_type, tags), build index from wiki/ pages, query with title-boost scoring (title match > content match), return ranked results with snippets in wiki-tool/src/search/mod.rs and wiki-tool/src/search/engine.rs
- [X] T024 [US5] Implement search subcommand with <QUERY> argument, --limit (default 10), --snippet flag, ranked output formatting, and performance timing (query_ms) in wiki-tool/src/commands/search.rs

**Checkpoint**: At this point, User Stories 1 AND 5 should both work independently. Users can ingest documents and search the wiki.

---

## Phase 5: User Story 2 — Query the Wiki (Priority: P2)

**Goal**: A user asks a question and the tool searches the wiki, retrieves relevant pages plus wikilink neighbors for context, sends to LLM for synthesis, and returns a cited answer. Optionally saves the answer as a synthesis page.

**Independent Test**: Run `wiki-tool query "What are the main themes?"` against a wiki with ingested content and verify a cited answer with `[[wikilink]]` references is returned.

### Implementation for User Story 2

- [X] T025 [US2] Implement query subcommand: search for relevant pages via tantivy → traverse wikilink neighbors for context expansion → assemble context window (--context N pages, default 5) → LLM synthesis with citation formatting → stream answer to stdout → optional --save flag creating synthesis page in wiki/syntheses/ with proper frontmatter in wiki-tool/src/commands/query.rs

**Checkpoint**: At this point, User Stories 1, 5, AND 2 should all work independently. Users can ingest, search, and query the wiki with cited answers.

---

## Phase 6: User Story 3 — Lint the Wiki (Priority: P3)

**Goal**: A user runs a health-check that scans for orphan pages, broken wikilinks, missing pages, contradictions, and stale content, reporting issues with file:line references.

**Independent Test**: Run `wiki-tool lint` against a wiki with known issues (orphan page, broken `[[wikilink]]`) and verify all issues are reported with file paths and line numbers.

### Implementation for User Story 3

- [X] T026 [US3] Implement lint subcommand: parse all wiki pages → check orphan pages (no inbound wikilinks) → check broken wikilinks (target page missing) → check missing pages (referenced but not created) → check contradictions (conflicting frontmatter across pages) → check stale content (last_updated age) → report issues with file:line references and suggestions → --fix auto-repair (rebuild index, remove broken links) → --category filter (orphans, broken-links, missing-pages, contradictions, stale) → exit code 0/1 in wiki-tool/src/commands/lint.rs

**Checkpoint**: At this point, User Stories 1, 5, 2, AND 3 should all work independently. The wiki has full ingest, search, query, and quality checking.

---

## Phase 7: User Story 4 — Build Knowledge Graph (Priority: P3)

**Goal**: A user builds or updates the knowledge graph from all wiki pages. The tool parses wikilinks, computes multi-signal relevance scores, runs Louvain community detection, and outputs a graph data file.

**Independent Test**: Run `wiki-tool graph --communities` and verify graph.json is created with nodes (including type and community assignment), edges (with weight), and community metrics.

### Implementation for User Story 4

- [X] T027 [P] [US4] Implement graph construction: scan all wiki pages, create petgraph DiGraph with GraphNode structs (id, title, node_type, community_id, link_count), add edges from parsed wikilinks in wiki-tool/src/graph/mod.rs and wiki-tool/src/graph/builder.rs
- [X] T028 [P] [US4] Implement 4-signal relevance scoring per research.md: direct links (weight 3.0), source overlap (weight 4.0), common neighbors via Adamic-Adar index (weight 1.5), type affinity (weight 1.0), combined into GraphEdge weight in wiki-tool/src/graph/relevance.rs
- [X] T029 [US4] Implement Louvain community detection algorithm on petgraph: modularity optimization, iterative node reassignment, community merging, output community_id assignments on GraphNode structs in wiki-tool/src/graph/community.rs
- [X] T030 [US4] Implement graph subcommand: build graph → compute relevance → optional --communities for Louvain detection → output to --output path (default graph.json) in --format json or dot → --related <PAGE> for single-page neighborhood → print stats (nodes, edges, communities) in wiki-tool/src/commands/graph.rs

**Checkpoint**: All user stories are now independently functional. The wiki supports ingest, search, query, lint, and knowledge graph visualization.

---

## Phase 8: Polish & Cross-Cutting Concerns

**Purpose**: Agent integration, documentation, structured output, and validation across all stories

- [X] T031 [P] Create agent schema files documenting all wiki-tool commands, usage patterns, and workflows tailored to each agent in wiki-tool/schema/CLAUDE.md, wiki-tool/schema/AGENTS.md, wiki-tool/schema/GEMINI.md, wiki-tool/schema/COPILOT.md
- [X] T032 [P] Write project README.md with installation (cargo build --release), setup, usage examples, architecture overview, dual-mode explanation, and performance targets in wiki-tool/README.md
- [X] T033 Implement --json structured output mode across all commands: ingest result (pages_created, pages_updated, review_items), search result (results array with score/snippet, query_ms), lint result (issues array with category/file/line), graph result (nodes/edges/communities/stats), cache result (cached boolean, hash) per CLI contract schemas in wiki-tool/src/commands/*.rs
- [X] T034 [P] Create test fixtures: sample raw/ source documents (markdown, text, PDF) and pre-built wiki/ state (pages with frontmatter, wikilinks, known lint issues) in wiki-tool/tests/fixtures/raw/ and wiki-tool/tests/fixtures/wiki/
- [X] T035 Validate quickstart.md end-to-end workflow: init → configure provider → ingest source → search → query → lint → graph, verifying each step produces expected output

---

## Dependencies & Execution Order

### Phase Dependencies

- **Setup (Phase 1)**: No dependencies — can start immediately
- **Foundational (Phase 2)**: Depends on Setup completion — **BLOCKS all user stories**
- **US1 Ingest (Phase 3)**: Depends on Foundational — can start immediately after Phase 2
- **US5 Search (Phase 4)**: Depends on Foundational — can start in parallel with US1 (no shared files)
- **US2 Query (Phase 5)**: Depends on US1 (LLM client) AND US5 (search engine)
- **US3 Lint (Phase 6)**: Depends on Foundational only — can start in parallel with US1/US5
- **US4 Graph (Phase 7)**: Depends on Foundational only — can start in parallel with US1/US5
- **Polish (Phase 8)**: Depends on all user stories being complete

### User Story Dependencies

```
Phase 1: Setup
    │
Phase 2: Foundational
    │
    ├── US1 Ingest (P1) ──────┐
    │                         ├──→ US2 Query (P2)
    ├── US5 Search (P2) ──────┘
    │
    ├── US3 Lint (P3) ── independent
    │
    ├── US4 Graph (P3) ── independent
    │
    └──→ Phase 8: Polish (after all stories)
```

- **US1 (P1)**: Depends on Foundational only — **MVP target**
- **US5 (P2)**: Depends on Foundational only — can run in parallel with US1
- **US2 (P2)**: Depends on US1 (LLM module) + US5 (search engine)
- **US3 (P3)**: Depends on Foundational only — can run in parallel with US1/US5
- **US4 (P3)**: Depends on Foundational only — can run in parallel with US1/US5

### Within Each User Story

- Models/data structures before services
- Core modules before command wrappers
- LLM client before provider abstraction
- Story complete before moving to next priority

### Parallel Opportunities

- All Setup tasks marked [P] can run in parallel
- All Foundational tasks marked [P] can run in parallel (T005-T008)
- US1: T011-T014 and T017, T019 can all run in parallel (separate files)
- US4: T027-T028 can run in parallel (builder and relevance are independent)
- US5: T022 can run in parallel with other stories (isolated file)
- US1 and US5 can proceed simultaneously (no shared source files)
- US3 and US4 can run in parallel with each other and with US1/US5
- Polish: T031, T032, T034 can run in parallel (different files)

---

## Parallel Example: User Story 1

```bash
# Launch all independent US1 modules together:
Task T011: "SHA256-based ingest cache in wiki-tool/src/cache/ingest_cache.rs"
Task T012: "Persistent ingest queue in wiki-tool/src/cache/queue.rs"
Task T013: "index.md catalog generation in wiki-tool/src/wiki/index.rs"
Task T014: "log.md entry appending in wiki-tool/src/wiki/log.rs"
Task T017: "Prompt templates in wiki-tool/src/llm/prompts.rs"
Task T019: "Extract subcommand in wiki-tool/src/commands/extract.rs"

# Then sequential (dependencies):
Task T015: "Streaming LLM client in wiki-tool/src/llm/client.rs"
Task T016: "Multi-provider abstraction in wiki-tool/src/llm/providers.rs" (depends on T015)
Task T018: "Cache subcommand in wiki-tool/src/commands/cache.rs" (depends on T011)
Task T020: "Ingest pipeline command in wiki-tool/src/commands/ingest.rs" (depends on T011-T017)
Task T021: "Index rebuild subcommand in wiki-tool/src/commands/index.rs" (depends on T013)
```

## Parallel Example: Cross-Story

```bash
# After Foundational phase, these stories can start simultaneously:
# Developer A: US1 Ingest (T011-T021)
# Developer B: US5 Search (T022-T024)
# Developer C: US3 Lint (T026) or US4 Graph (T027-T030)

# US2 Query (T025) starts after both US1 and US5 complete
```

---

## Implementation Strategy

### MVP First (User Story 1 Only)

1. Complete Phase 1: Setup
2. Complete Phase 2: Foundational (CRITICAL — blocks all stories)
3. Complete Phase 3: User Story 1 — Ingest
4. **STOP and VALIDATE**: Run `wiki-tool init` → `wiki-tool ingest raw/test.md` → verify pages, index, log
5. Deploy/demo if ready — wiki can be populated and managed

### Incremental Delivery

1. Setup + Foundational → Foundation ready
2. Add US1 Ingest → Test independently → **MVP!** (wiki can be populated)
3. Add US5 Search → Test independently → Wiki is searchable
4. Add US2 Query → Test independently → Wiki answers questions with citations
5. Add US3 Lint → Test independently → Wiki quality is monitored
6. Add US4 Graph → Test independently → Knowledge graph with communities
7. Polish → Schema files, docs, JSON output, fixtures
8. Each story adds value without breaking previous stories

### Parallel Team Strategy

With multiple developers:

1. Team completes Setup + Foundational together
2. Once Foundational is done:
   - Developer A: US1 Ingest (P1)
   - Developer B: US5 Search (P2) + US3 Lint (P3)
   - Developer C: US4 Graph (P3)
3. After US1 + US5 complete:
   - Developer A or B: US2 Query (P2)
4. Final: Team collaborates on Polish phase

---

## Notes

- [P] tasks = different files, no dependencies on incomplete tasks
- [Story] label maps task to specific user story for traceability
- Each user story is independently completable and testable
- Commit after each task or logical group
- Stop at any checkpoint to validate story independently
- Dual-mode architecture: agent-companion commands (search, lint, graph, cache, extract, index) require no API key; standalone commands (ingest, query) require LLM config
- Performance targets from spec: search 500 pages <200ms, graph 500 nodes <1s, binary <20MB
