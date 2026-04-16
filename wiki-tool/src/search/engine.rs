use std::path::Path;
use tantivy::collector::TopDocs;
use tantivy::query::QueryParser;
use tantivy::schema::*;
use tantivy::{doc, Index, IndexWriter, TantivyDocument};

use crate::search::tokenizer;
use crate::wiki::page::scan_wiki_pages;

/// Search result entry.
#[derive(Debug, Clone)]
pub struct SearchResult {
    pub path: String,
    pub title: String,
    pub score: f32,
    pub snippet: String,
    pub page_type: String,
}

/// Tantivy-based BM25 search engine.
pub struct SearchEngine {
    index: Index,
    #[allow(dead_code)]
    schema: Schema,
    title_field: Field,
    content_field: Field,
    path_field: Field,
    page_type_field: Field,
    tags_field: Field,
}

impl SearchEngine {
    /// Build a search index from all wiki pages.
    pub fn build(wiki_root: &Path, state_dir: &Path) -> crate::Result<Self> {
        let mut schema_builder = Schema::builder();

        let text_options = TextOptions::default()
            .set_indexing_options(
                TextFieldIndexing::default()
                    .set_tokenizer("default")
                    .set_index_option(IndexRecordOption::WithFreqsAndPositions),
            )
            .set_stored();

        let title_field = schema_builder.add_text_field("title", text_options.clone());
        let content_field = schema_builder.add_text_field("content", text_options.clone());
        let path_field = schema_builder.add_text_field("path", STORED);
        let page_type_field = schema_builder.add_text_field("page_type", STORED | STRING);
        let tags_field = schema_builder.add_text_field("tags", text_options);

        let schema = schema_builder.build();

        // Create index in state directory
        let index_dir = state_dir.join("search-index");
        std::fs::create_dir_all(&index_dir)
            .map_err(|e| crate::WikiToolError::Search(format!("Failed to create index dir: {}", e)))?;

        let index = Index::create_in_dir(&index_dir, schema.clone())
            .or_else(|_| {
                // If index exists, try opening and recreating
                let _ = std::fs::remove_dir_all(&index_dir);
                std::fs::create_dir_all(&index_dir)
                    .map_err(|e| crate::WikiToolError::Search(format!("Failed to recreate index dir: {}", e)))?;
                Index::create_in_dir(&index_dir, schema.clone())
                    .map_err(|e| crate::WikiToolError::Search(format!("Failed to create index: {}", e)))
            })?;

        let engine = Self {
            index,
            schema,
            title_field,
            content_field,
            path_field,
            page_type_field,
            tags_field,
        };

        // Index all wiki pages
        engine.index_pages(wiki_root)?;

        Ok(engine)
    }

    fn index_pages(&self, wiki_root: &Path) -> crate::Result<()> {
        let pages = scan_wiki_pages(wiki_root)?;

        let mut writer: IndexWriter = self
            .index
            .writer(50_000_000)
            .map_err(|e| crate::WikiToolError::Search(format!("Failed to create writer: {}", e)))?;

        for page in &pages {
            // For CJK content, also add tokenized bigrams to aid search
            let mut indexed_content = page.content.clone();
            let cjk_tokens = tokenizer::tokenize(&page.content);
            let cjk_only: Vec<&String> = cjk_tokens
                .iter()
                .filter(|t| t.chars().any(tokenizer::is_cjk))
                .collect();
            if !cjk_only.is_empty() {
                indexed_content.push(' ');
                indexed_content.push_str(&cjk_only.iter().map(|s| s.as_str()).collect::<Vec<_>>().join(" "));
            }

            let mut title_content = page.title.clone();
            let title_cjk = tokenizer::tokenize(&page.title);
            let title_cjk_only: Vec<&String> = title_cjk
                .iter()
                .filter(|t| t.chars().any(tokenizer::is_cjk))
                .collect();
            if !title_cjk_only.is_empty() {
                title_content.push(' ');
                title_content.push_str(&title_cjk_only.iter().map(|s| s.as_str()).collect::<Vec<_>>().join(" "));
            }

            writer
                .add_document(doc!(
                    self.title_field => title_content,
                    self.content_field => indexed_content,
                    self.path_field => page.path.clone(),
                    self.page_type_field => page.page_type.to_string(),
                    self.tags_field => page.tags.join(" "),
                ))
                .map_err(|e| crate::WikiToolError::Search(format!("Failed to add document: {}", e)))?;
        }

        writer.commit()
            .map_err(|e| crate::WikiToolError::Search(format!("Failed to commit: {}", e)))?;

        Ok(())
    }

    /// Search for pages matching the query.
    pub fn search(&self, query_str: &str, limit: usize, include_snippet: bool) -> crate::Result<Vec<SearchResult>> {
        let reader = self
            .index
            .reader()
            .map_err(|e| crate::WikiToolError::Search(format!("Failed to create reader: {}", e)))?;

        let searcher = reader.searcher();

        // Tokenize query for CJK support
        let tokens = tokenizer::tokenize(query_str);
        let expanded_query = if tokens.iter().any(|t| t.chars().any(tokenizer::is_cjk)) {
            tokens.join(" ")
        } else {
            query_str.to_string()
        };

        // Use title and content fields for search, title boosted
        let query_parser = QueryParser::for_index(
            &self.index,
            vec![self.title_field, self.content_field, self.tags_field],
        );

        let query = query_parser.parse_query(&expanded_query).map_err(|e| {
            crate::WikiToolError::Search(format!("Failed to parse query: {}", e))
        })?;

        let top_docs = searcher
            .search(&query, &TopDocs::with_limit(limit))
            .map_err(|e| crate::WikiToolError::Search(format!("Search failed: {}", e)))?;

        let mut results = Vec::new();

        for (score, doc_address) in top_docs {
            let doc: TantivyDocument = searcher.doc(doc_address)
                .map_err(|e| crate::WikiToolError::Search(format!("Failed to get doc: {}", e)))?;

            let title = doc
                .get_first(self.title_field)
                .and_then(|v| v.as_str())
                .unwrap_or("")
                .to_string();
            let path = doc
                .get_first(self.path_field)
                .and_then(|v| v.as_str())
                .unwrap_or("")
                .to_string();
            let page_type = doc
                .get_first(self.page_type_field)
                .and_then(|v| v.as_str())
                .unwrap_or("")
                .to_string();
            let content = doc
                .get_first(self.content_field)
                .and_then(|v| v.as_str())
                .unwrap_or("")
                .to_string();

            let snippet = if include_snippet {
                generate_snippet(&content, query_str, 150)
            } else {
                String::new()
            };

            results.push(SearchResult {
                path,
                title,
                score,
                snippet,
                page_type,
            });
        }

        Ok(results)
    }
}

/// Generate a snippet around the first occurrence of the query.
fn generate_snippet(content: &str, query: &str, max_len: usize) -> String {
    let lower_content = content.to_lowercase();
    let lower_query = query.to_lowercase();

    // Find best match position
    let pos = lower_content
        .find(&lower_query)
        .or_else(|| {
            // Try individual words
            lower_query
                .split_whitespace()
                .filter_map(|word| lower_content.find(word))
                .min()
        })
        .unwrap_or(0);

    // Extract snippet around match
    let start = pos.saturating_sub(max_len / 3);
    let end = (pos + max_len).min(content.len());

    // Align to word boundaries
    let snippet_start = if start == 0 {
        0
    } else {
        content[start..]
            .find(' ')
            .map(|i| start + i + 1)
            .unwrap_or(start)
    };

    let snippet_end = if end >= content.len() {
        content.len()
    } else {
        content[..end]
            .rfind(' ')
            .unwrap_or(end)
    };

    let mut snippet = content[snippet_start..snippet_end].to_string();

    if snippet_start > 0 {
        snippet = format!("...{}", snippet);
    }
    if snippet_end < content.len() {
        snippet.push_str("...");
    }

    // Clean up whitespace
    snippet
        .lines()
        .map(|l| l.trim())
        .filter(|l| !l.is_empty())
        .collect::<Vec<_>>()
        .join(" ")
}
