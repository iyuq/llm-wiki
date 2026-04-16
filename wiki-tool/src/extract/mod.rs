pub mod markdown;
pub mod pdf;
pub mod text;

use std::path::Path;

use crate::WikiToolError;

/// Extract plain text from a file based on its extension.
pub fn extract_text(path: &Path) -> crate::Result<String> {
    let ext = path
        .extension()
        .and_then(|e| e.to_str())
        .unwrap_or("")
        .to_lowercase();

    match ext.as_str() {
        "md" | "markdown" => markdown::extract(path),
        "txt" | "text" | "log" | "csv" | "tsv" | "json" | "yaml" | "yml" | "toml" | "xml"
        | "html" | "htm" | "rs" | "py" | "js" | "ts" | "go" | "java" | "c" | "cpp" | "h"
        | "sh" | "bash" | "zsh" => text::extract(path),
        "pdf" => pdf::extract(path),
        _ => Err(WikiToolError::Extraction(format!(
            "Unsupported file format: .{}",
            ext
        ))),
    }
}
