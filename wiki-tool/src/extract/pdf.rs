use std::path::Path;

/// Extract plain text from a PDF file.
pub fn extract(path: &Path) -> crate::Result<String> {
    let bytes = std::fs::read(path)?;
    pdf_extract::extract_text_from_mem(&bytes)
        .map_err(|e| crate::WikiToolError::Extraction(format!("PDF extraction failed: {}", e)))
}
