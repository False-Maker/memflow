// Integration tests for OCR enhancement module
// Tests end-to-end workflow: preprocess → OCR → postprocess → quality check

#[cfg(test)]
mod ocr_enhancement_integration_tests {
    use image::{GrayImage, ImageBuffer, Luma};
    use memflow_core::ocr_enhance::{
        calculate_cer, calculate_confidence, calculate_wer, detect_language, evaluate_ocr_quality,
        is_likely_code, postprocess_terminal_text, preprocess_terminal_image, suggest_corrections,
        ProgrammingLanguage,
    };
    use std::io::Cursor;

    // ============================================================================
    // Full Workflow Tests: Preprocess → OCR (simulated) → Postprocess → Quality
    // ============================================================================

    #[test]
    fn test_full_workflow_rust_code() {
        // Test complete enhancement workflow for Rust code
        let raw_ocr_text = r#"fn main() {
    let x = 1;
    println!("hello");
}"#;

        // Step 1: Detect if code
        let is_code = is_likely_code(raw_ocr_text);
        assert!(is_code, "Rust code should be detected");

        // Step 2: Detect language
        let lang = detect_language(raw_ocr_text);
        assert_eq!(lang, ProgrammingLanguage::Rust, "Should detect Rust");

        // Step 3: Postprocess (skip preprocess for text-only test)
        let enhanced = postprocess_terminal_text(raw_ocr_text);

        // Step 4: Evaluate quality
        let metrics = evaluate_ocr_quality(raw_ocr_text, &enhanced);

        // Verify enhancement preserves structure
        assert!(enhanced.contains("fn"), "Function keyword preserved");
        assert!(enhanced.contains("let"), "Variable declaration preserved");
        assert!(enhanced.contains("println"), "Macro preserved");

        // Verify quality metrics are reasonable
        assert!(metrics.character_error_rate >= 0.0, "CER is valid");
        assert!(metrics.word_error_rate >= 0.0, "WER is valid");
        assert!(
            metrics.confidence_score >= 0.0 && metrics.confidence_score <= 1.0,
            "Confidence in range"
        );
    }

    #[test]
    fn test_full_workflow_python_code() {
        // Test complete enhancement workflow for Python code
        let raw_ocr_text = r#"def calculate(x, y):
    return x + y

result = calculate(1, 2)"#;

        let is_code = is_likely_code(raw_ocr_text);
        assert!(is_code, "Python code should be detected");

        let lang = detect_language(raw_ocr_text);
        assert_eq!(lang, ProgrammingLanguage::Python, "Should detect Python");

        let enhanced = postprocess_terminal_text(raw_ocr_text);

        assert!(enhanced.contains("def"), "Function keyword preserved");
        assert!(enhanced.contains("return"), "Return statement preserved");
        assert!(enhanced.contains("calculate"), "Function name preserved");
    }

    #[test]
    fn test_full_workflow_noisy_terminal_output() {
        // Test realistic terminal output with noise
        let raw_ocr = r#"error: expected ';'
  --> src/main.rs:12:5
   |
12 |     let x = 1
   |         ^^^ expected ';'"#;

        let is_code = is_likely_code(raw_ocr);
        assert!(is_code, "Terminal error output should be detected as code");

        let enhanced = postprocess_terminal_text(raw_ocr);

        // Verify error message structure preserved
        assert!(enhanced.contains("error"), "Error keyword preserved");
        assert!(enhanced.contains("expected"), "Expected message preserved");
        assert!(enhanced.contains("let"), "Code keyword preserved");
    }

    // ============================================================================
    // Image Preprocessing Integration Tests
    // ============================================================================

    #[test]
    fn test_preprocess_terminal_image_integration() {
        // Create test image simulating terminal screenshot
        let mut img: GrayImage = ImageBuffer::new(100, 50);
        for y in 0..50 {
            for x in 0..100 {
                let value = if x < 80 && y < 30 { 200 } else { 50 };
                img.put_pixel(x, y, Luma([value]));
            }
        }

        let mut input = Vec::new();
        img.write_to(&mut Cursor::new(&mut input), image::ImageFormat::Png)
            .expect("Failed to create test image");

        // Apply preprocessing
        let start = std::time::Instant::now();
        let output = preprocess_terminal_image(&input);
        let duration = start.elapsed();

        // Verify preprocessing completed
        assert!(!output.is_empty(), "Preprocessing returns data");
        assert_ne!(output, input, "Preprocessing modifies image");

        // Verify output is valid image
        let output_img = image::load_from_memory(&output);
        assert!(output_img.is_ok(), "Output is valid image");

        // Verify binarization (should only have 0 or 255)
        let output_gray = output_img.unwrap().to_luma8();
        for pixel in output_gray.pixels() {
            assert!(
                pixel[0] == 0 || pixel[0] == 255,
                "Binarization applied: {}",
                pixel[0]
            );
        }

        // Performance check: should complete in reasonable time
        assert!(
            duration.as_millis() < 1000,
            "Preprocessing completes quickly: {}ms",
            duration.as_millis()
        );
    }

    #[test]
    fn test_preprocess_preserves_dimensions() {
        // Verify preprocessing preserves image dimensions
        let mut img: GrayImage = ImageBuffer::new(80, 40);
        for y in 0..40 {
            for x in 0..80 {
                img.put_pixel(x, y, Luma([128]));
            }
        }

        let mut input = Vec::new();
        img.write_to(&mut Cursor::new(&mut input), image::ImageFormat::Png)
            .expect("Failed to create test image");

        let output = preprocess_terminal_image(&input);
        let output_img = image::load_from_memory(&output).unwrap().to_luma8();

        assert_eq!(output_img.width(), 80, "Width preserved");
        assert_eq!(output_img.height(), 40, "Height preserved");
    }

    #[test]
    fn test_preprocess_handles_invalid_input() {
        // Verify preprocessing handles invalid input gracefully
        let invalid_input = b"not a valid image";
        let output = preprocess_terminal_image(invalid_input);

        // Should return original input if parsing fails
        assert_eq!(output, invalid_input, "Returns original on error");
    }

    // ============================================================================
    // Code Detection Gating Tests
    // ============================================================================

    #[test]
    fn test_code_detection_gating_rust() {
        // Test that enhancement is gated by code detection for Rust
        let rust_code = r#"fn main() {
    let x = 1;
    println!("{}", x);
}"#;

        let is_code = is_likely_code(rust_code);
        assert!(is_code, "Rust code detected");

        // Enhancement should be applied
        let enhanced = postprocess_terminal_text(rust_code);
        assert!(!enhanced.is_empty(), "Enhancement produces output");
    }

    #[test]
    fn test_code_detection_gating_python() {
        // Test that enhancement is gated by code detection for Python
        let python_code = r#"class MyClass:
    def __init__(self):
        self.value = 42

    def get_value(self):
        return self.value"#;

        let is_code = is_likely_code(python_code);
        assert!(is_code, "Python code detected");

        let lang = detect_language(python_code);
        assert_eq!(lang, ProgrammingLanguage::Python, "Language detected");
    }

    #[test]
    fn test_code_detection_gating_javascript() {
        // Test that enhancement is gated by code detection for JavaScript
        let js_code = r#"function greet(name) {
    const message = `Hello, ${name}!`;
    console.log(message);
}

greet("World");"#;

        let is_code = is_likely_code(js_code);
        assert!(is_code, "JavaScript code detected");

        let lang = detect_language(js_code);
        assert_eq!(lang, ProgrammingLanguage::JavaScript, "Language detected");
    }

    #[test]
    fn test_code_detection_gating_no_false_positives() {
        // Test that normal text doesn't trigger enhancement
        let normal_text = r#"Email: test@example.com
Phone: +1-234-567-8900
Address: 123 Main St"#;

        let is_code = is_likely_code(normal_text);
        assert!(!is_code, "Normal text not detected as code");

        // Enhancement should still work but preserve original
        let enhanced = postprocess_terminal_text(normal_text);
        assert!(enhanced.contains("@"), "Email preserved");
    }

    #[test]
    fn test_code_detection_gating_with_long_digits() {
        // Test that long digit sequences don't trigger false positives
        let text_with_ssn = "SSN: 123-45-6789\nPhone: 555-123-4567";

        let is_code = is_likely_code(text_with_ssn);
        assert!(!is_code, "Text with SSN/phone not detected as code");
    }

    #[test]
    fn test_code_detection_multi_language() {
        // Test code detection across multiple languages
        let rust = "fn main() { println!(\"test\"); }";
        let python = "def test(): pass";
        let js = "const x = () => {}";
        let cpp = "#include <iostream>";

        assert!(is_likely_code(rust), "Rust detected");
        assert!(is_likely_code(python), "Python detected");
        assert!(is_likely_code(js), "JavaScript detected");
        assert!(is_likely_code(cpp), "C++ detected");

        // Verify language detection
        assert_eq!(detect_language(rust), ProgrammingLanguage::Rust);
        assert_eq!(detect_language(python), ProgrammingLanguage::Python);
        assert_eq!(detect_language(js), ProgrammingLanguage::JavaScript);
        assert_eq!(detect_language(cpp), ProgrammingLanguage::Cpp);
    }

    // ============================================================================
    // Performance and Timing Tests
    // ============================================================================

    #[test]
    fn test_performance_preprocess_timing() {
        // Test that preprocessing completes within time limits
        let mut img: GrayImage = ImageBuffer::new(1920, 1080);
        for y in 0..1080 {
            for x in 0..1920 {
                img.put_pixel(x, y, Luma([128]));
            }
        }

        let mut input = Vec::new();
        img.write_to(&mut Cursor::new(&mut input), image::ImageFormat::Png)
            .expect("Failed to create test image");

        let start = std::time::Instant::now();
        let output = preprocess_terminal_image(&input);
        let duration = start.elapsed();

        // Full HD image should preprocess within 5 seconds
        assert!(
            duration.as_secs() < 5,
            "Preprocessing within time limit: {}s",
            duration.as_secs()
        );
        assert!(!output.is_empty(), "Preprocessing completed");
    }

    #[test]
    fn test_performance_postprocess_timing() {
        // Test that postprocessing completes within time limits
        let part1 = "fn main() {\n".repeat(1000);
        let part2 = "    let x = 1;\n".repeat(1000);
        let large_text = format!("{}{}", part1, part2);

        let start = std::time::Instant::now();
        let enhanced = postprocess_terminal_text(&large_text);
        let duration = start.elapsed();

        // Large text should postprocess within 100ms
        assert!(
            duration.as_millis() < 100,
            "Postprocessing within time limit: {}ms",
            duration.as_millis()
        );
        assert!(!enhanced.is_empty(), "Postprocessing completed");
    }

    #[test]
    fn test_performance_code_detection_timing() {
        // Test that code detection is fast
        let part1 = "fn main() {\n".repeat(100);
        let part2 = "    let x = 1;\n".repeat(100);
        let large_code = format!("{}{}", part1, part2);

        let start = std::time::Instant::now();
        let is_code = is_likely_code(&large_code);
        let duration = start.elapsed();

        assert!(is_code, "Code detected");
        assert!(
            duration.as_millis() < 50,
            "Detection within time limit: {}ms",
            duration.as_millis()
        );
    }

    #[test]
    fn test_performance_quality_metrics_timing() {
        // Test that quality calculation is reasonable
        let reference = "let x = 1;\n".repeat(100);
        let hypothesis = "1et x = 1;\n".repeat(100);

        let start = std::time::Instant::now();
        let metrics = evaluate_ocr_quality(&reference, &hypothesis);
        let duration = start.elapsed();

        assert!(
            duration.as_millis() < 500,
            "Quality calculation within time limit: {}ms",
            duration.as_millis()
        );
        assert!(metrics.character_error_rate > 0.0, "CER calculated");
    }

    // ============================================================================
    // Quality Metrics Integration Tests
    // ============================================================================

    #[test]
    fn test_quality_metrics_perfect_match() {
        // Test quality metrics for perfect match
        let text = "fn main() { println!(\"test\"); }";
        let metrics = evaluate_ocr_quality(text, text);

        assert_eq!(metrics.character_error_rate, 0.0, "Perfect CER");
        assert_eq!(metrics.word_error_rate, 0.0, "Perfect WER");
        assert_eq!(metrics.confidence_score, 1.0, "Perfect confidence");
    }

    #[test]
    fn test_quality_metrics_with_errors() {
        // Test quality metrics with realistic OCR errors
        let reference = "let x = 10;";
        let hypothesis = "1et x = lO;"; // l->1, 1->l, O->0 errors

        let metrics = evaluate_ocr_quality(reference, hypothesis);

        assert!(metrics.character_error_rate > 0.0, "CER detects errors");
        assert!(metrics.character_error_rate < 1.0, "CER not total failure");
        assert!(metrics.confidence_score < 1.0, "Confidence reflects errors");
    }

    #[test]
    fn test_quality_metrics_confidence_calculation() {
        // Test confidence calculation with various error rates
        let conf1 = calculate_confidence(0.0, 0.0);
        assert_eq!(conf1, 1.0, "No errors = perfect confidence");

        let conf2 = calculate_confidence(0.1, 0.1);
        assert!(conf2 > 0.8 && conf2 < 1.0, "Low errors = high confidence");

        let conf3 = calculate_confidence(0.5, 0.5);
        assert!(conf3 < 0.7, "High errors = low confidence");
    }

    #[test]
    fn test_quality_metrics_cer_weighted_higher_than_wer() {
        // Test that CER has higher weight than WER for code
        let conf1 = calculate_confidence(0.1, 0.0); // 10% CER, 0% WER
        let conf2 = calculate_confidence(0.0, 0.1); // 0% CER, 10% WER

        // CER should affect confidence more than WER
        assert!(
            conf1 < conf2,
            "CER weighted higher than WER: {} < {}",
            conf1,
            conf2
        );
    }

    // ============================================================================
    // Suggestion System Integration Tests
    // ============================================================================

    #[test]
    fn test_suggest_corrections_integration() {
        // Test correction suggestion with realistic errors
        let reference = "let x = 10;";
        let hypothesis = "1et x = lO;";

        let corrections = suggest_corrections(reference, hypothesis);

        // Should suggest common OCR confusions
        assert!(!corrections.is_empty(), "Corrections suggested");

        // Check for expected corrections
        let has_l_to_1 = corrections
            .iter()
            .any(|&(wrong, correct)| wrong == '1' && correct == 'l');
        let has_o_to_0 = corrections
            .iter()
            .any(|&(wrong, correct)| wrong == 'O' && correct == '0');

        assert!(has_l_to_1 || has_o_to_0, "Common confusions detected");
    }

    #[test]
    fn test_suggest_corrections_low_cer_threshold() {
        // Test that corrections are only suggested for significant CER
        let reference = "hello world";
        let hypothesis = "hello wor1d"; // 1 char error = 1/11 < 9%

        let corrections = suggest_corrections(reference, hypothesis);
        assert!(corrections.is_empty(), "No suggestions for low CER (< 5%)");
    }

    #[test]
    fn test_suggest_corrections_no_duplicates() {
        // Test that duplicate corrections are removed
        let reference = "let x = 1; let y = 2;";
        let hypothesis = "1et x = 1; 1et y = 2;"; // Repeated l->1 error

        let corrections = suggest_corrections(reference, hypothesis);

        // Check no duplicates
        let mut unique_corrections = corrections.clone();
        unique_corrections.sort();
        unique_corrections.dedup();

        assert_eq!(corrections, unique_corrections, "No duplicate corrections");
    }

    // ============================================================================
    // Bracket Fixing Integration Tests
    // ============================================================================

    #[test]
    fn test_bracket_fixing_preserves_strings() {
        // Test that brackets inside strings are preserved
        let input = r#"print("hello (world")"#;
        let enhanced = postprocess_terminal_text(input);

        // String content should be preserved
        assert!(enhanced.contains("(world"), "Brackets in strings preserved");
    }

    #[test]
    fn test_bracket_fixing_balances_unmatched() {
        // Test that unmatched brackets are balanced
        let input = "fn main() { println!(\"hello\");";
        let enhanced = postprocess_terminal_text(input);

        // Should add missing closing brace
        let open_braces = enhanced.matches('{').count();
        let close_braces = enhanced.matches('}').count();

        assert_eq!(open_braces, close_braces, "Braces balanced");
    }

    #[test]
    fn test_bracket_fixing_raw_strings() {
        // Test that raw strings are preserved
        let input = r##"r#"hello (world"#"##;
        let enhanced = postprocess_terminal_text(input);

        assert!(enhanced.contains("r#"), "Raw string marker preserved");
    }

    // ============================================================================
    // Whitespace Normalization Integration Tests
    // ============================================================================

    #[test]
    fn test_whitespace_preserves_indentation() {
        // Test that code indentation is preserved
        let input = "    fn main() {\n        println!(\"test\");\n    }";
        let enhanced = postprocess_terminal_text(input);

        // Check leading whitespace preserved
        let lines: Vec<&str> = enhanced.lines().collect();
        assert!(lines[0].starts_with("    "), "Outer indentation preserved");
        assert!(
            lines[1].starts_with("        "),
            "Inner indentation preserved"
        );
    }

    #[test]
    fn test_whitespace_normalizes_internal() {
        // Test that internal whitespace is normalized
        let input = "hello     world    test";
        let enhanced = postprocess_terminal_text(input);

        assert_eq!(
            enhanced, "hello world test",
            "Internal whitespace normalized"
        );
    }

    // ============================================================================
    // Symbol Correction Integration Tests
    // ============================================================================

    #[test]
    fn test_symbol_correction_in_code_context() {
        // Test that symbols are corrected in code context
        let input = "1et x = l0;"; // l->1, 0->O errors
        let enhanced = postprocess_terminal_text(input);

        // Should apply code-specific corrections
        // Note: exact output depends on correction rules
        assert!(!enhanced.is_empty(), "Corrections applied");
    }

    #[test]
    fn test_symbol_correction_no_context() {
        // Test that symbols are preserved in non-code context
        let input = "Hello World! 123";
        let enhanced = postprocess_terminal_text(input);

        // Should preserve original text
        assert!(enhanced.contains("Hello"), "Text preserved");
    }

    // ============================================================================
    // End-to-End Scenario Tests
    // ============================================================================

    #[test]
    fn test_e2e_terminal_screenshot_scenario() {
        // Simulate full workflow: screenshot → preprocess → OCR → postprocess
        // Create test image
        let mut img: GrayImage = ImageBuffer::new(200, 100);
        for y in 0..100 {
            for x in 0..200 {
                let value = if y < 80 { 220 } else { 60 };
                img.put_pixel(x, y, Luma([value]));
            }
        }

        let mut image_bytes = Vec::new();
        img.write_to(&mut Cursor::new(&mut image_bytes), image::ImageFormat::Png)
            .expect("Failed to create test image");

        // Step 1: Preprocess image
        let preprocessed = preprocess_terminal_image(&image_bytes);
        assert!(!preprocessed.is_empty(), "Preprocessing successful");

        // Step 2: Simulate OCR (normally done by OCR engine)
        let simulated_ocr = r#"fn main() {
    let x = 1;
    println!("test");
}"#;

        // Step 3: Detect if code
        let is_code = is_likely_code(simulated_ocr);
        assert!(is_code, "Code detected");

        // Step 4: Postprocess
        let enhanced = postprocess_terminal_text(simulated_ocr);

        // Step 5: Evaluate quality
        let metrics = evaluate_ocr_quality(simulated_ocr, &enhanced);

        // Verify workflow completed successfully
        assert!(enhanced.contains("fn"), "Structure preserved");
        assert!(metrics.confidence_score >= 0.0, "Quality evaluated");
    }

    #[test]
    fn test_e2e_error_recovery_scenario() {
        // Test error handling in enhancement workflow
        let invalid_image = b"invalid image data";

        // Should handle gracefully
        let preprocessed = preprocess_terminal_image(invalid_image);
        assert_eq!(preprocessed, invalid_image, "Invalid input handled");

        // Text processing should still work
        let text = "fn main() {}";
        let enhanced = postprocess_terminal_text(text);
        assert!(!enhanced.is_empty(), "Text processing continues");
    }

    #[test]
    fn test_e2e_performance_regression_scenario() {
        // Test for performance regression with typical workload
        let repeated_code = "fn main() {\n".repeat(100);
        let test_cases = vec![
            "fn main() { let x = 1; }",
            "def test(): pass",
            "const x = () => {}",
            &repeated_code,
        ];

        for test_case in test_cases {
            let start = std::time::Instant::now();
            let is_code = is_likely_code(test_case);
            let enhanced = postprocess_terminal_text(test_case);
            let duration = start.elapsed();

            assert!(is_code || !enhanced.is_empty(), "Processing completed");
            assert!(
                duration.as_millis() < 200,
                "Performance acceptable: {}ms",
                duration.as_millis()
            );
        }
    }

    // ============================================================================
    // Integration with OCR Worker Tests
    // ============================================================================

    #[test]
    fn test_worker_integration_enhancement_flow() {
        // Simulate the enhancement flow as used by ocr_worker.rs
        let ocr_text = r#"fn main() {
    let x = 1;
    println!("hello");
}"#;

        // Worker checks: is this code?
        let is_code = is_likely_code(ocr_text);
        assert!(is_code, "Worker detects code");

        // Worker applies enhancement if code detected
        let enhanced = if is_code {
            postprocess_terminal_text(ocr_text)
        } else {
            ocr_text.to_string()
        };

        // Verify enhancement applied
        assert!(
            enhanced.contains("fn"),
            "Enhancement preserves code structure"
        );
        assert!(!enhanced.is_empty(), "Enhancement produces output");
    }

    #[test]
    fn test_worker_integration_timing_validation() {
        // Test that enhancement timing is acceptable for worker
        let realistic_text = r#"fn main() {
    let x = 1;
    let y = 2;
    println!("{} {}", x, y);
}"#;

        let start = std::time::Instant::now();
        let is_code = is_likely_code(realistic_text);
        let enhanced = postprocess_terminal_text(realistic_text);
        let duration = start.elapsed();

        // Worker needs enhancement to be fast (< 50ms for typical text)
        assert!(
            duration.as_millis() < 50,
            "Enhancement fast enough for worker: {}ms",
            duration.as_millis()
        );
        assert!(is_code, "Code detected");
        assert!(!enhanced.is_empty(), "Enhancement completed");
    }

    #[test]
    fn test_worker_integration_preprocessing_timing() {
        // Test that image preprocessing is acceptable for worker
        let mut img: GrayImage = ImageBuffer::new(800, 600);
        for y in 0..600 {
            for x in 0..800 {
                img.put_pixel(x, y, Luma([128]));
            }
        }

        let mut input = Vec::new();
        img.write_to(&mut Cursor::new(&mut input), image::ImageFormat::Png)
            .expect("Failed to create test image");

        let start = std::time::Instant::now();
        let output = preprocess_terminal_image(&input);
        let duration = start.elapsed();

        // Worker needs preprocessing to be fast (< 1s for typical image)
        assert!(
            duration.as_secs() < 1,
            "Preprocessing fast enough for worker: {}s",
            duration.as_secs()
        );
        assert!(!output.is_empty(), "Preprocessing completed");
    }
}
