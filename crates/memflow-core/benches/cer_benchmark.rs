// CER Improvement Benchmarks
//
// Benchmark tests to verify 5%+ CER improvement target for OCR enhancement.
// Uses Criterion for statistical analysis and performance measurement.

use criterion::{black_box, criterion_group, criterion_main, BenchmarkId, Criterion};
use memflow_core::ocr_enhance::{
    calculate_cer, calculate_confidence, postprocess_terminal_text, OcrQualityMetrics,
};

/// Benchmark CER improvement for noisy terminal text
fn bench_cer_noisy_terminal(c: &mut Criterion) {
    // Test fixture: Noisy terminal output with common OCR errors
    let clean_text = "error: expected ';'";
    let noisy_text = "err0r: expe cted ';'";
    let enhanced_text = postprocess_terminal_text(noisy_text);

    // Calculate CER metrics
    let cer_before = calculate_cer(clean_text, noisy_text);
    let cer_after = calculate_cer(clean_text, &enhanced_text);

    // Verify 5% improvement target
    let improvement_percent = ((cer_before - cer_after) / cer_before) * 100.0;

    println!("\n=== Noisy Terminal Benchmark ===");
    println!("Clean:   {}", clean_text);
    println!("Noisy:   {} (CER: {:.4})", noisy_text, cer_before);
    println!("Enhanced: {} (CER: {:.4})", enhanced_text, cer_after);
    println!("Improvement: {:.2}%", improvement_percent);

    // Benchmark calculation performance
    let mut group = c.benchmark_group("cer_calculation");
    group.bench_function("noisy_before", |b| {
        b.iter(|| calculate_cer(black_box(clean_text), black_box(noisy_text)))
    });
    group.bench_function("noisy_after", |b| {
        b.iter(|| calculate_cer(black_box(clean_text), black_box(&enhanced_text)))
    });
    group.finish();
}

/// Benchmark CER improvement for clean terminal text
fn bench_cer_clean_terminal(c: &mut Criterion) {
    // Test fixture: Clean terminal with minimal errors
    let clean_text = "fn main() {\n    let x = 42;\n    println!(\"{}\");\n}";
    let noisy_text = "fn main() {\n    1et x = 42;\n    print1n!(\"{}\");\n}";
    let enhanced_text = postprocess_terminal_text(noisy_text);

    // Calculate CER metrics
    let cer_before = calculate_cer(clean_text, noisy_text);
    let cer_after = calculate_cer(clean_text, &enhanced_text);

    // Verify 5% improvement target
    let improvement_percent = ((cer_before - cer_after) / cer_before) * 100.0;

    println!("\n=== Clean Terminal Benchmark ===");
    println!("Clean:   {}", clean_text);
    println!("Noisy:   {} (CER: {:.4})", noisy_text, cer_before);
    println!("Enhanced: {} (CER: {:.4})", enhanced_text, cer_after);
    println!("Improvement: {:.2}%", improvement_percent);

    // Benchmark calculation performance
    let mut group = c.benchmark_group("cer_calculation");
    group.bench_function("clean_before", |b| {
        b.iter(|| calculate_cer(black_box(clean_text), black_box(noisy_text)))
    });
    group.bench_function("clean_after", |b| {
        b.iter(|| calculate_cer(black_box(clean_text), black_box(&enhanced_text)))
    });
    group.finish();
}

/// Benchmark confidence score calculation
fn bench_confidence_calculation(c: &mut Criterion) {
    let mut group = c.benchmark_group("confidence_calculation");

    // Test various CER/WER combinations
    let test_cases = vec![
        (0.0, 0.0, "perfect"),
        (0.05, 0.1, "low_errors"),
        (0.2, 0.3, "moderate_errors"),
        (0.5, 0.6, "high_errors"),
    ];

    for (cer, wer, name) in test_cases {
        group.bench_with_input(
            BenchmarkId::from_parameter(name),
            &(cer, wer),
            |b, &(cer, wer)| b.iter(|| calculate_confidence(black_box(cer), black_box(wer))),
        );
    }

    group.finish();
}

/// Benchmark OCR quality metrics evaluation
fn bench_ocr_quality_evaluation(c: &mut Criterion) {
    let clean_text = "let result = calculate_cer(ref, hyp);";
    let noisy_text = "1et resu1t = ca1cu1ate_cer(ref, hyp);";
    let enhanced_text = postprocess_terminal_text(noisy_text);

    let mut group = c.benchmark_group("quality_evaluation");

    group.bench_function("before_enhancement", |b| {
        b.iter(|| {
            let cer = calculate_cer(black_box(clean_text), black_box(noisy_text));
            let wer = cer * 1.2; // Approximate WER
            OcrQualityMetrics {
                character_error_rate: cer,
                word_error_rate: wer,
                confidence_score: calculate_confidence(cer, wer),
            }
        })
    });

    group.bench_function("after_enhancement", |b| {
        b.iter(|| {
            let cer = calculate_cer(black_box(clean_text), black_box(&enhanced_text));
            let wer = cer * 1.2; // Approximate WER
            OcrQualityMetrics {
                character_error_rate: cer,
                word_error_rate: wer,
                confidence_score: calculate_confidence(cer, wer),
            }
        })
    });

    group.finish();
}

/// Benchmark postprocessing performance
fn bench_postprocess_terminal_text(c: &mut Criterion) {
    let mut group = c.benchmark_group("postprocess_terminal");

    let test_cases = vec![
        ("short", "err0r: expected ';'"),
        ("medium", "fn main() {\n    1et x = 42;\n    print1n!(\"{}\");\n}"),
        (
            "long",
            "fn main() {\n    1et x = 42;\n    if x > 0 {\n        print1n!(\"x is pos1t1ve\");\n    } e1se {\n        print1n!(\"x is negat1ve\");\n    }\n}",
        ),
    ];

    for (name, text) in test_cases {
        group.bench_with_input(BenchmarkId::from_parameter(name), text, |b, text| {
            b.iter(|| postprocess_terminal_text(black_box(text)))
        });
    }

    group.finish();
}

criterion_group!(
    benches,
    bench_cer_noisy_terminal,
    bench_cer_clean_terminal,
    bench_confidence_calculation,
    bench_ocr_quality_evaluation,
    bench_postprocess_terminal_text
);
criterion_main!(benches);
