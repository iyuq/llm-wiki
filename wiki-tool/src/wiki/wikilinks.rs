use regex::Regex;
use std::sync::LazyLock;

/// Regex pattern for extracting `[[wikilinks]]` from markdown content.
static WIKILINK_RE: LazyLock<Regex> =
    LazyLock::new(|| Regex::new(r"\[\[([^\]]+)\]\]").expect("Invalid wikilink regex"));

/// Extract all `[[wikilinks]]` from markdown content.
/// Returns a deduplicated list of wikilink targets.
pub fn extract_wikilinks(content: &str) -> Vec<String> {
    let mut links: Vec<String> = WIKILINK_RE
        .captures_iter(content)
        .map(|cap| cap[1].trim().to_string())
        .collect();
    links.sort();
    links.dedup();
    links
}

/// Convert a page title to a filename slug.
///
/// Rules:
/// - Lowercase
/// - Replace spaces and special chars with hyphens
/// - Remove consecutive hyphens
/// - Strip leading/trailing hyphens
pub fn title_to_slug(title: &str) -> String {
    let slug: String = title
        .to_lowercase()
        .chars()
        .map(|c| {
            if c.is_alphanumeric() || c == '-' || c == '_' {
                c
            } else {
                '-'
            }
        })
        .collect();

    // Collapse consecutive hyphens
    let mut result = String::new();
    let mut prev_hyphen = false;
    for c in slug.chars() {
        if c == '-' {
            if !prev_hyphen {
                result.push(c);
            }
            prev_hyphen = true;
        } else {
            result.push(c);
            prev_hyphen = false;
        }
    }

    result.trim_matches('-').to_string()
}

/// Resolve a wikilink target to a filename.
/// `[[Transformer Architecture]]` → `transformer-architecture.md`
pub fn resolve_wikilink(target: &str) -> String {
    format!("{}.md", title_to_slug(target))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_extract_wikilinks() {
        let content = "See [[Transformers]] and [[GPT]] for details. Also [[Transformers]] again.";
        let links = extract_wikilinks(content);
        assert_eq!(links, vec!["GPT", "Transformers"]);
    }

    #[test]
    fn test_title_to_slug() {
        assert_eq!(title_to_slug("Transformer Architecture"), "transformer-architecture");
        assert_eq!(title_to_slug("GPT-4"), "gpt-4");
        assert_eq!(title_to_slug("Hello   World"), "hello-world");
    }

    #[test]
    fn test_resolve_wikilink() {
        assert_eq!(
            resolve_wikilink("Transformer Architecture"),
            "transformer-architecture.md"
        );
    }
}
