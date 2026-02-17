# Fix Workspace Build Failure (Problem 3)

## TL;DR

> **Quick Summary**: Fix `cargo build --workspace` failure caused by Tauri config referencing non-existent external binary paths for macOS/Linux targets that don't exist in this Windows development environment.
>
> **Deliverables**:
> - Updated `src-tauri/tauri.conf.json` with only valid externalBin entries
> - Fixed unused import warning in `crates/memflow-core/src/ocr_enhance.rs`
>
> **Estimated Effort**: Quick
> **Parallel Execution**: NO - single-file changes
> **Critical Path**: Fix tauri.conf.json → Verify build passes

---

## Context

### Original Request
From `doc/MCP_BLOCKING_ISSUES.md` Problem 3:
> `cargo build --workspace` fails (exit code 1), but `cargo check -p memflow-mcp` and `cargo check -p memflow-core` both pass. Error from Tauri app crate.

### Root Cause Analysis

**Error Signature**:
```
error: failed to run custom build command for `memflow v0.1.0 (D:\Demo\memflow\src-tauri)`
...
resource path `..\target\aarch64-apple-darwin\release\memflow-mcp-x86_64-pc-windows-msvc.exe` doesn't exist
```

**Problem Location**: `src-tauri/tauri.conf.json` lines 32-36
```json
"externalBin": [
  "../target/release/memflow-mcp",                           // Valid
  "../target/x86_64-pc-windows-msvc/release/memflow-mcp",    // Valid for Windows
  "../target/aarch64-apple-darwin/release/memflow-mcp",      // DOESN'T EXIST
  "../target/x86_64-unknown-linux-gnu/release/memflow-mcp"   // DOESN'T EXIST
]
```

**Additional Issue**: Minor warning in `crates/memflow-core/src/ocr_enhance.rs:9`
```
warning: unused import: `ImageBuffer`
```

### Interview Summary

**User's Decisions**:
- Create separate plans for each MCP blocking issue (this is Problem 3, first in sequence)
- Problem 2 (macOS terminal capture) will be skipped entirely
- Focus on minimal fix to unblock workspace build

**Metis Review Findings** (incorporated):
- Must capture pre-fix failure signature for verification
- Must define success as `cargo build --workspace` exits 0
- Must constrain edits to only these 2 files
- Must NOT expand into cross-platform packaging redesign

---

## Work Objectives

### Core Objective
Unblock the workspace build by removing references to non-existent external binaries in Tauri configuration.

### Concrete Deliverables
- Modified `src-tauri/tauri.conf.json` with only valid externalBin entries
- Cleaned `crates/memflow-core/src/ocr_enhance.rs` (unused import removed)

### Definition of Done
- [ ] `cargo build --workspace` completes with exit code 0
- [ ] `cargo build --workspace 2>&1` contains no "doesn't exist" errors
- [ ] `cargo check -p memflow-core` produces no warnings

### Must Have
- Remove non-existent externalBin paths from tauri.conf.json
- Fix unused import warning
- Verify build passes after changes

### Must NOT Have (Guardrails)
- **DO NOT** change workspace members in Cargo.toml
- **DO NOT** modify crate names or build system structure
- **DO NOT** expand into cross-platform packaging strategy redesign
- **DO NOT** touch Problem 1, 2, or 4 from MCP_BLOCKING_ISSUES.md
- **DO NOT** change CI configuration
- **DO NOT** add new externalBin entries for platforms that don't have binaries

---

## Verification Strategy (MANDATORY)

> **UNIVERSAL RULE: ZERO HUMAN INTERVENTION**
>
> ALL tasks in this plan MUST be verifiable WITHOUT any human action.
> This is NOT conditional — it applies to EVERY task.

### Test Decision
- **Infrastructure exists**: NO (Rust project, no test framework explicitly configured)
- **Automated tests**: None
- **Framework**: N/A

### Agent-Executed QA Scenarios (MANDATORY)

All verification will be done via cargo commands and file inspection.

**Baseline Failure Capture** (to be captured BEFORE fix):
```bash
cargo build --workspace 2>&1
```
Expected: Exit code 1, error contains "doesn't exist" for aarch64-apple-darwin path

---

## Execution Strategy

### Parallel Execution Waves
Single sequential task - no parallelization needed.

### Dependency Matrix

| Task | Depends On | Blocks | Can Parallelize With |
|------|------------|--------|---------------------|
| 1 | None | 2 | None |
| 2 | 1 | None | None |

---

## TODOs

- [ ] 1. Fix externalBin paths in tauri.conf.json

  **What to do**:
  1. Read `src-tauri/tauri.conf.json`
  2. Remove these non-existent externalBin entries:
     - `../target/aarch64-apple-darwin/release/memflow-mcp`
     - `../target/x86_64-unknown-linux-gnu/release/memflow-mcp`
  3. Keep these valid entries:
     - `../target/release/memflow-mcp` (cross-platform)
     - `../target/x86_64-pc-windows-msvc/release/memflow-mcp` (Windows-specific)
  4. Verify JSON syntax remains valid

  **Must NOT do**:
  - DO NOT add new externalBin entries
  - DO NOT modify other sections of tauri.conf.json
  - DO NOT change the bundle targets configuration

  **Recommended Agent Profile**:
  > Select category + skills based on task domain.
  - **Category**: `quick`
    - Reason: Simple config file edit, well-defined change
  - **Skills**: None needed for this straightforward edit
  - **Skills Evaluated but Omitted**:
    - All other skills: Not needed for single-line config edit

  **Parallelization**:
  - **Can Run In Parallel**: NO
  - **Parallel Group**: Sequential
  - **Blocks**: Task 2
  - **Blocked By**: None

  **References**:

  **Pattern References** (existing code to follow):
  - `src-tauri/tauri.conf.json:32-37` - Current externalBin configuration to modify

  **API/Type References**: None

  **Test References**: None

  **Documentation References** (specs and requirements):
  - `doc/MCP_BLOCKING_ISSUES.md:95-129` - Problem 3 description

  **External References** (libraries and frameworks):
  - Tauri 2.0 docs: https://tauri.app/v2/config/#bundleconfig.externalbin

  **WHY Each Reference Matters**:
  - `src-tauri/tauri.conf.json:32-37` - This is the exact location of the problematic configuration that needs fixing

  **Acceptance Criteria**:

  > **AGENT-EXECUTABLE VERIFICATION ONLY** — No human action permitted.

  - [ ] File edited: src-tauri/tauri.conf.json
  - [ ] Lines 32-37 now contain only 2 externalBin entries:
    - `../target/release/memflow-mcp`
    - `../target/x86_64-pc-windows-msvc/release/memflow-mcp`
  - [ ] Removed entries are confirmed absent:
    - `../target/aarch64-apple-darwin/release/memflow-mcp` (NOT in file)
    - `../target/x86_64-unknown-linux-gnu/release/memflow-mcp` (NOT in file)
  - [ ] JSON syntax is valid (file can be parsed)

  **Agent-Executed QA Scenarios (MANDATORY — per-scenario, ultra-detailed):**

  ```
  Scenario: Verify externalBin paths corrected
    Tool: Bash (grep/read)
    Preconditions: File has been edited
    Steps:
      1. Read src-tauri/tauri.conf.json
      2. Grep for "externalBin" section
      3. Count array entries - should be exactly 2
      4. Verify "../target/aarch64-apple-darwin/release/memflow-mcp" is NOT in file
      5. Verify "../target/x86_64-unknown-linux-gnu/release/memflow-mcp" is NOT in file
      6. Verify "../target/release/memflow-mcp" IS in file
      7. Verify "../target/x86_64-pc-windows-msvc/release/memflow-mcp" IS in file
    Expected Result: Exactly 2 externalBin entries, non-existent paths removed
    Evidence: File content captured
  ```

  **Commit**: NO (groups with Task 2)

---

- [ ] 2. Fix unused import warning in ocr_enhance.rs

  **What to do**:
  1. Read `crates/memflow-core/src/ocr_enhance.rs`
  2. Locate line 9: `use image::{GrayImage, ImageBuffer, Luma};`
  3. Remove `ImageBuffer` from the import list
  4. Result should be: `use image::{GrayImage, Luma};`

  **Must NOT do**:
  - DO NOT modify other parts of this file
  - DO NOT change the actual usage of GrayImage or Luma types
  - DO NOT add new imports

  **Recommended Agent Profile**:
  > Select category + skills based on task domain.
  - **Category**: `quick`
    - Reason: Simple unused import removal
  - **Skills**: None needed
  - **Skills Evaluated but Omitted**:
    - All other skills: Overkill for single import edit

  **Parallelization**:
  - **Can Run In Parallel**: NO
  - **Parallel Group**: Sequential (after Task 1)
  - **Blocks**: None
  - **Blocked By**: Task 1

  **References**:

  **Pattern References** (existing code to follow):
  - `crates/memflow-core/src/ocr_enhance.rs:9` - Line with unused import

  **API/Type References**: None

  **Test References**: None

  **Documentation References** (specs and requirements):
  - Compiler warning: `unused import: ImageBuffer at crates/memflow-core/src/ocr_enhance.rs:9`

  **External References** (libraries and frameworks):
  - Rust documentation on imports

  **WHY Each Reference Matters**:
  - `crates/memflow-core/src/ocr_enhance.rs:9` - Exact location of the warning to fix

  **Acceptance Criteria**:

  > **AGENT-EXECUTABLE VERIFICATION ONLY** — No human action permitted.

  - [ ] File edited: crates/memflow-core/src/ocr_enhance.rs
  - [ ] Line 9 no longer contains `ImageBuffer` in the import statement
  - [ ] `cargo check -p memflow-core` runs without warnings
  - [ ] File still contains `GrayImage` and `Luma` imports

  **Agent-Executed QA Scenarios (MANDATORY — per-scenario, ultra-detailed):**

  ```
  Scenario: Verify unused import removed
    Tool: Bash (cargo check + grep)
    Preconditions: File has been edited
    Steps:
      1. Run: cargo check -p memflow-core 2>&1
      2. Grep output for "warning: unused import: ImageBuffer"
      3. Verify warning is NOT present
      4. Grep crates/memflow-core/src/ocr_enhance.rs for "use image"
      5. Verify line contains "GrayImage" and "Luma" but NOT "ImageBuffer"
    Expected Result: No unused import warnings, ImageBuffer absent from import
    Evidence: cargo check output captured
  ```

  **Commit**: YES
  - Message: `fix(tauri): remove non-existent externalBin paths and unused import`
  - Files: `src-tauri/tauri.conf.json`, `crates/memflow-core/src/ocr_enhance.rs`
  - Pre-commit: `cargo check -p memflow-core`

---

- [ ] 3. Verify workspace build succeeds

  **What to do**:
  1. Run `cargo build --workspace` and capture full output
  2. Verify exit code is 0
  3. Verify no "doesn't exist" errors in output
  4. Run `cargo clippy --workspace` to check for warnings

  **Must NOT do**:
  - DO NOT modify any files in this task
  - DO NOT proceed if build fails - investigate and fix

  **Recommended Agent Profile**:
  > Select category + skills based on task domain.
  - **Category**: `quick`
    - Reason: Verification task with clear commands
  - **Skills**: None needed
  - **Skills Evaluated but Omitted**:
    - All other skills: Not needed for running cargo commands

  **Parallelization**:
  - **Can Run In Parallel**: NO
  - **Parallel Group**: Sequential (after Task 2)
  - **Blocks**: None (final verification)
  - **Blocked By**: Task 1, Task 2

  **References**:

  **Pattern References**: None

  **API/Type References**: None

  **Test References**: None

  **Documentation References** (specs and requirements):
  - `doc/MCP_BLOCKING_ISSUES.md:95-129` - Problem 3 requirements

  **External References**: None

  **WHY Each Reference Matters**: N/A

  **Acceptance Criteria**:

  > **AGENT-EXECUTABLE VERIFICATION ONLY** — No human action permitted.

  - [ ] Command: `cargo build --workspace`
  - [ ] Exit code: 0
  - [ ] Output does NOT contain: "doesn't exist"
  - [ ] Output does NOT contain: "failed to run custom build command"
  - [ ] Output contains: "Finished" or "Compiling memflow v0.1.0"
  - [ ] Command: `cargo clippy --workspace`
  - [ ] Clippy exit code: 0 (or only non-error warnings)

  **Agent-Executed QA Scenarios (MANDATORY — per-scenario, ultra-detailed):**

  ```
  Scenario: Verify workspace build succeeds
    Tool: Bash (cargo build)
    Preconditions: Tasks 1 and 2 completed
    Steps:
      1. Run: cargo build --workspace 2>&1
      2. Capture exit code
      3. Grep output for "doesn't exist"
      4. Grep output for "error:"
      5. Grep output for "Finished" or "Compiling memflow"
    Expected Result: Exit code 0, no "doesn't exist" errors, build completes
    Evidence: Build output captured

  Scenario: Verify no warnings remain
    Tool: Bash (cargo clippy)
    Preconditions: Build succeeded
    Steps:
      1. Run: cargo clippy --workspace 2>&1
      2. Grep output for "warning:"
      3. If warnings exist, check they are not blocking errors
      4. Verify exit code is 0 (clippy allows warnings by default)
    Expected Result: Clippy runs without blocking errors
    Evidence: Clippy output captured
  ```

  **Commit**: NO (verification only)

---

## Commit Strategy

| After Task | Message | Files | Verification |
|------------|---------|-------|--------------|
| 2 | `fix(tauri): remove non-existent externalBin paths and unused import` | src-tauri/tauri.conf.json, crates/memflow-core/src/ocr_enhance.rs | cargo check -p memflow-core |

---

## Success Criteria

### Verification Commands
```bash
# Primary verification
cargo build --workspace
# Expected: exit code 0, no "doesn't exist" errors

# Secondary verification
cargo clippy --workspace
# Expected: exit code 0 (or non-blocking warnings only)
```

### Final Checklist
- [ ] `cargo build --workspace` exits with code 0
- [ ] No "resource path doesn't exist" errors in output
- [ ] `src-tauri/tauri.conf.json` has only 2 externalBin entries
- [ ] `crates/memflow-core/src/ocr_enhance.rs` has no unused import warnings
- [ ] Non-existent paths (aarch64-apple-darwin, x86_64-unknown-linux-gnu) are removed

### Exclusions (Explicitly Out of Scope)
- CI/CD pipeline fixes
- Cross-platform packaging strategy redesign
- Problem 1 (ONNX Runtime version conflict)
- Problem 2 (macOS terminal capture)
- Problem 4 (Cursor/Claude Desktop integration verification)
