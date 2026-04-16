use std::time::Instant;

use super::Context;
use wiki_tool::search::SearchEngine;

/// Run the search command.
pub fn run(ctx: &Context, query: &str, limit: usize, snippet: bool) -> wiki_tool::Result<()> {
    let wiki_dir = ctx.wiki_dir();
    let state_dir = ctx.state_dir();

    if !wiki_dir.exists() {
        return Err(wiki_tool::WikiToolError::Wiki(
            "Wiki directory not found. Run 'wiki-tool init' first.".to_string(),
        ));
    }

    let start = Instant::now();

    let engine = SearchEngine::build(&wiki_dir, &state_dir)?;
    let results = engine.search(query, limit, snippet)?;

    let query_ms = start.elapsed().as_millis();

    if ctx.json {
        let result_json: Vec<serde_json::Value> = results
            .iter()
            .map(|r| {
                let mut obj = serde_json::json!({
                    "page": r.path,
                    "title": r.title,
                    "score": r.score,
                    "type": r.page_type,
                });
                if snippet {
                    obj["snippet"] = serde_json::Value::String(r.snippet.clone());
                }
                obj
            })
            .collect();

        let output = serde_json::json!({
            "results": result_json,
            "total": results.len(),
            "query_ms": query_ms,
        });
        println!("{}", serde_json::to_string_pretty(&output)?);
    } else {
        if results.is_empty() {
            println!("No results found for: {}", query);
        } else {
            println!("Search results for '{}' ({} ms):\n", query, query_ms);
            for (i, r) in results.iter().enumerate() {
                println!(
                    "  {}. {} [{}] (score: {:.1})",
                    i + 1,
                    r.title,
                    r.page_type,
                    r.score
                );
                println!("     {}", r.path);
                if snippet && !r.snippet.is_empty() {
                    println!("     {}", r.snippet);
                }
                println!();
            }
        }
        if ctx.verbose {
            println!("Query time: {} ms", query_ms);
        }
    }

    Ok(())
}
