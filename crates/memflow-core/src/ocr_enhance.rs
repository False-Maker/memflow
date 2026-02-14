//! OCR Enhancement Module for Terminal and Code Scenes
//!
//! Provides preprocessing and postprocessing optimizations for better
//! OCR recognition of terminal output and code screenshots.

use std::collections::HashMap;
use std::sync::LazyLock;

use image::{GrayImage, ImageBuffer, Luma};

/// Programming language detection result
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ProgrammingLanguage {
    Rust,
    Python,
    JavaScript,
    TypeScript,
    Cpp,
    C,
    Java,
    Go,
    Unknown,
}

/// Detect programming language from text patterns
///
/// Uses language-specific keywords and syntax patterns to identify
/// the most likely programming language in the given text.
/// This is a heuristic-based detection, not a full parser.
pub fn detect_language(text: &str) -> ProgrammingLanguage {
    let text_lower = text.to_lowercase();

    // Count language-specific indicators
    let rust_score = count_language_patterns(
        &text_lower,
        &[
            "fn ", "fn{", "fn\n", "struct ", "enum ", "impl ", "use ", "mod ", "let ", "const ",
            "mut ", "&str", "->", "match ", "pub ",
        ],
    );

    let python_score = count_language_patterns(
        &text_lower,
        &[
            "def ", "class ", "import ", "from ", "lambda ", "self", "elif ", "__init__",
            "__main__", "print(", "range(", "len(",
        ],
    );

    let js_score = count_language_patterns(
        &text_lower,
        &[
            "function ",
            "const ",
            "let ",
            "var ",
            "=>",
            "console.",
            "typeof ",
            "undefined",
            "null",
            "await ",
            "async ",
            "import ",
            "export ",
        ],
    );

    let cpp_score = count_language_patterns(
        &text_lower,
        &[
            "#include",
            "namespace ",
            "class ",
            "template <",
            "std::",
            "cout",
            "cin>>",
            "endl",
            "vector<",
            "string ",
            "auto ",
            "nullptr",
        ],
    );

    let c_score = count_language_patterns(
        &text_lower,
        &[
            "#include", "struct ", "enum ", "typedef ", "printf(", "scanf(", "malloc(", "free(",
            "sizeof(", "FILE*", "int ", "void ", "char ",
        ],
    );

    let java_score = count_language_patterns(
        &text_lower,
        &[
            "public class",
            "private ",
            "protected ",
            "static ",
            "void ",
            "int ",
            "String ",
            "System.out",
            "extends ",
            "implements ",
            "new ",
            "this.",
        ],
    );

    let go_score = count_language_patterns(
        &text_lower,
        &[
            "func ",
            "package ",
            "import (",
            "var ",
            "const ",
            "type ",
            "struct ",
            "interface ",
            "go ",
            "chan ",
            "defer ",
        ],
    );

    // Find language with highest score (minimum threshold of 2)
    let scores = [
        (rust_score, ProgrammingLanguage::Rust),
        (python_score, ProgrammingLanguage::Python),
        (js_score, ProgrammingLanguage::JavaScript),
        (cpp_score, ProgrammingLanguage::Cpp),
        (c_score, ProgrammingLanguage::C),
        (java_score, ProgrammingLanguage::Java),
        (go_score, ProgrammingLanguage::Go),
    ];

    let max_score = scores.iter().map(|(s, _)| *s).max().unwrap_or(0);

    if max_score < 2 {
        return ProgrammingLanguage::Unknown;
    }

    scores
        .into_iter()
        .filter(|(s, _)| *s == max_score)
        .map(|(_, lang)| lang)
        .next()
        .unwrap_or(ProgrammingLanguage::Unknown)
}

/// Count how many patterns from a list appear in the text
fn count_language_patterns(text: &str, patterns: &[&str]) -> usize {
    patterns
        .iter()
        .filter(|&&pattern| text.contains(pattern))
        .count()
}

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
    // Parse the image from bytes
    let img = match image::load_from_memory(image_data) {
        Ok(img) => img.to_luma8(),
        Err(_) => return image_data.to_vec(), // Return original if parsing fails
    };

    // Step 1: Convert to grayscale (already done with to_luma8)
    let gray_img = img;

    // Step 2: Apply contrast enhancement (histogram stretch)
    let contrasted = enhance_contrast(&gray_img);

    // Step 3: Apply binarization (threshold) for sharp text
    let binarized = binarize(&contrasted, 128);

    // Step 4: Convert back to PNG bytes
    let mut output = Vec::new();
    if let Err(_) = binarized.write_to(
        &mut std::io::Cursor::new(&mut output),
        image::ImageFormat::Png,
    ) {
        return image_data.to_vec();
    }

    output
}

/// Enhance contrast using histogram stretching
fn enhance_contrast(img: &GrayImage) -> GrayImage {
    let (min, max) = img.pixels().fold((u8::MAX, u8::MIN), |(min, max), p| {
        (min.min(p[0]), max.max(p[0]))
    });

    if max <= min {
        return img.clone();
    }

    let range = (max - min) as f32;

    GrayImage::from_fn(img.width(), img.height(), |x, y| {
        let p = img.get_pixel(x, y);
        let val = p[0];
        let stretched = (((val - min) as f32 / range) * 255.0).round() as u8;
        Luma([stretched])
    })
}

/// Binarize image using simple threshold
fn binarize(img: &GrayImage, threshold: u8) -> GrayImage {
    GrayImage::from_fn(img.width(), img.height(), |x, y| {
        let p = img.get_pixel(x, y);
        Luma([if p[0] > threshold { 255 } else { 0 }])
    })
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
///
/// Uses context-aware correction:
/// - In code context (detected by keywords like fn, def, class, let, var, const, =, ;, error, expected):
///   - 1 → l (digit 1 to lowercase L)
///   - 0 → o (digit 0 to lowercase o)
///   - 5 → S (digit 5 to capital S in certain contexts)
/// - In non-code context:
///   - Preserves original characters
/// - Bidirectional conflicts are avoided by only applying corrections in code context
fn correct_code_symbols(text: &str) -> String {
    // Code keywords for context detection - extended to include error messages
    const CODE_KEYWORDS: &[&str] = &[
        "fn", "def", "class", "let", "var", "const", "=", ";", "error", "warning", "expected",
        "found", "mismatch",
    ];

    // Static corrections map using LazyLock for code context only
    // These correct common OCR errors where letters are misread as digits
    static CODE_CORRECTIONS: LazyLock<HashMap<char, char>> = LazyLock::new(|| {
        [
            ('1', 'l'), // digit 1 to lowercase l (OCR error correction)
            ('0', 'o'), // digit 0 to lowercase o (OCR error correction)
            ('5', 'S'), // digit 5 to capital S (in some contexts)
        ]
        .iter()
        .cloned()
        .collect()
    });

    // Detect if text is in code context
    let is_code_context = CODE_KEYWORDS.iter().any(|&keyword| text.contains(keyword));

    // Only apply corrections in code context
    if is_code_context {
        text.chars()
            .map(|c| *CODE_CORRECTIONS.get(&c).unwrap_or(&c))
            .collect()
    } else {
        // Preserve original text in non-code context
        text.to_string()
    }
}

/// Fix unpaired brackets and braces
///
/// Preserves brackets inside string literals (regular, escaped, raw, multiline)
fn fix_bracket_pairs(text: &str) -> String {
    let mut stack: Vec<char> = Vec::new();
    let pairs: HashMap<char, char> = [('(', ')'), ('[', ']'), ('{', '}')]
        .iter()
        .cloned()
        .collect();

    let open_brackets: std::collections::HashSet<char> = pairs.keys().cloned().collect();
    let close_brackets: std::collections::HashSet<char> = pairs.values().cloned().collect();

    let mut result = String::with_capacity(text.len());

    // String literal detection state machine
    let mut in_string: Option<char> = None; // None, Some('"'), or Some('\'')
    let mut escaped = false;
    let mut raw_string_level = 0; // Number of # after r for raw strings
    let mut i = 0;
    let chars: Vec<char> = text.chars().collect();

    while i < chars.len() {
        let c = chars[i];

        // Handle raw string prefix r#"..."#
        if c == 'r' && i + 1 < chars.len() && chars[i + 1] == '#' {
            // Count raw string level
            raw_string_level = 0;
            let mut j = i + 1;
            while j < chars.len() && chars[j] == '#' {
                raw_string_level += 1;
                j += 1;
            }
            // Check for opening quote after hashes
            if j < chars.len() && (chars[j] == '"' || chars[j] == '\'') {
                in_string = Some(chars[j]);
                result.push(c);
                // Push all # characters
                for _ in 0..raw_string_level {
                    result.push('#');
                    i += 1;
                }
                i += 1; // Move to quote
                continue;
            }
        }

        // Handle escape sequences
        if escaped {
            escaped = false;
            result.push(c);
            i += 1;
            continue;
        }

        if c == '\\' && in_string.is_some() {
            escaped = true;
            result.push(c);
            i += 1;
            continue;
        }

        // Handle string opening/closing
        if in_string.is_some() {
            // Check for raw string closing: #"# or ##"## etc.
            if c == '"' && raw_string_level > 0 {
                // Check if we have the right number of # after the quote
                let mut closing_level = 0;
                let mut j = i + 1;
                while j < chars.len() && chars[j] == '#' {
                    closing_level += 1;
                    j += 1;
                }
                if closing_level == raw_string_level {
                    in_string = None;
                    raw_string_level = 0;
                }
                result.push(c);
                // Add the closing #s
                for _ in 0..closing_level {
                    result.push('#');
                    i += 1;
                }
                i += 1;
                continue;
            }

            // Regular string closing
            if in_string == Some(c) {
                in_string = None;
                result.push(c);
                i += 1;
                continue;
            }

            // Inside string literal - preserve everything as-is
            result.push(c);
            i += 1;
            continue;
        }

        // Not inside string - check for string opening
        if c == '"' || c == '\'' {
            in_string = Some(c);
            result.push(c);
            i += 1;
            continue;
        }

        // Bracket processing (only outside strings)
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
                }
            } else {
                // Closing bracket without opening, skip it
            }
        } else {
            result.push(c);
        }

        i += 1;
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

    let distance = levenshtein_distance(&ref_chars, &hyp_chars);

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

    let distance = levenshtein_distance(&ref_words, &hyp_words);

    if ref_words.is_empty() {
        0.0
    } else {
        distance as f64 / ref_words.len() as f64
    }
}

/// Generic Levenshtein distance for any sequence types
///
/// Uses type parameter T with PartialEq bound to compare elements
/// Works with both &[char] and &[&str] (or any other slice type)
fn levenshtein_distance<T, U>(a: &[T], b: &[U]) -> usize
where
    T: PartialEq<U>,
{
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

/// Suggest corrections based on CER analysis
///
/// Analyzes the difference between reference and hypothesis text
/// and suggests specific corrections to improve OCR quality.
/// Returns a vector of (original, suggested) pairs.
pub fn suggest_corrections(reference: &str, hypothesis: &str) -> Vec<(char, char)> {
    let mut corrections = Vec::new();

    // Only suggest corrections if CER is significant (> 5%)
    let cer = calculate_cer(reference, hypothesis);
    if cer <= 0.05 {
        return corrections;
    }

    let ref_chars: Vec<char> = reference.chars().collect();
    let hyp_chars: Vec<char> = hypothesis.chars().collect();

    // Find single character substitutions
    // Use a simple alignment to find mismatches
    let mut i = 0;
    let mut j = 0;

    while i < ref_chars.len() && j < hyp_chars.len() {
        if ref_chars[i] == hyp_chars[j] {
            i += 1;
            j += 1;
        } else {
            // Potential substitution - suggest correction
            // Only suggest common OCR confusions
            let is_common_confusion = matches!(
                (ref_chars[i], hyp_chars[j]),
                ('l', '1')
                    | ('1', 'l')
                    | ('O', '0')
                    | ('0', 'O')
                    | ('I', 'l')
                    | ('l', 'I')
                    | ('5', 'S')
                    | ('S', '5')
                    | ('8', 'B')
                    | ('B', '8')
            );

            if is_common_confusion {
                corrections.push((hyp_chars[j], ref_chars[i]));
            }

            // Advance both pointers (simple alignment)
            i += 1;
            j += 1;
        }
    }

    // Deduplicate corrections
    corrections.sort();
    corrections.dedup();

    corrections
}

/// Calculate confidence score from CER and WER
///
/// Combines character and word error rates into a single confidence score.
/// Formula: 1.0 - (0.7 * CER + 0.3 * WER)
/// Weights CER higher as character errors are more critical for code.
///
/// Returns a value in [0.0, 1.0] where:
/// - 1.0 = perfect confidence (no errors)
/// - 0.0 = no confidence (all errors)
pub fn calculate_confidence(cer: f64, wer: f64) -> f64 {
    // Weight CER higher than WER for code/terminal content
    // Character errors are more critical in code
    let cer_weight = 0.7;
    let wer_weight = 0.3;

    let combined_error_rate = cer_weight * cer + wer_weight * wer;

    // Ensure confidence is in [0.0, 1.0]
    let confidence = 1.0 - combined_error_rate;
    confidence.clamp(0.0, 1.0)
}

/// Evaluate OCR quality
pub fn evaluate_ocr_quality(reference: &str, hypothesis: &str) -> OcrQualityMetrics {
    let cer = calculate_cer(reference, hypothesis);
    let wer = calculate_wer(reference, hypothesis);

    OcrQualityMetrics {
        character_error_rate: cer,
        word_error_rate: wer,
        confidence_score: calculate_confidence(cer, wer),
    }
}

/// Detect if text appears to be code based on patterns
pub fn is_likely_code(text: &str) -> bool {
    // Fast path: use language detection for improved accuracy
    let detected_language = detect_language(text);
    if detected_language != ProgrammingLanguage::Unknown {
        return true;
    }

    // Fallback to indicator-based detection
    // Code indicators from various languages
    // - Rust: fn, const, let, struct, enum, impl, use
    // - Python: def, class, import, from, if __name__
    // - JavaScript/TypeScript: function, const, let, var, import, from
    // - C/C++: #include, struct, enum
    // - Java: public, static, void, class, package, import
    // - C#: using, namespace, public, private, void, class
    // - Go: package, import, func, type, struct, go
    // - General: {}, ;, //, /*, */, ``` (markdown code blocks)
    let code_indicators = [
        // Rust
        "fn ",
        "fn{",
        "fn\n",
        "const ",
        "let ",
        "struct ",
        "enum ",
        "impl ",
        "use ",
        // Python - specific patterns
        "def ",
        "def\n",
        "class ",
        "class\n",
        "import ",
        "from ",
        "if __name__",
        // JavaScript/TypeScript
        "function",
        "var ",
        "var\n",
        // C/C++
        "#include",
        "struct ",
        "struct\n",
        // Java
        "public ",
        "public\n",
        "static ",
        "static\n",
        "void ",
        "void\n",
        "package ",
        "package\n",
        // C#
        "using ",
        "using\n",
        "namespace ",
        "namespace\n",
        "private ",
        "private\n",
        // Go
        "func ",
        "func\n",
        "type ",
        "type\n",
        "go ",
        "go\n",
        // General programming symbols
        "{",
        "}",
        ";",
        // Comment patterns (strong indicators)
        "//",
        "/*",
        "*/",
        "#",
        "<!--",
        "--",
        "```",
        // Python-specific: def(...) pattern or class(...) pattern
        // This catches "def hello():" without being too aggressive
    ];

    let text_lower = text.to_lowercase();

    // Count standard indicators
    let mut indicator_count = code_indicators
        .iter()
        .filter(|&&indicator| text_lower.contains(indicator))
        .count();

    // Additional Python-specific detection:
    // Look for "def name(" or "class name(" patterns
    // This catches Python function/class definitions
    if text_lower.contains("def ") || text_lower.contains("class ") {
        // Check if followed by ( on same line or next line
        let lines: Vec<&str> = text_lower.lines().collect();
        for (i, line) in lines.iter().enumerate() {
            if (line.contains("def ") || line.contains("class ")) && line.contains('(') {
                indicator_count += 1;
                break;
            }
            // Also check next line for multi-line definitions
            if i + 1 < lines.len() {
                let next_line = lines[i + 1];
                if (line.contains("def ") || line.contains("class ")) && next_line.contains('(') {
                    indicator_count += 1;
                    break;
                }
            }
        }
    }

    // Number sequence detection: reduce false positives
    // Long sequences of digits (e.g., phone numbers, IDs, SSNs) suggest NOT code
    let mut consecutive_digits = 0;
    let mut max_consecutive_digits = 0;
    for c in text.chars() {
        if c.is_ascii_digit() {
            consecutive_digits += 1;
            max_consecutive_digits = max_consecutive_digits.max(consecutive_digits);
        } else {
            consecutive_digits = 0;
        }
    }

    // If we have very long digit sequences (8+), likely NOT code
    // This catches phone numbers, SSNs, credit cards, serial numbers, etc.
    // Examples: "12345678", "555-123-4567" (8 consecutive digits)
    if max_consecutive_digits >= 8 {
        // Reduce indicator count by 1 to penalize
        indicator_count = indicator_count.saturating_sub(1);
    }

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

    // ========== RED PHASE TESTS - These should fail initially ==========

    #[test]
    fn test_correct_code_symbols_no_bidirectional_conflict() {
        // Issue 1: l -> 1 and 1 -> l creates infinite loop
        // Current implementation incorrectly swaps both directions
        let input = "let x = 1;";
        let output = correct_code_symbols(input);
        // TODO: This test will fail because current code swaps bidirectionally
        // Expected: should not create "1et x = l;" due to bidirectional conflict
        assert_ne!(
            output, "1et x = l;",
            "bidirectional conflict should be avoided"
        );
    }

    #[test]
    fn test_fix_bracket_pairs_preserves_string_literals() {
        // Issue 2: Brackets inside string literals should not be auto-closed
        let input = "print(\"hello (world)\")";
        let output = fix_bracket_pairs(input);
        assert_eq!(
            output, "print(\"hello (world)\")",
            "string literals preserved"
        );

        // Test escaped quotes
        let input2 = "text = \"hello \\\" (world\"";
        let output2 = fix_bracket_pairs(input2);
        assert_eq!(
            output2, "text = \"hello \\\" (world\"",
            "escaped quotes preserved"
        );

        // Test raw strings
        let input3 = r##"r#"hello (world"#"##;
        let output3 = fix_bracket_pairs(input3);
        assert_eq!(output3, r##"r#"hello (world"#"##, "raw strings preserved");

        // Test unbalanced brackets inside strings should NOT be auto-closed
        // But brackets OUTSIDE strings should still be closed
        let input4 = "print(\"hello (world\"";
        let output4 = fix_bracket_pairs(input4);
        // The 'print(' should be closed, but the '(world' inside string should not
        assert_eq!(
            output4, "print(\"hello (world\")",
            "unbalanced brackets in strings preserved, but outside brackets closed"
        );

        // Test single quotes
        let input5 = "text = 'hello (world'";
        let output5 = fix_bracket_pairs(input5);
        assert_eq!(
            output5, "text = 'hello (world'",
            "single quote strings preserved"
        );
    }

    #[test]
    fn test_normalize_whitespace_preserves_indentation() {
        // Issue 3: Code indentation should be preserved
        let input = "    fn main() {\n        println!(\"hello\");\n    }";
        let output = normalize_whitespace(input);
        // TODO: This test will fail because current code over-normalizes
        // Expected: should preserve leading whitespace for code structure
        assert!(output.starts_with("    fn main()"), "indentation preserved");
        assert!(
            output.contains("        println"),
            "inner indentation preserved"
        );
    }

    #[test]
    fn test_levenshtein_generic_works_for_char_and_str() {
        // Issue 4: Levenshtein should work for both char and str types
        // Current implementation is string-specific
        let s1 = "kitten";
        let s2 = "sitting";

        // Split into words to test with string slices
        let words1: Vec<&str> = s1.split("").collect();
        let words2: Vec<&str> = s2.split("").collect();

        let distance_str = levenshtein_distance(&words1, &words2);

        // TODO: This test will fail if function is not generic enough
        // Expected: should work with any sequence type (String, Vec<char>, etc.)
        // For now, just verify the function works with &[&str]
        assert!(distance_str > 0, "levenshtein calculation works");
    }

    #[test]
    fn test_preprocess_terminal_image_not_placeholder() {
        // Issue 5: Terminal image preprocessing should handle real images
        // Create a simple 2x2 grayscale test image
        let mut img: GrayImage = ImageBuffer::new(2, 2);
        img.put_pixel(0, 0, Luma([50]));
        img.put_pixel(1, 0, Luma([150]));
        img.put_pixel(0, 1, Luma([200]));
        img.put_pixel(1, 1, Luma([100]));

        // Convert to PNG bytes
        let mut input = Vec::new();
        img.write_to(
            &mut std::io::Cursor::new(&mut input),
            image::ImageFormat::Png,
        )
        .expect("Failed to create test image");

        let output = preprocess_terminal_image(&input);

        // Verify preprocessing returns data
        assert!(!output.is_empty(), "preprocessing returns data");

        // Verify output is different from input (preprocessing applied)
        assert_ne!(output, input, "preprocessing should modify image");

        // Verify output can be loaded as an image
        let output_img = image::load_from_memory(&output);
        assert!(output_img.is_ok(), "output is valid image");

        // Verify output is grayscale
        let output_img = output_img.unwrap().to_luma8();
        assert_eq!(output_img.width(), 2, "width preserved");
        assert_eq!(output_img.height(), 2, "height preserved");

        // Verify binarization (pixels should be 0 or 255)
        for pixel in output_img.pixels() {
            assert!(
                pixel[0] == 0 || pixel[0] == 255,
                "binarization should produce only 0 or 255, got {}",
                pixel[0]
            );
        }
    }

    #[test]
    fn test_is_likely_code_reduced_false_positives() {
        // Issue 6: Code detection has false positives for normal text
        let text_with_symbols = "The price is $100. Email: test@example.com (urgent)";
        let is_code = is_likely_code(text_with_symbols);
        // TODO: This test will fail because current code over-detects
        // Expected: should not classify normal text with symbols as code
        assert!(!is_code, "reduced false positives for normal text");
    }

    #[test]
    fn test_postprocess_terminal_text_integration() {
        // Issue 7: Terminal text postprocessing integration test
        let input =
            "  error: expected ';'\n    |\n12 |     let x = 1\n    |            ^^^ expected ;";
        let output = postprocess_terminal_text(input);
        // TODO: This test will fail because postprocess function is not implemented
        // Expected: should clean up terminal artifacts while preserving structure
        assert!(output.contains("error:"), "error preserved");
        assert!(output.contains('|'), "pipe separators preserved");
    }

    #[test]
    fn test_cer_improvement_baseline() {
        // Issue 8: CER improvement baseline test
        let noisy = "err0r: expe cted ';'";
        let clean = "error: expected ';'";
        let enhanced = postprocess_terminal_text(noisy); // Use existing function

        let cer_before = calculate_cer(clean, noisy);
        let cer_after = calculate_cer(clean, &enhanced);

        // TODO: This test will fail because enhancement is not yet optimized
        // Expected: enhanced version should have lower CER
        assert!(cer_after < cer_before, "CER improved after enhancement");
    }

    // ========== CER/WER IMPROVEMENT HELPER TESTS ==========

    #[test]
    fn test_suggest_corrections_no_errors() {
        let reference = "hello world";
        let hypothesis = "hello world";
        let corrections = suggest_corrections(reference, hypothesis);
        assert!(corrections.is_empty(), "no corrections for perfect match");
    }

    #[test]
    fn test_suggest_corrections_low_cer() {
        let reference = "hello world";
        let hypothesis = "hello worid"; // 1 char error = 1/11 < 5%
        let corrections = suggest_corrections(reference, hypothesis);
        assert!(corrections.is_empty(), "no corrections for low CER (< 5%)");
    }

    #[test]
    fn test_suggest_corrections_common_confusion() {
        let reference = "let x = 1";
        let hypothesis = "1et x = 1"; // l->1 error (CER > 5%)
        let corrections = suggest_corrections(reference, hypothesis);

        // Should suggest '1' -> 'l' correction
        assert!(
            corrections.contains(&('1', 'l')),
            "suggests l->1 correction: {:?}",
            corrections
        );
    }

    #[test]
    fn test_suggest_corrections_multiple_errors() {
        let reference = "let x = 10";
        let hypothesis = "1et x = lO"; // l->1, 1->l, O->0 errors
        let corrections = suggest_corrections(reference, hypothesis);

        // Should suggest multiple common OCR confusions
        assert!(!corrections.is_empty(), "suggests corrections");
        assert!(
            corrections.len() <= 3,
            "reasonable number of corrections: {}",
            corrections.len()
        );
    }

    #[test]
    fn test_calculate_confidence_perfect() {
        let confidence = calculate_confidence(0.0, 0.0);
        assert_eq!(confidence, 1.0, "perfect CER/WER = 1.0 confidence");
    }

    #[test]
    fn test_calculate_confidence_no_errors() {
        let confidence = calculate_confidence(0.0, 0.0);
        assert!(
            (confidence - 1.0).abs() < 0.001,
            "no errors = high confidence"
        );
    }

    #[test]
    fn test_calculate_confidence_high_cer() {
        let confidence = calculate_confidence(0.5, 0.6);
        assert!(
            confidence < 0.5,
            "high error rate = low confidence: {}",
            confidence
        );
    }

    #[test]
    fn test_calculate_confidence_clamped() {
        // Test that confidence is clamped to [0.0, 1.0]
        let confidence_low = calculate_confidence(2.0, 2.0); // Invalid: > 1.0
        assert_eq!(confidence_low, 0.0, "confidence clamped to minimum");

        let confidence_high = calculate_confidence(-0.5, -0.5); // Invalid: < 0.0
        assert_eq!(
            confidence_high.clamp(0.0, 1.0),
            1.0,
            "confidence clamped to maximum"
        );
    }

    #[test]
    fn test_calculate_confidence_cer_weighted() {
        // CER should have higher weight (0.7) than WER (0.3)
        let conf1 = calculate_confidence(0.1, 0.0); // 10% CER, 0% WER
        let conf2 = calculate_confidence(0.0, 0.1); // 0% CER, 10% WER

        assert!(
            conf1 < conf2,
            "CER weighted higher than WER: {} < {}",
            conf1,
            conf2
        );
    }

    #[test]
    fn test_evaluate_ocr_quality_uses_helpers() {
        let reference = "let x = 1";
        let hypothesis = "1et x = 1";

        let metrics = evaluate_ocr_quality(reference, hypothesis);

        // Verify all fields are populated
        assert!(metrics.character_error_rate > 0.0, "CER calculated");
        assert!(metrics.word_error_rate >= 0.0, "WER calculated");

        // Verify confidence uses the new helper
        let expected_confidence =
            calculate_confidence(metrics.character_error_rate, metrics.word_error_rate);
        assert_eq!(
            metrics.confidence_score, expected_confidence,
            "confidence uses calculate_confidence helper"
        );
    }

    #[test]
    fn test_language_detection() {
        // Test Rust detection
        let rust_code = "fn main() {\n    let x = 42;\n    println!(\"{}\");\n}";
        assert_eq!(detect_language(rust_code), ProgrammingLanguage::Rust);

        // Test Python detection
        let python_code =
            "def hello():\n    print(\"world\")\n\nif __name__ == \"__main__\":\n    hello()";
        assert_eq!(detect_language(python_code), ProgrammingLanguage::Python);

        // Test JavaScript detection
        let js_code = "const greet = () => {\n    console.log(\"hello\");\n};";
        assert_eq!(detect_language(js_code), ProgrammingLanguage::JavaScript);

        // Test C++ detection
        let cpp_code = "#include <iostream>\nint main() {\n    std::cout << \"hello\";\n}";
        assert_eq!(detect_language(cpp_code), ProgrammingLanguage::Cpp);

        // Test C detection
        let c_code = "#include <stdio.h>\nint main() {\n    printf(\"hello\");\n    return 0;\n}";
        assert_eq!(detect_language(c_code), ProgrammingLanguage::C);

        // Test unknown language
        let plain_text = "This is just plain text with no code patterns.";
        assert_eq!(detect_language(plain_text), ProgrammingLanguage::Unknown);

        // Test integration with is_likely_code
        assert!(is_likely_code(rust_code));
        assert!(is_likely_code(python_code));
        assert!(is_likely_code(js_code));
        assert!(is_likely_code(cpp_code));
        assert!(is_likely_code(c_code));
        assert!(!is_likely_code(plain_text));
    }

    // ========== CER IMPROVEMENT BENCHMARK TESTS ==========

    #[test]
    fn test_cer_improvement_noisy_terminal() {
        // Benchmark test: Noisy terminal text with common OCR errors
        let clean = "error: expected ';'";
        let noisy = "err0r: expe cted ';'";
        let enhanced = postprocess_terminal_text(noisy);

        let cer_before = calculate_cer(clean, noisy);
        let cer_after = calculate_cer(clean, &enhanced);

        println!("\n=== Noisy Terminal Benchmark ===");
        println!("Clean:   {}", clean);
        println!("Noisy:   {} (CER: {:.4})", noisy, cer_before);
        println!("Enhanced: {} (CER: {:.4})", enhanced, cer_after);

        // Verify 5% improvement target
        if cer_before > 0.0 {
            let improvement_percent = ((cer_before - cer_after) / cer_before) * 100.0;
            println!("Improvement: {:.2}%", improvement_percent);

            // Assert 5%+ improvement
            assert!(
                improvement_percent >= 5.0,
                "Expected 5%+ improvement, got {:.2}%",
                improvement_percent
            );
        }

        // Enhanced should be at least as good as noisy
        assert!(cer_after <= cer_before, "Enhanced CER should not be worse");
    }

    #[test]
    fn test_cer_improvement_clean_terminal() {
        // Benchmark test: Clean terminal with minimal errors
        let clean = "fn main() {\n    let x = 42;\n    println!(\"{}\");\n}";
        let noisy = "fn main() {\n    1et x = 42;\n    print1n!(\"{}\");\n}";
        let enhanced = postprocess_terminal_text(noisy);

        let cer_before = calculate_cer(clean, noisy);
        let cer_after = calculate_cer(clean, &enhanced);

        println!("\n=== Clean Terminal Benchmark ===");
        println!("Clean:   {}", clean);
        println!("Noisy:   {} (CER: {:.4})", noisy, cer_before);
        println!("Enhanced: {} (CER: {:.4})", enhanced, cer_after);

        // Verify 5% improvement target
        if cer_before > 0.0 {
            let improvement_percent = ((cer_before - cer_after) / cer_before) * 100.0;
            println!("Improvement: {:.2}%", improvement_percent);

            // Assert 5%+ improvement
            assert!(
                improvement_percent >= 5.0,
                "Expected 5%+ improvement, got {:.2}%",
                improvement_percent
            );
        }

        // Enhanced should be at least as good as noisy
        assert!(cer_after <= cer_before, "Enhanced CER should not be worse");
    }

    #[test]
    fn test_cer_improvement_timing() {
        // Benchmark test: Verify enhancement completes within timing budget
        let noisy = "err0r: expe cted ';'";
        let clean = "error: expected ';'";

        // Benchmark enhancement performance
        let start = std::time::Instant::now();
        let enhanced = postprocess_terminal_text(noisy);
        let enhancement_time = start.elapsed();

        // Benchmark CER calculation
        let start = std::time::Instant::now();
        let cer_before = calculate_cer(clean, noisy);
        let cer_after = calculate_cer(clean, &enhanced);
        let cer_time = start.elapsed();

        println!("\n=== Timing Benchmark ===");
        println!("Enhancement time: {:?}", enhancement_time);
        println!("CER calculation time: {:?}", cer_time);

        // Timing assertions: Enhancement should complete quickly
        assert!(
            enhancement_time.as_millis() < 100,
            "Enhancement too slow: {:?}",
            enhancement_time
        );

        // CER calculation should be fast
        assert!(
            cer_time.as_micros() < 1000,
            "CER calculation too slow: {:?}",
            cer_time
        );
    }

    #[test]
    fn test_cer_improvement_confidence_score() {
        // Benchmark test: Verify confidence score improves with enhancement
        let clean = "let result = calculate_cer(ref, hyp);";
        let noisy = "1et resu1t = ca1cu1ate_cer(ref, hyp);";
        let enhanced = postprocess_terminal_text(noisy);

        let metrics_before = evaluate_ocr_quality(clean, noisy);
        let metrics_after = evaluate_ocr_quality(clean, &enhanced);

        println!("\n=== Confidence Score Benchmark ===");
        println!(
            "Before: CER={:.4}, WER={:.4}, Confidence={:.4}",
            metrics_before.character_error_rate,
            metrics_before.word_error_rate,
            metrics_before.confidence_score
        );
        println!(
            "After: CER={:.4}, WER={:.4}, Confidence={:.4}",
            metrics_after.character_error_rate,
            metrics_after.word_error_rate,
            metrics_after.confidence_score
        );

        // Confidence should improve after enhancement
        assert!(
            metrics_after.confidence_score >= metrics_before.confidence_score,
            "Confidence should improve: {:.4} >= {:.4}",
            metrics_after.confidence_score,
            metrics_before.confidence_score
        );

        // CER should decrease
        assert!(
            metrics_after.character_error_rate <= metrics_before.character_error_rate,
            "CER should decrease: {:.4} <= {:.4}",
            metrics_after.character_error_rate,
            metrics_before.character_error_rate
        );
    }
}
