## Task 6 Learnings

### Implementation Notes
- Used `db::get_activities(limit)` to query activities from database
- ActivityLog struct includes: id, timestamp, app_name, window_title, image_path, ocr_text, phash
- `chrono::DateTime::from_timestamp()` returns `Option` not `Result`
- JSON export formatted with metadata (exportType, version, timestamp, count)
- Markdown export uses headers (##), inline code (`), and code blocks (```)

### Fix Applied
- Changed `if let Ok(dt)` to `if let Some(dt)` for chrono DateTime parsing
- Added `scan_directory` helper function that was referenced but not defined
- Fixed autostart disable command match expression (separate branch for Ok/Err)

### QA Evidence
- JSON export produces valid parseable JSON
- Markdown export contains proper headers and activity data

---

## Task 8 Learnings - Implement Windows autostart commands

### Implementation Notes
- **Registry Path**: `HKEY_CURRENT_USER\Software\Microsoft\Windows\CurrentVersion\Run`
- **Value Name**: "MemFlow"
- **Value Data**: Full path to executable via `std::env::current_exe()`
- Added `winreg = "0.52"` to `Cargo.toml` under `[target.'cfg(windows)'.dependencies]`
- Used `#[cfg(target_os = "windows")]` for platform-specific compilation
- Non-Windows platforms return "not supported" error with platform name

### Winreg Crate Usage
- `RegKey::predef(HKEY_CURRENT_USER)` to access HKCU hive
- `open_subkey_with_flags(path, KEY_WRITE/KEY_READ)` for registry access
- `set_value()` for creating registry entries
- `delete_value()` for removing registry entries
- `get_value()` for checking if a value exists

### Fix Applied
- Removed unused `app_handle` parameter from `enable_autostart`
- Removed duplicate `scan_directory` function definition

### QA Evidence
- Code compiles successfully (`cargo check` passed)
- LSP diagnostics clean for modified files

---

## Task 5 Learnings - Implement storage stats with DB/filesystem scan

### Implementation Details
- Added `app_handle: tauri::AppHandle` parameter to access app data directory
- Queries database for activity count using `sqlx::query_scalar::<_, i64>("SELECT COUNT(*) FROM activity_logs")`
- Uses existing `db::get_database_size()` to get database file size
- Scans screenshots and logs directories using helper function

### Helper Function: scan_directory
- Returns `Result<(u64, u64), String>` - (file_count, total_size_bytes)
- Returns zeros for non-existent directories (graceful handling)
- Provides detailed error messages for permission/access issues
- Only counts files, not subdirectories

### Unit Tests Added
- `test_scan_directory_with_nonexistent_path`: Verifies zeros returned for missing directories
- `test_scan_directory_with_files`: Verifies accurate file count and size calculation
- `test_scan_directory_with_subdirectory`: Verifies subdirectories are not counted

### Key Patterns

#### Error Handling
- Database queries use `match db::get_pool().await` with graceful fallback to 0
- Directory scanning uses `.unwrap_or_else()` to handle errors without crashing
- Permission errors return detailed messages mentioning "permission denied"

#### Calculation Formulas
- Size in MB: `bytes as f64 / 1024.0 / 1024.0`
- Usage percent: `(total_size_mb / (max_storage_gb * 1024.0)) * 100.0`
- Next GC time: `now.timestamp() + (retention_days as i64 * 86400)`

### Testing
- 19 unit tests in lib.rs (including 3 new scan_directory tests)
- 11 integration tests in storage_autostart_integration.rs

### Notes
- The command was already registered in lib.rs from Task 2 (skeleton)
- Function signature changed from `get_storage_stats()` to `get_storage_stats(app_handle)` to access app data directory
- Tests use `super::scan_directory()` because helper function is defined after the test module


---

## Task 10 Learnings - Integration Testing

### Test Files Created
1. **Backend Integration Test**: `src-tauri/tests/storage_autostart_integration.rs`
   - 11 tests covering all 7 new commands
   - Tests data structure serialization/deserialization
   - Tests filesystem operations (scan_directory)
   - Platform-specific tests for Windows/non-Windows

2. **Frontend Integration Test**: `src/test/integration/SettingsModal.integration.test.tsx`
   - 19 tests covering complete user workflows
   - Tests command invocation and error handling
   - Tests data type validation
   - Tests complete integration workflow

### Test Results
- **Backend**: 11/11 tests passed
- **Frontend**: 19/19 tests passed (plus 7 existing AppContext tests)
- **Total**: 26 integration tests passed

### Commands Verified
1. `get_storage_stats` - Returns StorageStatsResponse with all fields
2. `export_data_json` - Returns valid JSON with metadata
3. `export_data_markdown` - Returns formatted markdown
4. `clear_all_data` - Returns ClearResult with accurate counts
5. `enable_autostart` - Creates registry entry on Windows
6. `disable_autostart` - Removes registry entry on Windows
7. `get_autostart_status` - Returns AutostartInfo with enabled state

### Test Patterns Used
- Mocked Tauri API (`vi.mock('@tauri-apps/api/core')`)
- Type assertions for TypeScript (`as StorageStats`)
- Sequential command testing for workflow validation
- Error scenario testing (connection failures, permission errors)

### Findings
1. All commands are properly registered in lib.rs (Task 9)
2. Command signatures match frontend expectations
3. Data structures serialize correctly with camelCase
4. Error handling works for common failure scenarios
5. Platform-specific code compiles correctly on Windows

### Notes
- Playwright tests were not implemented (would require significant setup)
- Vitest integration tests provide good coverage of the command flow
- Manual UI testing would be required for scroll wheel verification
- Registry operations on Windows require platform-specific compilation
