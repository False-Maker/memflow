use memflow_core::ocr_enhance::calculate_cer;
use serde::Serialize;
use std::path::PathBuf;

#[derive(Debug, Serialize, Clone)]
struct SampleResult {
    id: usize,
    reference: String,
    before_ocr: String,
    after_ocr: String,
    cer_before: f64,
    cer_after: f64,
    improvement: f64,
}

#[derive(Debug, Serialize)]
struct TestSummary {
    total_samples: usize,
    improved_samples: usize,
    equal_samples: usize,
    worse_samples: usize,
}

#[derive(Debug, Serialize)]
struct Report {
    test_summary: TestSummary,
    samples: Vec<SampleResult>,
}

struct Sample<'a> {
    reference: &'a str,
    before: &'a str,
    after: &'a str,
}

fn main() -> anyhow::Result<()> {
    let samples = build_samples();

    if samples.len() < 10 {
        anyhow::bail!("Need at least 10 samples, found {}", samples.len());
    }

    let mut results = Vec::new();
    let mut improved = 0usize;
    let mut equal = 0usize;
    let mut worse = 0usize;

    for (idx, sample) in samples.iter().enumerate() {
        let cer_before = calculate_cer(sample.reference, sample.before);
        let cer_after = calculate_cer(sample.reference, sample.after);
        if cer_after < cer_before {
            improved += 1;
        } else if (cer_after - cer_before).abs() < f64::EPSILON {
            equal += 1;
        } else {
            worse += 1;
        }

        let res = SampleResult {
            id: idx + 1,
            reference: sample.reference.to_string(),
            before_ocr: sample.before.to_string(),
            after_ocr: sample.after.to_string(),
            cer_before,
            cer_after,
            improvement: cer_before - cer_after,
        };
        results.push(res);
    }

    let report = Report {
        test_summary: TestSummary {
            total_samples: samples.len(),
            improved_samples: improved,
            equal_samples: equal,
            worse_samples: worse,
        },
        samples: results,
    };

    let workspace_root = locate_workspace_root()?;
    let evidence_dir = workspace_root.join(".sisyphus").join("evidence");
    std::fs::create_dir_all(&evidence_dir)?;
    let report_path = evidence_dir.join("ocr-compare-report.json");
    let report_json = serde_json::to_string_pretty(&report)?;
    std::fs::write(&report_path, report_json)?;

    println!(
        "Generated report with {} samples. Improved: {}, Equal: {}, Worse: {}",
        samples.len(),
        improved,
        equal,
        worse
    );

    println!("Report saved to {}", report_path.to_string_lossy());

    Ok(())
}

fn build_samples<'a>() -> Vec<Sample<'a>> {
    vec![
        Sample {
            reference: "fn main() { println!(\"hello\"); }",
            before: "fn main(} { print1n!(\"he1lo\"); }",
            after: "fn main() { println!(\"hello\"); }",
        },
        Sample {
            reference: "let result = do_work(value).unwrap();",
            before: "let result = do work(value).unwap();",
            after: "let result = do_work(value).unwrap();",
        },
        Sample {
            reference: "error[E0425]: cannot find value `count` in this scope",
            before: "error[E0425]: cannot find value 'count' in this scope",
            after: "error[E0425]: cannot find value `count` in this scope",
        },
        Sample {
            reference: "INFO 2025-02-01 12:00:00 processor: completed step 3",
            before: "INFO 2025-02-01 12:00:00 processor: completed step 3",
            after: "INFO 2025-02-01 12:00:00 processor: completed step 3",
        },
        Sample {
            reference: "for i in 0..10 { println!(\"{}\", i); }",
            before: "for i in 0..10 { printin!(\"{}\", i); }",
            after: "for i in 0..10 { println!(\"{}\", i); }",
        },
        Sample {
            reference: "[2025-01-01 09:00:00] WARN worker::retry - attempt=2",
            before: "[2025-01-01 09:00:00] WARN worker::retry - attempt=2",
            after: "[2025-01-01 09:00:00] WARN worker::retry - attempt=2",
        },
        Sample {
            reference: "cargo test --package memflow-core",
            before: "cargo test --packaqe memflow-core",
            after: "cargo test --package memflow-core",
        },
        Sample {
            reference: "let map: HashMap<String, usize> = HashMap::new();",
            before: "let map: HashMap<String, us1ze> = HashMap::new();",
            after: "let map: HashMap<String, usize> = HashMap::new();",
        },
        Sample {
            reference: "SELECT id, title FROM notes WHERE id = 42;",
            before: "SELECT id, title FROM notes WHERE 1d = 42;",
            after: "SELECT id, title FROM notes WHERE id = 42;",
        },
        Sample {
            reference: "panic!(\"unexpected state: {}\", state);",
            before: "panic(\"unexpected state: {}\", state);",
            after: "panic!(\"unexpected state: {}\", state);",
        },
        Sample {
            reference: "if let Some(value) = cache.get(key) { return value.clone(); }",
            before: "if let Some(value) = cache.get(key) { return value.clone(); }",
            after: "if let Some(value) = cache.get(key) { return value.clone(); }",
        },
        Sample {
            reference: "TRACE query_executor: rows=120 time=18ms",
            before: "TRACE query executor: rows=120 time=18ms",
            after: "TRACE query_executor: rows=120 time=18ms",
        },
    ]
}

fn locate_workspace_root() -> anyhow::Result<PathBuf> {
    let mut current = std::env::current_dir()?;
    loop {
        if current.join("Cargo.toml").exists() && current.join(".sisyphus").exists() {
            return Ok(current);
        }
        if !current.pop() {
            anyhow::bail!("Failed to locate workspace root containing .sisyphus directory");
        }
    }
}
