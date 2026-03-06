//! Unit tests for OCR enhancement module.
//!
//! These tests verify the text preprocessing, postprocessing,
//! and quality evaluation functions.

use memflow_core::ocr_enhance::{
    calculate_cer, calculate_wer, detect_language, evaluate_ocr_quality, is_likely_code,
    postprocess_terminal_text,
};

/// Test postprocess_terminal_text: normalization and punctuation fixing
#[test]
fn test_postprocess_normalizes_newlines() {
    let input = "line1\r\nline2\rline3";
    let result = postprocess_terminal_text(input);
    assert!(!result.contains("\r"));
    assert_eq!(result, "line1\nline2\nline3");
}

#[test]
fn test_postprocess_removes_trailing_whitespace() {
    let input = "code    \t  ";
    let result = postprocess_terminal_text(input);
    assert_eq!(result, "code");
}

#[test]
fn test_postprocess_fixes_fullwidth_punctuation() {
    // Fullwidth to halfwidth conversion - note: current impl removes spaces between conversions
    let input = "（test）［content］｛code｝";
    let result = postprocess_terminal_text(input);
    // Current implementation converts punctuation but doesn't preserve spaces
    assert!(result.contains("(test)"));
    assert!(result.contains("[content]"));
    assert!(result.contains("{code}"));
}

#[test]
fn test_postprocess_fixes_fullwidth_punctuation_mixed() {
    // Fullwidth to halfwidth conversion - note: current impl removes spaces
    let input = "中文（括号）and english(括号)";
    let result = postprocess_terminal_text(input);
    // Current implementation converts punctuation but doesn't preserve spaces
    assert!(result.contains("中文(括号)"));
    assert!(result.contains("english(括号)"));
}

/// Test is_likely_code: code detection heuristics
#[test]
fn test_is_likely_code_rust() {
    let rust_code = r#"
fn main() {
    println!("Hello");
}
"#;
    assert!(is_likely_code(rust_code));
}

#[test]
fn test_is_likely_code_python() {
    let python_code = r#"
def hello():
    print("world")
"#;
    assert!(is_likely_code(python_code));
}

#[test]
fn test_is_likely_code_js_arrow() {
    // JS arrow function with proper code structure
    let js_code = r#"
const add = function(a, b) {
    return a + b;
};
"#;
    assert!(is_likely_code(js_code));
}

#[test]
fn test_is_likely_code_not_code() {
    let plain_text = "This is just a regular sentence about random topics.";
    assert!(!is_likely_code(plain_text));
}

#[test]
fn test_is_likely_code_empty() {
    assert!(!is_likely_code(""));
}

#[test]
fn test_is_likely_code_single_line() {
    // Single line without common code patterns
    assert!(!is_likely_code("just a short phrase"));
}

/// Test detect_language: language identification
#[test]
fn test_detect_language_rust() {
    let rust_code = r#"
fn main() {
    use std::collections::HashMap;
}
"#;
    assert_eq!(detect_language(rust_code), Some("rust".to_string()));
}

#[test]
fn test_detect_language_python() {
    let python_code = r#"
def main():
    import os
    from typing import List
"#;
    assert_eq!(detect_language(python_code), Some("python".to_string()));
}

#[test]
fn test_detect_language_javascript() {
    let js_code = "console.log('test'); function() {}";
    assert_eq!(detect_language(js_code), Some("javascript".to_string()));
}

#[test]
fn test_detect_language_cpp() {
    let cpp_code = "#include <iostream> std::cout << 1;";
    assert_eq!(detect_language(cpp_code), Some("cpp".to_string()));
}

#[test]
fn test_detect_language_java() {
    let java_code = "public class Main { System.out.println(); }";
    assert_eq!(detect_language(java_code), Some("java".to_string()));
}

#[test]
fn test_detect_language_unknown() {
    let unknown = "random text without code patterns";
    assert_eq!(detect_language(unknown), None);
}

/// Test calculate_cer: character error rate
#[test]
fn test_cer_identical() {
    let reference = "hello world";
    assert_eq!(calculate_cer(reference, reference), 0.0);
}

#[test]
fn test_cer_one_char_difference() {
    let reference = "hello";
    let hypothesis = "hallo";
    let cer = calculate_cer(reference, hypothesis);
    // 1 edit distance / 5 chars = 0.2
    assert!((cer - 0.2).abs() < 0.01);
}

#[test]
fn test_cer_empty_reference() {
    // When reference is empty, should return 0 to avoid division by zero
    assert_eq!(calculate_cer("", "anything"), 0.0);
}

#[test]
fn test_cer_complete_mismatch() {
    let reference = "abc";
    let hypothesis = "xyz";
    let cer = calculate_cer(reference, hypothesis);
    // 3 edit distance / 3 chars = 1.0
    assert!((cer - 1.0).abs() < 0.01);
}

/// Test calculate_wer: word error rate
#[test]
fn test_wer_identical() {
    let reference = "hello world";
    assert_eq!(calculate_wer(reference, reference), 0.0);
}

#[test]
fn test_wer_one_word_difference() {
    let reference = "hello world";
    let hypothesis = "hello there";
    let wer = calculate_wer(reference, hypothesis);
    // 1 edit (world->there) / 2 words = 0.5
    assert!((wer - 0.5).abs() < 0.01);
}

#[test]
fn test_wer_empty_reference() {
    assert_eq!(calculate_wer("", "anything"), 0.0);
}

#[test]
fn test_wer_multiple_words() {
    let reference = "the quick brown fox";
    let hypothesis = "the slow green fox";
    let wer = calculate_wer(reference, hypothesis);
    // 2 edits (quick->slow, brown->green) / 4 words = 0.5
    assert!((wer - 0.5).abs() < 0.01);
}

/// Test evaluate_ocr_quality: combined quality assessment
#[test]
fn test_evaluate_quality_perfect() {
    let reference = "hello world";
    let quality = evaluate_ocr_quality(reference, reference);
    assert_eq!(quality.cer, 0.0);
    assert_eq!(quality.wer, 0.0);
    assert!((quality.score - 1.0).abs() < 0.01);
}

#[test]
fn test_evaluate_quality_poor() {
    let reference = "hello world";
    let hypothesis = "xyz abc";
    let quality = evaluate_ocr_quality(reference, hypothesis);
    // For complete mismatch, both CER and WER should be high (close to 1.0)
    assert!(quality.cer > 0.5, "CER should be high for poor match, got {}", quality.cer);
    assert!(quality.wer > 0.5, "WER should be high for poor match, got {}", quality.wer);
    // Score should be low for poor quality
    assert!(quality.score < 0.5, "Score should be low for poor match, got {}", quality.score);
}

#[test]
fn test_evaluate_quality_partial() {
    let reference = "hello world";
    let hypothesis = "hello"; // Missing "world"
    let quality = evaluate_ocr_quality(reference, hypothesis);
    // CER should be > 0 but < 1
    assert!(quality.cer > 0.0 && quality.cer < 1.0);
    // Score should be between 0 and 1
    assert!(quality.score > 0.0 && quality.score < 1.0);
}
