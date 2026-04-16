# Quickstart: LLM Wiki Agent (wiki-tool)

## Installation

```bash
# Build from source
cd wiki-tool
cargo build --release

# Binary at target/release/wiki-tool
# Copy to PATH or use directly
```

## Setup

```bash
# Initialize a new wiki project
wiki-tool init

# This creates:
#   raw/          — drop your source documents here
#   wiki/         — LLM-maintained wiki (don't edit manually)
#   .wiki-tool.toml — configuration
#   CLAUDE.md, AGENTS.md, GEMINI.md, COPILOT.md — agent schemas
```

## Configure LLM Provider

Edit `.wiki-tool.toml`:

```toml
[llm]
provider = "anthropic"  # or "openai", "ollama", "custom"

[llm.providers.anthropic]
api_url = "https://api.anthropic.com/v1/messages"
model = "claude-sonnet-4-20250514"
max_tokens = 8192
timeout_secs = 900

[llm.providers.openai]
api_url = "https://api.openai.com/v1/chat/completions"
model = "gpt-4o"
max_tokens = 8192

[llm.providers.ollama]
api_url = "http://localhost:11434/api/chat"
model = "llama3"
max_tokens = 4096
```

Set your API key:
```bash
export ANTHROPIC_API_KEY="sk-ant-..."
# or
export OPENAI_API_KEY="sk-..."
```

## Basic Workflow

### 1. Add a source and ingest it

```bash
# Copy a document to raw/
cp ~/documents/interesting-article.md raw/

# Ingest it into the wiki
wiki-tool ingest raw/interesting-article.md
```

The tool will:
- Analyze the document (Pass 1)
- Generate wiki pages (Pass 2)
- Update index.md and log.md
- Report created/updated pages

### 2. Search the wiki

```bash
wiki-tool search "transformer architecture"
wiki-tool search "注意力机制"  # CJK supported
```

### 3. Ask questions

```bash
wiki-tool query "What are the key differences between GPT and BERT?"

# Save the answer as a wiki page
wiki-tool query --save "Compare all attention mechanisms"
```

### 4. Check wiki health

```bash
wiki-tool lint
wiki-tool lint --fix  # auto-fix simple issues
```

### 5. Build knowledge graph

```bash
wiki-tool graph --communities
# Outputs graph.json with nodes, edges, and community clusters
```

## Using with Coding Agents

The schema files tell your coding agent how to use wiki-tool:

- **Claude Code**: Reads `CLAUDE.md` automatically
- **Codex/OpenCode**: Reads `AGENTS.md`
- **Gemini CLI**: Reads `GEMINI.md`
- **Copilot CLI**: Reads `COPILOT.md`

Just open the project in your preferred agent and say:
- "Ingest raw/my-paper.pdf"
- "What does the wiki say about X?"
- "Lint the wiki and fix any issues"
- "Build the knowledge graph"

The agent reads the schema and invokes `wiki-tool` commands.
