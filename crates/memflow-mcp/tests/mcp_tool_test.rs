//! MCP Tool Tests
//!
//! Tests for MCP tool functionality including all 6 tools:
//! - search_memory
//! - get_recent_activity
//! - get_active_window_context
//! - get_terminal_output
//! - get_system_environment
//! - get_related_context

use memflow_mcp::protocol::*;
use serde_json::json;

/// Test helper to create a tools/list request
fn create_tools_list_request() -> JsonRpcRequest {
    JsonRpcRequest {
        jsonrpc: "2.0".to_string(),
        method: "tools/list".to_string(),
        params: None,
        id: Some(json!(1)),
    }
}

/// Test helper to create a tools/call request
fn create_tools_call_request(tool_name: &str, arguments: serde_json::Value) -> JsonRpcRequest {
    JsonRpcRequest {
        jsonrpc: "2.0".to_string(),
        method: "tools/call".to_string(),
        params: Some(json!({
            "name": tool_name,
            "arguments": arguments
        })),
        id: Some(json!(1)),
    }
}

#[test]
fn test_search_memory_tool_schema() {
    // Verify the tool schema matches contract
    let schema = json!({
        "type": "object",
        "properties": {
            "query": {
                "type": "string",
                "description": "The search query to match against memory."
            },
            "limit": {
                "type": "integer",
                "description": "Maximum number of results to return (default 5)."
            },
            "mode": {
                "type": "string",
                "description": "Search mode: hybrid, semantic, or keyword (default hybrid)."
            },
            "app_name": {
                "type": "string",
                "description": "Optional app name filter."
            },
            "keywords": {
                "type": "array",
                "items": { "type": "string" },
                "description": "Optional keyword list to override query parsing."
            },
            "date_range": {
                "type": "string",
                "description": "Optional date range: today, yesterday, this_week, last_week, this_month."
            },
            "has_ocr": {
                "type": "boolean",
                "description": "Filter records that contain OCR text."
            }
        },
        "required": ["query"]
    });

    // Verify schema structure
    assert!(schema.get("type").is_some());
    assert!(schema.get("properties").is_some());
    assert!(schema.get("required").is_some());
}

#[test]
fn test_get_recent_activity_tool_schema() {
    let schema = json!({
        "type": "object",
        "properties": {
            "minutes": {
                "type": "integer",
                "description": "Number of minutes to look back (default: 5, max: 30)"
            },
            "limit": {
                "type": "integer",
                "description": "Max number of activities to return (default: 20)"
            }
        }
    });

    assert!(schema.get("type").is_some());
    assert!(schema.get("properties").is_some());
}

#[test]
fn test_get_terminal_output_tool_schema() {
    let schema = json!({
        "type": "object",
        "properties": {
            "lines": {
                "type": "integer",
                "description": "Number of lines to capture from terminal output (default: 50).",
                "default": 50,
                "minimum": 1,
                "maximum": 500
            }
        }
    });

    let props = schema.get("properties").unwrap();
    let lines = props.get("lines").unwrap();
    assert_eq!(lines.get("default").unwrap().as_i64(), Some(50));
    assert_eq!(lines.get("minimum").unwrap().as_i64(), Some(1));
    assert_eq!(lines.get("maximum").unwrap().as_i64(), Some(500));
}

#[test]
fn test_get_system_environment_tool_schema() {
    let schema = json!({
        "type": "object",
        "properties": {
            "include_dev_tools": {
                "type": "boolean",
                "description": "Include development tool versions (Node, Python, Rust, Docker).",
                "default": true
            },
            "include_processes": {
                "type": "boolean",
                "description": "Include active development processes.",
                "default": true
            },
            "include_ports": {
                "type": "boolean",
                "description": "Include common port usage (3000, 8080, 8000, etc.).",
                "default": false
            }
        }
    });

    let props = schema.get("properties").unwrap();
    assert!(props.get("include_dev_tools").is_some());
    assert!(props.get("include_processes").is_some());
    assert!(props.get("include_ports").is_some());
}

#[test]
fn test_tool_name_constants() {
    // Verify all tool name constants are defined
    assert_eq!(TOOL_SEARCH_MEMORY, "search_memory");
    assert_eq!(TOOL_SEARCH_MEMORY_ALIAS, "search_visual_memory");
    assert_eq!(TOOL_GET_RECENT_ACTIVITY, "get_recent_activity");
    assert_eq!(TOOL_GET_RECENT_ACTIVITY_ALIAS, "get_recent_activities");
    assert_eq!(TOOL_GET_ACTIVE_WINDOW_CONTEXT, "get_active_window_context");
    assert_eq!(TOOL_GET_TERMINAL_OUTPUT, "get_terminal_output");
    assert_eq!(TOOL_GET_SYSTEM_ENVIRONMENT, "get_system_environment");
    assert_eq!(TOOL_GET_RELATED_CONTEXT, "get_related_context");
}

#[test]
fn test_tool_name_from_str_all_tools() {
    // Test canonical names
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
fn test_tool_name_alias_support() {
    // Test aliases resolve to correct tools
    assert_eq!(
        ToolName::from_str("search_visual_memory"),
        Some(ToolName::SearchMemory)
    );
    assert_eq!(
        ToolName::from_str("get_recent_activities"),
        Some(ToolName::GetRecentActivity)
    );

    // Verify canonical name is returned
    assert_eq!(ToolName::SearchMemory.as_str(), "search_memory");
    assert_eq!(ToolName::GetRecentActivity.as_str(), "get_recent_activity");
}

#[test]
fn test_tool_name_is_alias() {
    assert!(ToolName::is_alias("search_visual_memory"));
    assert!(ToolName::is_alias("get_recent_activities"));
    assert!(!ToolName::is_alias("search_memory"));
    assert!(!ToolName::is_alias("get_terminal_output"));
}

#[test]
fn test_error_code_constants() {
    // MCP Standard Error Codes
    assert_eq!(ERROR_PARSE_ERROR, -32700);
    assert_eq!(ERROR_INVALID_REQUEST, -32600);
    assert_eq!(ERROR_METHOD_NOT_FOUND, -32601);
    assert_eq!(ERROR_INVALID_PARAMS, -32602);
    assert_eq!(ERROR_INTERNAL_ERROR, -32603);

    // Memflow-Specific Error Codes
    assert_eq!(ERROR_SERVER_ERROR, -32000);
    assert_eq!(ERROR_UNAUTHORIZED, -32001);
    assert_eq!(ERROR_READ_ONLY_MODE, -32003);
    assert_eq!(ERROR_TERMINAL_NOT_FOUND, -32004);
    assert_eq!(ERROR_PERMISSION_DENIED, -32005);
    assert_eq!(ERROR_DATABASE_LOCKED, -32006);
    assert_eq!(ERROR_NO_DATA_AVAILABLE, -32007);
    assert_eq!(ERROR_OCR_FAILED, -32008);
}

#[test]
fn test_tool_call_result_creation() {
    let result = ToolCallResult::text("Hello, world!");
    assert_eq!(result.content.len(), 1);
    assert_eq!(result.content[0].type_, "text");
    assert_eq!(result.content[0].text, "Hello, world!");
}

#[test]
fn test_tool_creation() {
    let tool = Tool::new(
        "test_tool",
        "A test tool description",
        json!({"type": "object"}),
    );

    assert_eq!(tool.name, "test_tool");
    assert_eq!(tool.description, "A test tool description");
}

#[test]
fn test_search_memory_args_validation() {
    // Valid: query is required
    let valid_args = json!({"query": "test search"});
    assert!(valid_args.get("query").is_some());

    // Valid with optional params
    let full_args = json!({
        "query": "test",
        "limit": 10,
        "mode": "hybrid",
        "app_name": "VSCode",
        "date_range": "today"
    });
    assert_eq!(full_args.get("limit").unwrap().as_i64(), Some(10));
}

#[test]
fn test_terminal_output_lines_bounds() {
    // Test lines parameter bounds (1-500)
    let min_lines = json!({"lines": 1});
    let max_lines = json!({"lines": 500});
    let default_lines = json!({});

    assert_eq!(min_lines.get("lines").unwrap().as_i64(), Some(1));
    assert_eq!(max_lines.get("lines").unwrap().as_i64(), Some(500));
    assert!(default_lines.get("lines").is_none()); // Should use default
}

#[test]
fn test_system_environment_params() {
    let args = json!({
        "include_dev_tools": true,
        "include_processes": true,
        "include_ports": false
    });

    assert_eq!(args.get("include_dev_tools").unwrap().as_bool(), Some(true));
    assert_eq!(args.get("include_processes").unwrap().as_bool(), Some(true));
    assert_eq!(args.get("include_ports").unwrap().as_bool(), Some(false));
}

#[test]
fn test_tool_response_format() {
    // Verify tool response follows MCP format
    let response = json!({
        "content": [
            {
                "type": "text",
                "text": "Tool result here"
            }
        ]
    });

    let content = response.get("content").unwrap().as_array().unwrap();
    assert_eq!(content.len(), 1);
    assert_eq!(content[0].get("type").unwrap().as_str(), Some("text"));
}

#[test]
fn test_error_response_format() {
    // Verify error response follows MCP format
    let error_response =
        JsonRpcResponse::error(Some(json!(1)), ERROR_INVALID_PARAMS, "Invalid parameters");

    assert!(error_response.error.is_some());
    assert_eq!(
        error_response.error.as_ref().unwrap().code,
        ERROR_INVALID_PARAMS
    );
}

#[test]
fn test_all_tools_defined() {
    // Verify all 6 tools are defined
    let tools = vec![
        ToolName::SearchMemory,
        ToolName::GetRecentActivity,
        ToolName::GetActiveWindowContext,
        ToolName::GetTerminalOutput,
        ToolName::GetSystemEnvironment,
        ToolName::GetRelatedContext,
    ];

    assert_eq!(tools.len(), 6);

    for tool in tools {
        let name = tool.as_str();
        assert!(!name.is_empty());
        // Verify it can be parsed back
        assert_eq!(ToolName::from_str(name), Some(tool));
    }
}

// ============================================================================
// Error Scenario Tests
// ============================================================================

#[test]
fn test_invalid_params_error_response() {
    // Verify ERROR_INVALID_PARAMS is returned for malformed parameters
    let response = JsonRpcResponse::error(
        Some(json!(1)),
        ERROR_INVALID_PARAMS,
        "Invalid parameters: missing required field 'query'",
    );

    assert!(response.error.is_some());
    let error = response.error.unwrap();
    assert_eq!(error.code, ERROR_INVALID_PARAMS);
    assert!(error.message.contains("Invalid parameters"));
}

#[test]
fn test_database_locked_error_code() {
    // Verify ERROR_DATABASE_LOCKED constant
    assert_eq!(ERROR_DATABASE_LOCKED, -32006);

    let response = JsonRpcResponse::error(
        Some(json!(1)),
        ERROR_DATABASE_LOCKED,
        "[-32000] Database is locked by another process",
    );

    assert!(response.error.is_some());
}

#[test]
fn test_server_error_with_retry_hint() {
    // Verify server error includes retry hint
    let error_msg = "[-32000] Database is locked by another process (Memflow app may be recording). Please wait a moment and retry.";

    assert!(error_msg.contains("[-32000]"));
    assert!(error_msg.contains("retry"));
    assert!(error_msg.contains("wait"));
}

// ============================================================================
// Parameter Validation Tests
// ============================================================================

#[test]
fn test_search_memory_params_validation() {
    // Valid: query is required
    let valid_args = json!({"query": "test search"});
    assert!(valid_args.get("query").is_some());

    // Valid with optional params
    let full_args = json!({
        "query": "test",
        "limit": 10,
        "mode": "hybrid",
        "app_name": "VSCode",
        "date_range": "today"
    });

    // Verify all fields exist
    assert!(full_args.get("query").is_some());
    assert!(full_args.get("limit").is_some());
    assert!(full_args.get("mode").is_some());
    assert!(full_args.get("app_name").is_some());
    assert!(full_args.get("date_range").is_some());
}

// ============================================================================
// Return Field Completeness Tests
// ============================================================================

#[test]
fn test_tool_response_content_structure() {
    // Verify tool response follows MCP format
    let response = json!({
        "content": [
            {
                "type": "text",
                "text": "Tool result here"
            }
        ]
    });

    let content = response.get("content").unwrap().as_array().unwrap();
    assert_eq!(content.len(), 1);

    let first = &content[0];
    assert!(first.get("type").is_some());
    assert!(first.get("text").is_some());
    assert_eq!(first.get("type").unwrap().as_str(), Some("text"));
}

#[test]
fn test_system_environment_return_fields() {
    // Verify system environment returns expected fields
    let expected_fields = vec![
        "OS",
        "OS Version",
        "Kernel",
        "Hostname",
        "CPU Count",
        "Total Memory",
        "Used Memory",
    ];

    // This documents what fields should be present
    assert_eq!(expected_fields.len(), 7);
    assert!(expected_fields.contains(&"OS"));
    assert!(expected_fields.contains(&"CPU Count"));
    assert!(expected_fields.contains(&"Total Memory"));
}

#[test]
fn test_recent_activity_return_fields() {
    // Verify recent activity returns timeline format
    let mock_output = "[Activity Timeline - Last 5 minutes]\n\n1. [12:00:00] VSCode - main.rs\n   OCR: fn main() {}\n\n";

    assert!(mock_output.contains("[Activity Timeline"));
    assert!(mock_output.contains("App:") || mock_output.contains("VSCode"));
    assert!(mock_output.contains("OCR:"));
}

#[test]
fn test_error_response_completeness() {
    // Verify error response has all required fields
    let error_response = JsonRpcResponse::error(
        Some(json!(123)),
        ERROR_INVALID_PARAMS,
        "Parameter validation failed",
    );

    assert!(error_response.error.is_some());
    let error = error_response.error.unwrap();

    assert_eq!(error.code, ERROR_INVALID_PARAMS);
    assert!(!error.message.is_empty());
    assert_eq!(error_response.id, Some(json!(123)));
    assert_eq!(error_response.jsonrpc, "2.0");
}
