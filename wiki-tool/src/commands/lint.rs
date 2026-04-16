use std::collections::{HashMap, HashSet};

use super::Context;
use wiki_tool::wiki::page::scan_wiki_pages;
use wiki_tool::wiki::wikilinks::{extract_wikilinks, title_to_slug};

/// A lint issue found in the wiki.
#[derive(Debug, serde::Serialize)]
struct LintIssue {
    category: String,
    file: String,
    line: usize,
    message: String,
    suggestion: String,
}

/// Run the lint command.
pub fn run(ctx: &Context, fix: bool, category: Option<&str>) -> wiki_tool::Result<()> {
    let wiki_dir = ctx.wiki_dir();

    if !wiki_dir.exists() {
        return Err(wiki_tool::WikiToolError::Wiki(
            "Wiki directory not found. Run 'wiki-tool init' first.".to_string(),
        ));
    }

    let pages = scan_wiki_pages(&wiki_dir)?;
    let mut issues: Vec<LintIssue> = Vec::new();

    // Build page slug lookup
    let page_slugs: HashSet<String> = pages
        .iter()
        .filter_map(|p| {
            std::path::Path::new(&p.path)
                .file_stem()
                .and_then(|s| s.to_str())
                .map(|s| s.to_string())
        })
        .collect();

    // Build inbound link map
    let mut inbound_links: HashMap<String, Vec<String>> = HashMap::new();
    for slug in &page_slugs {
        inbound_links.insert(slug.clone(), Vec::new());
    }
    for page in &pages {
        let source_slug = std::path::Path::new(&page.path)
            .file_stem()
            .and_then(|s| s.to_str())
            .unwrap_or("")
            .to_string();

        for link in &page.wikilinks {
            let target_slug = title_to_slug(link);
            inbound_links
                .entry(target_slug)
                .or_default()
                .push(source_slug.clone());
        }
    }

    // Check each page
    for page in &pages {
        let page_slug = std::path::Path::new(&page.path)
            .file_stem()
            .and_then(|s| s.to_str())
            .unwrap_or("")
            .to_string();

        let full_path = wiki_dir.join(&page.path);
        let content = std::fs::read_to_string(&full_path).unwrap_or_default();
        let lines: Vec<&str> = content.lines().collect();

        // Check for broken wikilinks
        if category.is_none() || category == Some("broken-links") {
            for (line_num, line) in lines.iter().enumerate() {
                let links = extract_wikilinks(line);
                for link in &links {
                    let target_slug = title_to_slug(link);
                    if !page_slugs.contains(&target_slug) {
                        issues.push(LintIssue {
                            category: "broken-link".to_string(),
                            file: page.path.clone(),
                            line: line_num + 1,
                            message: format!("[[{}]] links to missing page", link),
                            suggestion: format!(
                                "Create wiki page for '{}' or fix the link",
                                link
                            ),
                        });
                    }
                }
            }
        }

        // Check for orphan pages (no inbound links)
        if category.is_none() || category == Some("orphans") {
            let inbound = inbound_links.get(&page_slug);
            let has_inbound = inbound.is_some_and(|links| !links.is_empty());
            if !has_inbound {
                issues.push(LintIssue {
                    category: "orphan".to_string(),
                    file: page.path.clone(),
                    line: 1,
                    message: format!("'{}' has no inbound wikilinks", page.title),
                    suggestion: "Add [[wikilinks]] from other pages or consider removing".to_string(),
                });
            }
        }

        // Check for stale content
        if category.is_none() || category == Some("stale") {
            if page.last_updated.is_empty() {
                issues.push(LintIssue {
                    category: "stale".to_string(),
                    file: page.path.clone(),
                    line: 1,
                    message: "Missing last_updated date in frontmatter".to_string(),
                    suggestion: "Add last_updated field to frontmatter".to_string(),
                });
            }
        }
    }

    // Check for missing pages (referenced but don't exist)
    if category.is_none() || category == Some("missing-pages") {
        let mut missing: HashSet<String> = HashSet::new();
        for page in &pages {
            for link in &page.wikilinks {
                let target_slug = title_to_slug(link);
                if !page_slugs.contains(&target_slug) {
                    missing.insert(link.clone());
                }
            }
        }
        // These are already reported as broken-links on a per-location basis,
        // but this gives a summary of unique missing pages
        for link in &missing {
            let _target_slug = title_to_slug(link);
            if !issues.iter().any(|i| i.category == "missing-page" && i.message.contains(link)) {
                issues.push(LintIssue {
                    category: "missing-page".to_string(),
                    file: String::new(),
                    line: 0,
                    message: format!("Page '{}' is referenced but does not exist", link),
                    suggestion: format!(
                        "Create the page or update references",
                    ),
                });
            }
        }
    }

    // Auto-fix if requested
    if fix {
        // Rebuild index
        wiki_tool::wiki::index::write_index(&wiki_dir)?;
        if !ctx.quiet && !ctx.json {
            println!("✓ Rebuilt index.md");
        }
    }

    // Report
    if ctx.json {
        let result = serde_json::json!({
            "issues": issues,
            "total": issues.len(),
        });
        println!("{}", serde_json::to_string_pretty(&result)?);
    } else if issues.is_empty() {
        println!("✓ No issues found.");
    } else {
        println!("Found {} issue(s):\n", issues.len());
        for issue in &issues {
            if issue.file.is_empty() {
                println!("  [{}] {}", issue.category, issue.message);
            } else {
                println!(
                    "  [{}] {}:{} — {}",
                    issue.category, issue.file, issue.line, issue.message
                );
            }
            println!("    → {}", issue.suggestion);
            println!();
        }
    }

    if !issues.is_empty() && !ctx.json {
        std::process::exit(1);
    }

    Ok(())
}
