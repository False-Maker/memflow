# Fix ONNX Runtime Version Conflict (Problem 1)

## TL;DR

> **Quick Summary**: Fix ONNX Runtime version mismatch (1.17.1 vs required >= 1.23.x) by downloading onnxruntime.dll 1.24.1 to project directory, enabling the embedding model to load successfully.
>
> **Deliverables**:
> - Downloaded onnxruntime.dll v1.24.1 placed in project directory
> - Verified memflow-mcp loads the new DLL successfully
> - Semantic search functionality confirmed working
>
> **Estimated Effort**: Quick
> **Parallel Execution**: NO - sequential download and verification
> **Critical Path**: Download DLL → Verify loading → Test semantic search

---

## Context

### Original Request
From `doc/MCP_BLOCKING_ISSUES.md` Problem 1:
> ort 2.0.0-rc.11 is not compatible with the ONNX Runtime binary found at `onnxruntime.dll`;
> expected version >= '1.23.x', but got '1.17.1'

### Root Cause Analysis

**Error Signature**:
```
Failed to load ONNX Runtime dylib: ort 2.0.0-rc.11 is not compatible with
the ONNX Runtime binary found at `onnxruntime.dll`;
expected version >= '1.23.x', but got '1.17.1'
```

**Current State**:
- System DLL at `C:\Windows\System32\onnxruntime.dll` is version 1.17.1
- Project dependency in `crates/memflow-mcp/Cargo.toml` uses ort 2.0.0-rc.11
- ort 2.0.0-rc.11 requires ONNX Runtime >= 1.23.x
- Embedding model initialization fails, semantic search degraded to placeholder

**Solution Strategy**:
- Download ONNX Runtime 1.24.1 (latest stable) from official GitHub releases
- Place onnxruntime.dll in `crates/memflow-mcp/` directory (next to where the binary runs)
- ort's `load-dynamic` feature will search the executable directory first

### Interview Summary

**User's Decisions**:
- **Strategy**: Update DLL (not downgrade ort crate)
- **Placement**: Project directory (safer than System32)
- **Scope**: Fix only Problem 1, don't touch Problems 2/3/4

**Metis Review Findings** (incorporated):
- Must verify runtime loaded DLL path and version, not just file presence
- Must place DLL in deterministic location: `crates/memflow-mcp/` (next to executable)
- Must validate with command-based checks (file version, runtime logs, functional test)
- Must NOT modify global system state (System32, PATH, registry)

---

## Work Objectives

### Core Objective
Resolve ONNX Runtime version conflict by providing a compatible onnxruntime.dll in the project directory, enabling the embedding model to load successfully.

### Concrete Deliverables
- Downloaded `onnxruntime.dll` v1.24.1 in `crates/memflow-mcp/` directory
- Verification that the DLL is loaded at runtime (not the System32 version)
- Confirmation that semantic search works with real embeddings

### Definition of Done
- [ ] `onnxruntime.dll` v1.24.1 exists in `crates/memflow-mcp/`
- [ ] File version verified as 1.24.1.x
- [ ] memflow-mcp starts without "not compatible" error
- [ ] Logs confirm successful ONNX Runtime loading
- [ ] Semantic search returns real embeddings (not placeholder)

### Must Have
- Download from official GitHub releases (microsoft/onnxruntime)
- Use x64 Windows package
- Place in `crates/memflow-mcp/` directory
- Verify DLL architecture and version
- Test semantic search functionality

### Must NOT Have (Guardrails)
- **DO NOT** modify C:\Windows\System32\onnxruntime.dll
- **DO NOT** modify Cargo.toml dependencies
- **DO NOT** change ort crate version
- **DO NOT** modify embedding model configuration
- **DO NOT** touch Problem 2, 3, or 4 from MCP_BLOCKING_ISSUES.md
- **DO NOT** modify system PATH or registry

---

## Verification Strategy (MANDATORY)

> **UNIVERSAL RULE: ZERO HUMAN INTERVENTION**
>
> ALL tasks in this plan MUST be verifiable WITHOUT any human action.
> This is NOT conditional — it applies to EVERY task.

### Test Decision
- **Infrastructure exists**: NO (binary executable test)
- **Automated tests**: None
- **Framework**: N/A

### Agent-Executed QA Scenarios (MANDATORY)

All verification will be done via PowerShell commands, file inspection, and memflow-mcp execution.

---

## Execution Strategy

### Parallel Execution Waves
Single sequential task - no parallelization needed.

### Dependency Matrix

| Task | Depends On | Blocks | Can Parallelize With |
|------|------------|--------|---------------------|
| 1 | None | 2 | None |
| 2 | 1 | 3 | None |
| 3 | 2 | None | None |

---

## TODOs

- [ ] 1. Download ONNX Runtime 1.24.1

  **What to do**:
  1. Create temp directory for download: `D:\Demo\memflow\tmp\onnx`
  2. Download ONNX Runtime 1.24.1 Windows x64 package from GitHub releases
  3. URL: `https://github.com/microsoft/onnxruntime/releases/download/v1.24.1/onnxruntime-win-x64-1.24.1.zip`
  4. Extract the ZIP file
  5. Locate `onnxruntime.dll` in the extracted files
  6. Copy `onnxruntime.dll` to `crates/memflow-mcp/` directory

  **Must NOT do**:
  - DO NOT download from unofficial sources
  - DO NOT modify System32 or any system directories
  - DO NOT download x86 or ARM64 packages (x64 only)

  **Recommended Agent Profile**:
  > Select category + skills based on task domain.
  - **Category**: `quick`
    - Reason: Straightforward download and copy operation
  - **Skills**: None needed
  - **Skills Evaluated but Omitted**:
    - All other skills: Not needed for file download/copy

  **Parallelization**:
  - **Can Run In Parallel**: NO
  - **Parallel Group**: Sequential
  - **Blocks**: Task 2
  - **Blocked By**: None

  **References**:

  **Pattern References** (existing code to follow): None

  **API/Type References**: None

  **Test References**: None

  **Documentation References** (specs and requirements):
  - `doc/MCP_BLOCKING_ISSUES.md:8-48` - Problem 1 description

  **External References** (libraries and frameworks):
  - ONNX Runtime GitHub Releases: https://github.com/microsoft/onnxruntime/releases/tag/v1.24.1
  - Direct download: https://github.com/microsoft/onnxruntime/releases/download/v1.24.1/onnxruntime-win-x64-1.24.1.zip

  **WHY Each Reference Matters**:
  - Official GitHub releases page is the authoritative source for ONNX Runtime downloads
  - Direct URL ensures we get exactly version 1.24.1

  **Acceptance Criteria**:

  > **AGENT-EXECUTABLE VERIFICATION ONLY** — No human action permitted.

  - [ ] Directory created: D:\Demo\memflow\tmp\onnx (or temp dir)
  - [ ] File downloaded: onnxruntime-win-x64-1.24.1.zip
  - [ ] ZIP extracted successfully
  - [ ] onnxruntime.dll located in extracted files
  - [ ] onnxruntime.dll copied to crates/memflow-mcp/
  - [ ] Target file exists: D:\Demo\memflow\crates\memflow-mcp\onnxruntime.dll

  **Agent-Executed QA Scenarios (MANDATORY — per-scenario, ultra-detailed):**

  ```
  Scenario: Verify ONNX DLL downloaded and placed correctly
    Tool: Bash (PowerShell)
    Preconditions: Download task completed
    Steps:
      1. Test-Path "D:\Demo\memflow\crates\memflow-mcp\onnxruntime.dll"
      2. (Get-Item "D:\Demo\memflow\crates\memflow-mcp\onnxruntime.dll").Length -gt 0
      3. (Get-Item "D:\Demo\memflow\crates\memflow-mcp\onnxruntime.dll").VersionInfo.FileVersion
      4. Assert FileVersion starts with "1.24.1"
      5. Assert File is x64 architecture (check via file properties)
    Expected Result: File exists, size > 0, version is 1.24.1.x, architecture is x64
    Evidence: File properties captured
  ```

  **Commit**: NO (wait for verification)

---

- [ ] 2. Verify DLL loading and ONNX Runtime version

  **What to do**:
  1. Build memflow-mcp if not already built: `cargo build -p memflow-mcp`
  2. Run memflow-mcp and capture startup logs
  3. Check logs for ONNX Runtime loading messages
  4. Verify NO "not compatible" or "expected version >= 1.23.x" errors
  5. Verify ONNX Runtime initializes successfully

  **Must NOT do**:
  - DO NOT modify any source code
  - DO NOT proceed if errors are found - investigate

  **Recommended Agent Profile**:
  > Select category + skills based on task domain.
  - **Category**: `quick`
    - Reason: Verification task with clear commands
  - **Skills**: None needed
  - **Skills Evaluated but Omitted**:
    - All other skills: Not needed for running cargo and checking logs

  **Parallelization**:
  - **Can Run In Parallel**: NO
  - **Parallel Group**: Sequential (after Task 1)
  - **Blocks**: Task 3
  - **Blocked By**: Task 1

  **References**:

  **Pattern References**: None

  **API/Type References**: None

  **Test References**: None

  **Documentation References** (specs and requirements):
  - `doc/MCP_BLOCKING_ISSUES.md:8-48` - Problem 1 description

  **External References**:
  - ort crate documentation: https://docs.rs/ort/

  **WHY Each Reference Matters**: N/A

  **Acceptance Criteria**:

  > **AGENT-EXECUTABLE VERIFICATION ONLY** — No human action permitted.

  - [ ] Command: `cargo run -p memflow-mcp -- --help` (or similar basic invocation)
  - [ ] Output does NOT contain: "not compatible"
  - [ ] Output does NOT contain: "expected version >= 1.23.x"
  - [ ] Output contains: successful initialization message (or no ONNX errors)
  - [ ] Process exits cleanly (or can be terminated)

  **Agent-Executed QA Scenarios (MANDATORY — per-scenario, ultra-detailed):**

  ```
  Scenario: Verify memflow-mcp loads ONNX DLL without errors
    Tool: Bash (PowerShell)
    Preconditions: Task 1 completed, DLL in place
    Steps:
      1. cargo build -p memflow-mcp 2>&1 | Tee-Object -FilePath tmp\build.log
      2. Assert build exits with code 0
      3. $process = Start-Process -FilePath "cargo" -ArgumentList "run -p memflow-mcp -- --help" -RedirectStandardOutput tmp\mcp_out.log -RedirectStandardError tmp\mcp_err.log -PassThru -NoNewWindow
      4. Start-Sleep -Seconds 5
      5. Stop-Process -Id $process.Id -Force -ErrorAction SilentlyContinue
      6. Get-Content tmp\mcp_err.log | Select-String -Pattern "not compatible|expected version"
      7. Assert NO matches for version mismatch errors
      8. Get-Content tmp\mcp_out.log | Select-String -Pattern "ONNX|initialized|embedding"
    Expected Result: No version mismatch errors, ONNX Runtime initializes successfully
    Evidence: Log files captured
  ```

  **Commit**: NO (wait for functional test)

---

- [ ] 3. Test semantic search functionality

  **What to do**:
  1. Run memflow-mcp server (if it's a server) or execute a search command
  2. Send a test search_memory request
  3. Verify semantic search returns real results (not placeholder)
  4. Verify embeddings are generated successfully
  5. Document the test results

  **Must NOT do**:
  - DO NOT modify search configuration or parameters
  - DO NOT skip this test - functional verification is critical

  **Recommended Agent Profile**:
  > Select category + skills based on task domain.
  - **Category**: `quick`
    - Reason: Functional test with clear success criteria
  - **Skills**: None needed
  - **Skills Evaluated but Omitted**:
    - All other skills: Not needed for basic functional test

  **Parallelization**:
  - **Can Run In Parallel**: NO
  - **Parallel Group**: Sequential (after Task 2)
  - **Blocks**: None (final verification)
  - **Blocked By**: Task 1, Task 2

  **References**:

  **Pattern References**:
  - `crates/memflow-mcp/src/main.rs` - Entry point for running memflow-mcp
  - `doc/E2E_VALIDATION_REPORT.md` - Previous test patterns

  **API/Type References**: None

  **Test References**: None

  **Documentation References** (specs and requirements):
  - `doc/MCP_BLOCKING_ISSUES.md:8-48` - Problem 1 description
  - `doc/E2E_VALIDATION_REPORT.md` - Test patterns for memflow-mcp

  **External References**: None

  **WHY Each Reference Matters**:
  - E2E_VALIDATION_REPORT.md contains previous test commands that worked

  **Acceptance Criteria**:

  > **AGENT-EXECUTABLE VERIFICATION ONLY** — No human action permitted.

  - [ ] memflow-mcp starts successfully
  - [ ] Search request executes without errors
  - [ ] Search results include embeddings (not empty/placeholder)
  - [ ] No ONNX-related errors in logs

  **Agent-Executed QA Scenarios (MANDATORY — per-scenario, ultra-detailed):**

  ```
  Scenario: Verify semantic search works with real embeddings
    Tool: Bash (PowerShell + curl/cargo)
    Preconditions: Tasks 1 and 2 completed
    Steps:
      1. Start memflow-mcp server in background
      2. Wait 5 seconds for startup
      3. Send test search request (via MCP protocol or direct API)
      4. Check response contains non-empty results
      5. Check logs for "embedding" or "semantic" success indicators
      6. Terminate server
    Expected Result: Search returns results with real embeddings, no ONNX errors
    Evidence: Response output and log files captured
  ```

  **Commit**: YES
  - Message: `fix(onnx): add onnxruntime.dll 1.24.1 for ort compatibility`
  - Files: `crates/memflow-mcp/onnxruntime.dll`
  - Pre-commit: Verify tests pass

---

## Commit Strategy

| After Task | Message | Files | Verification |
|------------|---------|-------|--------------|
| 3 | `fix(onnx): add onnxruntime.dll 1.24.1 for ort compatibility` | crates/memflow-mcp/onnxruntime.dll | Manual check (binary file) |

---

## Success Criteria

### Verification Commands
```powershell
# 1. Check DLL file version
(Get-Item "D:\Demo\memflow\crates\memflow-mcp\onnxruntime.dll").VersionInfo.FileVersion
# Expected: 1.24.1.x

# 2. Run memflow-mcp and check for errors
cargo run -p memflow-mcp -- --help 2>&1 | Select-String -Pattern "not compatible"
# Expected: No matches

# 3. Test semantic search
# (actual command depends on memflow-mcp interface)
```

### Final Checklist
- [ ] `onnxruntime.dll` exists in `crates/memflow-mcp/`
- [ ] File version is 1.24.1.x (>= 1.23.x requirement met)
- [ ] memflow-mcp starts without ONNX version mismatch errors
- [ ] Semantic search returns real embeddings (not placeholder)
- [ ] System32 onnxruntime.dll remains unchanged (1.17.1)

### Exclusions (Explicitly Out of Scope)
- Modifying C:\Windows\System32\onnxruntime.dll
- Downgrading ort crate in Cargo.toml
- Modifying fastembed dependency
- Changing embedding model configuration
- Problem 2 (macOS terminal capture)
- Problem 3 (workspace build - already fixed)
- Problem 4 (Cursor/Claude Desktop integration)

---

## Appendix: DLL Resolution Order

The Windows DLL search order (when `load-dynamic` is used):
1. Application directory (where memflow-mcp.exe runs from)
2. Current working directory
3. System directory (C:\Windows\System32)
4. Windows directory (C:\Windows)

By placing `onnxruntime.dll` in `crates/memflow-mcp/`, the ort crate will find our v1.24.1 DLL before the System32 v1.17.1 version.
