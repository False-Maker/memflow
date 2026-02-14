# Task 11 Learnings - Performance Benchmarks and Tauri Concurrency Tests

## Date: 2026-02-14

### Performance Benchmark Framework

**File**: `crates/memflow-mcp/tests/perf_benchmark.rs`

**Key Patterns Discovered**:
1. **Benchmark Structure**: Uses a clean `PerformanceBenchmark` struct that collects results with `BenchmarkResult` containing tool_name, iteration, duration_ms, and success status
2. **Percentile Calculation**: Implements p50/p95/p99/min/max/avg metrics calculation using sorted durations array
3. **Concurrent Testing**: Has a test `test_concurrent_requests_handling` that simulates 10 concurrent requests using threads and atomic counters
4. **Criteria Validation**: `meets_criteria` method checks if all tools meet p95 latency threshold

**Observations**:
- The framework is well-designed for measuring tool latency percentiles
- Tests are unit tests, not integration tests with actual MCP tool calls
- No actual tool execution benchmarking occurs - only framework validation tests

### Concurrency Test Implementation

**File**: `crates/memflow-mcp/tests/tauri_concurrency_test.rs`

**Implementation Approach**:
1. **Isolation Strategy**: Use separate test databases in temp directory to avoid conflicts with production DB
2. **WAL Mode Verification**: Explicitly check `PRAGMA journal_mode=WAL` before running concurrent tests
3. **Concurrent Read Test**: Spawns 10 tasks performing 5 reads each (50 total reads)
4. **MCP/Tauri Concurrent Test**: Simulates 1 writer (Tauri) + 5 readers (MCP) with 10 writes and 25 reads

**Key Success Patterns**:
- Use `tokio::spawn` for true concurrent execution
- Track lock errors using `Arc<Mutex>` shared state
- Verify data integrity after concurrent operations
- Write evidence logs to `.sisphus/evidence/` directory

### Database WAL Configuration

**File**: `crates/memflow-core/src/db.rs`

**Configuration Details**:
```rust
SqliteConnectOptions::new()
    .filename(&db_path)
    .create_if_missing(true)
    .journal_mode(SqliteJournalMode::Wal)
    .busy_timeout(Duration::from_secs(5));
```

**Key Learnings**:
- WAL mode is enabled at connection time, not as PRAGMA statement
- 5-second busy timeout provides grace period for edge cases
- WAL allows multiple readers + single writer without blocking

**Verification Results**:
- 50 concurrent reads: 0 lock errors ✓
- Concurrent read/write: 0 lock errors ✓
- Data integrity: No corruption detected ✓

### Test Execution Best Practices

**Do's**:
1. Use `std::env::temp_dir()` for test databases to avoid production data
2. Clean up existing test databases before running tests
3. Use `Arc<Mutex>` for shared state in concurrent tests
4. Capture evidence logs for audit trail
5. Verify WAL mode explicitly before concurrent tests

**Don'ts**:
1. Don't use production database path for concurrent tests
2. Don't skip WAL mode verification - it's critical for correctness
3. Don't mix unit tests and integration tests in same file
4. Don't forget to check for database lock errors in concurrent scenarios

### Evidence Collection Pattern

**Standard Evidence Files**:
- `task-11-perf-benchmark.log`: Full test execution output
- `task-11-concurrency.log`: Concurrency test execution output
- `task-11-concurrent-read.log`: Specific test results summary
- `task-11-mcp-tauri-concurrent.log`: MCP/Tauri concurrent access summary
- `task-11-wal-mode.log`: WAL mode verification report

**Pattern**: Create both raw execution logs AND summary evidence files for human readability.
