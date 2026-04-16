use std::path::Path;

use super::Context;
use wiki_tool::cache::IngestCache;

/// Run `cache check <SOURCE_PATH>`.
pub fn run_check(ctx: &Context, source_path: &Path) -> wiki_tool::Result<()> {
    let state_dir = ctx.state_dir();
    let cache = IngestCache::load(&state_dir)?;

    let path_str = source_path.to_string_lossy().to_string();
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

    let current_hash = IngestCache::compute_hash(&full_path)?;

    if ctx.json {
        if let Some(entry) = cache.lookup(&path_str, &current_hash) {
            let result = serde_json::json!({
                "cached": true,
                "hash": entry.hash,
                "files": entry.files_written,
                "timestamp": entry.timestamp,
            });
            println!("{}", serde_json::to_string_pretty(&result)?);
        } else if let Some(entry) = cache.get(&path_str) {
            let result = serde_json::json!({
                "cached": false,
                "reason": "content_changed",
                "old_hash": entry.hash,
                "new_hash": current_hash,
            });
            println!("{}", serde_json::to_string_pretty(&result)?);
        } else {
            let result = serde_json::json!({
                "cached": false,
                "reason": "not_ingested",
                "hash": current_hash,
            });
            println!("{}", serde_json::to_string_pretty(&result)?);
        }
    } else if let Some(entry) = cache.lookup(&path_str, &current_hash) {
        println!("cached (hash: {})", &entry.hash[..12]);
        println!("Files: {}", entry.files_written.join(", "));
    } else if cache.get(&path_str).is_some() {
        println!("not-cached (content changed, hash: {})", &current_hash[..12]);
    } else {
        println!("not-cached (hash: {})", &current_hash[..12]);
    }

    Ok(())
}

/// Run `cache list`.
pub fn run_list(ctx: &Context) -> wiki_tool::Result<()> {
    let state_dir = ctx.state_dir();
    let cache = IngestCache::load(&state_dir)?;

    if ctx.json {
        let entries: Vec<serde_json::Value> = cache
            .entries
            .iter()
            .map(|(path, entry)| {
                serde_json::json!({
                    "source": path,
                    "hash": entry.hash,
                    "timestamp": entry.timestamp,
                    "files": entry.files_written,
                })
            })
            .collect();
        let result = serde_json::json!({ "cached_sources": entries });
        println!("{}", serde_json::to_string_pretty(&result)?);
    } else if cache.entries.is_empty() {
        println!("No cached sources.");
    } else {
        println!("Cached sources:");
        let mut sources: Vec<_> = cache.entries.iter().collect();
        sources.sort_by_key(|(k, _)| (*k).clone());
        for (path, entry) in sources {
            println!(
                "  {} (hash: {}, {} files)",
                path,
                &entry.hash[..12],
                entry.files_written.len()
            );
        }
    }

    Ok(())
}

/// Run `cache clear [SOURCE_PATH]`.
pub fn run_clear(ctx: &Context, source_path: Option<&Path>) -> wiki_tool::Result<()> {
    let state_dir = ctx.state_dir();
    let mut cache = IngestCache::load(&state_dir)?;

    if let Some(path) = source_path {
        let path_str = path.to_string_lossy().to_string();
        if cache.remove(&path_str) {
            cache.save(&state_dir)?;
            if !ctx.quiet {
                println!("Cleared cache for: {}", path_str);
            }
        } else if !ctx.quiet {
            println!("No cache entry found for: {}", path_str);
        }
    } else {
        let count = cache.entries.len();
        cache.clear_all();
        cache.save(&state_dir)?;
        if !ctx.quiet {
            println!("Cleared {} cache entries.", count);
        }
    }

    Ok(())
}
