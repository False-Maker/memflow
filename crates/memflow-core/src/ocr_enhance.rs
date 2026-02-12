//! OCR Enhancement Module for Terminal and Code Scenes
//!
//! Provides preprocessing and postprocessing optimizations for better
//! OCR recognition of terminal output and code screenshots.

use std::collections::HashMap;

/// OCR quality metrics
#[derive(Debug, Clone)]
pub struct OcrQualityMetrics {
    pub character_error_rate: f64,
    pub word_error_rate: f64,
    pub confidence_score: f64,
}

impl Default for OcrQualityMetrics {
    fn default() -> Self {
        Self {
            character_error_rate: 0.0,
            word_error_rate: 0.0,
            confidence_score: 0.0,
        }
    }
}

/// Preprocess image for terminal OCR
///
/// Optimizations for terminal/code scenes:
/// 1. Convert to grayscale
/// 2. Apply contrast enhancement
/// 3. Binarization for sharp text
/// 4. Noise reduction
pub fn preprocess_terminal_image(image_data: &[u8]) -> Vec<u8> {
    // Placeholder implementation - would use image crate for actual processing
    // For now, just return the original data
    // TODO: Implement actual image preprocessing using image crate
    image_data.to_vec()
}

/// Postprocess OCR text for code/terminal content
///
/// Corrections and enhancements:
/// 1. Code symbol correction
/// 2. Bracket/brace pairing check
/// 3. Indentation preservation
/// 4. Line structure restoration
pub fn postprocess_terminal_text(text: &str) -> String {
    let mut result = text.to_string();

    // Apply code symbol corrections
    result = correct_code_symbols(&result);

    // Check and fix bracket pairing
    result = fix_bracket_pairs(&result);

    // Normalize whitespace while preserving indentation
    result = normalize_whitespace(&result);

    result
}

/// Correct common OCR errors in code symbols
fn correct_code_symbols(text: &str) -> String {
    let corrections: HashMap<char, char> = [
        ('0', '0'),  // Zero that looks like O
        ('O', '0'),  // O that should be zero
        ('l', '1'),  // lowercase L that looks like 1
        ('1', 'l'),  // 1 that looks like l
        ('`', '\''), // backtick vs single quote
        ('"', '"'),  // normalize quotes
        ('"', '"'),  // normalize quotes
        ('—', '-'),  // em-dash to hyphen
        ('–', '-'),  // en-dash to hyphen
    ]
    .iter()
    .cloned()
    .collect();

    text.chars()
        .map(|c| *corrections.get(&c).unwrap_or(&c))
        .collect()
}

/// Fix unpaired brackets and braces
fn fix_bracket_pairs(text: &str) -> String {
    let mut stack: Vec<char> = Vec::new();
    let pairs: HashMap<char, char> = [('(', ')'), ('[', ']'), ('{', '}')]
        .iter()
        .cloned()
        .collect();

    let open_brackets: std::collections::HashSet<char> = pairs.keys().cloned().collect();
    let close_brackets: std::collections::HashSet<char> = pairs.values().cloned().collect();

    let mut result = String::with_capacity(text.len());

    for c in text.chars() {
        if open_brackets.contains(&c) {
            stack.push(c);
            result.push(c);
        } else if close_brackets.contains(&c) {
            // Check if it matches the last opened bracket
            if let Some(last_open) = stack.last() {
                if pairs.get(last_open) == Some(&c) {
                    stack.pop();
                    result.push(c);
                } else {
                    // Mismatched bracket, skip it
                    continue;
                }
            } else {
                // Closing bracket without opening, skip it
                continue;
            }
        } else {
            result.push(c);
        }
    }

    // Add missing closing brackets
    while let Some(open) = stack.pop() {
        if let Some(close) = pairs.get(&open) {
            result.push(*close);
        }
    }

    result
}

/// Normalize whitespace while preserving indentation structure
fn normalize_whitespace(text: &str) -> String {
    let lines: Vec<&str> = text.lines().collect();
    let mut result = Vec::new();

    for line in lines {
        // Preserve leading whitespace (indentation)
        let trimmed = line.trim_start();
        let indent = &line[..line.len() - trimmed.len()];

        // Normalize internal whitespace
        let normalized: String = trimmed.split_whitespace().collect::<Vec<_>>().join(" ");

        result.push(format!("{}{}", indent, normalized));
    }

    result.join("\n")
}

/// Calculate Character Error Rate (CER)
///
/// CER = (Substitutions + Insertions + Deletions) / Total Characters in Reference
pub fn calculate_cer(reference: &str, hypothesis: &str) -> f64 {
    let ref_chars: Vec<char> = reference.chars().collect();
    let hyp_chars: Vec<char> = hypothesis.chars().collect();

    let distance = levenshtein_distance_chars(&ref_chars, &hyp_chars);

    if ref_chars.is_empty() {
        0.0
    } else {
        distance as f64 / ref_chars.len() as f64
    }
}

/// Calculate Word Error Rate (WER)
///
/// WER = (Substitutions + Insertions + Deletions) / Total Words in Reference
pub fn calculate_wer(reference: &str, hypothesis: &str) -> f64 {
    let ref_words: Vec<&str> = reference.split_whitespace().collect();
    let hyp_words: Vec<&str> = hypothesis.split_whitespace().collect();

    let distance = levenshtein_distance_str(&ref_words, &hyp_words);

    if ref_words.is_empty() {
        0.0
    } else {
        distance as f64 / ref_words.len() as f64
    }
}

/// Levenshtein distance for character sequences
fn levenshtein_distance_chars(a: &[char], b: &[char]) -> usize {
    let len_a = a.len();
    let len_b = b.len();

    if len_a == 0 {
        return len_b;
    }
    if len_b == 0 {
        return len_a;
    }

    let mut matrix = vec![vec![0; len_b + 1]; len_a + 1];

    for i in 0..=len_a {
        matrix[i][0] = i;
    }
    for j in 0..=len_b {
        matrix[0][j] = j;
    }

    for i in 1..=len_a {
        for j in 1..=len_b {
            let cost = if a[i - 1] == b[j - 1] { 0 } else { 1 };
            matrix[i][j] = (matrix[i - 1][j] + 1)
                .min(matrix[i][j - 1] + 1)
                .min(matrix[i - 1][j - 1] + cost);
        }
    }

    matrix[len_a][len_b]
}

/// Levenshtein distance for string sequences
fn levenshtein_distance_str(a: &[&str], b: &[&str]) -> usize {
    let len_a = a.len();
    let len_b = b.len();

    if len_a == 0 {
        return len_b;
    }
    if len_b == 0 {
        return len_a;
    }

    let mut matrix = vec![vec![0; len_b + 1]; len_a + 1];

    for i in 0..=len_a {
        matrix[i][0] = i;
    }
    for j in 0..=len_b {
        matrix[0][j] = j;
    }

    for i in 1..=len_a {
        for j in 1..=len_b {
            let cost = if a[i - 1] == b[j - 1] { 0 } else { 1 };
            matrix[i][j] = (matrix[i - 1][j] + 1)
                .min(matrix[i][j - 1] + 1)
                .min(matrix[i - 1][j - 1] + cost);
        }
    }

    matrix[len_a][len_b]
}

/// Evaluate OCR quality
pub fn evaluate_ocr_quality(reference: &str, hypothesis: &str) -> OcrQualityMetrics {
    OcrQualityMetrics {
        character_error_rate: calculate_cer(reference, hypothesis),
        word_error_rate: calculate_wer(reference, hypothesis),
        confidence_score: 1.0 - calculate_cer(reference, hypothesis),
    }
}

/// Detect if text appears to be code based on patterns
pub fn is_likely_code(text: &str) -> bool {
    let code_indicators = [
        "fn ", "function", "def ", "class ", "import ", "#include", "{", "}", ";", "//", "/*",
        "*/", "```",
    ];

    let text_lower = text.to_lowercase();
    let indicator_count = code_indicators
        .iter()
        .filter(|&&indicator| text_lower.contains(indicator))
        .count();

    // If more than 2 indicators found, likely code
    indicator_count >= 2
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_fix_bracket_pairs() {
        let input = "(hello world";
        let output = fix_bracket_pairs(input);
        assert_eq!(output, "(hello world)");

        let input2 = "{[()]";
        let output2 = fix_bracket_pairs(input2);
        assert_eq!(output2, "{[()]}");
    }

    #[test]
    fn test_calculate_cer() {
        let reference = "hello world";
        let hypothesis = "hello wrld";
        let cer = calculate_cer(reference, hypothesis);
        assert!(cer > 0.0 && cer < 1.0);
    }

    #[test]
    fn test_calculate_wer() {
        let reference = "the quick brown fox";
        let hypothesis = "the brown fox";
        let wer = calculate_wer(reference, hypothesis);
        assert!(wer > 0.0 && wer < 1.0);
    }

    #[test]
    fn test_is_likely_code() {
        assert!(is_likely_code("fn main() { println!(\"hello\"); }"));
        assert!(is_likely_code("def hello():\n    pass"));
        assert!(!is_likely_code("This is just plain text"));
    }

    #[test]
    fn test_normalize_whitespace() {
        let input = "  hello   world  ";
        let output = normalize_whitespace(input);
        assert_eq!(output, "  hello world");
    }
}
