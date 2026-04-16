use super::Context;

/// Run the index command.
pub fn run(ctx: &Context, check: bool) -> wiki_tool::Result<()> {
    let wiki_dir = ctx.wiki_dir();

    if !wiki_dir.exists() {
        return Err(wiki_tool::WikiToolError::Wiki(
            "Wiki directory not found. Run 'wiki-tool init' first.".to_string(),
        ));
    }

    if check {
        let is_current = wiki_tool::wiki::index::check_index(&wiki_dir)?;
        if ctx.json {
            let result = serde_json::json!({
                "up_to_date": is_current,
            });
            println!("{}", serde_json::to_string_pretty(&result)?);
        } else if is_current {
            println!("✓ index.md is up-to-date.");
        } else {
            println!("✗ index.md is stale. Run 'wiki-tool index' to rebuild.");
        }
        if !is_current {
            std::process::exit(1);
        }
    } else {
        wiki_tool::wiki::index::write_index(&wiki_dir)?;
        if ctx.json {
            let result = serde_json::json!({
                "rebuilt": true,
            });
            println!("{}", serde_json::to_string_pretty(&result)?);
        } else if !ctx.quiet {
            println!("✓ Rebuilt wiki/index.md");
        }

        // Log the rebuild
        wiki_tool::wiki::log::append_log(
            &wiki_dir,
            wiki_tool::wiki::log::LogOperation::Rebuild,
            "Rebuilt index.md",
        )?;
    }

    Ok(())
}
