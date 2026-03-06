use criterion::{criterion_group, criterion_main, Criterion};
use memflow_core::ocr_enhance;

fn cer_on_long_snippet(c: &mut Criterion) {
    let reference = r#"
fn main() {
    println!("Hello, world!");
    for i in 0..100 {
        println!("value = {}", i);
    }
}
"#;

    // Introduce a small amount of synthetic noise to simulate OCR errors.
    let hypothesis = reference.replace("println!", "printin!")
        .replace("value", "vaule");

    c.bench_function("ocr_enhance::calculate_cer_long_snippet", |b| {
        b.iter(|| {
            let _ = ocr_enhance::calculate_cer(reference, &hypothesis);
        });
    });
}

fn wer_on_long_snippet(c: &mut Criterion) {
    let reference = "the quick brown fox jumps over the lazy dog";
    let hypothesis = "the slow blue fox jump over the lazy cat";

    c.bench_function("ocr_enhance::calculate_wer_long_snippet", |b| {
        b.iter(|| {
            let _ = ocr_enhance::calculate_wer(reference, hypothesis);
        });
    });
}

fn postprocess_text(c: &mut Criterion) {
    let input = r#"
fn main() {
    // This is a comment
    let x = 42;
    println!("Hello, world!");
}
"#;

    c.bench_function("ocr_enhance::postprocess_terminal_text", |b| {
        b.iter(|| {
            let _ = ocr_enhance::postprocess_terminal_text(input);
        });
    });
}

fn is_likely_code_bench(c: &mut Criterion) {
    let rust_code = r#"
fn main() {
    let result = vec![1, 2, 3].iter().map(|x| x * 2).collect::<Vec<_>>();
    for item in result {
        println!("{}", item);
    }
}
"#;

    c.bench_function("ocr_enhance::is_likely_code", |b| {
        b.iter(|| {
            let _ = ocr_enhance::is_likely_code(rust_code);
        });
    });
}

fn detect_language_bench(c: &mut Criterion) {
    let python_code = r#"
def main():
    import os
    from typing import List, Dict
    
    class MyClass:
        def __init__(self):
            self.value = 42
"#;

    c.bench_function("ocr_enhance::detect_language", |b| {
        b.iter(|| {
            let _ = ocr_enhance::detect_language(python_code);
        });
    });
}

fn evaluate_ocr_quality_bench(c: &mut Criterion) {
    let reference = r#"MemFlow is a smart desktop activity recording and analysis tool.
It focuses on visual memory functionality. Through automatic screenshots,
OCR text extraction, AI analysis and other technologies, it helps users
record and analyze computer activities to build a personal knowledge graph."#;

    let hypothesis = r#"MemFlow is a smart desltop activity recording and analysis tool.
It focus on visual memor functionality. Through automatic screenshot,
OCR text extraction, AI analisys and other technologis, it helps users
record and analize computer activities to build a personal knowledge graph."#;

    c.bench_function("ocr_enhance::evaluate_ocr_quality", |b| {
        b.iter(|| {
            let _ = ocr_enhance::evaluate_ocr_quality(reference, hypothesis);
        });
    });
}

criterion_group!(
    benches,
    cer_on_long_snippet,
    wer_on_long_snippet,
    postprocess_text,
    is_likely_code_bench,
    detect_language_bench,
    evaluate_ocr_quality_bench
);
criterion_main!(benches);

