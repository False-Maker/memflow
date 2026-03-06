# Fix Recording Screenshots Directory Issue

## TL;DR

> **Quick Summary**: Fix two initialization bugs that prevent screenshot capture when recording starts: (1) DB initialization happens after recorder init, leaving screenshots_dir unset; (2) `clone_for_task()` resets screenshots_dir to None, losing state in spawned event loop.
>
> **Deliverables**:
> - Reordered startup initialization ensuring DB → recorder sequence
> - Fixed collector clone state propagation to preserve screenshots_dir
> - Added readiness gate for recording start
>
> **Estimated Effort**: Short (1-3 files, focused changes)
> **Parallel Execution**: NO - sequential changes required
> **Critical Path**: lib.rs init reorder → collector.rs clone fix → verification

---

## Context

### Original Request
User reports clicking the record button doesn't start screenshot capture. Application logs show:
```
WARN memflow_core::collection::collector: Event-driven capture failed: Screenshots directory not set
WARN memflow_core::collection::collector: Heartbeat capture failed: Screenshots directory not set
INFO memflow::commands: Recorder started successfully (local)
INFO memflow_core::collection::collector: Activity collector started
```

### Interview Summary
**Key Discussions**:
- User confirmed log output showing the error
- Analysis traced code flow from frontend (Layout.tsx) → commands → recorder → collector
- Found initialization timing issue in `lib.rs` setup function

**Research Findings**:
- `SCREENSHOTS_DIR` static variable in `memflow_core::db` only set during `init_db_with_path()`
- `get_screenshots_dir()` returns `Option<PathBuf>` - None if not initialized
- `recorder::init()` calls `db::get_screenshots_dir().await` during setup
- Metis identified **second bug**: `clone_for_task()` resets `screenshots_dir` to `None`

### Metis Review
**Identified Gaps** (addressed):
- **Bug 2 discovered**: `clone_for_task()` in `collector.rs:542` creates a new collector with `screenshots_dir: AsyncMutex::new(None)`, wiping the directory even if startup order is fixed
- **Validation needed**: Should start_recording fail fast if prerequisites missing?
- **Edge case**: User starts recording before DB init completes

**Risk Mitigation**:
- Fix both root causes (init order + clone state propagation)
- Confine changes to 3 files only
- Preserve existing recording/event-loop/OCR behavior

---

## Work Objectives

### Core Objective
Fix screenshot capture failure by ensuring `screenshots_dir` is properly initialized and propagated to the recording loop.

### Concrete Deliverables
- Modified `src-tauri/src/lib.rs` with correct init order
- Fixed `crates/memflow-core/src/collection/collector.rs` clone_for_task to preserve screenshots_dir
- Updated `src-tauri/src/recorder.rs` if needed for readiness gate

### Definition of Done
- [ ] `cargo check --manifest-path src-tauri/Cargo.toml` passes
- [ ] `cargo test --manifest-path src-tauri/Cargo.toml -p memflow` passes
- [ ] App starts without "Screenshots directory not set" warnings
- [ ] Clicking record button creates *.webp screenshot files

### Must Have
- Database initialization completes before recorder initialization
- screenshots_dir is preserved in collector clones
- Recording start waits for prerequisites (ready gate)

### Must NOT Have (Guardrails)
- No changes to OCR worker, scheduler, tray, or AI modules
- No new abstractions or framework rewrites
- No behavior changes outside screenshot readiness path
- No modifications to UI components

---

## Verification Strategy

> **ZERO HUMAN INTERVENTION** — ALL verification is agent-executed.

### Test Decision
- **Infrastructure exists**: YES (cargo test)
- **Automated tests**: YES (Tests after - verify fix with cargo test + manual verification)
- **Framework**: cargo test + runtime log verification

### QA Policy
Every task includes agent-executed QA scenarios with evidence capture.

---

## Execution Strategy

### Parallel Execution Waves

Sequential execution required due to dependency chain.

```
Wave 1 (Foundation - DB init order):
├── Task 1: Reorder lib.rs initialization [deep]

Wave 2 (Core Fix - Clone state propagation):
├── Task 2: Fix collector.rs clone_for_task [deep]

Wave 3 (Verification):
├── Task 3: Cargo check + test [quick]
├── Task 4: Runtime verification [unspecified-high]

Critical Path: Task 1 → Task 2 → Task 3 → Task 4
Sequential: Changes must be ordered correctly
```

---

## TODOs

- [x] 1. **Reorder lib.rs initialization - DB before Recorder**

  **What to do**:
  - Move `db::init_db()` call to execute BEFORE `recorder::init()`
  - Ensure DB init is awaited (not spawned to separate task)
  - Keep OCR service spawn in parallel (unchanged)
  - Keep config init in existing async spawn (can remain parallel)

  **Current problematic order** (`lib.rs:158-164`):
  ```rust
  // Line 159: recorder init happens IMMEDIATELY
  recorder::init(app_handle.clone());

  // Line 167-204: DB init spawned to async task (DELAYED)
  tauri::async_runtime::spawn(async move {
      db::init_db(app_handle.clone()).await  // ← Too late!
      // ...
  });
  ```

  **Target order**:
  ```rust
  // First: Initialize database synchronously (block on this)
  let db_handle = app_handle.clone();
  tauri::async_runtime::block_on(async move {
      if let Err(e) = db::init_db(db_handle).await {
          tracing::error!("CRITICAL: Database init failed: {:#}", e);
      }
  });

  // Second: Initialize recorder (now DB is ready)
  recorder::init(app_handle.clone());

  // Third: Other async spawns (OCR, config) can run parallel
  tauri::async_runtime::spawn(async move {
      app_config::init_config(app_handle.clone()).await
      // ...
  });
  ```

  **Must NOT do**:
  - Don't change OCR service initialization
  - Don't modify tray icon setup
  - Don't touch global shortcut registration
  - Don't move ocr_worker spawn (stays parallel)

  **Recommended Agent Profile**:
  > - **Category**: `deep`
    - Reason: Requires understanding async runtime and initialization dependencies
  - **Skills**: `[]`

  **Parallelization**:
  - **Can Run In Parallel**: NO
  - **Parallel Group**: Sequential (Task 1 → Task 2)
  - **Blocks**: Task 2 (collector fix depends on this)
  - **Blocked By**: None (can start immediately)

  **References**:

  **Pattern References**:
  - `src-tauri/src/lib.rs:158-164` - Current init order (problematic)
  - `src-tauri/src/lib.rs:167-204` - DB init spawn (needs to move earlier)
  - `src-tauri/src/lib.rs:147-156` - OCR service spawn pattern (stays as-is)

  **API/Type References**:
  - `tauri::async_runtime::block_on` - How to block on async during setup
  - `src-tauri/src/db.rs:37-44` - `init_db` function signature

  **External References**:
  - Tauri async runtime: https://tauri.app/v2/api/rust/tauri/async_runtime/

  **Acceptance Criteria**:
  - [ ] File modified: src-tauri/src/lib.rs
  - [ ] db::init_db() called and awaited before recorder::init()
  - [ ] cargo check passes (no compilation errors)

  **QA Scenarios (MANDATORY)**:

  ```bash
  Scenario: Verify compilation after init reorder
    Tool: Bash (cargo)
    Preconditions: Code changes applied
    Steps:
      1. cd /d D:\Demo\memflow && cargo check --manifest-path src-tauri/Cargo.toml
    Expected Result: Exit code 0, "Finished" with no errors
    Failure Indicators: "error:", "expected", "not found"
    Evidence: .sisyphus/evidence/task-1-cargo-check.txt

  Scenario: Verify init order in code
    Tool: Grep
    Preconditions: Changes applied
    Steps:
      1. Search for "recorder::init" and "db::init_db" in lib.rs
      2. Verify db::init_db appears before recorder::init
    Expected Result: db::init_db found at earlier line number than recorder::init
    Failure Indicators: recorder::init appears before db::init_db
    Evidence: .sisyphus/evidence/task-1-init-order-grep.txt
  ```

  **Evidence to Capture**:
  - [ ] cargo check output
  - [ ] grep results showing init order

  **Commit**: NO (groups with Task 2)
  - Message: `fix(recorder): ensure screenshots_dir initialized before recording`
  - Files: src-tauri/src/lib.rs

---

- [x] 2. **Fix collector.rs clone_for_task state propagation**

  **What to do**:
  - Modify `clone_for_task()` in `collector.rs` to preserve `screenshots_dir` from parent
  - Instead of resetting to `AsyncMutex::new(None)`, clone the existing Arc reference
  - Ensure spawned event loop has access to screenshots directory

  **Current buggy code** (`collector.rs:532-546`):
  ```rust
  fn clone_for_task(&self) -> Self {
      Self {
          config: self.config.clone(),
          state: self.state.clone(),
          event_tx: self.event_tx.clone(),
          recording: AtomicBool::new(self.recording.load(Ordering::SeqCst)),
          last_hash: AsyncMutex::new(None),
          last_text_hash: AsyncMutex::new(None),
          heartbeat_ms: AtomicU64::new(self.heartbeat_ms.load(Ordering::Relaxed)),
          base_interval_ms: AtomicU64::new(self.base_interval_ms.load(Ordering::Relaxed)),
          screenshots_dir: AsyncMutex::new(None),  // ← BUG: Loses parent's dir!
          // ...
      }
  }
  ```

  **Target fix**:
  ```rust
  fn clone_for_task(&self) -> Self {
      Self {
          config: self.config.clone(),
          state: self.state.clone(),
          event_tx: self.event_tx.clone(),
          recording: AtomicBool::new(self.recording.load(Ordering::SeqCst)),
          last_hash: AsyncMutex::new(None),
          last_text_hash: AsyncMutex::new(None),
          heartbeat_ms: AtomicU64::new(self.heartbeat_ms.load(Ordering::Relaxed)),
          base_interval_ms: AtomicU64::new(self.base_interval_ms.load(Ordering::Relaxed)),
          screenshots_dir: self.screenshots_dir.clone(),  // ← FIX: Clone the Arc<AsyncMutex>
          event_recorder: AsyncMutex::new(None),
          proactive_callback: AsyncMutex::new(None),
      }
  }
  ```

  **Key change**: `screenshots_dir: AsyncMutex::new(None)` → `screenshots_dir: self.screenshots_dir.clone()`

  **Must NOT do**:
  - Don't change the type of screenshots_dir field
  - Don't modify other clone behavior (last_hash, last_text_hash should reset)
  - Don't change init() or start() methods (unless directly related)

  **Recommended Agent Profile**:
  > - **Category**: `deep`
    - Reason: Requires understanding Arc, AsyncMutex, and Rust clone semantics
  - **Skills**: `[]`

  **Parallelization**:
  - **Can Run In Parallel**: NO
  - **Parallel Group**: Sequential (after Task 1)
  - **Blocks**: Task 3 (verification)
  - **Blocked By**: Task 1 (should verify init order fix first)

  **References**:

  **Pattern References**:
  - `crates/memflow-core/src/collection/collector.rs:532-546` - clone_for_task implementation (buggy)
  - `crates/memflow-core/src/collection/collector.rs:56-58` - screenshots_dir field declaration

  **API/Type References**:
  - `tokio::sync::RwLock::clone` - How to clone Arc-wrapped async primitives

  **External References**:
  - Tokio RwLock: https://docs.rs/tokio/latest/tokio/sync/struct.RwLock.html

  **Acceptance Criteria**:
  - [ ] File modified: crates/memflow-core/src/collection/collector.rs
  - [ ] `screenshots_dir: self.screenshots_dir.clone()` in clone_for_task
  - [ ] cargo check passes (compiles without errors)
  - [ ] cargo test passes (no test breakage)

  **QA Scenarios (MANDATORY)**:

  ```bash
  Scenario: Verify compilation after collector fix
    Tool: Bash (cargo)
    Preconditions: Code changes applied
    Steps:
      1. cd /d D:\Demo\memflow && cargo check --manifest-path src-tauri/Cargo.toml
    Expected Result: Exit code 0, "Finished" with no errors
    Failure Indicators: "error:", "expected", "not found"
    Evidence: .sisyphus/evidence/task-2-cargo-check.txt

  Scenario: Verify clone_for_task preserves screenshots_dir
    Tool: Grep
    Preconditions: Changes applied
    Steps:
      1. Search for "screenshots_dir:" in collector.rs clone_for_task function
      2. Verify line reads "screenshots_dir: self.screenshots_dir.clone()"
    Expected Result: Found "screenshots_dir: self.screenshots_dir.clone()"
    Failure Indicators: Found "screenshots_dir: AsyncMutex::new(None)"
    Evidence: .sisyphus/evidence/task-2-clone-fix-grep.txt

  Scenario: Run unit tests
    Tool: Bash (cargo test)
    Preconditions: Changes applied
    Steps:
      1. cd /d D:\Demo\memflow && cargo test --manifest-path src-tauri/Cargo.toml -p memflow-core
    Expected Result: "test result: ok" with passing tests
    Failure Indicators: "FAILED", "panicked", "error: test failed"
    Evidence: .sisyphus/evidence/task-2-unit-tests.txt
  ```

  **Evidence to Capture**:
  - [ ] cargo check output
  - [ ] grep showing clone_for_task fix
  - [ ] cargo test output

  **Commit**: YES (groups with Task 1)
  - Message: `fix(recorder): ensure screenshots_dir initialized before recording`
  - Files: src-tauri/src/lib.rs, crates/memflow-core/src/collection/collector.rs
  - Pre-commit: `cargo check --manifest-path src-tauri/Cargo.toml && cargo test --manifest-path src-tauri/Cargo.toml -p memflow-core`

---

- [ ] 3. **Runtime verification - Recording starts successfully**

  **What to do**:
  - Start the application in dev mode
  - Click the record button
  - Verify screenshots are being created
  - Check logs for absence of "Screenshots directory not set" errors

  **This is an integration verification task - no code changes.**

  **Verification steps**:
  1. Start `pnpm tauri:dev`
  2. Wait for app to fully load
  3. Click the record button in UI
  4. Wait 10+ seconds
  5. Check logs for:
     - "Activity collector started" (should appear)
     - "Screenshots directory not set" (should NOT appear)
  6. Check screenshots directory for new *.webp files
  7. Stop recording
  8. Count screenshot files created

  **Expected outcome**:
  - No "Screenshots directory not set" warnings
  - At least 1-2 screenshot files created (depends on timing)
  - Screenshot files named like: `{timestamp}_{hash}.webp`

  **Screenshots directory location**:
  ```
  C:\Users\{user}\AppData\Roaming\com.memflow.app\screenshots\
  ```

  **Must NOT do**:
  - Don't make any code changes
  - Don't modify test files

  **Recommended Agent Profile**:
  > - **Category**: `unspecified-high`
    - Reason: Runtime verification, requires launching app and checking filesystem
  - **Skills**: `[]`

  **Parallelization**:
  - **Can Run In Parallel**: NO
  - **Parallel Group**: Sequential (after Tasks 1-2)
  - **Blocks**: Final verification wave
  - **Blocked By**: Tasks 1 and 2 (code changes must be complete)

  **References**:

  **Pattern References**:
  - `src-tauri/src/lib.rs` - App entry point for understanding startup
  - `src/contexts/AppContext.tsx:175-185` - Frontend startRecording function

  **External References**:
  - None

  **Acceptance Criteria**:
  - [ ] App starts without crashing
  - [ ] Record button clickable
  - [ ] No "Screenshots directory not set" in logs
  - [ ] At least one *.webp file created

  **QA Scenarios (MANDATORY)**:

  ```bash
  Scenario: Verify app starts and recording works
    Tool: Bash (dev server + filesystem)
    Preconditions: Code changes from Tasks 1-2 applied
    Steps:
      1. cd /d D:\Demo\memflow
      2. Start dev mode: pnpm tauri:dev (background, timeout 60s)
      3. Wait 30 seconds for app to fully initialize
      4. Check logs for "Activity collector started"
      5. Check logs for "Screenshots directory not set" (should NOT exist)
      6. Get screenshot count from directory
      7. Wait 15 seconds
      8. Get screenshot count again
      9. Verify count increased
    Expected Result: "Activity collector started" in logs, no "Screenshots directory not set", screenshot count > 0
    Failure Indicators: "Screenshots directory not set" in logs, screenshot count = 0
    Evidence: .sisyphus/evidence/task-3-verification.txt

  Scenario: Manual recording test (via logs)
    Tool: Bash (log analysis)
    Preconditions: App running, recording started
    Steps:
      1. Extract log lines containing "WARN" and "collector"
      2. Verify none contain "Screenshots directory not set"
      3. Extract log lines containing "Captured activity:"
      4. Verify at least one exists
    Expected Result: 0 "Screenshots directory not set" lines, >=1 "Captured activity" lines
    Failure Indicators: Any "Screenshots directory not set" warnings
    Evidence: .sisyphus/evidence/task-3-log-analysis.txt
  ```

  **Evidence to Capture**:
  - [ ] App startup logs
  - [ ] Screenshot directory listing
  - [ ] Log analysis results

  **Commit**: NO (verification only)

---

## Final Verification Wave

- [ ] F1. **Build Check** — `quick`
  Run `cargo check --manifest-path src-tauri/Cargo.toml`
  Assert: exit code 0, no compilation errors

- [ ] F2. **Unit Tests** — `quick`
  Run `cargo test --manifest-path src-tauri/Cargo.toml -p memflow-core`
  Assert: all tests pass

- [ ] F3. **Runtime Log Verification** — `unspecified-high`
  Start app with `pnpm tauri:dev`, click record button
  Assert: logs include "Activity collector started" WITHOUT "Screenshots directory not set"
  Evidence: `.sisyphus/evidence/runtime-log.txt`

- [ ] F4. **Screenshot Artifact Verification** — `unspecified-high`
  After recording starts, check screenshots directory
  Assert: at least one new *.webp file exists
  Evidence: `.sisyphus/evidence/screenshot-list.txt`

---

## Commit Strategy

- **1**: `fix(recorder): ensure screenshots_dir initialized before recording` — lib.rs, collector.rs, recorder.rs

---

## Success Criteria

### Verification Commands
```bash
# Build check
cargo check --manifest-path src-tauri/Cargo.toml
# Expected: exit code 0

# Unit tests
cargo test --manifest-path src-tauri/Cargo.toml -p memflow-core
# Expected: all pass

# Runtime (manual step - agent executes)
pnpm tauri:dev
# Click record, check logs, verify *.webp files created
```

### Final Checklist
- [ ] Database init completes before recorder init
- [ ] screenshots_dir preserved in collector clones
- [ ] No "Screenshots directory not set" warnings in logs
- [ ] Screenshot files created when recording starts
