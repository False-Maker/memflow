//! MCP Test Suite
//!
//! This module serves as the entry point for all MCP-related tests.

mod mocks;
mod protocol_test;
mod tauri_concurrency_test;

// Re-export mock types for use in other tests
pub use mocks::mock_context::{MockContext, SystemInfo, WindowInfo};
pub use mocks::mock_db::{ActivityRecord, MemoryRecord, MockDb};
