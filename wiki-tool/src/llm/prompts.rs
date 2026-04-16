/// Prompt templates for LLM-driven ingest and query operations.

/// Pass 1: Analyze a source document to extract entities, concepts, and contradictions.
pub fn ingest_pass1_system() -> &'static str {
    r#"You are a knowledge extraction agent. Analyze the given source document and produce a structured analysis.

Output a YAML block with:
- entities: list of named entities (people, organizations, projects, products) with brief descriptions
- concepts: list of key concepts, ideas, methods, or theories with brief descriptions
- contradictions: any claims that might conflict with common knowledge, with explanation
- tags: relevant categorization tags
- summary: one-paragraph summary of the document

Format:
```yaml
entities:
  - name: "Entity Name"
    description: "Brief description"
concepts:
  - name: "Concept Name"
    description: "Brief description"
contradictions:
  - claim: "The contradictory claim"
    context: "Why this might be contradictory"
tags: [tag1, tag2, tag3]
summary: "One paragraph summary"
```"#
}

/// Pass 1: User prompt with the source document content.
pub fn ingest_pass1_user(source_name: &str, content: &str) -> String {
    format!(
        "Analyze this source document: **{}**\n\n---\n\n{}",
        source_name, content
    )
}

/// Pass 2: Generate wiki pages from the analysis.
pub fn ingest_pass2_system() -> &'static str {
    r#"You are a wiki page generator. Given a source document analysis and the original content, generate wiki pages in markdown format with YAML frontmatter.

Generate these page types:
1. **Source page** (one per source): Summary with links to entities and concepts
2. **Entity pages** (one per entity): Detailed entry for each entity found
3. **Concept pages** (one per concept): Detailed entry for each concept found

Each page must use this format:
```
---FILE: <type>/<slug>.md---
---
title: "Page Title"
type: <source|entity|concept>
tags: [tag1, tag2]
sources: [source-slug]
last_updated: <today's date>
---

Page content with [[Wikilinks]] to related pages.
```

Rules:
- Use [[Wikilinks]] to connect pages to each other
- Each entity/concept page should reference its source(s) using [[Source Name]]
- Include relevant context and details from the source
- The source slug is the source filename without extension, lowercased with hyphens
- Use `---FILE: <path>---` as the delimiter between pages"#
}

/// Pass 2: User prompt with the analysis and original content.
pub fn ingest_pass2_user(
    source_name: &str,
    analysis: &str,
    content: &str,
    today: &str,
) -> String {
    format!(
        "Source document: **{}**\nToday's date: {}\n\n## Analysis\n\n{}\n\n## Original Content\n\n{}",
        source_name, today, analysis, content
    )
}

/// Query: System prompt for synthesizing an answer from wiki context.
pub fn query_system() -> &'static str {
    r#"You are a knowledge synthesis agent. Given context from a personal wiki, answer the user's question with citations.

Rules:
- Use [[Wikilinks]] to cite your sources (e.g., "According to [[Transformers]], ...")
- Only use information present in the provided context
- If the context doesn't contain enough information, say so clearly
- Structure your answer clearly with paragraphs and lists as appropriate
- Be concise but thorough"#
}

/// Query: User prompt with context pages and the question.
pub fn query_user(question: &str, context_pages: &[(String, String)]) -> String {
    let mut prompt = String::from("## Wiki Context\n\n");

    for (title, content) in context_pages {
        prompt.push_str(&format!("### {}\n\n{}\n\n---\n\n", title, content));
    }

    prompt.push_str(&format!("## Question\n\n{}", question));
    prompt
}

/// Parse `---FILE: path---` delimited blocks from LLM output.
pub fn parse_file_blocks(output: &str) -> Vec<(String, String)> {
    let mut files = Vec::new();
    let mut current_path: Option<String> = None;
    let mut current_content = String::new();

    for line in output.lines() {
        if let Some(path) = line.strip_prefix("---FILE:") {
            // Save previous block
            if let Some(ref path) = current_path {
                files.push((path.clone(), current_content.trim().to_string()));
            }
            current_path = Some(path.trim_end_matches("---").trim().to_string());
            current_content.clear();
        } else if current_path.is_some() {
            current_content.push_str(line);
            current_content.push('\n');
        }
    }

    // Save last block
    if let Some(path) = current_path {
        files.push((path, current_content.trim().to_string()));
    }

    files
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_file_blocks() {
        let output = r#"Here are the pages:

---FILE: sources/my-article.md---
---
title: "My Article"
type: source
tags: [test]
sources: []
last_updated: 2026-01-01
---

Summary of the article.

---FILE: entities/example-entity.md---
---
title: "Example Entity"
type: entity
tags: [test]
sources: [my-article]
last_updated: 2026-01-01
---

Details about the entity.
"#;

        let blocks = parse_file_blocks(output);
        assert_eq!(blocks.len(), 2);
        assert_eq!(blocks[0].0, "sources/my-article.md");
        assert!(blocks[0].1.contains("My Article"));
        assert_eq!(blocks[1].0, "entities/example-entity.md");
        assert!(blocks[1].1.contains("Example Entity"));
    }
}
