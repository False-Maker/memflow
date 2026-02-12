# Terminal Capture - Learnings

## Patterns and Conventions

### Windows API Usage in Rust (windows-rs crate v0.58)

1. **Module Organization**
   - `Win32::Foundation` - Core types (BOOL, HWND, LPARAM, CloseHandle)
   - `Win32::UI::WindowsAndMessaging` - Window APIs (EnumWindows, GetWindowText*, GetClassNameW)
   - `Win32::System::Threading` - Process APIs (OpenProcess, QueryFullProcessImageNameW, PROCESS_*)
   - Note: Process-related APIs moved from `ProcessStatus` to `Threading` in windows 0.58

2. **EnumWindows Callback Pattern**
   ```rust
   use std::cell::RefCell;

   let collection = RefCell::new(Vec::new());

   unsafe extern "system" fn callback(hwnd: HWND, lparam: LPARAM) -> BOOL {
       let collection = &*(lparam.0 as *const RefCell<Vec<Item>>);
       // Process window
       BOOL(1) // Continue enumeration
   }

   EnumWindows(
       Some(callback),
       LPARAM(&collection as *const _ as isize),
   )?;
   ```

3. **Process Name Extraction**
   ```rust
   let handle = OpenProcess(PROCESS_QUERY_INFORMATION | PROCESS_VM_READ, false, pid)?;
   let mut name = [0u16; 512];
   let mut size = name.len() as u32;
   QueryFullProcessImageNameW(
       handle,
       PROCESS_NAME_FORMAT(0), // Important: Wrap in enum, not just 0
       windows::core::PWSTR(name.as_mut_ptr()),
       &mut size,
   )?;
   ```

4. **Window Metadata Retrieval**
   ```rust
   // Get class name
   let mut class_name = [0u16; 256];
   let len = GetClassNameW(hwnd, &mut class_name);

   // Get process ID
   let mut pid = 0u32;
   GetWindowThreadProcessId(hwnd, Some(&mut pid));

   // Get window title
   let len = GetWindowTextLengthW(hwnd);
   let mut buffer: Vec<u16> = vec![0; (len + 1) as usize];
   let copied = GetWindowTextW(hwnd, &mut buffer);
   ```

### Terminal Detection Logic

1. **Window Class Names** (primary filter)
   - `ConsoleWindowClass` - cmd.exe
   - `Cascadia.Terminal` - Windows Terminal
   - `CASCADIA_HOSTING_WINDOW_CLASS` - Windows Terminal (alternate)

2. **Process Names** (secondary filter)
   - `cmd.exe` - Command Prompt
   - `powershell.exe` - Windows PowerShell
   - `pwsh.exe` - PowerShell Core
   - `wt.exe` - Windows Terminal launcher
   - `WindowsTerminal.exe` - Windows Terminal main process
   - `conhost.exe` - Console Host (often paired with other terminals)

3. **Validation Steps**
   - Check window class name contains terminal class substring
   - Check process name matches terminal process (case-insensitive)
   - Ensure process name is not empty (OpenProcess may fail)
   - Get and validate window title is non-empty
   - Only include windows that pass all filters

### Cargo.toml Configuration

```toml
[target.'cfg(windows)'.dependencies]
windows = { version = "0.58", features = [
    "Win32_Foundation",
    "Win32_UI_WindowsAndMessaging",
    "Win32_System_Threading",
    "Win32_System_ProcessStatus",
] }
```

### Error Handling Patterns

1. **Windows API Result Conversion**
   ```rust
   EnumWindows(...).map_err(|e| TerminalError::CaptureFailed(format!("...: {:?}", e)))?;
   ```

2. **Safe Unwrap for Process Name**
   ```rust
   let process_name = get_process_name(pid).unwrap_or_else(|_| String::new());
   ```

3. **Filter Before Collection**
   ```rust
   if is_terminal && !process_name.is_empty() && !window_title.is_empty() {
       // Only add valid entries
   }
   ```

### Testing Strategy

1. **Test Multiple Scenarios**
   - Test when terminals are found (validate structure)
   - Test when no terminals found (no panic)
   - Test metadata validation (name, pid, title)

2. **Use Flexible Assertions**
   ```rust
   if let Ok(terminals) = result {
       if !terminals.is_empty() {
           // Validate individual terminals
       }
   }
   ```

## Successful Approaches

1. **RefCell for Callback Data Collection**
   - Works well with EnumWindows synchronous callback
   - Avoids Arc/Mutex complexity for single-threaded callback

2. **String Matching Strategy**
   - Use `contains()` for class names (may be longer)
   - Use `eq_ignore_ascii_case()` for process names (exact match)

3. **Empty String Filtering**
   - Skip terminals with empty names (OpenProcess failed)
   - Skip terminals with empty titles (invisible/minimized)

4. **Platform-Specific Tests**
   - Use `#[cfg(target_os = "windows")]` for Windows-specific tests
   - Handle both success and error cases gracefully

## Key Gotchas

1. **Windows Crate Version Changes**
   - v0.58 moved process APIs from `ProcessStatus` to `Threading`
   - `QueryFullProcessImageNameW` requires `PROCESS_NAME_FORMAT` enum, not raw `0`

2. **Unsafe Callback Context**
   - Must use `unsafe extern "system"` for callbacks
   - Passing pointers via LPARAM requires careful type casting

3. **Process Access Failures**
   - Some processes may not be accessible (permissions)
   - Always provide fallback (empty string) for process name

4. **Multiple Window Instances**
    - Same process may have multiple windows (e.g., conhost.exe)
    - Filter by window title to avoid duplicates

## Background Cache System Implementation (Task 5)

### Architecture

1. **TerminalCache Structure**
   - `content: Arc<RwLock<String>>` - Thread-safe cached terminal content
   - `last_update: Arc<RwLock<Instant>>` - Timestamp of last cache update
   - `max_lines: usize` - Maximum lines to cache (500)

2. **Cache Lifecycle**
   - Created as `Lazy<TerminalCache>` global singleton
   - Initial state: empty content, stale timestamp (10 seconds ago)
   - Updated on-demand via `capture_terminal_output()` when enabled

3. **Cache Freshness Check**
   - `CACHE_FRESHNESS_WINDOW: Duration = 1 second`
   - Cache is considered "fresh" for 1 second after update
   - Subsequent calls within this window return cached data

4. **Line Limiting**
   - Cache stores max 500 lines globally
   - User requests for fewer lines are applied on-demand
   - Truncation takes LAST N lines (most recent)

### Environment Variable Configuration

```rust
MEMFLOW_TERMINAL_CACHE_POLLING=1  // Enable cache (values: 1, true, yes)
```

### Send-Safety Constraint (Important Gotcha)

**Problem**: Windows API types (HWND, etc.) are not `Send`, preventing `tokio::spawn`.

**Impact**: Background polling via `tokio::spawn` is disabled.

**Workaround**: Cache still functional but updates on-demand instead of via background task.

**Future Solution Options**:
1. Wrap capture calls in `spawn_blocking` (requires tokio runtime complexity)
2. Refactor to use `Arc<Mutex<Option<HWND>>>` pattern
3. Use cross-platform Send-safe abstractions

### Testing Strategy

1. **Cache Refresh Interval Test**
   - Verify 1-second freshness window
   - Allow 100ms tolerance for timing variations
   - Test: fresh at 900ms, stale at 1100ms

2. **Cache Max Lines Test**
   - Verify 600-line input is truncated to 500
   - Verify truncation takes LAST 500 lines
   - Test: Line 101 to Line 600 cached

3. **Cache Freshness Test**
   - Multiple calls within freshness window return cached data
   - Cache expires after 1 second
   - Test: calls at 0ms, 500ms (fresh), 1100ms (stale)

4. **Limit Lines Function Test**
   - Under limit: preserve all lines
   - At limit: preserve all lines
   - Over limit: truncate to last N lines
   - Limit of 0: return all content (edge case)

5. **Background Polling Enabled Test**
   - Parse environment variable (case-insensitive)
   - True values: "1", "true", "TRUE", "yes"
   - False values: "0", "false", "no"
   - Undefined: disabled by default

### Public API (Unchanged)

```rust
pub async fn capture_terminal_output(lines: usize) -> Result<String, TerminalError>
pub async fn start_background_refresh()  // New function
```

### Implementation Details

1. **Cache Update Flow**
   ```rust
   if is_background_polling_enabled() && CACHE.is_fresh(1s) {
       return CACHE.get_cached();  // Cache hit
   }
   // Cache miss - perform capture
   let result = capture_terminal_output_windows(lines).await;
   CACHE.update(result.clone());
   return result;
   ```

2. **Line Limiting Logic**
   ```rust
   fn limit_lines(content: &str, max_lines: usize) -> String {
       if max_lines == 0 || content.lines().count() <= max_lines {
           return content.to_string();
       }
       let lines: Vec<&str> = content.lines().collect();
       lines[lines.len() - max_lines..].join("\n")  // Last N lines
   }
   ```

3. **Environment Variable Parsing**
   ```rust
   std::env::var("MEMFLOW_TERMINAL_CACHE_POLLING")
       .map(|val| val.eq_ignore_ascii_case("1") || ...)
       .unwrap_or(false)  // Default: disabled
   ```

## Decisions Made

1. **Background Polling Disabled**: Due to Windows API Send-safety constraints. Cache still works on-demand.
2. **500 Line Limit**: Balances memory usage with useful terminal history.
3. **1 Second Freshness Window**: Fast enough for interactive use, slow enough to avoid excessive captures.
4. **Opt-In by Default**: Environment variable required to enable caching (respect user control).
5. **Public API Unchanged**: `capture_terminal_output()` signature preserved for backward compatibility.

## Issues and Limitations

1. **Send-Safety Constraint**: Cannot spawn background tasks with Windows API types. Requires architectural refactoring to fix.
2. **Manual Cache Trigger**: Without background polling, cache only updates on-demand. Users must call `capture_terminal_output()` to refresh.
3. **Environment Variable Only**: No runtime API to enable/disable caching programmatically.

## Future Improvements

1. **Fix Send-Safety**: Refactor to use `Arc<Mutex<Option<HWND>>>` or similar pattern to enable background polling.
2. **Runtime Configuration**: Add API to enable/disable cache without environment variable.
3. **Configurable Line Limit**: Allow users to adjust cache size via configuration.
4. **Cache Statistics**: Add metrics for cache hit rate, refresh frequency, etc.

## Documentation and Integration Tests (Task 6)

### Rust Documentation Patterns

1. **Module-Level Documentation (`//!`)**
   - Explain architecture and design decisions
   - Document platform support clearly
   - Include usage examples at module level
   - List important warnings and gotchas
   - Reference related functions and types

2. **Function-Level Documentation (`///`)**
   - Describe what the function does (first line is summary)
   - Document all parameters with `# Arguments`
   - Document return types with `# Returns`
   - List errors in `# Errors` section with variants and when they occur
   - Include multiple usage examples in `# Examples`
   - Document performance considerations where relevant
   - Note platform-specific behavior

3. **Documentation Examples**
   - Use `#no_run` for examples that require external resources (like actual terminals)
   - Show both simple and advanced usage patterns
   - Demonstrate error handling patterns
   - Include filtering and selection examples for collection-returning functions
   - Keep examples concise but complete

4. **Doc Test Best Practices**
   - Mark examples that shouldn't compile with `#no_run` when they use async or external dependencies
   - Verify doc examples compile with `cargo test --doc`
   - Ensure doc examples follow Rust coding standards
   - Use descriptive comments in examples

### Integration Testing Strategy

1. **Full Flow Tests**
   - Test complete workflows (detect → capture)
   - Validate intermediate states
   - Handle both success and graceful failure scenarios
   - Don't panic if terminals aren't available in test environment

2. **Mixed Strategy Tests**
   - Verify both primary and fallback code paths
   - Test with different line limits
   - Ensure error handling doesn't panic
   - Validate parameter handling (0 limit, large limits)

3. **Cache Integration Tests**
   - Test cache lifecycle (empty → populated → read → stale)
   - Verify cache freshness logic (within/without 1s window)
   - Test line limiting (truncation behavior)
   - Validate cached content accuracy

4. **Error Handling Tests**
   - Test all error variants are handled gracefully
   - Verify no panics occur on failures
   - Test edge cases (no terminals, permission denied, etc.)
   - Validate error messages are informative

5. **Test Design Principles**
   - Tests should never panic due to missing resources
   - Acceptable outcomes: success or known errors (NotFound, PermissionDenied, CaptureFailed)
   - Use conditional assertions (if let Ok(...) rather than unwrap())
   - Print informative messages for expected failures in test environment

### Environment Variable Testing

1. **Isolation Issues**
   - Tests that modify environment variables should run sequentially
   - Use `-- --test-threads=1` to avoid conflicts between tests
   - Clean up environment variables after tests

2. **Test Thread Conflicts**
   - Parallel tests that set env vars can interfere with each other
   - Solution: run with single thread or isolate env var modifications
   - The `test_is_background_polling_enabled` test is particularly susceptible

### Testing Without External Dependencies

1. **Terminal Availability**
   - Tests should pass even if no terminals are present
   - Design tests to be idempotent: work whether terminals exist or not
   - Use conditional logic: if terminals found → test capture; else → skip gracefully

2. **Permission Variations**
   - Test environments may have restricted permissions
   - Accept `PermissionDenied` and `CaptureFailed` errors as valid test outcomes
   - Don't hardcode success expectations for resource-dependent operations

### Documentation Quality Checklist

1. **Module Docs**
   - ✓ Architecture explanation
   - ✓ Usage examples
   - ✓ Platform support matrix
   - ✓ Important warnings (Microsoft deprecation, Send-safety)
   - ✓ Configuration options

2. **Function Docs**
   - ✓ Clear summary
   - ✓ All parameters documented
   - ✓ Return types documented
   - ✓ All error variants listed
   - ✓ Multiple usage examples
   - ✓ Performance considerations
   - ✓ Platform-specific notes

3. **Doc Tests**
   - ✓ All examples compile (cargo test --doc passes)
   - ✓ Examples marked with #no_run where appropriate
   - ✓ Examples demonstrate real-world usage
   - ✓ Examples are complete and executable

### Verification Commands

```bash
# Run all unit and integration tests
cargo test -p memflow-core --lib

# Run specific module tests
cargo test -p memflow-core --lib terminal

# Run tests with single thread (for env var tests)
cargo test -p memflow-core --lib terminal -- --test-threads=1

# Build documentation
cargo doc --no-deps -p memflow-core

# Test doc examples
cargo test --doc -p memflow-core

# Combined verification
cargo test -p memflow-core --lib && cargo doc --no-deps -p memflow-core && cargo test --doc -p memflow-core
```

### Key Learnings

1. **Documentation Completeness Matters**
   - Users rely on docs for API understanding
   - Include all error cases, not just success paths
   - Provide multiple examples for different use cases
   - Document platform-specific behavior explicitly

2. **Test Design for CI/CD**
   - Tests must pass in automated environments without manual intervention
   - Handle all error cases gracefully, never panic
   - Design tests to be deterministic (avoid flakiness)
   - Use conditional assertions for resource-dependent tests

3. **Environment Variable Testing**
   - Run tests sequentially when modifying env vars
   - Always clean up env vars after tests
   - Be aware that env var state persists between parallel tests

4. **Doc Tests as Verification**
   - Doc examples are executable tests
   - Use `#no_run` for examples requiring external resources
   - Verify doc examples compile and run with `cargo test --doc`
   - Doc tests ensure examples stay up-to-date with API changes

