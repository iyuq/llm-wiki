/// CJK-aware bigram tokenizer for tantivy.
///
/// Detects CJK Unicode ranges and splits them into bigrams.
/// Latin text passes through the standard tokenizer.

/// Check if a character is in CJK Unicode ranges.
pub fn is_cjk(c: char) -> bool {
    matches!(c,
        '\u{4E00}'..='\u{9FFF}'   | // CJK Unified Ideographs
        '\u{3400}'..='\u{4DBF}'   | // CJK Unified Ideographs Extension A
        '\u{F900}'..='\u{FAFF}'   | // CJK Compatibility Ideographs
        '\u{3040}'..='\u{309F}'   | // Hiragana
        '\u{30A0}'..='\u{30FF}'   | // Katakana
        '\u{AC00}'..='\u{D7AF}'     // Hangul Syllables
    )
}

/// Tokenize text with CJK bigram support.
/// CJK characters are split into overlapping bigrams.
/// Latin words are split on whitespace and lowercased.
pub fn tokenize(text: &str) -> Vec<String> {
    let mut tokens = Vec::new();
    let mut cjk_buffer = Vec::new();
    let mut latin_buffer = String::new();

    for c in text.chars() {
        if is_cjk(c) {
            // Flush latin buffer
            if !latin_buffer.is_empty() {
                for word in latin_buffer.split_whitespace() {
                    let lower = word.to_lowercase();
                    let cleaned: String = lower.chars().filter(|c| c.is_alphanumeric()).collect();
                    if !cleaned.is_empty() {
                        tokens.push(cleaned);
                    }
                }
                latin_buffer.clear();
            }
            cjk_buffer.push(c);
        } else {
            // Flush CJK buffer as bigrams
            if !cjk_buffer.is_empty() {
                flush_cjk_bigrams(&cjk_buffer, &mut tokens);
                cjk_buffer.clear();
            }
            latin_buffer.push(c);
        }
    }

    // Flush remaining buffers
    if !cjk_buffer.is_empty() {
        flush_cjk_bigrams(&cjk_buffer, &mut tokens);
    }
    if !latin_buffer.is_empty() {
        for word in latin_buffer.split_whitespace() {
            let lower = word.to_lowercase();
            let cleaned: String = lower.chars().filter(|c| c.is_alphanumeric()).collect();
            if !cleaned.is_empty() {
                tokens.push(cleaned);
            }
        }
    }

    tokens
}

fn flush_cjk_bigrams(chars: &[char], tokens: &mut Vec<String>) {
    if chars.len() == 1 {
        tokens.push(chars[0].to_string());
    } else {
        for window in chars.windows(2) {
            tokens.push(window.iter().collect());
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_latin_tokenize() {
        let tokens = tokenize("Hello World");
        assert_eq!(tokens, vec!["hello", "world"]);
    }

    #[test]
    fn test_cjk_bigrams() {
        let tokens = tokenize("注意力机制");
        assert_eq!(tokens, vec!["注意", "意力", "力机", "机制"]);
    }

    #[test]
    fn test_mixed_cjk_latin() {
        let tokens = tokenize("The 注意力 mechanism");
        assert_eq!(tokens, vec!["the", "注意", "意力", "mechanism"]);
    }

    #[test]
    fn test_single_cjk() {
        let tokens = tokenize("猫");
        assert_eq!(tokens, vec!["猫"]);
    }
}
