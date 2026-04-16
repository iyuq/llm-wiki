# wiki-tool — Schema for Gemini CLI

This project uses `wiki-tool`, a Rust CLI for building and maintaining
a personal knowledge base. You (Gemini CLI) can use it directly.

## Commands (no API key needed)

```bash
wiki-tool search "query"           # BM25 keyword search with CJK support
wiki-tool search --snippet "query" # Include content snippets
wiki-tool lint                     # Check wiki health (orphans, broken links)
wiki-tool lint --fix               # Auto-fix simple issues
wiki-tool graph --communities      # Build knowledge graph with clusters
wiki-tool graph --related "Page"   # Find pages related to a specific page
wiki-tool cache check raw/file.md  # Check if source is already ingested
wiki-tool extract raw/file.pdf     # Extract text from PDF/DOCX
wiki-tool index                    # Rebuild wiki/index.md
wiki-tool index --check            # Verify index is current
wiki-tool init                     # Initialize new wiki project
```

## Standalone Commands (requires .wiki-tool.toml with LLM config)

```bash
wiki-tool ingest raw/file.md       # Two-pass LLM ingest
wiki-tool query "question"         # LLM-synthesized answer with citations
wiki-tool query --save "question"  # Save answer as synthesis page
```

## Wiki Conventions

- **Page format**: Markdown with YAML frontmatter (title, type, tags, sources, last_updated)
- **Page types**: source, entity, concept, synthesis
- **Cross-references**: Use `[[PageName]]` wikilinks
- **Directory structure**: wiki/sources/, wiki/entities/, wiki/concepts/, wiki/syntheses/
- **Special files**: wiki/index.md (catalog), wiki/log.md (chronological log)
- **Raw sources**: Placed in raw/, never modified by the tool

## Ingest Workflow (Agent-Companion Mode)

When the user asks you to ingest a source:

1. Run `wiki-tool cache check raw/<file>` — skip if cached
2. Run `wiki-tool extract raw/<file>` — get plain text
3. Read the extracted text and existing wiki context (index.md, relevant pages)
4. Generate wiki pages with proper frontmatter and wikilinks
5. Write pages to wiki/sources/, wiki/entities/, wiki/concepts/ as appropriate
6. Run `wiki-tool index` to rebuild index.md
7. Append to wiki/log.md: `## [YYYY-MM-DD] ingest | <title>`
