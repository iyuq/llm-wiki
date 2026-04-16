# Copilot Instructions — LLM Wiki

## Repository Overview

This is a monorepo containing two related projects around the "LLM Wiki" pattern — using LLMs to incrementally build and maintain a persistent, interlinked personal knowledge base from source documents.

### `doc/llm_wiki/` — Desktop Application (Tauri + React)

A cross-platform desktop app implementing the LLM Wiki pattern. The frontend is React 19 + TypeScript + Vite; the backend is Rust via Tauri 2.

**Build & run (from `doc/llm_wiki/`):**

```bash
npm install
npm run dev          # Vite dev server (frontend only)
npm run build        # tsc && vite build
npm run test         # vitest run (all tests)
npx vitest run src/lib/__tests__/llm-providers.test.ts  # single test file
npm run tauri dev    # full Tauri app (frontend + Rust backend)
cargo build          # Rust backend only (from src-tauri/)
```

**CI** runs on `doc/llm_wiki/.github/workflows/ci.yml`: frontend build (`npx vite build`) + Rust build (`cargo build`) across macOS, Ubuntu, Windows.

### `doc/llm-wiki-agent/` — Coding Agent Skill

A standalone agent skill (no build step) that works with Claude Code, Codex/OpenCode, or Gemini CLI. Drop sources into `raw/`, use slash commands (`/wiki-ingest`, `/wiki-query`, `/wiki-lint`, `/wiki-graph`). Schema files: `CLAUDE.md`, `AGENTS.md`, `GEMINI.md`. Python tools in `tools/` require `ANTHROPIC_API_KEY`.

### `doc/llm-wiki.md` — Karpathy's Original Pattern

The abstract design document describing the LLM Wiki concept. This repo implements it concretely.

## Architecture — Desktop App (`doc/llm_wiki/`)

**Three-layer data model:**
1. **Raw sources** (`raw/`) — immutable user documents. Never modified by the app.
2. **Wiki** (`wiki/`) — LLM-generated markdown with YAML frontmatter. Pages are typed: `source`, `entity`, `concept`, `synthesis`. Cross-referenced via `[[wikilinks]]`.
3. **Schema** (`schema.md` + `purpose.md`) — rules for how the wiki is structured and why it exists.

**Frontend (React + TypeScript):**
- State management: Zustand stores in `src/stores/` — `wiki-store.ts` (main app state), `chat-store.ts`, `research-store.ts`, `review-store.ts`, `activity-store.ts`
- Core logic in `src/lib/` — ingest pipeline (`ingest.ts`, `ingest-queue.ts`, `ingest-cache.ts`), search (`search.ts`), graph (`wiki-graph.ts`, `graph-relevance.ts`, `graph-insights.ts`), LLM client (`llm-client.ts`, `llm-providers.ts`)
- UI: shadcn/ui components (`src/components/ui/`), layout components (`src/components/layout/`), feature panels (editor, graph, chat, lint, review, search, settings)
- i18n: English and Chinese (`src/i18n/`)

**Backend (Rust/Tauri):**
- Tauri commands exposed in `src-tauri/src/commands/` — file system ops (`fs.rs`), project management (`project.rs`), vector store via LanceDB (`vectorstore.rs`)
- Clip server (`clip_server.rs`) — local HTTP server for Chrome extension web clipping
- File preprocessing: PDF (`pdf-extract`), DOCX (`docx-rs`), Excel (`calamine`), ZIP extraction

**Ingest pipeline** is two-step chain-of-thought: Step 1 (analysis) reads source + wiki context → structured analysis; Step 2 (generation) takes analysis → wiki page files. SHA256 cache skips unchanged sources. Persistent queue with crash recovery.

**Knowledge graph** uses graphology + sigma.js with a 4-signal relevance model (direct links ×3.0, source overlap ×4.0, Adamic-Adar ×1.5, type affinity ×1.0) and Louvain community detection.

## Key Conventions

- Every wiki page has YAML frontmatter with `title`, `type`, `tags`, `sources[]`, `last_updated`
- Wiki page naming: source pages use `kebab-case.md`, entity/concept pages use `TitleCase.md`
- `[[wikilinks]]` are the standard cross-reference format throughout
- `wiki/index.md` is the content catalog (updated on every ingest); `wiki/log.md` is the append-only chronological record with format `## [YYYY-MM-DD] <operation> | <title>`
- Tauri commands are the bridge between frontend and backend — defined in Rust with `#[tauri::command]`, invoked from TypeScript via `@tauri-apps/api`
- App state persisted via `@tauri-apps/plugin-store` in `app-state.json`

## Spec Kit Integration

This repo uses [Spec Kit](https://github.com/Luro02/spec-kit) (`.specify/` directory) for feature specification workflows. Spec Kit agents and prompts live in `.github/agents/` and `.github/prompts/`. The constitution template in `.specify/memory/constitution.md` is a placeholder — not yet configured for this project.

## Active Technologies
- Rust 1.75+ (2021 edition) (001-llm-wiki-agent-rust)
- File-based (JSON cache files, markdown wiki pages) (001-llm-wiki-agent-rust)

## Recent Changes
- 001-llm-wiki-agent-rust: Added Rust 1.75+ (2021 edition)
