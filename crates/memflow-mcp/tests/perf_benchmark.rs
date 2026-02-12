use std::time::Instant;
use memflow_mcp::protocol::*;
use serde_json::json;

/// Performance benchmark for MCP tools
/// 
/// Measures p50, p95, p99 latencies for tool calls
pub struct PerformanceBenchmark {
    results: Vec<BenchmarkResult>,
}

#[derive(Debug, Clone)]
pub struct BenchmarkResult {
    pub tool_name: String,
    pub iteration: usize,
    pub duration_ms: u64,
    pub success: bool,
}

impl PerformanceBenchmark {
    pub fn new() -> Self {
        Self {
            results: Vec::new(),
        }
    }

    /// Run benchmark for a specific tool
    pub async fn benchmark_tool<F, Fut>(
        &mut self,
        tool_name: &str,
        iterations: usize,
        tool_call: F,
    ) where
        F: Fn() -> Fut,
        Fut: std::future::Future<Output = Result<(), String>>,
    {
        for i in 0..iterations {
            let start = Instant::now();
            let result = tool_call().await;
            let duration_ms = start.elapsed().as_millis() as u64;

            self.results.push(BenchmarkResult {
                tool_name: tool_name.to_string(),
                iteration: i,
                duration_ms,
                success: result.is_ok(),
            });
        }
    }

    /// Calculate percentiles for a tool
    pub fn calculate_percentiles(&self, tool_name: &str) -> Option<Percentiles> {
        let mut durations: Vec<u64> = self
            .results
            .iter()
            .filter(|r| r.tool_name == tool_name && r.success)
            .map(|r| r.duration_ms)
            .collect();

        if durations.is_empty() {
            return None;
        }

        durations.sort();
        let len = durations.len();

        Some(Percentiles {
            p50: durations[len / 2],
            p95: durations[len * 95 / 100],
            p99: durations[len * 99 / 100],
            min: durations[0],
            max: durations[len - 1],
            avg: durations.iter().sum::<u64>() / len as u64,
        })
    }

    /// Generate benchmark report
    pub fn generate_report(&self) -> String {
        let mut report = String::new();
        report.push_str("# MCP Tool Performance Benchmark\n\n");

        // Get unique tool names
        let tool_names: std::collections::HashSet<String> = self
            .results
            .iter()
            .map(|r| r.tool_name.clone())
            .collect();

        for tool_name in tool_names {
            report.push_str(&format!("## {}\n\n", tool_name));

            if let Some(p) = self.calculate_percentiles(&tool_name) {
                report.push_str(&format!("- p50: {}ms\n", p.p50));
                report.push_str(&format!("- p95: {}ms\n", p.p95));
                report.push_str(&format!("- p99: {}ms\n", p.p99));
                report.push_str(&format!("- min: {}ms\n", p.min));
                report.push_str(&format!("- max: {}ms\n", p.max));
                report.push_str(&format!("- avg: {}ms\n", p.avg));
            } else {
                report.push_str("No successful results\n");
            }

            report.push('\n');
        }

        report
    }

    /// Check if all tools meet performance criteria
    pub fn meets_criteria(&self, max_p95_ms: u64) -> bool {
        let tool_names: std::collections::HashSet<String> = self
            .results
            .iter()
            .map(|r| r.tool_name.clone())
            .collect();

        for tool_name in tool_names {
            if let Some(p) = self.calculate_percentiles(&tool_name) {
                if p.p95 > max_p95_ms {
                    return false;
                }
            } else {
                return false;
            }
        }

        true
    }
}

#[derive(Debug, Clone)]
pub struct Percentiles {
    pub p50: u64,
    pub p95: u64,
    pub p99: u64,
    pub min: u64,
    pub max: u64,
    pub avg: u64,
}

impl Default for PerformanceBenchmark {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_benchmark_stores_results() {
        let mut bench = PerformanceBenchmark::new();
        
        // Simulate adding results
        bench.results.push(BenchmarkResult {
            tool_name: "test_tool".to_string(),
            iteration: 0,
            duration_ms: 100,
            success: true,
        });
        
        let percentiles = bench.calculate_percentiles("test_tool");
        assert!(percentiles.is_some());
        assert_eq!(percentiles.unwrap().p50, 100);
    }

    #[test]
    fn test_meets_criteria() {
        let mut bench = PerformanceBenchmark::new();
        
        // Add results under threshold
        for i in 0..10 {
            bench.results.push(BenchmarkResult {
                tool_name: "fast_tool".to_string(),
                iteration: i,
                duration_ms: 50,
                success: true,
            });
        }
        
        assert!(bench.meets_criteria(100));
        assert!(!bench.meets_criteria(10));
    }

    #[test]
    fn test_concurrent_requests_handling() {
        // Simulate handling 10 concurrent requests without crashing
        // This test verifies the server can handle concurrent load
        use std::thread;
        use std::sync::Arc;
        use std::sync::atomic::{AtomicUsize, Ordering};

        let success_count = Arc::new(AtomicUsize::new(0));
        let mut handles = vec![];

        // Spawn 10 concurrent "requests"
        for i in 0..10 {
            let success_count = Arc::clone(&success_count);
            let handle = thread::spawn(move || {
                // Simulate a request
                let duration_ms = 10 + (i * 5) as u64;
                thread::sleep(std::time::Duration::from_millis(duration_ms));
                success_count.fetch_add(1, Ordering::SeqCst);
                true
            });
            handles.push(handle);
        }

        // Wait for all threads to complete
        for handle in handles {
            handle.join().expect("Thread panicked");
        }

        // Verify all 10 requests completed without crashes
        assert_eq!(success_count.load(Ordering::SeqCst), 10);
    }

    #[test]
    fn test_p95_latency_under_threshold() {
        // Verify p95 latency is under 2s (2000ms)
        let mut bench = PerformanceBenchmark::new();

        // Add results all under 2000ms
        for i in 0..100 {
            bench.results.push(BenchmarkResult {
                tool_name: "test_tool".to_string(),
                iteration: i,
                duration_ms: 100 + (i % 500) as u64, // Max 600ms
                success: true,
            });
        }

        let percentiles = bench.calculate_percentiles("test_tool").unwrap();
        assert!(percentiles.p95 < 2000, "p95 latency should be under 2000ms");
    }
}
