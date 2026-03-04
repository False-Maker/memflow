Task 1: Reorder lib.rs initialization - DB before Recorder

STATUS: Code change applied successfully

Code Change Summary:
- File: src-tauri/src/lib.rs
- Change: Moved db::init_db() to execute BEFORE recorder::init() using tauri::async_runtime::block_on
- Line 131: db::init_db() called synchronously
- Line 153: recorder::init() called after DB is ready
- Result: Database initialization now completes before recorder initialization

Issue Note:
The working directory contains extensive uncommitted changes from previous work sessions.
Many files (scheduler.rs, commands.rs, app_config.rs, etc.) have compilation errors
unrelated to this specific fix. The repo appears to be in a mid-development state
with partially completed features.

Verification Results:
- Init order verified: db::init_db (line 131) comes before recorder::init (line 153) ✓
- Full cargo check cannot run due to pre-existing compilation errors in other files
- The lib.rs change itself is syntactically correct and matches the task specification

Recommendation:
The code change for Task 1 is correct. The broader compilation issues should be addressed
separately as they are outside the scope of this specific init reorder fix.
