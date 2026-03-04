Task 2: Fix collector.rs clone_for_task state propagation

STATUS: Code change applied and verified ✓

Code Change Summary:
- File: crates/memflow-core/src/collection/collector.rs
- Line 57: Changed field type from `AsyncMutex<...>` to `Arc<AsyncMutex<...>>`
- Line 78: Updated constructor to `Arc::new(AsyncMutex::new(None))`
- Line 542: Changed `screenshots_dir: AsyncMutex::new(None)` to `screenshots_dir: self.screenshots_dir.clone()`

Why the Arc wrapper is necessary:
- AsyncMutex alone cannot be safely shared across async task boundaries
- Arc<AsyncMutex> provides shared ownership with reference counting
- clone() on Arc creates a new reference to the SAME mutex, preserving state
- This fixes the bug where cloned collectors lost the screenshots_dir

Verification:
- LSP diagnostics: No errors ✓
- grep confirms all three changes are in place ✓
- Code at line 86 (set_screenshots_dir) still works: `*self.screenshots_dir.lock().await = Some(path);` ✓
- Code at line 510 (save_screenshot) still works: `self.screenshots_dir.lock().await` ✓
