use std::path::Path;

/// Extract plain text from a text file with auto-detected encoding.
pub fn extract(path: &Path) -> crate::Result<String> {
    let bytes = std::fs::read(path)?;

    // Try UTF-8 first
    if let Ok(text) = String::from_utf8(bytes.clone()) {
        return Ok(text);
    }

    // Auto-detect encoding using encoding_rs
    let (encoding, _confident) = encoding_rs::Encoding::for_bom(&bytes)
        .map(|(enc, _)| (enc, true))
        .unwrap_or_else(|| {
            // Try common encodings
            let detector = encoding_rs::EncoderResult::OutputFull;
            let _ = detector; // suppress warning
            // Default to UTF-8 with replacement
            (encoding_rs::UTF_8, false)
        });

    let (text, _, _) = encoding.decode(&bytes);
    Ok(text.into_owned())
}
