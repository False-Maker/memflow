# Wave 4: 测试与稳定化

## TL;DR

> **Quick Summary**: 完成 MemFlow MCP Server 的全面测试验证与端到端集成测试，确保所有工具在 Cursor/Claude 中正常工作
>
> **Deliverables**:
> - 完整的测试套件验证报告
> - 性能基准测试结果
> - Cursor/Claude 端到端验证报告
> - 并发测试与数据完整性验证
>
> **Estimated Effort**: Medium
> **Parallel Execution**: NO - sequential verification tasks
> **Critical Path**: Task 10 → Task 11 → Task 12

---

## Context

### Original Request
Wave 4 的测试与稳定化工作需要完成以下验证：
- Task 10: MCP 自动化测试套件（文件存在，需验证编译和通过）
- Task 11: 集成测试与性能调优（部分完成，需运行基准测试）
- Task 12: Cursor/Claude 端到端验证（完全未完成）

### Current State
经过调研发现：
- ✅ **测试文件齐全**:
  - `protocol_test.rs` (188 lines, 15 tests) - 协议层测试
  - `schema_validation_test.rs` (298 lines, 10 tests) - Schema 验证
  - `mcp_tool_test.rs` (496 lines, 30+ tests) - 工具测试
  - `perf_benchmark.rs` (228 lines, 5 tests) - 性能基准测试
  - `tauri_concurrency_test.rs` - 并发测试（空实现）
- ✅ **所有测试编译通过**: `cargo test -p memflow-mcp` 显示 35 个测试全部通过
- ✅ **MCP_INTEGRATION_GUIDE.md 存在**: 7.6KB 完整集成文档
- ❌ **缺少端到端验证**: 无证据表明在 Cursor/Claude 中实际测试过
- ❌ **性能基准未运行**: perf_benchmark.rs 存在但无实际运行结果
- ❌ **并发测试未实现**: tauri_concurrency_test.rs 是空文件

### Interview Summary
用户要求对 Wave 4 的测试与稳定化工作制定完整计划。核心需求是验证已有测试文件并补充缺失的端到端验证。

### Research Findings
- Cargo 测试已通过：35 个单元测试全部 OK
- 缺少集成测试：需要实际在 Cursor/Claude 中测试 MCP 工具
- 性能基准测试框架存在但未执行

---

## Work Objectives

### Core Objective
验证 MemFlow MCP Server 的测试覆盖率、性能表现和端到端集成功能，确保在 Cursor/Claude 中可正常使用。

### Concrete Deliverables
- 测试验证报告（.sisyphus/evidence/ 目录）
- 性能基准测试结果（p50/p95/p99 延迟数据）
- Cursor/Claude 集成验证报告（带截图）
- 并发压力测试报告

### Definition of Done
- [ ] `cargo test -p memflow-mcp` 全部通过（已验证 ✅）
- [ ] `cargo test -p memflow-mcp --test perf_benchmark` 执行完成
- [ ] 端到端验证：在 Cursor 中调用至少 3 个工具并截图
- [ ] 并发测试：验证与 Tauri App 同时运行无数据损坏
- [ ] 所有验证结果保存到 `.sisyphus/evidence/`

### Must Have
- 所有现有测试必须通过
- 性能基准测试必须执行并记录结果
- 端到端验证必须有实际截图证据
- 并发测试必须验证数据库 WAL 模式是否工作

### Must NOT Have (Guardrails)
- 不得修改已有测试逻辑（只补充缺失测试）
- 不得在端到端验证时破坏生产数据库
- 不得提交包含敏感信息的截图（PII 脱敏）
- 不得使用模拟数据做性能基准测试（需要真实数据库）

---

## Verification Strategy

### Test Decision
- **Infrastructure exists**: YES (Cargo test framework)
- **Automated tests**: Tests-after (单元测试已存在，需补充集成测试)
- **Framework**: cargo test (Rust 内置)

### Agent-Executed QA Scenarios (MANDATORY)

**Scenario: Verify all existing tests pass**
  Tool: Bash
  Preconditions: memflow-mcp crate compiles
  Steps:
    1. cd D:\Demo\memflow
    2. cargo test -p memflow-mcp 2>&1 | tee .sisyphus/evidence/task-10-unit-tests.log
    3. grep -E "test result:|passed|failed" .sisyphus/evidence/task-10-unit-tests.log
  Expected Result: All tests pass, output shows "X passed; 0 failed"
  Evidence: .sisyphus/evidence/task-10-unit-tests.log

**Scenario: Run performance benchmark tests**
  Tool: Bash
  Preconditions: memflow-mcp built in debug mode
  Steps:
    1. cd D:\Demo\memflow
    2. cargo test -p memflow-mcp --test perf_benchmark -- --nocapture 2>&1 | tee .sisyphus/evidence/task-11-perf-benchmark.log
    3. grep -E "p50|p95|p99|avg" .sisyphus/evidence/task-11-perf-benchmark.log
  Expected Result: Benchmark report shows latency metrics
  Evidence: .sisyphus/evidence/task-11-perf-benchmark.log

**Scenario: Verify MCP server binary exists**
  Tool: Bash
  Preconditions: Rust toolchain installed
  Steps:
    1. test -f D:\Demo\memflow\target\release\memflow-mcp.exe && echo "EXISTS" || echo "NOT_FOUND"
    2. if [ ! -f D:\Demo\memflow\target\release\memflow-mcp.exe ]; then cargo build --release -p memflow-mcp; fi
    3. D:\Demo\memflow\target\release\memflow-mcp.exe --version
  Expected Result: Binary exists and version outputs
  Evidence: Version string captured

**Scenario: Test MCP server startup**
  Tool: Bash
  Preconditions: Binary built
  Steps:
    1. timeout 5 D:\Demo\memflow\target\release\memflow-mcp.exe 2>&1 | head -20 || true
    2. echo "Exit code: $?"
  Expected Result: Server starts, initializes database, waits for stdin
  Evidence: Startup log captured

---

## Execution Strategy

### Parallel Execution Waves

所有任务有依赖关系，顺序执行：

```
Wave 1:
├── Task 10: 验证现有测试套件

Wave 2 (after 10):
├── Task 11: 运行性能基准测试

Wave 3 (after 11):
└── Task 12: 端到端集成验证

Critical Path: Task 10 → Task 11 → Task 12
```

### Dependency Matrix

| Task | Depends On | Blocks | Can Parallelize With |
|------|------------|--------|---------------------|
| 10 | None | 11 | None |
| 11 | 10 | 12 | None |
| 12 | 11 | None | None (final verification) |

---

## TODOs

- [x] 10. 验证 MCP 自动化测试套件

  **What to do**:
  - 运行完整的 cargo test 命令验证所有测试
  - 生成测试覆盖率报告（使用 cargo-llvm-cov 如果可用）
  - 记录测试结果到 evidence 目录

  **Must NOT do**:
  - 不得修改现有测试（除非编译失败）
  - 不得跳过任何测试文件

  **Recommended Agent Profile**:
  > **Category**: `unspecified-low`
    - Reason: Simple verification task, run existing tests
  > **Skills**: [`git-master`]
    - `git-master`: For checking if tests were run previously via git history

  **Parallelization**:
  - **Can Run In Parallel**: NO
  - **Parallel Group**: Sequential
  - **Blocks**: Task 11
  - **Blocked By**: None (can start immediately)

  **References**:

  **Pattern References**:
  - `crates/memflow-mcp/tests/*.rs` - All test files to verify
  - `.cargo/config.toml` - Cargo test configuration (if exists)

  **API/Type References**:
  - `memflow_mcp::protocol::*` - Protocol types used in tests
  - `memflow_mcp::context::McpContext` - Context for integration tests

  **Test References**:
  - `crates/memflow-mcp/tests/protocol_test.rs` - Protocol layer tests (15 tests)
  - `crates/memflow-mcp/tests/schema_validation_test.rs` - Schema tests (10 tests)
  - `crates/memflow-mcp/tests/mcp_tool_test.rs` - Tool tests (30+ tests)

  **Documentation References**:
  - `Cargo.toml` (memflow-mcp) - Test dependencies and configuration

  **External References**:
  - Official docs: `https://doc.rust-lang.org/cargo/commands/cargo-test.html` - Cargo test options
  - `https://github.com/taiki-e/cargo-llvm-cov` - Test coverage tool

  **Acceptance Criteria**:
  - [ ] cargo test -p memflow-mcp executes successfully
  - [ ] All 35+ tests pass (0 failed)
  - [ ] Test results saved to .sisyphus/evidence/task-10-unit-tests.log
  - [ ] Test summary shows: "test result: ok. X passed; 0 failed"

  **Agent-Executed QA Scenarios (MANDATORY):**

  ```
  Scenario: Run all unit tests
    Tool: Bash
    Preconditions: Rust toolchain installed, memflow-mcp compiles
    Steps:
      1. cd D:\Demo\memflow
      2. cargo test -p memflow-mcp --verbose 2>&1 | tee .sisyphus/evidence/task-10-unit-tests.log
      3. grep -E "(running|test )+tests" .sisyphus/evidence/task-10-unit-tests.log | tail -20
    Expected Result: All test files executed, summary shows 0 failed
    Evidence: .sisyphus/evidence/task-10-unit-tests.log

  Scenario: Verify test file coverage
    Tool: Bash
    Preconditions: Test files exist
    Steps:
      1. ls -la D:\Demo\memflow\crates\memflow-mcp\tests\ | tee .sisyphus/evidence/task-10-test-files.log
      2. grep -c "test.*rs" .sisyphus/evidence/task-10-test-files.log
    Expected Result: At least 4 test files listed
    Evidence: .sisyphus/evidence/task-10-test-files.log

  Scenario: Check for test compilation warnings
    Tool: Bash
    Preconditions: Tests compile
    Steps:
      1. cargo test -p memflow-mcp --no-run 2>&1 | grep -E "warning:|unused" > .sisyphus/evidence/task-10-warnings.log || echo "No warnings"
      2. wc -l .sisyphus/evidence/task-10-warnings.log
    Expected Result: Warnings documented (acceptable if not critical)
    Evidence: .sisyphus/evidence/task-10-warnings.log
  ```

  **Evidence to Capture**:
  - [ ] .sisyphus/evidence/task-10-unit-tests.log (full test output)
  - [ ] .sisyphus/evidence/task-10-test-files.log (test file list)
  - [ ] .sisyphus/evidence/task-10-warnings.log (compilation warnings)

  **Commit**: NO
  - This task only runs tests, no code changes

---

- [x] 11. 集成测试与性能调优

  **What to do**:
  - 运行 perf_benchmark.rs 获取实际性能数据
  - 实现并运行 tauri_concurrency_test.rs（当前为空）
  - 验证数据库 WAL 模式下的并发读写
  - 记录性能基线（p50/p95/p99 延迟）

  **Must NOT do**:
  - 不得在生产数据库上运行破坏性测试
  - 不得修改现有 perf_benchmark 框架

  **Recommended Agent Profile**:
  > **Category**: `unspecified-low`
    - Reason: Benchmark execution, some test implementation
  > **Skills**: [`git-master`]
    - `git-master`: Check benchmark history

  **Parallelization**:
  - **Can Run In Parallel**: NO
  - **Parallel Group**: Sequential
  - **Blocks**: Task 12
  - **Blocked By**: Task 10

  **References**:

  **Pattern References**:
  - `crates/memflow-mcp/tests/perf_benchmark.rs` - Benchmark framework (reference for implementing new benchmarks)
  - `crates/memflow-mcp/tests/tauri_concurrency_test.rs` - Empty test file to implement
  - `src-tauri/src/db.rs` - Database WAL mode implementation
  - `crates/memflow-core/src/db.rs` - Core database operations

  **API/Type References**:
  - `memflow_mcp::context::McpContext` - Context for MCP operations
  - `memflow_core::db::Database` - Database client for concurrent access

  **Test References**:
  - `src-tauri/tests/ocr_integration_test.rs` - Integration test patterns

  **Documentation References**:
  - `doc/MCP_INTEGRATION_GUIDE.md` - Integration guide (troubleshooting section for database locks)

  **External References**:
  - SQLite WAL: `https://www.sqlite.org/wal.html` - WAL mode documentation
  - Tokio concurrency: `https://tokio.rs/tokio/tutorial/spawning` - Async task spawning

  **Acceptance Criteria**:
  - [ ] perf_benchmark tests execute with --nocapture
  - [ ] Performance metrics recorded (p50/p95/p99 for all tools)
  - [ ] tauri_concurrency_test.rs implemented with at least 2 tests
  - [ ] Concurrent access test verifies no data corruption
  - [ ] Results saved to .sisyphus/evidence/

  **Agent-Executed QA Scenarios (MANDATORY):**

  ```
  Scenario: Run performance benchmarks
    Tool: Bash
    Preconditions: memflow-mcp tests pass
    Steps:
      1. cd D:\Demo\memflow
      2. cargo test -p memflow-mcp --test perf_benchmark -- --nocapture 2>&1 | tee .sisyphus/evidence/task-11-perf-benchmark.log
      3. grep -E "p50|p95|p99|avg|min|max" .sisyphus/evidence/task-11-perf-benchmark.log
    Expected Result: Benchmark report with percentile metrics
    Evidence: .sisyphus/evidence/task-11-perf-benchmark.log

  Scenario: Verify concurrent test implementation
    Tool: Bash
    Preconditions: tauri_concurrency_test.rs exists
    Steps:
      1. wc -l D:\Demo\memflow\crates\memflow-mcp\tests\tauri_concurrency_test.rs
      2. cargo test -p memflow-mcp --test tauri_concurrency_test 2>&1 | tee .sisyphus/evidence/task-11-concurrency.log
    Expected Result: File has >50 lines (implemented), tests run
    Evidence: .sisyphus/evidence/task-11-concurrency.log

  Scenario: Test database WAL mode concurrent reads
    Tool: Bash
    Preconditions: SQLite database exists
    Steps:
      1. cd D:\Demo\memflow
      2. sqlite3 memflow.db "PRAGMA journal_mode;" 2>&1 | grep -i wal
      3. sqlite3 memflow.db "PRAGMA locking_mode;" 2>&1 | grep -i normal
    Expected Result: WAL mode enabled, locking_mode=NORMAL
    Evidence: Mode settings captured

  Scenario: Verify no database locks during concurrent access
    Tool: Bash
    Preconditions: MCP server and Tauri app both running
    Steps:
      1. Start Tauri app (pnpm tauri:dev) in background
      2. Start MCP server (memflow-mcp) in background
      3. Query database from both (simultaneous SELECT)
      4. Check logs for "database is locked" errors
      5. Kill both processes
    Expected Result: No "database locked" errors in logs
    Evidence: Log output captured
  ```

  **Evidence to Capture**:
  - [ ] .sisyphus/evidence/task-11-perf-benchmark.log (benchmark metrics)
  - [ ] .sisyphus/evidence/task-11-concurrency.log (concurrency test results)
  - [ ] .sisyphus/evidence/task-11-wal-mode.log (WAL mode verification)
  - [ ] .sisyphus/evidence/task-11-concurrent-access.log (concurrent access test)

  **Commit**: YES (if implementing tauri_concurrency_test.rs)
  - Message: `test(mcp): add Tauri concurrency integration tests`
  - Files: `crates/memflow-mcp/tests/tauri_concurrency_test.rs`
  - Pre-commit: `cargo test -p memflow-mcp`

---

- [x] 12. Cursor/Claude 端到端验证

  **What to do**:
  - 验证 MCP_INTEGRATION_GUIDE.md 中的配置步骤
  - 在 Cursor 或 Claude Desktop 中配置 memflow MCP
  - 测试所有 6 个工具调用
  - 生成带截图的验证报告
  - 验证错误处理和重试逻辑

  **Must NOT do**:
  - 不得在生产 IDE 配置中暴露敏感数据
  - 不得在截图中包含 API 密钥或个人信息

  **Recommended Agent Profile**:
  > **Category**: `webapp-testing`
    - Reason: Requires browser/IDE automation and screenshot capture
  > **Skills**: [`playwright`]
    - `playwright`: For screenshot capture and UI verification
  - **Skills Evaluated but Omitted**:
    - `dev-browser`: Not suitable - Cursor/Claude are desktop apps, not web browsers

  **Parallelization**:
  - **Can Run In Parallel**: NO
  - **Parallel Group**: Sequential
  - **Blocks**: None (final task)
  - **Blocked By**: Task 11

  **References**:

  **Pattern References**:
  - `doc/MCP_INTEGRATION_GUIDE.md` - Step-by-step integration guide
  - `crates/memflow-mcp/src/server.rs` - MCP server implementation
  - `crates/memflow-mcp/src/tools/` - Tool implementations to verify

  **API/Type References**:
  - MCP Tool Contract: `tools/list`, `tools/call` - JSON-RPC methods
  - Tool parameters: Each tool's schema defined in mcp_tool_test.rs

  **Test References**:
  - `crates/memflow-mcp/tests/mcp_tool_test.rs` - Tool schema tests (reference for parameters)

  **Documentation References**:
  - `doc/MCP_INTEGRATION_GUIDE.md` - Full integration guide including examples

  **External References**:
  - MCP spec: `https://spec.modelcontextprotocol.io/` - Protocol specification
  - Cursor MCP: `https://cursor.sh/docs/features/mcp` - Cursor MCP integration docs
  - Claude MCP: `https://docs.anthropic.com/claude/docs/mcp` - Claude MCP integration docs

  **Acceptance Criteria**:
  - [ ] MCP server binary built and verified
  - [ ] Cursor or Claude configured with memflow MCP
  - [ ] At least 3 tools called successfully from IDE
  - [ ] Screenshots captured showing tool calls and results
  - [ ] Error handling verified (invalid params, empty results)
  - [ ] Verification report saved to .sisyphus/evidence/

  **Agent-Executed QA Scenarios (MANDATORY):**

  **IMPORTANT**: Cursor and Claude Desktop are not web browsers. Playwright cannot directly interact with them.
  Use tmux/interactive_bash to run CLI-based MCP testing instead.

  ```
  Scenario: Verify MCP server binary
    Tool: Bash
    Preconditions: Rust toolchain installed
    Steps:
      1. test -f D:\Demo\memflow\target\release\memflow-mcp.exe || cargo build --release -p memflow-mcp
      2. D:\Demo\memflow\target\release\memflow-mcp.exe --version 2>&1 | tee .sisyphus/evidence/task-12-mcp-version.txt
      3. file D:\Demo\memflow\target\release\memflow-mcp.exe
    Expected Result: Binary exists, version outputs, file type is PE executable
    Evidence: .sisyphus/evidence/task-12-mcp-version.txt

  Scenario: Test MCP tools/list via stdio
    Tool: interactive_bash (tmux)
    Preconditions: MCP server binary exists
    Steps:
      1. tmux new-session -d -s mcp-test "D:\Demo\memflow\target\release\memflow-mcp.exe"
      2. sleep 2
      3. tmux send-keys -t mcp-test '{"jsonrpc":"2.0","method":"tools/list","id":1}' Enter
      4. sleep 1
      5. tmux capture-pane -t mcp-test -p > .sisyphus/evidence/task-12-tools-list.txt
      6. tmux kill-session -t mcp-test
      7. cat .sisyphus/evidence/task-12-tools-list.txt
    Expected Result: JSON-RPC response with tools array containing 6 tools
    Evidence: .sisyphus/evidence/task-12-tools-list.txt

  Scenario: Test MCP tools/call search_memory
    Tool: interactive_bash (tmux)
    Preconditions: MCP server binary exists, database has data
    Steps:
      1. tmux new-session -d -s mcp-test "D:\Demo\memflow\target\release\memflow-mcp.exe"
      2. sleep 2
      3. tmux send-keys -t mcp-test '{"jsonrpc":"2.0","method":"tools/call","params":{"name":"search_memory","arguments":{"query":"test","limit":5}},"id":2}' Enter
      4. sleep 2
      5. tmux capture-pane -t mcp-test -p > .sisyphus/evidence/task-12-search-memory.txt
      6. tmux kill-session -t mcp-test
      7. grep -E "search_memory|content" .sisyphus/evidence/task-12-search-memory.txt
    Expected Result: JSON-RPC response with search results
    Evidence: .sisyphus/evidence/task-12-search-memory.txt

  Scenario: Test MCP tools/call get_recent_activity
    Tool: interactive_bash (tmux)
    Preconditions: MCP server binary exists
    Steps:
      1. tmux new-session -d -s mcp-test "D:\Demo\memflow\target\release\memflow-mcp.exe"
      2. sleep 2
      3. tmux send-keys -t mcp-test '{"jsonrpc":"2.0","method":"tools/call","params":{"name":"get_recent_activity","arguments":{"minutes":5}},"id":3}' Enter
      4. sleep 2
      5. tmux capture-pane -t mcp-test -p > .sisyphus/evidence/task-12-recent-activity.txt
      6. tmux kill-session -t mcp-test
      7. cat .sisyphus/evidence/task-12-recent-activity.txt
    Expected Result: JSON-RPC response with activity timeline
    Evidence: .sisyphus/evidence/task-12-recent-activity.txt

  Scenario: Test MCP error handling with invalid params
    Tool: interactive_bash (tmux)
    Preconditions: MCP server binary exists
    Steps:
      1. tmux new-session -d -s mcp-test "D:\Demo\memflow\target\release\memflow-mcp.exe"
      2. sleep 2
      3. tmux send-keys -t mcp-test '{"jsonrpc":"2.0","method":"tools/call","params":{"name":"search_memory","arguments":{}},"id":4}' Enter
      4. sleep 1
      5. tmux capture-pane -t mcp-test -p > .sisyphus/evidence/task-12-error-handling.txt
      6. tmux kill-session -t mcp-test
      7. grep -E "error|-32602|Invalid params" .sisyphus/evidence/task-12-error-handling.txt
    Expected Result: JSON-RPC error response with code -32602
    Evidence: .sisyphus/evidence/task-12-error-handling.txt

  Scenario: Generate verification report
    Tool: Bash
    Preconditions: All MCP test outputs captured
    Steps:
      1. cat > .sisyphus/evidence/task-12-verification-report.md << 'EOF'
  # MCP End-to-End Verification Report

  ## Tests Executed
  - tools/list: [STATUS]
  - search_memory: [STATUS]
  - get_recent_activity: [STATUS]
  - Error handling: [STATUS]

  ## Results Summary
  EOF
      2. for f in .sisyphus/evidence/task-12-*.txt; do echo "## $f" >> .sisyphus/evidence/task-12-verification-report.md; echo '```' >> .sisyphus/evidence/task-12-verification-report.md; head -50 "$f" >> .sisyphus/evidence/task-12-verification-report.md; echo '```' >> .sisyphus/evidence/task-12-verification-report.md; done
      3. cat .sisyphus/evidence/task-12-verification-report.md
    Expected Result: Complete verification report with all test outputs
    Evidence: .sisyphus/evidence/task-12-verification-report.md
  ```

  **Evidence to Capture**:
  - [ ] .sisyphus/evidence/task-12-mcp-version.txt (server version)
  - [ ] .sisyphus/evidence/task-12-tools-list.txt (tools/list response)
  - [ ] .sisyphus/evidence/task-12-search-memory.txt (search_memory call)
  - [ ] .sisyphus/evidence/task-12-recent-activity.txt (get_recent_activity call)
  - [ ] .sisyphus/evidence/task-12-error-handling.txt (error handling)
  - [ ] .sisyphus/evidence/task-12-verification-report.md (final report)

  **Commit**: NO (verification only, no code changes)

---

## Commit Strategy

| After Task | Message | Files | Verification |
|------------|---------|-------|--------------|
| 11 | `test(mcp): add Tauri concurrency integration tests` | crates/memflow-mcp/tests/tauri_concurrency_test.rs | cargo test -p memflow-mcp |

---

## Success Criteria

### Verification Commands
```bash
# Task 10: Unit tests
cargo test -p memflow-mcp 2>&1 | grep "test result:"

# Task 11: Performance benchmarks
cargo test -p memflow-mcp --test perf_benchmark -- --nocapture | grep -E "p50|p95"

# Task 11: Concurrency tests
cargo test -p memflow-mcp --test tauri_concurrency

# Task 12: MCP server verification
target/release/memflow-mcp.exe --version

# Evidence check
ls -la .sisyphus/evidence/task-*.log
ls -la .sisyphus/evidence/task-*.txt
```

### Final Checklist
- [ ] All 35+ unit tests pass
- [ ] Performance benchmarks executed with metrics recorded
- [ ] Concurrency tests implemented and pass
- [ ] MCP tools verified via stdio (at least 3 tools)
- [ ] Error handling verified (invalid params returns -32602)
- [ ] All evidence files saved to .sisyphus/evidence/
- [ ] Verification report generated
