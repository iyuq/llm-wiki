use super::Context;
use wiki_tool::llm::client::LlmClient;
use wiki_tool::llm::prompts;
use wiki_tool::search::SearchEngine;
use wiki_tool::wiki::page::scan_wiki_pages;

/// Run the query command (standalone mode — requires LLM config).
pub async fn run(
    ctx: &Context,
    question: &str,
    save: bool,
    context_count: usize,
    no_stream: bool,
) -> wiki_tool::Result<()> {
    let wiki_dir = ctx.wiki_dir();
    let state_dir = ctx.state_dir();

    if !wiki_dir.exists() {
        return Err(wiki_tool::WikiToolError::Wiki(
            "Wiki directory not found. Run 'wiki-tool init' first.".to_string(),
        ));
    }

    // Search for relevant pages
    if !ctx.quiet && !ctx.json {
        println!("🔍 Searching for relevant pages...");
    }

    let engine = SearchEngine::build(&wiki_dir, &state_dir)?;
    let search_results = engine.search(question, context_count, false)?;

    if search_results.is_empty() {
        if ctx.json {
            let result = serde_json::json!({
                "answer": "No relevant pages found in the wiki.",
                "sources": [],
            });
            println!("{}", serde_json::to_string_pretty(&result)?);
        } else {
            println!("No relevant pages found in the wiki.");
        }
        return Ok(());
    }

    // Load pages and expand context via wikilink neighbors
    let all_pages = scan_wiki_pages(&wiki_dir)?;
    let mut context_pages: Vec<(String, String)> = Vec::new();
    let mut seen_paths = std::collections::HashSet::new();

    for result in &search_results {
        if seen_paths.contains(&result.path) {
            continue;
        }
        if let Some(page) = all_pages.iter().find(|p| p.path == result.path) {
            context_pages.push((page.title.clone(), page.content.clone()));
            seen_paths.insert(result.path.clone());

            // Add wikilink neighbors for context expansion
            for link in &page.wikilinks {
                if context_pages.len() >= context_count * 2 {
                    break;
                }
                let slug = wiki_tool::wiki::wikilinks::title_to_slug(link);
                if let Some(neighbor) = all_pages.iter().find(|p| {
                    let p_slug = std::path::Path::new(&p.path)
                        .file_stem()
                        .and_then(|s| s.to_str())
                        .unwrap_or("");
                    p_slug == slug && !seen_paths.contains(&p.path)
                }) {
                    context_pages.push((neighbor.title.clone(), neighbor.content.clone()));
                    seen_paths.insert(neighbor.path.clone());
                }
            }
        }
    }

    // Get LLM client
    let provider_config = ctx.config.active_provider()?;
    let client = LlmClient::new(provider_config)?;

    if !ctx.quiet && !ctx.json {
        println!(
            "📚 Using {} context pages. Synthesizing answer...\n",
            context_pages.len()
        );
    }

    // Generate answer
    let answer = if no_stream {
        client
            .complete(
                prompts::query_system(),
                &prompts::query_user(question, &context_pages),
            )
            .await?
    } else {
        client
            .complete_streaming(
                prompts::query_system(),
                &prompts::query_user(question, &context_pages),
                |chunk| {
                    if !ctx.json {
                        print!("{}", chunk);
                    }
                },
            )
            .await?
    };

    if !no_stream && !ctx.json {
        println!();
    }

    // Optionally save as synthesis page
    if save {
        let slug = wiki_tool::wiki::wikilinks::title_to_slug(question);
        let today = chrono::Local::now().format("%Y-%m-%d").to_string();
        let page_content = format!(
            "---\ntitle: \"{}\"\ntype: synthesis\ntags: [query]\nsources: []\nlast_updated: {}\n---\n\n{}\n",
            question, today, answer
        );
        let page_path = wiki_dir.join(format!("syntheses/{}.md", slug));
        if let Some(parent) = page_path.parent() {
            std::fs::create_dir_all(parent)?;
        }
        std::fs::write(&page_path, page_content)?;

        wiki_tool::wiki::index::write_index(&wiki_dir)?;
        wiki_tool::wiki::log::append_log(
            &wiki_dir,
            wiki_tool::wiki::log::LogOperation::Query,
            &format!("Saved synthesis: {}", question),
        )?;

        if !ctx.quiet && !ctx.json {
            println!("\n✓ Saved as wiki/syntheses/{}.md", slug);
        }
    }

    if ctx.json {
        let sources: Vec<String> = search_results.iter().map(|r| r.path.clone()).collect();
        let result = serde_json::json!({
            "answer": answer,
            "sources": sources,
            "context_pages": context_pages.len(),
        });
        println!("{}", serde_json::to_string_pretty(&result)?);
    }

    Ok(())
}
