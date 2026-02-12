//! MCP Protocol Tests
//!
//! This module contains tests for the MCP protocol layer including:
//! - JSON-RPC serialization/deserialization
//! - Tool name parsing and alias handling
//! - Error code validation

use memflow_mcp::protocol::*;
use serde_json::json;

#[test]
fn test_json_rpc_request_deserialization() {
    let json = json!({
        "jsonrpc": "2.0",
        "method": "tools/list",
        "id": 1
    });

    let req: JsonRpcRequest = serde_json::from_value(json).unwrap();
    assert_eq!(req.jsonrpc, "2.0");
    assert_eq!(req.method, "tools/list");
}

#[test]
fn test_json_rpc_response_success() {
    let result = json!({"tools": []});
    let response = JsonRpcResponse::success(Some(json!(1)), result);

    assert!(response.error.is_none());
    assert!(response.result.is_some());
}

#[test]
fn test_json_rpc_response_error() {
    let response =
        JsonRpcResponse::error(Some(json!(1)), ERROR_METHOD_NOT_FOUND, "Method not found");

    assert!(response.result.is_none());
    assert!(response.error.is_some());
    let error = response.error.unwrap();
    assert_eq!(error.code, ERROR_METHOD_NOT_FOUND);
}

#[test]
fn test_tool_name_from_str_canonical() {
    assert_eq!(
        ToolName::from_str("search_memory"),
        Some(ToolName::SearchMemory)
    );
    assert_eq!(
        ToolName::from_str("get_recent_activity"),
        Some(ToolName::GetRecentActivity)
    );
    assert_eq!(
        ToolName::from_str("get_active_window_context"),
        Some(ToolName::GetActiveWindowContext)
    );
    assert_eq!(
        ToolName::from_str("get_terminal_output"),
        Some(ToolName::GetTerminalOutput)
    );
    assert_eq!(
        ToolName::from_str("get_system_environment"),
        Some(ToolName::GetSystemEnvironment)
    );
    assert_eq!(
        ToolName::from_str("get_related_context"),
        Some(ToolName::GetRelatedContext)
    );
}

#[test]
fn test_tool_name_from_str_aliases() {
    assert_eq!(
        ToolName::from_str("search_visual_memory"),
        Some(ToolName::SearchMemory)
    );
    assert_eq!(
        ToolName::from_str("get_recent_activities"),
        Some(ToolName::GetRecentActivity)
    );
}

#[test]
fn test_tool_name_from_str_invalid() {
    assert_eq!(ToolName::from_str("invalid_tool"), None);
    assert_eq!(ToolName::from_str(""), None);
}

#[test]
fn test_tool_name_as_str() {
    assert_eq!(ToolName::SearchMemory.as_str(), "search_memory");
    assert_eq!(ToolName::GetRecentActivity.as_str(), "get_recent_activity");
    assert_eq!(
        ToolName::GetActiveWindowContext.as_str(),
        "get_active_window_context"
    );
}

#[test]
fn test_tool_name_is_alias() {
    assert!(ToolName::is_alias("search_visual_memory"));
    assert!(ToolName::is_alias("get_recent_activities"));
    assert!(!ToolName::is_alias("search_memory"));
    assert!(!ToolName::is_alias("get_recent_activity"));
}

#[test]
fn test_tool_constants() {
    assert_eq!(TOOL_SEARCH_MEMORY, "search_memory");
    assert_eq!(TOOL_SEARCH_MEMORY_ALIAS, "search_visual_memory");
    assert_eq!(TOOL_GET_RECENT_ACTIVITY, "get_recent_activity");
    assert_eq!(TOOL_GET_RECENT_ACTIVITY_ALIAS, "get_recent_activities");
}

#[test]
fn test_error_codes() {
    assert_eq!(ERROR_METHOD_NOT_FOUND, -32601);
    assert_eq!(ERROR_INVALID_PARAMS, -32602);
    assert_eq!(ERROR_SERVER_ERROR, -32000);
    assert_eq!(ERROR_TERMINAL_NOT_FOUND, -32004);
}

#[test]
fn test_error_message() {
    assert_eq!(error_message(ERROR_METHOD_NOT_FOUND), "Method not found");
    assert_eq!(error_message(ERROR_INVALID_PARAMS), "Invalid params");
    assert_eq!(error_message(ERROR_SERVER_ERROR), "Server error");
    assert_eq!(
        error_message(ERROR_TERMINAL_NOT_FOUND),
        "Terminal not found"
    );
    assert_eq!(error_message(99999), "Unknown error");
}

#[test]
fn test_tool_creation() {
    let tool = Tool::new("test_tool", "A test tool", json!({"type": "object"}));

    assert_eq!(tool.name, "test_tool");
    assert_eq!(tool.description, "A test tool");
}

#[test]
fn test_tool_call_result_text() {
    let result = ToolCallResult::text("Hello, world!");

    assert_eq!(result.content.len(), 1);
    assert_eq!(result.content[0].type_, "text");
    assert_eq!(result.content[0].text, "Hello, world!");
}

#[test]
fn test_alias_routing_parity() {
    // Test that canonical and alias names resolve to the same enum variant
    let canonical_variant = ToolName::from_str("search_memory");
    let alias_variant = ToolName::from_str("search_visual_memory");

    assert_eq!(canonical_variant, alias_variant);
    assert_eq!(canonical_variant, Some(ToolName::SearchMemory));
    assert_eq!(alias_variant, Some(ToolName::SearchMemory));

    let canonical_variant2 = ToolName::from_str("get_recent_activity");
    let alias_variant2 = ToolName::from_str("get_recent_activities");

    assert_eq!(canonical_variant2, alias_variant2);
    assert_eq!(canonical_variant2, Some(ToolName::GetRecentActivity));
    assert_eq!(alias_variant2, Some(ToolName::GetRecentActivity));
}

#[test]
fn test_alias_returns_canonical_name() {
    // Test that aliases return canonical names via as_str()
    let alias_variant = ToolName::from_str("search_visual_memory").unwrap();
    let alias_variant2 = ToolName::from_str("get_recent_activities").unwrap();

    // Both should return their canonical string representation
    assert_eq!(alias_variant.as_str(), "search_memory");
    assert_eq!(alias_variant2.as_str(), "get_recent_activity");

    // Verify they match the canonical variants
    let canonical_variant = ToolName::SearchMemory;
    let canonical_variant2 = ToolName::GetRecentActivity;

    assert_eq!(alias_variant.as_str(), canonical_variant.as_str());
    assert_eq!(alias_variant2.as_str(), canonical_variant2.as_str());
}
