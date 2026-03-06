use std::path::PathBuf;

use memflow_core::ocr_enhance;

/// Basic integration-style test for OCR enhancement utilities.
///
/// This does not spin up the full Tauri app; instead it focuses on making
/// sure the core `ocr_enhance` APIs behave sanely when used from the
/// desktop OCR pipeline:
/// - `preprocess_terminal_image` should be callable from a dummy path.
/// - `postprocess_terminal_text` should normalize line endings and
///   typical full-width punctuation.
/// - Code/语言检测应在典型代码片段上给出合理结果。
/// - `calculate_cer` / `calculate_wer` / `evaluate_ocr_quality`
///   应在简单样例上返回可预期的数值关系。
#[test]
fn ocr_enhancement_smoke_test() {
    // 1) preprocess should be a no-op for non-existent / dummy path: we only
    // verify that the function can be called and returns a Result. The actual
    // OCR worker covers real images in end-to-end runs.
    let dummy_path = PathBuf::from("tests/fixtures/nonexistent.png");
    let _ = ocr_enhance::preprocess_terminal_image(&dummy_path, 800, 3_000_000);

    // 2) postprocess should fix newlines and common full-width punctuation.
    let raw = "fn main（）｛\r\n    println！（\"hello，world\"）；\r\n｝";
    let processed = ocr_enhance::postprocess_terminal_text(raw);
    assert!(
        processed.contains("fn main()"),
        "expected full-width parens to be normalized"
    );
    assert!(
        processed.contains("println!(\"hello,world\");")
            || processed.contains("println!(\"hello, world\");"),
        "expected comma/semicolon normalization in println! line"
    );
    assert!(
        !processed.contains('\r'),
        "expected CR characters to be normalized to LF"
    );

    // 3) is_likely_code + detect_language should identify this as Rust code.
    assert!(
        ocr_enhance::is_likely_code(&processed),
        "expected processed snippet to be recognized as code"
    );
    let detected_lang = ocr_enhance::detect_language(&processed);
    assert!(
        detected_lang.as_deref() == Some("rust"),
        "expected detected language to be rust, got {:?}",
        detected_lang
    );

    // 4) CER / WER / evaluate_ocr_quality basic relationships.
    let reference = "let value = 42;";
    let hypothesis_same = "let value = 42;";
    let hypothesis_noisy = "let value = 43;";

    let cer_same = ocr_enhance::calculate_cer(reference, hypothesis_same);
    let cer_noisy = ocr_enhance::calculate_cer(reference, hypothesis_noisy);
    let wer_same = ocr_enhance::calculate_wer(reference, hypothesis_same);
    let wer_noisy = ocr_enhance::calculate_wer(reference, hypothesis_noisy);

    assert!(
        (cer_same - 0.0).abs() < f64::EPSILON,
        "CER for identical strings should be 0, got {}",
        cer_same
    );
    assert!(
        cer_noisy > cer_same,
        "CER for noisy hypothesis should be greater than identical case"
    );
    assert!(
        (wer_same - 0.0).abs() < f64::EPSILON,
        "WER for identical strings should be 0, got {}",
        wer_same
    );
    assert!(
        wer_noisy >= wer_same,
        "WER for noisy hypothesis should be >= identical case"
    );

    let quality_same = ocr_enhance::evaluate_ocr_quality(reference, hypothesis_same);
    let quality_noisy = ocr_enhance::evaluate_ocr_quality(reference, hypothesis_noisy);

    assert!(
        (quality_same.cer - cer_same).abs() < 1e-9
            && (quality_same.wer - wer_same).abs() < 1e-9,
        "evaluate_ocr_quality should expose CER/WER from underlying helpers"
    );
    assert!(
        quality_same.score >= quality_noisy.score,
        "quality score for perfect match should be >= noisy hypothesis"
    );
}

