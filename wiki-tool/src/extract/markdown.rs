use std::path::Path;

/// Extract plain text from a Markdown file.
/// Strips YAML frontmatter and returns the body content.
pub fn extract(path: &Path) -> crate::Result<String> {
    let raw = std::fs::read_to_string(path)?;
    Ok(strip_frontmatter(&raw))
}

/// Strip YAML frontmatter from markdown content.
fn strip_frontmatter(raw: &str) -> String {
    let trimmed = raw.trim_start();
    if !trimmed.starts_with("---") {
        return raw.to_string();
    }

    let after_first = &trimmed[3..];
    if let Some(end_idx) = after_first.find("\n---") {
        let content = &after_first[end_idx + 4..];
        // Use comrak to convert markdown to plain text
        let arena = comrak::Arena::new();
        let options = comrak::Options::default();
        let root = comrak::parse_document(&arena, content.trim(), &options);

        let mut text = String::new();
        collect_text(root, &mut text);
        text
    } else {
        raw.to_string()
    }
}

/// Recursively collect text content from a comrak AST node.
fn collect_text<'a>(node: &'a comrak::nodes::AstNode<'a>, output: &mut String) {
    use comrak::nodes::NodeValue;
    match &node.data.borrow().value {
        NodeValue::Text(t) => {
            output.push_str(t);
        }
        NodeValue::Code(c) => {
            output.push_str(&c.literal);
        }
        NodeValue::CodeBlock(c) => {
            output.push_str(&c.literal);
            output.push('\n');
        }
        NodeValue::SoftBreak | NodeValue::LineBreak => {
            output.push('\n');
        }
        NodeValue::Paragraph => {
            if !output.is_empty() && !output.ends_with('\n') {
                output.push('\n');
            }
        }
        _ => {}
    }

    for child in node.children() {
        collect_text(child, output);
    }

    // Add newline after paragraphs, headings
    match &node.data.borrow().value {
        NodeValue::Paragraph | NodeValue::Heading(_) => {
            if !output.ends_with('\n') {
                output.push('\n');
            }
        }
        _ => {}
    }
}
