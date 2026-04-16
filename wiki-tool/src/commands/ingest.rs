use std::path::Path;

use super::Context;
use wiki_tool::cache::IngestCache;
use wiki_tool::llm::client::LlmClient;
use wiki_tool::llm::prompts;
use wiki_tool::wiki::log::{append_log, LogOperation};
use wiki_tool::wiki::index::write_index;

/// Run the ingest command (standalone mode — requires LLM config).
pub async fn run(
    ctx: &Context,
    source_path: &Path,
    force: bool,
    dry_run: bool,
    no_stream: bool,
) -> wiki_tool::Result<()> {
    let full_path = if source_path.is_relative() {
        ctx.project_root.join(source_path)
    } else {
        source_path.to_path_buf()
    };

    if !full_path.exists() {
        return Err(wiki_tool::WikiToolError::FileNotFound(
            full_path.display().to_string(),
        ));
    }

    let state_dir = ctx.state_dir();
    let wiki_dir = ctx.wiki_dir();
    let path_str = source_path.to_string_lossy().to_string();

    // Cache check
    let mut cache = IngestCache::load(&state_dir)?;
    let current_hash = IngestCache::compute_hash(&full_path)?;

    if !force {
        if let Some(entry) = cache.lookup(&path_str, &current_hash) {
            if ctx.json {
                let result = serde_json::json!({
                    "source": path_str,
                    "cached": true,
                    "hash": entry.hash,
                    "files": entry.files_written,
                });
                println!("{}", serde_json::to_string_pretty(&result)?);
            } else if !ctx.quiet {
                println!("⚡ Skipping {} (cached, hash: {})", path_str, &entry.hash[..12]);
            }
            return Ok(());
        }
    }

    // Extract source content
    let content = wiki_tool::extract::extract_text(&full_path)?;
    let source_name = full_path
        .file_stem()
        .and_then(|s| s.to_str())
        .unwrap_or("unknown");

    if !ctx.quiet && !ctx.json {
        println!("📄 Ingesting: {}", path_str);
    }

    // Get LLM client
    let provider_config = ctx.config.active_provider()?;
    let client = LlmClient::new(provider_config)?;

    // Pass 1: Analysis
    if !ctx.quiet && !ctx.json {
        println!("  Pass 1: Analyzing...");
    }

    let analysis = if no_stream {
        client
            .complete(prompts::ingest_pass1_system(), &prompts::ingest_pass1_user(source_name, &content))
            .await?
    } else {
        client
            .complete_streaming(
                prompts::ingest_pass1_system(),
                &prompts::ingest_pass1_user(source_name, &content),
                |chunk| {
                    if ctx.verbose {
                        eprint!("{}", chunk);
                    }
                },
            )
            .await?
    };

    if ctx.verbose && !no_stream {
        eprintln!();
    }

    // Pass 2: Generation
    if !ctx.quiet && !ctx.json {
        println!("  Pass 2: Generating wiki pages...");
    }

    let today = chrono::Local::now().format("%Y-%m-%d").to_string();
    let generation = if no_stream {
        client
            .complete(
                prompts::ingest_pass2_system(),
                &prompts::ingest_pass2_user(source_name, &analysis, &content, &today),
            )
            .await?
    } else {
        client
            .complete_streaming(
                prompts::ingest_pass2_system(),
                &prompts::ingest_pass2_user(source_name, &analysis, &content, &today),
                |chunk| {
                    if ctx.verbose {
                        eprint!("{}", chunk);
                    }
                },
            )
            .await?
    };

    if ctx.verbose && !no_stream {
        eprintln!();
    }

    // Parse generated file blocks
    let file_blocks = prompts::parse_file_blocks(&generation);

    if file_blocks.is_empty() {
        return Err(wiki_tool::WikiToolError::Parse(
            "LLM output contained no ---FILE: blocks".to_string(),
        ));
    }

    if dry_run {
        if ctx.json {
            let result = serde_json::json!({
                "source": path_str,
                "dry_run": true,
                "pages_would_create": file_blocks.iter().map(|(p, _)| p).collect::<Vec<_>>(),
            });
            println!("{}", serde_json::to_string_pretty(&result)?);
        } else {
            println!("\n  Dry run — would create:");
            for (path, _) in &file_blocks {
                println!("    wiki/{}", path);
            }
        }
        return Ok(());
    }

    // Write wiki pages
    let mut pages_created = Vec::new();
    let mut pages_updated = Vec::new();

    for (relative_path, content) in &file_blocks {
        let page_path = wiki_dir.join(relative_path);
        let is_update = page_path.exists();

        if let Some(parent) = page_path.parent() {
            std::fs::create_dir_all(parent)?;
        }
        std::fs::write(&page_path, content)?;

        let display_path = format!("wiki/{}", relative_path);
        if is_update {
            pages_updated.push(display_path);
        } else {
            pages_created.push(display_path);
        }
    }

    // Update index.md
    write_index(&wiki_dir)?;

    // Append log entry
    let log_msg = format!(
        "Ingested {} → {} created, {} updated",
        path_str,
        pages_created.len(),
        pages_updated.len()
    );
    append_log(&wiki_dir, LogOperation::Ingest, &log_msg)?;

    // Update cache
    let files_written: Vec<String> = file_blocks
        .iter()
        .map(|(p, _)| format!("wiki/{}", p))
        .collect();
    cache.insert(path_str.clone(), current_hash, files_written);
    cache.save(&state_dir)?;

    // Report results
    if ctx.json {
        let result = serde_json::json!({
            "source": path_str,
            "cached": false,
            "pages_created": pages_created,
            "pages_updated": pages_updated,
            "review_items": [],
        });
        println!("{}", serde_json::to_string_pretty(&result)?);
    } else if !ctx.quiet {
        println!("\n✓ Ingest complete:");
        for p in &pages_created {
            println!("  + {}", p);
        }
        for p in &pages_updated {
            println!("  ~ {}", p);
        }
    }

    Ok(())
}
