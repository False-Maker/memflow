# Task 4: Handler Integration Tests - Learnings

## Date
2025-02-16

## Context
Task 4 from `doc/MCP_REMAINING_TASKS.md` required implementing handler-level integration tests for the memflow-mcp crate. The existing tests (`mcp_tool_test.rs` and `schema_validation_test.rs`) only validated JSON Schema structure but did not execute the actual handler logic.

## Key Learnings

### 1. Integration Test Location in Rust Binaries

**Problem**: The handler functions (`call_get_system_environment`, `call_get_terminal_output`, etc.) are private functions in `src/main.rs` (a binary crate). Integration tests in `tests/` directory cannot access private items from the binary.

**Solution**: Place integration tests directly in `main.rs` using a `#[cfg(test)]` module. This allows the tests to access private functions in the same file.

```rust
#[cfg(test)]
mod handler_integration_tests {
    use super::*;

    #[tokio::test]
    async fn test_something() {
        // Can call private functions from main.rs
        call_get_system_environment(false, false, false).await;
    }
}
```

**Key Insight**:
- `#[cfg(test)]` in lib.rs does NOT apply to integration tests in `tests/` directory
- Integration tests are compiled as separate crates and can only access public API items
- For testing binary-private functions, inline tests in the binary file are the standard approach

### 2. Running Tests in a Binary Crate

**Command**: `cargo test -p memflow-mcp --bin memflow-mcp`

This specifically runs tests in the binary (main.rs), not the library.

### 3. Error Handling for Database Uninitialized State

**Learning**: The database may not be initialized in test environments. The error message can be in Chinese ("数据库未初始化") due to the underlying codebase using Chinese error messages.

**Solution**: Test for both English and Chinese error patterns:

```rust
let is_db_error = error_msg.to_lowercase().contains("not initialized")
    || error_msg.contains("未初始化")  // Chinese: "not initialized"
    || error_msg.contains("数据库");     // Chinese: "database"
```

### 4. TerminalError Enum Variant Types

**Learning**: `TerminalError::CaptureFailed` is a tuple variant with a String payload, not a unit variant.

```rust
// Correct
Err(memflow_core::terminal::TerminalError::CaptureFailed(_)) => { }

// Incorrect - will cause compile error
Err(memflow_core::terminal::TerminalError::CaptureFailed) => { }
```

### 5. Test Design for Environment-Dependent Features

**Approach**: Tests that depend on external environment (terminal, database, dev tools) should accept multiple outcomes:

```rust
match result {
    Ok(output) => {
        // Success path - validate output format
    }
    Err(ExpectedError::Variant1) => {
        // Acceptable failure - environment not set up
    }
    Err(ExpectedError::Variant2) => {
        // Another acceptable failure
    }
    Err(other) => {
        // Unexpected error - fail the test
    }
}
```

### 6. Test Coverage Goals

The 5 required tests plus 3 additional tests cover:
1. **System Environment** (3 tests): Basic info, dev tools, processes
2. **Terminal Output** (1 test): Graceful handling of no terminal
3. **Search Memory** (2 tests): Empty query validation
4. **Recent Activities** (1 test): Database uninitialized handling

## Test Results

All 8 tests pass:
- `test_get_system_environment_returns_os_info` ✓
- `test_get_system_environment_with_dev_tools` ✓
- `test_get_terminal_output_handles_no_terminal` ✓
- `test_search_memory_empty_query` ✓
- `test_get_recent_activities_default_params` ✓
- `test_get_system_environment_includes_processes` ✓
- `test_get_system_environment_all_features` ✓
- `test_search_memory_none_query` ✓

## Commands Used

```bash
# Run all memflow-mcp tests
cargo test -p memflow-mcp

# Run only handler integration tests
cargo test -p memflow-mcp --bin memflow-mcp handler_integration_tests
```

## Files Modified

1. `crates/memflow-mcp/src/main.rs` - Added `handler_integration_tests` module
2. `crates/memflow-mcp/tests/mod.rs` - No changes needed (originally added handler_integration_test, but removed when switching to inline tests)

## Files Not Used

- `crates/memflow-mcp/tests/handler_integration_test.rs` - Created but then deleted; tests moved inline to main.rs
