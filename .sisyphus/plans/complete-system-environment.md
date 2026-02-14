# Complete get_system_environment Implementation

## TL;DR

> **Quick Summary**: Complete the `call_get_system_environment` function (main.rs:1133-1159) which currently ignores all three parameters (`include_dev_tools`, `include_processes`, `include_ports`). Implement development tool version detection, filtered process listing, and port usage checking with graceful error handling.
>
> **Deliverables**:
> - Updated `call_get_system_environment` function implementing all three parameters
> - Helper functions for tool version detection, process filtering, and port checking
> - Updated unit tests covering the new functionality
>
> **Estimated Effort**: Medium
> **Parallel Execution**: NO - sequential implementation within single function
> **Critical Path**: Add dependencies → Implement helpers → Update main function → Add tests

---

## Context

### Original Request
User identified that `get_system_environment` (main.rs:1133-1159) is only 30% complete. Three function parameters are ignored:
- `include_dev_tools`: Should detect Node, Python, Rust, Docker versions
- `include_processes`: Should list active development processes
- `include_ports`: Should check common port usage

### Interview Summary
**Key Discussions**:
- **Process filtering**: User confirmed to filter only development-related processes (VSCode, node, python, rustc, cargo, docker, java, go)
- **Port checking**: User confirmed to support custom port list with defaults (3000, 3001, 8000, 8080, 5000, 4200, 5173, 4000, 9000)
- **Error handling**: User confirmed to include error messages when tools are not installed (not fail-fast)

**Research Findings**:
- Project uses `sysinfo 0.30`, `tokio`, `anyhow`, `tracing`
- No existing pattern for shell command execution in codebase
- Function must return `Result<String>` with formatted text output
- Tests exist in `crates/memflow-mcp/tests/mcp_tool_test.rs`

### Metis Review
**Critical Questions Identified** (resolved with defaults):
1. **Command execution**: Use `tokio::process::Command` with 2-3s timeout per tool
2. **Output format**: Sectioned text output (Tools, Processes, Ports) with stable ordering
3. **Process details**: Include process name, PID, CPU %, memory usage
4. **Port reporting**: Report both "in use" and "free" ports with process name if available
5. **Error aggregation**: Inline errors per item (fail-soft, don't abort entire report)
6. **Process limit**: Cap at 50 processes to avoid huge outputs
7. **Port list behavior**: Merge user-provided ports with defaults (deduplicate)
8. **Process name normalization**: Case-insensitive, strip `.exe` extension
9. **Python detection**: Try `python --version` first, fallback to `python3 --version`
10. **Truncation**: Capture first line only for version outputs

**Guardrails Applied**:
- Timeouts per external command (2-3s) to avoid hanging
- Fail-soft: one tool failure doesn't abort report
- Case-insensitive process name matching with extension stripping
- Deduplicate ports; cap reported processes at 50
- Whitelist commands only (no arbitrary user input)
- Consistent, stable ordering for testability
- Truncate overly long outputs

---

## Work Objectives

### Core Objective
Complete the `call_get_system_environment` function to honor all three parameters with proper error handling and cross-platform compatibility.

### Concrete Deliverables
1. Updated `call_get_system_environment` function implementing all parameters
2. Helper module/functions for:
   - Tool version detection (async with timeout)
   - Process filtering and formatting
   - Port availability checking
3. Unit tests covering new functionality
4. Error handling that reports failures without crashing

### Definition of Done
- [ ] All three parameters properly implemented and used
- [ ] `cargo test --package memflow-mcp` passes
- [ ] Manual verification with `include_dev_tools=true`, `include_processes=true`, `include_ports=true`
- [ ] Error handling verified by testing on system without some tools installed
- [ ] Cross-platform behavior verified (Windows process name handling)

### Must Have
- Implement all three parameters (no partial completion)
- Graceful error handling (fail-soft per item)
- Cross-platform compatibility (Windows/Linux/macOS)
- 2-3s timeout per external command
- Case-insensitive process name matching
- Stable output ordering for testability

### Must NOT Have (Guardrails)
- **No JSON serialization** - keep human-readable text output only
- **No exhaustive process metrics** - limit to name, PID, CPU %, memory
- **No port scanning beyond list** - only check provided/default ports
- **No arbitrary command execution** - whitelist commands only
- **No adding more dev tools** - only specified set (Node, Python, Rust, Docker)
- **No shell pipelines** - use direct `tokio::process::Command` without pipes
- **No deep Docker inspection** - just version availability

---

## Verification Strategy (MANDATORY)

> **UNIVERSAL RULE: ZERO HUMAN INTERVENTION**
>
> ALL tasks in this plan MUST be verifiable WITHOUT any human action.
> ALL verification is executed by the agent using tools (Bash, cargo test). No exceptions.

### Test Decision
- **Infrastructure exists**: YES (cargo test framework)
- **Automated tests**: YES (Tests-after)
- **Framework**: cargo test (Rust built-in)

### Test Approach
Since this is completing an existing function with a well-defined interface, we'll use **Tests-after** approach:
1. Implement the functionality first
2. Add unit tests that verify each section (tools, processes, ports)
3. Use mocking/stubbing for platform-independent tests where possible

### Test Structure

**New test file**: `crates/memflow-mcp/tests/system_environment_test.rs`

```rust
#[tokio::test]
async fn test_system_environment_dev_tools() {
    // Test tool detection with mocked commands or real system calls
}

#[tokio::test]
async fn test_system_environment_processes() {
    // Test process filtering logic
}

#[tokio::test]
async fn test_system_environment_ports() {
    // Test port checking logic
}

#[tokio::test]
async fn test_system_environment_error_handling() {
    // Test that missing tools don't crash the function
}

#[tokio::test]
async fn test_process_name_normalization() {
    // Test case-insensitive matching and .exe stripping
}

#[tokio::test]
async fn test_output_format_stability() {
    // Test that ordering is consistent
}
```

### Agent-Executed QA Scenarios (MANDATORY — per-task, ultra-detailed)

```
Scenario: Verify all three parameters work correctly
  Tool: Bash (cargo + RUST_LOG)
  Preconditions: Project builds, at least one dev tool installed (node/python/rust)
  Steps:
    1. cargo build --package memflow-mcp
    2. cargo test --package memflow-mcp system_environment
    3. Run integration test: call_get_system_environment(true, true, true)
    4. Assert output contains "[System Environment]" section
    5. Assert output contains "Dev Tools" section when include_dev_tools=true
    6. Assert output contains "Processes" section when include_processes=true
    7. Assert output contains "Port Usage" section when include_ports=true
    8. Verify at least one tool version or error message appears
    9. Verify process list contains only dev-related processes (case-insensitive)
    10. Verify port list shows status for each default port
  Expected Result: All sections present, properly formatted, no crashes
  Evidence: Test output captured to .sisyphus/evidence/task-2-test-output.txt

Scenario: Verify parameter exclusion works
  Tool: Bash (cargo test)
  Preconditions: None
  Steps:
    1. Call get_system_environment with all params=false
    2. Assert output contains basic system info (OS, memory, etc.)
    3. Assert output does NOT contain "Dev Tools" section
    4. Assert output does NOT contain "Processes" section
    5. Assert output does NOT contain "Port Usage" section
  Expected Result: Only basic info, no additional sections
  Evidence: Test output

Scenario: Verify error handling for missing tools
  Tool: Bash (cargo test + uninstall simulation)
  Preconditions: System without at least one dev tool (e.g., Docker)
  Steps:
    1. Call get_system_environment(include_dev_tools=true, ...)
    2. Assert output includes error message for missing tool (e.g., "docker: not installed")
    3. Assert other tools still detected successfully
    4. Assert function returns Ok() even with missing tools
  Expected Result: Graceful degradation, no panic
  Evidence: Test output

Scenario: Verify process filtering and normalization
  Tool: Bash (cargo test)
  Preconditions: At least one dev process running (e.g., code.exe, node)
  Steps:
    1. Start a test process if needed: node --version
    2. Call get_system_environment(include_processes=true, ...)
    3. Assert process list includes dev processes (case-insensitive)
    4. Assert non-dev processes excluded (e.g., notepad, explorer)
    5. Assert .exe extensions stripped from names
    6. Assert each process includes PID, CPU %, memory usage
  Expected Result: Only dev-related processes, properly formatted
  Evidence: Process output captured

Scenario: Verify port checking functionality
  Tool: Bash (cargo test + netcat/socat)
  Preconditions: Network access available
  Steps:
    1. Start test server on port 3000: python -m http.server 3000 &
    2. Call get_system_environment(include_ports=true, ...)
    3. Assert output shows "3000: in use"
    4. Assert output shows process using port 3000 (python/http.server)
    5. Assert other ports show "free" status
    6. Kill test server
  Expected Result: Correct port status reporting
  Evidence: Port output captured

Scenario: Verify timeout handling
  Tool: Bash (cargo test + mock hanging command)
  Preconditions: Test environment with mock tools
  Steps:
    1. Create mock command that hangs: sleep 10
    2. Call get_system_environment with 2s timeout
    3. Assert function completes within 5s total
    4. Assert timeout error recorded for hanging tool
    5. Assert other tools still detected successfully
  Expected Result: Function completes, timeout handled gracefully
  Evidence: Execution time captured

Scenario: Cross-platform compatibility (Windows)
  Tool: Bash (cargo test)
  Preconditions: Running on Windows
  Steps:
    1. Call get_system_environment with all params=true
    2. Assert process names normalized (Code.exe → code)
    3. Assert paths handled correctly in tool detection
    4. Assert Windows-specific errors handled (e.g., command not found)
  Expected Result: Works correctly on Windows
  Evidence: Test output

Scenario: Cross-platform compatibility (Unix)
  Tool: Bash (cargo test)
  Preconditions: Running on Linux/macOS
  Steps:
    1. Call get_system_environment with all params=true
    2. Assert python3 fallback works if python not found
    3. Assert process names without extensions handled
    4. Assert Unix-specific commands work
  Expected Result: Works correctly on Unix-like systems
  Evidence: Test output
```

---

## Execution Strategy

### Parallel Execution Waves

```
Wave 1 (Start Immediately):
└── Task 1: Review current implementation and add missing dependencies

Wave 2 (After Task 1):
├── Task 2: Implement helper functions
└── Task 3: Update main function

Wave 3 (After Tasks 2, 3):
└── Task 4: Add tests and verify

Critical Path: Task 1 → Task 2 → Task 3 → Task 4 (sequential)
```

### Dependency Matrix

| Task | Depends On | Blocks | Can Parallelize With |
|------|------------|--------|---------------------|
| 1 | None | 2, 3 | None (foundation) |
| 2 | 1 | 3 | None |
| 3 | 2 | 4 | None |
| 4 | 3 | None | None (verification) |

### Agent Dispatch Summary

| Wave | Tasks | Recommended Agents |
|------|-------|-------------------|
| 1 | 1 | `task(category="quick", load_skills=[], ...)` |
| 2 | 2 | `task(category="unspecified-high", load_skills=[], ...)` |
| 3 | 3 | `task(category="unspecified-high", load_skills=[], ...)` |
| 4 | 4 | `task(category="quick", load_skills=[], ...)` |

---

## TODOs

> Implementation + Test = ONE Task. Never separate.
> EVERY task MUST have: Recommended Agent Profile + Parallelization info.

- [ ] 1. Review current implementation and prepare for changes

  **What to do**:
  - Read current `call_get_system_environment` implementation
  - Verify existing dependencies in `Cargo.toml`
  - Identify what needs to be added (tokio::process::Command is already available)
  - Plan helper function structure
  - Review existing test patterns in `mcp_tool_test.rs`

  **Must NOT do**:
  - Don't modify any code yet
  - Don't add new dependencies unless necessary (tokio already provides process::Command)
  - Don't change the function signature

  **Recommended Agent Profile**:
  > **Category**: `unspecified-low`
    - Reason: Simple code review and preparation task
  > **Skills**: None needed for this task
  > **Skills Evaluated but Omitted**: None

  **Parallelization**:
  - **Can Run In Parallel**: NO
  - **Parallel Group**: Sequential
  - **Blocks**: Task 2
  - **Blocked By**: None (can start immediately)

  **References** (CRITICAL):

  **Pattern References** (existing code to follow):
  - `crates/memflow-mcp/src/main.rs:1133-1159` - Current function implementation
  - `crates/memflow-mcp/Cargo.toml:6-35` - Existing dependencies (confirm tokio features)
  - `crates/memflow-mcp/tests/mcp_tool_test.rs:124-151` - Test schema pattern for get_system_environment

  **API/Type References** (contracts to implement against):
  - MCP schema at `main.rs:325-346` - Tool definition with three boolean parameters
  - Function signature: `async fn call_get_system_environment(include_dev_tools: bool, include_processes: bool, include_ports: bool) -> Result<String>`

  **Test References** (testing patterns to follow):
  - `crates/memflow-mcp/tests/mcp_tool_test.rs:1-165` - Test structure and assertion patterns
  - Look for how other async functions are tested

  **Documentation References** (specs and requirements):
  - Project README (root) - Tech stack section mentions Rust, Tauri, tokio
  - This plan - Requirements section for detailed behavior specs

  **External References** (libraries and frameworks):
  - Tokio process documentation: https://docs.rs/tokio/latest/tokio/process/ - Command usage
  - sysinfo 0.30 docs: https://docs.rs/sysinfo/latest/sysinfo/ - Process and system info APIs

  **WHY Each Reference Matters**:
  - `main.rs:1133-1159`: Understanding current stub implementation to extend
  - `Cargo.toml`: Confirm no new dependencies needed (tokio::process::Command available)
  - `mcp_tool_test.rs`: Follow existing test patterns for consistency
  - Tokio docs: Learn `Command::new()` with timeout pattern
  - sysinfo docs: Understand process filtering and system info APIs

  **Acceptance Criteria**:

  > **AGENT-EXECUTABLE VERIFICATION ONLY**

  - [ ] Review completed: Current implementation notes recorded
  - [ ] Dependencies confirmed: tokio already has process::Command available
  - [ ] Test patterns understood: Know how to structure new tests
  - [ ] Helper functions planned: Structure for tool/process/port helpers documented

  **Commit**: NO (continue to Task 2)

---

- [ ] 2. Implement helper functions

  **What to do**:
  - Create three helper functions:
    1. `detect_tool_version(tool_name: &str) -> Result<String>` (async with 2s timeout)
       - Commands: `node -v`, `python --version` (fallback `python3 --version`), `rustc --version`, `docker --version`
       - Use `tokio::process::Command` with `timeout()`
       - Capture stdout first line only
       - Return error message on failure (e.g., "node: not installed")
    2. `filter_dev_processes(sys: &sysinfo::System) -> Vec<ProcessInfo>`
       - Whitelist: code, node, python, rustc, cargo, docker, java, go (case-insensitive)
       - Normalize names: lowercase, strip `.exe` extension
       - Return at most 50 processes
       - Include: name, PID, CPU %, memory usage
    3. `check_port_availability(ports: &[u16]) -> Vec<PortStatus>`
       - Try binding to each port via `std::net::TcpListener::bind()`
       - Return "in use" or "free" status
       - If in use, try to get process name via sysinfo (best effort)
  - Add these helpers before `call_get_system_environment` function
  - Ensure all helpers are cross-platform (handle Windows paths, extensions)

  **Must NOT do**:
  - Don't modify the main `call_get_system_environment` function yet (that's Task 3)
  - Don't add arbitrary command execution - whitelist only
  - Don't forget timeouts for tool version checks
  - Don't use shell pipelines - use direct Command

  **Recommended Agent Profile**:
  > **Category**: `unspecified-high`
    - Reason: Multi-function implementation requiring async, error handling, cross-platform considerations
  > **Skills**: None (pure Rust implementation)
  > **Skills Evaluated but Omitted**:
    - All skills are frontend/UX/doc related; this is pure backend Rust logic

  **Parallelization**:
  - **Can Run In Parallel**: NO
  - **Parallel Group**: Sequential
  - **Blocks**: Task 3
  - **Blocked By**: Task 1

  **References** (CRITICAL):

  **Pattern References** (existing code to follow):
  - `crates/memflow-mcp/src/main.rs:1139-1156` - Current sysinfo usage pattern
  - `crates/memflow-mcp/src/main.rs:1-20` - Import patterns (use statements)
  - Any existing `use tokio::process::Command` patterns (if found in codebase)

  **API/Type References** (contracts to implement against):
  - `tokio::process::Command` - Spawn commands and capture output
  - `tokio::time::timeout` - Add 2s timeout to command execution
  - `sysinfo::System::processes()` - Get process list
  - `std::net::TcpListener::bind()` - Check port availability

  **Test References** (testing patterns to follow):
  - `crates/memflow-mcp/tests/mcp_tool_test.rs` - Test structure
  - Plan to write tests in Task 4

  **Documentation References** (specs and requirements):
  - This plan "Work Objectives" section - Detailed helper specs
  - This plan "Must NOT Have" section - Constraints to respect

  **External References** (libraries and frameworks):
  - Tokio process: https://docs.rs/tokio/latest/tokio/process/struct.Command.html
  - Tokio timeout: https://docs.rs/tokio/latest/tokio/time/fn.timeout.html
  - sysinfo process: https://docs.rs/sysinfo/latest/sysinfo/struct.System.html#method.processes
  - TcpListener: https://doc.rust-lang.org/std/net/struct.TcpListener.html

  **WHY Each Reference Matters**:
  - Current sysinfo usage: Maintain consistent patterns
  - Tokio docs: Correct async + timeout implementation
  - sysinfo docs: Process filtering API details
  - TcpListener docs: Port checking approach

  **Acceptance Criteria**:

  > **AGENT-EXECUTABLE VERIFICATION ONLY**

  - [ ] Helper functions added before main function
  - [ ] `cargo build --package memflow-mcp` succeeds
  - [ ] `detect_tool_version` implemented with timeout and error handling
  - [ ] `filter_dev_processes` implements case-insensitive filtering with .exe stripping
  - [ ] `check_port_availability` attempts bind and returns status
  - [ ] No compiler warnings or errors

  **Agent-Executed QA Scenarios**:

  ```
  Scenario: Verify helpers compile successfully
    Tool: Bash (cargo build)
    Preconditions: Rust toolchain installed
    Steps:
      1. cd D:\Demo\memflow
      2. cargo build --package memflow-mcp 2>&1 | tee .sisyphus/evidence/task-2-build.log
      3. Assert exit code is 0
      4. Assert no "error[E" in build output
      5. Assert helpers are present in compiled binary (nm or objdump if available)
    Expected Result: Clean build with helpers compiled
    Evidence: .sisyphus/evidence/task-2-build.log

  Scenario: Verify detect_tool_version timeout behavior
    Tool: Bash (cargo test + manual inspection)
    Preconditions: Code built
    Steps:
      1. Write quick integration test invoking detect_tool_version with mock hanging command
      2. Run test with RUST_LOG=info
      3. Assert test completes within 3s (2s timeout + overhead)
      4. Assert timeout error returned
    Expected Result: Timeout enforced, function returns quickly
    Evidence: Test output with timing

  Scenario: Verify process filtering case-insensitivity
    Tool: Bash (cargo test)
    Preconditions: At least one dev process running
    Steps:
      1. Call filter_dev_processes with system containing "Code.exe" or "node" or "Node"
      2. Assert process list includes the process
      3. Assert name is normalized to lowercase
      4. Assert .exe extension stripped
    Expected Result: Process correctly filtered and normalized
    Evidence: Test output

  Scenario: Verify port checking logic
    Tool: Bash (cargo test + netcat)
    Preconditions: Network available
    Steps:
      1. Start listener on port 9999: python -m http.server 9999 &
      2. Call check_port_availability with [9999, 10000]
      3. Assert 9999 returns "in use"
      4. Assert 10000 returns "free"
      5. Kill listener
    Expected Result: Correct port statuses
    Evidence: Test output
  ```

  **Evidence to Capture**:
  - [ ] Build log: .sisyphus/evidence/task-2-build.log
  - [ ] Test output for each scenario
  - [ ] Code screenshot of helper functions

  **Commit**: NO (continue to Task 3 for unified changes)

---

- [ ] 3. Update call_get_system_environment function

  **What to do**:
  - Remove underscore prefixes from parameters: `_include_dev_tools` → `include_dev_tools`, etc.
  - Add conditional sections based on parameters:
    - If `include_dev_tools == true`:
      - Call `detect_tool_version` for each tool (node, python, rustc, cargo, docker)
      - Format as "[Dev Tools]" section with one line per tool
      - Include error messages for missing tools
    - If `include_processes == true`:
      - Call `filter_dev_processes` with current system
      - Format as "[Active Dev Processes]" section
      - Show name, PID, CPU %, memory for each process (max 50)
      - If no dev processes, show "None" message
    - If `include_ports == true`:
      - Define default port list: [3000, 3001, 8000, 8080, 5000, 4200, 5173, 4000, 9000]
      - Call `check_port_availability`
      - Format as "[Port Usage]" section
      - Show port number and status (in use / free)
      - If in use, show process name if available
  - Maintain existing basic system info section (unchanged)
  - Ensure stable ordering: Basic info → Dev tools → Processes → Ports
  - Use consistent formatting (section headers, indentation)
  - All sections are fail-soft (errors in one section don't affect others)

  **Must NOT do**:
  - Don't change the function signature or return type
  - Don't modify the basic system info section
  - Don't add JSON output (keep human-readable text)
  - Don't fail fast - continue on errors

  **Recommended Agent Profile**:
  > **Category**: `unspecified-high`
    - Reason: Main function update requiring careful integration of all helpers
  > **Skills**: None (pure Rust implementation)
  > **Skills Evaluated but Omitted**: All skills are frontend/UX/doc related

  **Parallelization**:
  - **Can Run In Parallel**: NO
  - **Parallel Group**: Sequential
  - **Blocks**: Task 4
  - **Blocked By**: Task 2

  **References** (CRITICAL):

  **Pattern References** (existing code to follow):
  - `crates/memflow-mcp/src/main.rs:1133-1159` - Current function to modify
  - `crates/memflow-mcp/src/main.rs:1146-1156` - Existing string formatting pattern (push_str)

  **API/Type References** (contracts to implement against):
  - Helper functions from Task 2: `detect_tool_version`, `filter_dev_processes`, `check_port_availability`
  - MCP tool definition at `main.rs:325-346` - Parameter behavior contract

  **Test References** (testing patterns to follow):
  - Existing test in `mcp_tool_test.rs:125-151` - Schema test to update if needed

  **Documentation References** (specs and requirements):
  - This plan "Work Objectives" section - Output format requirements
  - Metis review questions #2, #4, #5 - Output format details

  **External References** (libraries and frameworks):
  - None needed (all external docs covered in Task 2)

  **WHY Each Reference Matters**:
  - Current function: Know what to modify
  - Existing formatting pattern: Maintain consistent style
  - MCP schema: Honor parameter contract

  **Acceptance Criteria**:

  > **AGENT-EXECUTABLE VERIFICATION ONLY**

  - [ ] Function updated with all three parameters used (no underscore prefixes)
  - [ ] `cargo build --package memflow-mcp` succeeds
  - [ ] Output includes "[Dev Tools]" section when include_dev_tools=true
  - [ ] Output includes "[Active Dev Processes]" section when include_processes=true
  - [ ] Output includes "[Port Usage]" section when include_ports=true
  - [ ] All sections are optional (excluded when param=false)
  - [ ] Stable ordering maintained (Basic → Tools → Processes → Ports)
  - [ ] Error messages included for missing tools
  - [ ] Function returns Result<String> successfully even with errors

  **Agent-Executed QA Scenarios**:

  ```
  Scenario: Verify all parameters enabled
    Tool: Bash (cargo test + RUST_LOG)
    Preconditions: Code built, dev tools installed
    Steps:
      1. Write integration test calling function with (true, true, true)
      2. Run test: cargo test --package memflow-mcp verify_all_params -- --nocapture
      3. Assert output contains "[System Environment]"
      4. Assert output contains "[Dev Tools]"
      5. Assert output contains at least one tool version (node/python/rustc/cargo/docker)
      6. Assert output contains "[Active Dev Processes]"
      7. Assert output contains process list or "None"
      8. Assert output contains "[Port Usage]"
      9. Assert output contains port statuses for all default ports
      10. Capture output to .sisyphus/evidence/task-3-all-params.txt
    Expected Result: All sections present, correctly formatted
    Evidence: .sisyphus/evidence/task-3-all-params.txt

  Scenario: Verify parameter exclusion
    Tool: Bash (cargo test)
    Preconditions: Code built
    Steps:
      1. Call function with (false, false, false)
      2. Assert output contains "[System Environment]" (basic info)
      3. Assert output does NOT contain "[Dev Tools]"
      4. Assert output does NOT contain "[Active Dev Processes]"
      5. Assert output does NOT contain "[Port Usage]"
      6. Assert basic info still present (OS, memory, CPU, etc.)
    Expected Result: Only basic system info, no extra sections
    Evidence: Test output

  Scenario: Verify graceful error handling
    Tool: Bash (cargo test)
    Preconditions: System missing at least one tool (e.g., Docker)
    Steps:
      1. Call function with include_dev_tools=true
      2. Assert output includes "docker: not installed" or similar error
      3. Assert other tools still detected successfully
      4. Assert function returns Ok(String) not Err
      5. Assert processes and ports sections still work
    Expected Result: Missing tool reported, function continues
    Evidence: Test output with error message visible

  Scenario: Verify output stability
    Tool: Bash (cargo test)
    Preconditions: Same system state
    Steps:
      1. Call function 5 times with same params
      2. Compare outputs - assert identical ordering
      3. Assert section headers in same order each time
      4. Assert tool/process/port lists in stable order (e.g., sorted)
    Expected Result: Deterministic output
    Evidence: Test output showing identical results

  Scenario: Verify process filtering
    Tool: Bash (cargo test)
    Preconditions: Dev processes running
    Steps:
      1. Start test process: node -e "setInterval(()=>{},1000)" &
      2. Call function with include_processes=true
      3. Assert process list includes "node"
      4. Assert process list does NOT include explorer.exe, notepad.exe
      5. Assert each process has PID, CPU %, memory info
      6. Kill test process
    Expected Result: Only dev processes listed
    Evidence: Process list captured

  Scenario: Verify port checking
    Tool: Bash (cargo test + python)
    Preconditions: Network available
    Steps:
      1. Start server: python -m http.server 3000 &
      2. Call function with include_ports=true
      3. Assert output shows "3000: in use"
      4. Assert output shows process name (python/http.server)
      5. Assert other ports show "free"
      6. Kill server
    Expected Result: Correct port statuses
    Evidence: Port section captured
  ```

  **Evidence to Capture**:
  - [ ] Full output with all params: .sisyphus/evidence/task-3-all-params.txt
  - [ ] Output with all params false: .sisyphus/evidence/task-3-no-params.txt
  - [ ] Error handling output: .sisyphus/evidence/task-3-errors.txt
  - [ ] Process filtering output
  - [ ] Port checking output

  **Commit**: NO (wait for Task 4 tests)

---

- [ ] 4. Add tests and verify functionality

  **What to do**:
  - Create new test file: `crates/memflow-mcp/tests/system_environment_test.rs`
  - Add tests covering:
    1. `test_all_parameters_enabled`: Verify all sections present
    2. `test_all_parameters_disabled`: Verify only basic info
    3. `test_tool_version_detection`: Test tool version parsing
    4. `test_process_filtering`: Test whitelist and normalization
    5. `test_process_name_normalization`: Test case-insensitive, .exe stripping
    6. `test_port_checking`: Test port availability logic
    7. `test_error_handling`: Test missing tools don't crash
    8. `test_output_stability`: Test deterministic ordering
  - Run `cargo test --package memflow-mcp` and ensure all pass
  - Manual verification: Run the function and inspect output format
  - Fix any test failures or edge cases discovered

  **Must NOT do**:
  - Don't modify the implementation unless tests reveal bugs
  - Don't skip error handling tests
  - Don't forget cross-platform considerations (Windows vs Unix)

  **Recommended Agent Profile**:
  > **Category**: `quick`
    - Reason: Test writing and verification task
  > **Skills**: None (pure Rust testing)
  > **Skills Evaluated but Omitted**: All skills unrelated

  **Parallelization**:
  - **Can Run In Parallel**: NO
  - **Parallel Group**: Sequential (final verification)
  - **Blocks**: None (completion)
  - **Blocked By**: Task 3

  **References** (CRITICAL):

  **Pattern References** (existing code to follow):
  - `crates/memflow-mcp/tests/mcp_tool_test.rs` - Existing test patterns
  - `crates/memflow-mcp/tests/mcp_tool_test.rs:124-151` - System environment schema test

  **API/Type References** (contracts to implement against):
  - `call_get_system_environment` function signature
  - Helper functions from Task 2

  **Test References** (testing patterns to follow):
  - Same file - follow existing patterns

  **Documentation References** (specs and requirements):
  - This plan "Verification Strategy" section - Test scenarios to implement
  - This plan "Agent-Executed QA Scenarios" - Specific test cases

  **External References** (libraries and frameworks):
  - Rust testing docs: https://doc.rust-lang.org/book/ch11-00-testing.html
  - tokio::test: https://docs.rs/tokio/latest/tokio/attr.test.html

  **WHY Each Reference Matters**:
  - Existing tests: Maintain consistency with project test style
  - Function signature: Test against correct API
  - This plan: Implement planned test scenarios

  **Acceptance Criteria**:

  > **AGENT-EXECUTABLE VERIFICATION ONLY**

  - [ ] Test file created: `crates/memflow-mcp/tests/system_environment_test.rs`
  - [ ] All 8 tests implemented
  - [ ] `cargo test --package memflow-mcp` passes all tests
  - [ ] Manual verification output inspected and looks correct
  - [ ] No compiler warnings
  - [ ] Cross-platform behavior verified (at least one Windows and one Unix test)

  **Agent-Executed QA Scenarios**:

  ```
  Scenario: Verify all tests pass
    Tool: Bash (cargo test)
    Preconditions: Code built
    Steps:
      1. cd D:\Demo\memflow
      2. cargo test --package memflow-mcp system_environment 2>&1 | tee .sisyphus/evidence/task-4-test-results.log
      3. Assert exit code is 0
      4. Assert "test result: ok" in output
      5. Assert all 8 tests passed
      6. Assert no ignored tests
    Expected Result: All tests pass
    Evidence: .sisyphus/evidence/task-4-test-results.log

  Scenario: Verify test coverage
    Tool: Bash (cargo test + tarpaulin if available)
    Preconditions: Tests pass
    Steps:
      1. cargo test --package memflow-mcp
      2. Review test output for coverage
      3. Assert all helper functions tested
      4. Assert main function tested with various parameter combinations
      5. Assert error paths tested
    Expected Result: Good coverage of new code
    Evidence: Test coverage report

  Scenario: Manual verification with real tools
    Tool: Bash (cargo run + manual inspection)
    Preconditions: Dev tools installed
    Steps:
      1. Run manual integration test calling function with all params true
      2. Inspect output visually
      3. Verify sections are clearly separated
      4. Verify tool versions are readable
      5. Verify process list is clean (only dev processes)
      6. Verify port statuses are clear
      7. Capture output to .sisyphus/evidence/task-4-manual-output.txt
    Expected Result: Human-readable, well-formatted output
    Evidence: .sisyphus/evidence/task-4-manual-output.txt

  Scenario: Verify no regressions
    Tool: Bash (cargo test)
    Preconditions: All tests exist
    Steps:
      1. cargo test --package memflow-mcp
      2. Assert existing tests still pass (mcp_tool_test.rs)
      3. Assert no new warnings introduced
      4. Assert no existing functionality broken
    Expected Result: No regressions
    Evidence: Full test output
  ```

  **Evidence to Capture**:
  - [ ] Test results: .sisyphus/evidence/task-4-test-results.log
  - [ ] Manual verification output: .sisyphus/evidence/task-4-manual-output.txt
  - [ ] Screenshot of well-formatted output (if applicable)

  **Commit**: YES (Final commit for all changes)
  - Message: `feat(mcp): complete get_system_environment implementation`
  - Files:
    - `crates/memflow-mcp/src/main.rs` (helpers + updated function)
    - `crates/memflow-mcp/tests/system_environment_test.rs` (new file)
  - Pre-commit: `cargo test --package memflow-mcp`

---

## Commit Strategy

| After Task | Message | Files | Verification |
|------------|---------|-------|--------------|
| 4 | `feat(mcp): complete get_system_environment implementation` | `crates/memflow-mcp/src/main.rs`<br>`crates/memflow-mcp/tests/system_environment_test.rs` | `cargo test --package memflow-mcp` |

---

## Success Criteria

### Verification Commands
```bash
# Build check
cargo build --package memflow-mcp

# Run all tests
cargo test --package memflow-mcp

# Run specific tests
cargo test --package memflow-mcp system_environment

# Manual verification
cargo run --package memflow-mcp -- --tool get_system_environment '{"include_dev_tools":true,"include_processes":true,"include_ports":true}'
```

### Final Checklist
- [ ] All three parameters implemented and functional
- [ ] Helper functions for tools, processes, ports working
- [ ] Error handling graceful (fail-soft)
- [ ] Cross-platform compatibility verified
- [ ] All tests pass (cargo test)
- [ ] No regressions in existing tests
- [ ] Output is human-readable and well-formatted
- [ ] Timeouts enforced for external commands
- [ ] Process filtering works (case-insensitive, .exe stripped)
- [ ] Port checking reports correct status
