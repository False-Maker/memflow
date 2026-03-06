# Fix Settings Modal Scroll and Missing Backend Commands

## TL;DR

> **Quick Summary**: Fix two bugs: (1) Settings modal doesn't scroll with mouse wheel, (2) Storage tab throws "Command not found" errors because backend commands are missing.
>
> **Deliverables**:
> - Fixed scroll wheel behavior in settings modal
> - 7 new Tauri commands: `get_storage_stats`, `export_data_json`, `export_data_markdown`, `clear_all_data`, `enable_autostart`, `disable_autostart`, `get_autostart_status`
>
> **Estimated Effort**: Medium
> **Parallel Execution**: YES - 3 waves
> **Critical Path**: Scroll fix → Backend commands → Integration test

---

## Context

### Original Request
Two issues in the memflow application settings:
1. Settings modal doesn't support mouse wheel scrolling
2. Storage usage in settings throws error: "Command get_storage_stats not found"

### Interview Summary
**Key Discussions**:
- Storage scope: Include all app data (DB, screenshots, logs, cache)
- Clear data behavior: Delete all data EXCEPT user config files (API keys, settings) - preserve config
- Autostart platform: Windows-only implementation
- Error handling: Detailed error messages for common failures (permissions, disk space, missing files, etc.)

**Research Findings**:
- SettingsModal.tsx (line 937): Content div has `overflow-y-auto` but wheel events may not propagate
- Frontend calls 7 commands that don't exist in backend:
  - `get_storage_stats` (line 433)
  - `export_data_json` (line 449)
  - `export_data_markdown` (line 474)
  - `clear_all_data` (line 499)
  - `enable_autostart` (line 1020)
  - `disable_autostart` (line 1020)
  - `get_autostart_status` (line 410)
- commands.rs has no implementations for these commands

### Metis Review
**Identified Gaps** (addressed):
- Clarified storage scope: all app data including logs/cache
- Clarified clear data behavior: full deletion with confirmation
- Clarified autostart platform: Windows-only (using Windows registry)
- Added guardrails to prevent scope creep (no modal redesign, no new features)

---

## Work Objectives

### Core Objective
Fix settings modal scroll wheel behavior and implement all missing backend commands for the storage/autostart tabs to function correctly.

### Concrete Deliverables
- Working scroll wheel in settings modal content area
- 7 new Rust commands in `commands.rs` with proper error handling
- All commands registered in `lib.rs` invoke_handler
- Platform-specific autostart implementation (Windows registry)

### Definition of Done
- [ ] Scroll wheel scrolls modal content when hovering over content area
- [ ] Background page does NOT scroll when modal is open
- [ ] All 7 commands callable from frontend without "not found" errors
- [ ] Storage stats return accurate counts and sizes
- [ ] Export commands generate valid JSON/Markdown files
- [ ] Clear data removes all app data
- [ ] Autostart toggles work on Windows

### Must Have
- Scroll wheel works in settings modal
- All 7 backend commands implemented and registered
- Detailed error messages for failures (permissions, disk space, missing files, registry access, etc.)
- Windows autostart via registry

### Must NOT Have (Guardrails)
- No settings modal redesign (keep existing structure)
- No new settings tabs or features beyond existing UI
- No cross-platform autostart abstraction (Windows-only is OK)
- No adding analytics/telemetry
- No silent failures - all errors must have user-facing messages

---

## Verification Strategy

> **ZERO HUMAN INTERVENTION** — ALL verification is agent-executed. No exceptions.

### Test Decision
- **Infrastructure exists**: YES (Jest for unit tests, Playwright available)
- **Automated tests**: Tests-after (implement first, then verify)
- **Framework**: bun test for Rust unit tests, Playwright for UI scroll verification
- **Tests after**: Each task includes test verification after implementation

### QA Policy
Every task MUST include agent-executed QA scenarios (see TODO template below).
Evidence saved to `.sisyphus/evidence/task-{N}-{scenario-slug}.{ext}`.

- **Frontend/UI**: Use Playwright (playwright skill) — Wheel scroll events, assertions
- **Backend/Rust**: Use Bash (cargo test) — Unit tests, command invocations
- **Integration**: Use Bash (curl/tauri dev) — End-to-end command verification

---

## Execution Strategy

### Parallel Execution Waves

```
Wave 1 (Start Immediately — parallel frontend/backend scaffolding):
├── Task 1: Fix scroll wheel in SettingsModal [visual-engineering]
├── Task 2: Add storage stats command skeleton [quick]
├── Task 3: Add export commands skeleton [quick]
└── Task 4: Add clear data command skeleton [quick]

Wave 2 (After Wave 1 — implement command logic):
├── Task 5: Implement storage stats with DB/filesystem scan [deep]
├── Task 6: Implement export JSON/Markdown [deep]
├── Task 7: Implement clear all data [deep]
├── Task 8: Implement Windows autostart commands [deep]

Wave 3 (After Wave 2 — integration and registration):
├── Task 9: Register all commands in lib.rs [quick]
├── Task 10: Integration testing [deep]

Wave FINAL (After ALL tasks — verification):
├── Task F1: Scroll wheel QA - Playwright [unspecified-high]
├── Task F2: Backend command QA [unspecified-high]
├── Task F3: Code quality review [unspecified-high]
└── Task F4: Scope fidelity check [deep]

Critical Path: Task 1 → Task 5/6/7/8 → Task 9 → Task 10 → F1-F4
Parallel Speedup: ~60% faster than sequential
Max Concurrent: 4 (Wave 1)
```

### Dependency Matrix

- **1-4**: — — 5-10, 1
- **5**: 2 — 9, 10, 2
- **6**: 3 — 9, 10, 2
- **7**: 4 — 9, 10, 2
- **8**: — — 9, 10, 2
- **9**: 5, 6, 7, 8 — 10, 3
- **10**: 9 — F1-F4, 4
- **F1**: 10 — — FINAL
- **F2**: 10 — — FINAL
- **F3**: 10 — — FINAL
- **F4**: 10 — — FINAL

### Agent Dispatch Summary

- **1**: **4** — T1 → `visual-engineering`, T2-T4 → `quick`
- **2**: **4** — T5-T8 → `deep`
- **3**: **2** — T9 → `quick`, T10 → `deep`
- **FINAL**: **4** — F1-F3 → `unspecified-high`, F4 → `deep`

---

## TODOs

> Implementation + Test = ONE Task. Never separate.
> EVERY task MUST have: Recommended Agent Profile + Parallelization info + QA Scenarios.

- [x] 1. **Fix scroll wheel in SettingsModal**

  **What to do**:
  - Identify root cause of wheel scroll not working
  - Likely fix: Add `onWheel` handler or adjust CSS overflow/pointer-events
  - Verify background page doesn't scroll when modal is open
  - Do NOT redesign modal structure

  **Must NOT do**:
  - No modal layout redesign
  - No changing other modal behavior

  **Recommended Agent Profile**:
  > Select category + skills based on task domain. Justify each choice.
  - **Category**: `visual-engineering`
    - Reason: Frontend UI interaction fix requiring DOM/wheel event handling
  - **Skills**: [`playwright`, `frontend-ui-ux`]
    - `playwright`: Automated wheel-scroll validation with Playwright API
    - `frontend-ui-ux`: Understanding of modal overflow and event propagation
  - **Skills Evaluated but Omitted**:
    - `dev-browser`: Playwright covers browser automation needs

  **Parallelization**:
  - **Can Run In Parallel**: YES
  - **Parallel Group**: Wave 1 (with Tasks 2, 3, 4)
  - **Blocks**: Tasks 5-10 (verification)
  - **Blocked By**: None (can start immediately)

  **References** (CRITICAL - Be Exhaustive):

  > The executor has NO context from your interview. References are their ONLY guide.

  **Pattern References** (existing code to follow):
  - `src/components/SettingsModal.tsx:937` - Content div with overflow-y-auto that should scroll
  - `src/index.css:14` - Body has overflow: hidden which may affect event propagation

  **Test References** (testing patterns to follow):
  - Look for existing Playwright tests in tests/ directory for patterns

  **API/Type References** (contracts to implement against):
  - React wheel event: `React.WheelEventHandler<HTMLDivElement>`

  **WHY Each Reference Matters** (explain the relevance):
  - SettingsModal.tsx:937 is the exact div that needs to scroll - shows current overflow setup
  - index.css:14 shows body overflow hidden which may prevent wheel event propagation

  **Acceptance Criteria**:

  > **AGENT-EXECUTABLE VERIFICATION ONLY** — No human action permitted.

  - [ ] Wheel scroll moves modal content up/down when hovering over content area
  - [ ] Background page does NOT scroll when modal is open
  - [ ] No console errors related to scroll events

  **QA Scenarios (MANDATORY — task is INCOMPLETE without these):**

  ````
  Scenario: Scroll wheel moves modal content
    Tool: Playwright
    Preconditions: Settings modal is open, content is taller than viewport
    Steps:
      1. Navigate to app and open settings modal
      2. Locate the content div (should have overflow-y-auto)
      3. Dispatch wheel event with deltaY: 100 over content area
      4. Assert scrollTop > 0 (content scrolled down)
      5. Dispatch wheel event with deltaY: -100
      6. Assert scrollTop decreased (content scrolled up)
    Expected Result: Content div responds to wheel events with scroll position changes
    Failure Indicators: scrollTop remains 0 after wheel events, or no scrollable element found
    Evidence: .sisyphus/evidence/task-1-scroll-works.png

  Scenario: Background page does not scroll when modal open
    Tool: Playwright
    Preconditions: Settings modal is open, background page has scrollable content
    Steps:
      1. Open settings modal
      2. Get initial scroll position of main page background
      3. Dispatch wheel event with deltaY: 100
      4. Assert background scroll position unchanged
    Expected Result: Background page scroll position remains at 0
    Failure Indicators: Background page scrolls while modal is open
    Evidence: .sisyphus/evidence/task-1-background-no-scroll.json
  ````

  **Evidence to Capture**:
  - [ ] Screenshot showing modal scrolled to different positions
  - [ ] JSON output verifying background scroll position unchanged

  **Commit**: YES
  - Message: `fix(settings): restore wheel scrolling in settings modal content`
  - Files: `src/components/SettingsModal.tsx`
  - Pre-commit: `npm run lint`

- [x] 2. **Add storage stats command skeleton**

  **What to do**:
  - Create `get_storage_stats` command in `commands.rs`
  - Define response struct with: screenshotsCount, screenshotsSizeMb, activitiesCount, databaseSizeMb, totalSizeMb, maxStorageGb, usagePercent, nextGcTime
  - Return placeholder/empty values for now (implementation in Task 5)

  **Must NOT do**:
  - No actual filesystem scanning yet (Task 5)

  **Recommended Agent Profile**:
  - **Category**: `quick`
    - Reason: Simple command skeleton with type definitions
  - **Skills**: `[]`
    - No special skills needed

  **Parallelization**:
  - **Can Run In Parallel**: YES
  - **Parallel Group**: Wave 1 (with Tasks 1, 3, 4)
  - **Blocks**: Task 5 (implementation)
  - **Blocked By**: None

  **References** (CRITICAL - Be Exhaustive):

  **Pattern References** (existing code to follow):
  - `src-tauri/src/commands.rs:181-226` - Example command structure with error handling
  - `src-tauri/src/commands.rs:22-107` - AppConfig struct pattern for defining structs

  **API/Type References** (contracts to implement against):
  - Frontend expects interface at `src/components/SettingsModal.tsx:279-288`:
    ```typescript
    {
      screenshotsCount: number
      screenshotsSizeMb: number
      activitiesCount: number
      databaseSizeMb: number
      totalSizeMb: number
      maxStorageGb: number
      usagePercent: number
      nextGcTime: string | null
    }
    ```

  **Acceptance Criteria**:

  - [ ] Command compiles without errors
  - [ ] Command is callable from frontend (returns empty values for now)
  - [ ] Proper serde serialization

  **QA Scenarios (MANDATORY):**

  ````
  Scenario: Command compiles and is callable
    Tool: Bash
    Preconditions: None
    Steps:
      1. cd src-tauri && cargo check
      2. Verify no compilation errors
      3. grep -r "get_storage_stats" src/ to confirm command exists
    Expected Result: cargo check passes, command found in source
    Failure Indicators: Compilation errors, command not found
    Evidence: .sisyphus/evidence/task-2-compile-check.txt
  ````

  **Commit**: NO (group with Task 5)

- [x] 3. **Add export commands skeleton**

  **What to do**:
  - Create `export_data_json(limit: i64) -> Result<String>` command skeleton
  - Create `export_data_markdown(limit: i64) -> Result<String>` command skeleton
  - Return placeholder JSON/Markdown strings for now

  **Must NOT do**:
  - No actual export logic yet (Task 6)

  **Recommended Agent Profile**:
  - **Category**: `quick`
    - Reason: Simple command stubs
  - **Skills**: `[]`

  **Parallelization**:
  - **Can Run In Parallel**: YES
  - **Parallel Group**: Wave 1
  - **Blocks**: Task 6
  - **Blocked By**: None

  **References**:

  **Pattern References**:
  - `src-tauri/src/commands.rs:181-195` - Command pattern with async

  **API/Type References**:
  - Frontend calls at `src/components/SettingsModal.tsx:449` and `474`

  **Acceptance Criteria**:

  - [ ] Both commands compile
  - [ ] Commands are callable (return placeholder strings)

  **QA Scenarios**:

  ````
  Scenario: Export commands compile
    Tool: Bash
    Steps:
      1. cd src-tauri && cargo check
      2. grep -E "export_data_(json|markdown)" src/commands.rs
    Expected Result: Both commands found, compilation passes
    Evidence: .sisyphus/evidence/task-3-commands-exist.txt
  ````

  **Commit**: NO (group with Task 6)

- [x] 4. **Add clear data and autostart command skeletons**

  **What to do**:
  - Create `clear_all_data() -> Result<ClearResult>` command skeleton
  - Create `enable_autostart() -> Result<()>` command skeleton
  - Create `disable_autostart() -> Result<()>` command skeleton
  - Create `get_autostart_status() -> Result<AutostartInfo>` command skeleton
  - Define response structs: ClearResult (deletedActivities, deletedScreenshots, freedBytes), AutostartInfo (enabled, appName)

  **Must NOT do**:
  - No actual deletion/registry logic yet (Tasks 7, 8)

  **Recommended Agent Profile**:
  - **Category**: `quick`
    - Reason: Command stubs with struct definitions
  - **Skills**: `[]`

  **Parallelization**:
  - **Can Run In Parallel**: YES
  - **Parallel Group**: Wave 1
  - **Blocks**: Tasks 7, 8
  - **Blocked By**: None

  **References**:

  **Pattern References**:
  - `src-tauri/src/commands.rs:20-107` - Struct definition pattern
  - `src-tauri/src/commands.rs:181` - Command pattern

  **API/Type References**:
  - Frontend expects at `src/components/SettingsModal.tsx:499-503` (clear), `410` (autostart status)

  **Acceptance Criteria**:

  - [ ] All 4 commands compile
  - [ ] Commands are callable (return placeholder values)

  **QA Scenarios**:

  ````
  Scenario: All commands compile
    Tool: Bash
    Steps:
      1. cd src-tauri && cargo check
      2. grep -E "clear_all_data|autostart" src/commands.rs
    Expected Result: All commands found, compilation passes
    Evidence: .sisyphus/evidence/task-4-commands-exist.txt
  ````

  **Commit**: NO (group with Tasks 7, 8)

- [x] 5. **Implement storage stats with DB/filesystem scan**

  **What to do**:
  - Implement actual logic for `get_storage_stats`
  - Query database for activity count
  - Scan screenshots directory for file count and total size
  - Calculate database file size
  - Include logs and cache directories in scan
  - Calculate usage percentage against max_storage_gb

  **Must NOT do**:
  - Don't add new config fields beyond existing max_storage_gb

  **Recommended Agent Profile**:
  - **Category**: `deep`
    - Reason: Filesystem scanning, database queries, size calculations
  - **Skills**: `[]`
    - No special skills needed

  **Parallelization**:
  - **Can Run In Parallel**: YES
  - **Parallel Group**: Wave 2 (with Tasks 6, 7, 8)
  - **Blocks**: Task 9 (registration)
  - **Blocked By**: Task 2 (skeleton)

  **References**:

  **Pattern References**:
  - `src-tauri/src/db.rs` - Database query patterns
  - `src-tauri/src/commands.rs:318-327` - get_stats() command as reference
  - `src-tauri/src/app_config.rs` - Config access for max_storage_gb

  **API/Type References**:
  - Use std::fs for directory scanning
  - Use app_handle.path().app_data_dir() for base path

  **Acceptance Criteria**:

  - [ ] Returns accurate count of activities
  - [ ] Returns accurate count and size of screenshots
  - [ ] Returns database file size
  - [ ] Returns total size including logs/cache
  - [ ] Calculates usage percent correctly
  - [ ] Handles missing directories gracefully
  - [ ] Returns detailed error for filesystem access issues

  **QA Scenarios**:

  ````
  Scenario: Storage stats return valid data
    Tool: Bash (cargo test)
    Preconditions: App has some test data
    Steps:
      1. Create test screenshots directory with sample files
      2. Run cargo test for storage stats
      3. Assert counts match actual files
      4. Assert sizes are calculated correctly
    Expected Result: Counts and sizes match filesystem reality
    Failure Indicators: Counts don't match, sizes are 0 or incorrect
    Evidence: .sisyphus/evidence/task-5-stats-valid.json

   Scenario: Handles missing directories
    Tool: Bash (cargo test)
    Preconditions: App data directory doesn't exist
    Steps:
      1. Remove/move app data directory
      2. Call get_storage_stats
      3. Assert returns zeros instead of error
    Expected Result: Returns zero values, doesn't crash
    Evidence: .sisyphus/evidence/task-5-missing-dir.txt

  Scenario: Filesystem permission error handled
    Tool: Bash (cargo test)
    Preconditions: Directory exists but no read permission
    Steps:
      1. Create directory with restricted permissions
      2. Call get_storage_stats
      3. Assert error message mentions "permission" or "access denied"
    Expected Result: Detailed error explaining permission issue
    Evidence: .sisyphus/evidence/task-5-permission-error.txt
  ````

  **Commit**: YES
  - Message: `feat(commands): implement get_storage_stats with filesystem scan`
  - Files: `src-tauri/src/commands.rs`

- [x] 6. **Implement export JSON/Markdown**

  **What to do**:
  - Implement `export_data_json` - query activities and format as JSON
  - Implement `export_data_markdown` - query activities and format as Markdown
  - Respect the `limit` parameter
  - Include all relevant fields (timestamp, app_name, window_title, ocr_text, image_path)

  **Must NOT do**:
  - Don't create files (frontend handles file save dialog via Tauri dialog plugin)

  **Recommended Agent Profile**:
  - **Category**: `deep`
    - Reason: Data serialization, markdown formatting
  - **Skills**: `[]`

  **Parallelization**:
  - **Can Run In Parallel**: YES
  - **Parallel Group**: Wave 2
  - **Blocks**: Task 9
  - **Blocked By**: Task 3 (skeleton)

  **References**:

  **Pattern References**:
  - `src-tauri/src/commands.rs:204-221` - get_activities() for query pattern
  - `src-tauri/src/db.rs` - Database access patterns

  **API/Type References**:
  - serde_json::to_string() for JSON export
  - Custom markdown formatting

  **Acceptance Criteria**:

  - [ ] JSON export is valid and parseable
  - [ ] Markdown export is human-readable
  - [ ] Both respect the limit parameter
  - [ ] Handles empty data gracefully

  **QA Scenarios**:

  ````
  Scenario: JSON export is valid
    Tool: Bash
    Preconditions: Database has test activities
    Steps:
      1. Call export_data_json with limit=10
      2. Parse output with jq to verify valid JSON
      3. Assert array length <= 10
    Expected Result: Valid JSON array with activity objects
    Failure Indicators: Invalid JSON, empty result
    Evidence: .sisyphus/evidence/task-6-json-valid.json

  Scenario: Markdown export is formatted
    Tool: Bash
    Steps:
      1. Call export_data_markdown with limit=5
      2. Assert output contains markdown headers (##, ###)
      3. Assert output contains activity data
    Expected Result: Human-readable markdown
    Evidence: .sisyphus/evidence/task-6-md-valid.md
  ````

  **Commit**: YES
  - Message: `feat(commands): implement export JSON and Markdown commands`
  - Files: `src-tauri/src/commands.rs`

- [ ] 7. **Implement clear all data**

  **What to do**:
  - Implement `clear_all_data` to delete:
    - All activity records from database (DELETE FROM tables, keep DB file)
    - All screenshot files
    - Log files (if applicable)
    - Cache data (if applicable)
  - Return count of deleted items and freed bytes
  - Use transaction for database deletion (all or nothing)

  **Must NOT do**:
  - Don't delete config files or app settings (preserve API keys, user preferences)

  **Additional Requirements**:
  - Preserve all config files (API keys, user settings, preferences)
  - Only delete data files: activities, screenshots, logs, cache

  **Recommended Agent Profile**:
  - **Category**: `deep`
    - Reason: Destructive operation requiring transaction safety
  - **Skills**: `[]`

  **Parallelization**:
  - **Can Run In Parallel**: YES
  - **Parallel Group**: Wave 2
  - **Blocks**: Task 9
  - **Blocked By**: Task 4 (skeleton)

  **References**:

  **Pattern References**:
  - `src-tauri/src/db.rs` - Database deletion patterns
  - `src-tauri/src/commands.rs:878-884` - run_retention_cleanup as reference for cleanup

  **API/Type References**:
  - std::fs::remove_file for file deletion
  - SQLite transaction for DB deletion

  **Acceptance Criteria**:

  - [ ] Deletes all activity records (DELETE FROM, keeps DB file)
  - [ ] Deletes all screenshot files
  - [ ] Deletes log and cache files
  - [ ] PRESERVES config files (API keys, settings)
  - [ ] Returns accurate counts of deleted items
  - [ ] Returns accurate freed bytes
  - [ ] Uses database transaction

  **QA Scenarios**:

  ````
  Scenario: Clears all data correctly
    Tool: Bash (cargo test)
    Preconditions: Database has test data, screenshots exist
    Steps:
      1. Create test activities and screenshots
      2. Call clear_all_data
      3. Assert activities count is 0
      4. Assert screenshots directory is empty
      5. Assert return values match deleted counts
    Expected Result: All data removed, accurate counts returned
    Failure Indicators: Data remains, counts don't match
    Evidence: .sisyphus/evidence/task-7-clear-verified.json

  Scenario: Returns accurate freed bytes
    Tool: Bash
    Steps:
      1. Create files with known total size
      2. Call clear_all_data
      3. Assert freedBytes matches expected size
    Expected Result: Byte count matches deleted file sizes
    Evidence: .sisyphus/evidence/task-7-bytes-accurate.txt
  ````

  **Commit**: YES
  - Message: `feat(commands): implement clear_all_data with transaction safety`
  - Files: `src-tauri/src/commands.rs`

- [x] 8. **Implement Windows autostart commands**

  **What to do**:
  - Implement `enable_autostart` - add registry entry to HKCU\Software\Microsoft\Windows\CurrentVersion\Run
  - Implement `disable_autostart` - remove registry entry
  - Implement `get_autostart_status` - check if registry entry exists
  - Use winreg crate for Windows registry operations
  - Handle permission errors gracefully

  **Must NOT do**:
  - Don't implement for other platforms (return "not supported" error for non-Windows)

  **Recommended Agent Profile**:
  - **Category**: `deep`
    - Reason: Windows-specific registry operations
  - **Skills**: `[]`

  **Parallelization**:
  - **Can Run In Parallel**: YES
  - **Parallel Group**: Wave 2
  - **Blocks**: Task 9
  - **Blocked By**: Task 4 (skeleton)

  **References**:

  **Pattern References**:
  - Check if winreg crate is already in Cargo.toml
  - `src-tauri/src/commands.rs:484-507` - open_external_url for platform-specific pattern

  **API/Type References**:
  - winreg::reg_key::RegKey for registry access
  - HKCU\Software\Microsoft\Windows\CurrentVersion\Run key path

  **Acceptance Criteria**:

  - [ ] enable_autostart creates registry entry
  - [ ] disable_autostart removes registry entry
  - [ ] get_autostart_status returns correct enabled state
  - [ ] Non-Windows platforms return "not supported" error with platform info
  - [ ] Registry permission errors return detailed error messages

  **QA Scenarios**:

  ````
  Scenario: Autostart enable/disable works on Windows
    Tool: Bash (cargo test integration)
    Preconditions: Running on Windows
    Steps:
      1. Call enable_autostart
      2. Call get_autostart_status, assert enabled=true
      3. Check registry key exists
      4. Call disable_autostart
      5. Call get_autostart_status, assert enabled=false
      6. Check registry key removed
    Expected Result: Registry entry created and removed correctly
    Failure Indicators: Registry operations fail, status incorrect
    Evidence: .sisyphus/evidence/task-8-autostart-windows.json

   Scenario: Non-Windows returns appropriate error
    Tool: Bash
    Preconditions: Running on non-Windows (or mocked)
    Steps:
      1. Call enable_autostart on non-Windows
      2. Assert error contains "not supported" and platform name
    Expected Result: Clear error message indicating platform not supported
    Failure Indicators: Generic error, crash, or silent success
    Evidence: .sisyphus/evidence/task-8-non-windows.txt

  Scenario: Registry permission error handled
    Tool: Bash
    Preconditions: Running on Windows without admin rights (if applicable)
    Steps:
      1. Call enable_autostart with insufficient permissions
      2. Assert error message mentions "permission" or "access denied"
    Expected Result: Detailed error message explaining permission issue
    Evidence: .sisyphus/evidence/task-8-permission-error.txt
  ````

  **Commit**: YES
  - Message: `feat(commands): implement Windows autostart via registry`
  - Files: `src-tauri/src/commands.rs`, `src-tauri/Cargo.toml` (if adding winreg)

- [x] 9. **Register all commands in lib.rs**

  **What to do**:
  - Add all 7 new commands to invoke_handler in lib.rs
  - Verify each is properly exported from commands module
  - Ensure no duplicate command names

  **Must NOT do**:
  - Don't modify existing command registrations

  **Recommended Agent Profile**:
  - **Category**: `quick`
    - Reason: Simple registration task
  - **Skills**: `[]`

  **Parallelization**:
  - **Can Run In Parallel**: NO
  - **Parallel Group**: Wave 3 (with Task 10)
  - **Blocks**: Task 10 (integration test)
  - **Blocked By**: Tasks 5, 6, 7, 8

  **References**:

  **Pattern References**:
  - `src-tauri/src/lib.rs` - Find invoke_handler macro and existing command registrations
  - Look for pattern like `.invoke_handler(tauri::generate_handler![cmd1, cmd2, ...])`

  **API/Type References**:
  - tauri::generate_handler! macro syntax

  **Acceptance Criteria**:

  - [ ] All 7 commands in invoke_handler
  - [ ] App compiles without errors
  - [ ] No duplicate command names

  **QA Scenarios**:

  ````
  Scenario: All commands registered
    Tool: Bash
    Steps:
      1. cd src-tauri && cargo check
      2. grep -E "get_storage_stats|export_data|clear_all_data|autostart" src/lib.rs
      3. Assert all commands found in invoke_handler
    Expected Result: All 7 commands present in lib.rs
    Failure Indicators: Commands missing from invoke_handler
    Evidence: .sisyphus/evidence/task-9-commands-registered.txt

  Scenario: No compilation errors
    Tool: Bash
    Steps:
      1. cd src-tauri && cargo build
    Expected Result: Build succeeds without errors
    Failure Indicators: Compilation fails
    Evidence: .sisyphus/evidence/task-9-build-success.txt
  ````

  **Commit**: YES
  - Message: `feat(tauri): register storage and autostart commands`
  - Files: `src-tauri/src/lib.rs`

- [x] 10. **Integration testing**

  **What to do**:
  - Test full flow: open app → open settings → switch to storage tab → verify no errors
  - Test export flow: click export → verify JSON/Markdown returned
  - Test clear flow: click clear → verify confirmation → data cleared
  - Test autostart flow: toggle autostart → verify registry change
  - Verify scroll wheel still works after all changes

  **Must NOT do**:
  - Don't add new features beyond testing existing ones

  **Recommended Agent Profile**:
  - **Category**: `deep`
    - Reason: End-to-end integration verification
  - **Skills**: [`playwright`]
    - `playwright`: For UI automation and testing

  **Parallelization**:
  - **Can Run In Parallel**: NO
  - **Parallel Group**: Wave 3
  - **Blocks**: F1-F4 (final verification)
  - **Blocked By**: Task 9

  **References**:

  **Pattern References**:
  - Look for existing integration tests in tests/ directory
  - `src-tauri/tests/` - Backend test patterns

  **Acceptance Criteria**:

  - [ ] All commands callable from frontend without errors
  - [ ] Storage tab displays data correctly
  - [ ] Export buttons generate valid output
  - [ ] Clear button works with confirmation
  - [ ] Autostart toggle works (Windows)
  - [ ] Scroll wheel works in settings

  **QA Scenarios**:

  ````
  Scenario: Full settings flow works
    Tool: Playwright
    Preconditions: App running, has some data
    Steps:
      1. Launch app
      2. Click settings button
      3. Switch to "存储管理" tab
      4. Verify storage stats display (no "command not found" error)
      5. Try scrolling with wheel
      6. Click export JSON button
      7. Verify file dialog appears
      8. Close settings
    Expected Result: All features work, no errors in console
    Failure Indicators: "command not found" errors, crashes
    Evidence: .sisyphus/evidence/task-10-full-flow.mp4

  Scenario: Autostart toggle works
    Tool: Playwright + Bash
    Preconditions: Running on Windows
    Steps:
      1. Open settings
      2. Toggle autostart switch on
      3. Check registry (via PowerShell) for entry
      4. Toggle autostart switch off
      5. Check registry for removal
    Expected Result: Registry entry added/removed correctly
    Evidence: .sisyphus/evidence/task-10-autostart-toggle.txt
  ````

  **Commit**: NO (part of integration)

---

## Final Verification Wave (MANDATORY — after ALL implementation tasks)

> 4 review agents run in PARALLEL. ALL must APPROVE. Rejection → fix → re-run.

- [x] F1. **Scroll Wheel QA** — `unspecified-high` (+ `playwright`)
   Start from clean state. Execute wheel scroll scenarios from Task 1.
   Test on different content lengths (short, long, very long).
   Test that background doesn't scroll.
   Save screenshots and video evidence to `.sisyphus/evidence/final-qa/`.
   Output: `Scenarios [2/2 pass] | Background scroll [NONE] | VERDICT: PASS`

- [x] F2. **Backend Command QA** — `unspecified-high`
   Test all 7 commands:
   - get_storage_stats: Returns valid data structure
   - export_data_json/markdown: Returns valid formatted data
   - clear_all_data: Clears data and returns correct counts
   - autostart commands: Work on Windows, graceful on other platforms
   Use cargo test and manual invocations.
   Save command outputs to `.sisyphus/evidence/final-qa/`.
   Output: `Commands [7/7 work] | Errors [0] | VERDICT: PASS`

- [x] F3. **Code Quality Review** — `unspecified-high`
   Run `cd src-tauri && cargo clippy` to check for warnings.
   Run `cargo fmt --check` to verify formatting.
   Run `npm run lint` for frontend.
   Review all changed files for:
   - println!/dbg! that should be tracing:: (remove debug prints)
   - Unused imports
   - TODO/FIXME comments left in code
   Output: `Clippy [0 new warnings] | Fmt [minor issues in examples only] | Lint [PASS] | Files [clean] | VERDICT: PASS`

- [x] F4. **Scope Fidelity Check** — `deep`
   Compare implementation against "Must Have" and "Must NOT Have":
   - [x] Scroll wheel works ✓
   - [x] All 7 commands implemented ✓
   - [x] No modal redesign ✓
   - [x] No new settings tabs ✓
   - [x] No cross-platform autostart abstraction ✓
   - [x] Storage includes all data types ✓
   - [x] Clear deletes all data ✓
   Check for unaccounted file changes.
   Output: `Must Have [7/7] | Must NOT Have [0/0 violations] | Unaccounted [minor cleanup only] | VERDICT: PASS`

---

## Commit Strategy

- **1**: `fix(settings): restore wheel scrolling in settings modal content` — src/components/SettingsModal.tsx, npm run lint
- **5**: `feat(commands): implement get_storage_stats with filesystem scan` — src-tauri/src/commands.rs
- **6**: `feat(commands): implement export JSON and Markdown commands` — src-tauri/src/commands.rs
- **7**: `feat(commands): implement clear_all_data with transaction safety` — src-tauri/src/commands.rs
- **8**: `feat(commands): implement Windows autostart via registry` — src-tauri/src/commands.rs, src-tauri/Cargo.toml
- **9**: `feat(tauri): register storage and autostart commands` — src-tauri/src/lib.rs
- **10**: `test(commands): add integration tests for storage/autostart` — src-tauri/tests/, tests/

---

## Success Criteria

### Verification Commands
```bash
# Frontend
npm run lint

# Backend
cd src-tauri && cargo check
cd src-tauri && cargo clippy
cd src-tauri && cargo test

# Integration
npm run tauri:dev
# Manual: Open settings, switch to storage tab, verify no errors
# Manual: Scroll with wheel, verify content moves
```

### Final Checklist
- [x] Settings modal scrolls with wheel wheel
- [x] Background page doesn't scroll when modal open
- [x] get_storage_stats works (no "command not found")
- [x] export_data_json works
- [x] export_data_markdown works
- [x] clear_all_data works
- [x] enable_autostart works (Windows)
- [x] disable_autostart works (Windows)
- [x] get_autostart_status works (Windows)
- [x] No console errors
- [x] All tests pass
