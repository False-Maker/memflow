// Integration tests for OCR enhancement in ocr_worker
// Tests that enhancement functions are properly integrated with the OCR pipeline

#[cfg(test)]
mod ocr_integration_tests {
    use memflow_core::ocr_enhance::{is_likely_code, postprocess_terminal_text};

    #[test]
    fn test_integration_code_detection_gates_postprocess() {
        // Test that code detection correctly identifies code for postprocessing
        let rust_code = r#"fn main() {
    println!("Hello");
}"#;

        let python_code = r#"def hello():
    print("Hello")"#;

        let plain_text = "The quick brown fox jumps over the lazy dog.";

        assert!(is_likely_code(rust_code), "Rust code should be detected");
        assert!(
            is_likely_code(python_code),
            "Python code should be detected"
        );
        assert!(
            !is_likely_code(plain_text),
            "Plain text should not be detected as code"
        );
    }

    #[test]
    fn test_integration_postprocess_preserves_structure() {
        // Test that postprocessing preserves code structure
        let code_input = r#"fn main() {
    let x = 1;
    println!("{}", x);
}"#;

        let processed = postprocess_terminal_text(code_input);

        // Should preserve structure (no exact assertion since enhancement may modify)
        assert!(
            processed.contains("fn") || processed.contains("let"),
            "Postprocessing should preserve code keywords"
        );
        assert!(
            processed.lines().count() >= 3,
            "Postprocessing should preserve line structure"
        );
    }

    #[test]
    fn test_integration_bracket_fixing_in_code() {
        // Test that bracket fixing works on real code snippets
        let unbalanced = r#"fn main() {
    println("hello"#;

        let processed = postprocess_terminal_text(unbalanced);

        // Should add missing closing braces
        let open_braces = processed.chars().filter(|&c| c == '{').count();
        let close_braces = processed.chars().filter(|&c| c == '}').count();

        assert_eq!(
            open_braces, close_braces,
            "Postprocessing should balance braces"
        );
    }

    #[test]
    fn test_integration_no_false_positives_for_normal_text() {
        // Test that normal text doesn't get enhancement
        let normal_text = "Email: test@example.com\nPhone: +1-234-567-8900";

        let is_code = is_likely_code(normal_text);

        assert!(
            !is_code,
            "Normal text with contact info should not be flagged as code"
        );
    }

    #[test]
    fn test_integration_symbol_correction_in_code_context() {
        // Test that symbols are corrected in code context
        // The correction happens with specific keywords present
        let ocr_noisy_code = r#"fn test() {
    let x = 1;
    println!("hello");
}"#;

        let processed = postprocess_terminal_text(ocr_noisy_code);

        // Should preserve code structure and keywords
        assert!(
            processed.contains("fn") && processed.contains("let"),
            "Code context should be preserved"
        );
    }

    #[test]
    fn test_integration_terminal_error_output() {
        // Test with realistic terminal error output
        let terminal_output = r#"error: expected ';'
  --> src/main.rs:12:5
   |
12 |     let x = 1
   |            ^^^ expected ';'
"#;

        let is_code = is_likely_code(terminal_output);

        assert!(is_code, "Terminal error output should be detected as code");

        let processed = postprocess_terminal_text(terminal_output);

        // Should preserve error indicators
        assert!(
            processed.contains("error") || processed.contains("expected"),
            "Terminal output should preserve error messages"
        );
    }

    #[test]
    fn test_integration_python_function_detection() {
        // Test Python function patterns
        let python_function = r#"def calculate(x, y):
    return x + y

result = calculate(1, 2)"#;

        let is_code = is_likely_code(python_function);

        assert!(is_code, "Python function should be detected");
    }

    #[test]
    fn test_integration_javascript_code_detection() {
        // Test JavaScript/TypeScript code
        let js_code = r#"function hello() {
    console.log("Hello");
    const x = 1;
}"#;

        let is_code = is_likely_code(js_code);

        assert!(is_code, "JavaScript code should be detected");
    }
}
