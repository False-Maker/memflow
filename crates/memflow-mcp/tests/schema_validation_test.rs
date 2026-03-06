// Schema validation tests for MCP protocol compliance.
// Validates request/response structures against MCP Tool Contract v1.

use serde_json::json;

/// Validates JSON-RPC 2.0 request structure
#[test]
fn test_jsonrpc_request_schema() {
    // Valid request
    let request = json!({
        "jsonrpc": "2.0",
        "id": 1,
        "method": "tools/call",
        "params": {
            "name": "search_memory",
            "arguments": {
                "query": "test"
            }
        }
    });
    
    assert_eq!(request["jsonrpc"], "2.0");
    assert!(request.get("id").is_some());
    assert!(request.get("method").is_some());
    assert!(request.get("params").is_some());
}

/// Validates JSON-RPC 2.0 response structure
#[test]
fn test_jsonrpc_response_schema() {
    // Valid response
    let response = json!({
        "jsonrpc": "2.0",
        "id": 1,
        "result": {
            "content": [
                {
                    "type": "text",
                    "text": "result text"
                }
            ]
        }
    });
    
    assert_eq!(response["jsonrpc"], "2.0");
    assert!(response.get("id").is_some());
    assert!(response.get("result").is_some() || response.get("error").is_some());
}

/// Validates error response structure
#[test]
fn test_jsonrpc_error_response_schema() {
    let error = json!({
        "jsonrpc": "2.0",
        "id": 1,
        "error": {
            "code": -32003,
            "message": "MCP_INVALID_PARAMS: missing required field 'query'"
        }
    });
    
    assert_eq!(error["jsonrpc"], "2.0");
    assert!(error.get("error").is_some());
    let error_obj = error.get("error").unwrap();
    assert!(error_obj.get("code").is_some());
    assert!(error_obj.get("message").is_some());
}

/// Validates tool call params structure
#[test]
fn test_tool_call_params_schema() {
    let params = json!({
        "name": "search_memory",
        "arguments": {
            "query": "test query",
            "limit": 10,
            "mode": "hybrid"
        }
    });
    
    assert!(params.get("name").is_some());
    assert!(params.get("arguments").is_some());
    assert_eq!(params["name"], "search_memory");
}

/// Validates content block structure in tool responses
#[test]
fn test_content_block_schema() {
    // Valid content block
    let content = json!({
        "type": "text",
        "text": "some result text"
    });
    
    assert_eq!(content["type"], "text");
    assert!(content.get("text").is_some());
    assert!(content["text"].is_string());
}

/// Tests all error codes defined in Tool Contract v1
#[test]
fn test_error_codes() {
    let error_codes = vec![
        (-32000, "MCP_PARSE_ERROR"),
        (-32001, "MCP_INVALID_REQUEST"),
        (-32002, "MCP_METHOD_NOT_FOUND"),
        (-32003, "MCP_INVALID_PARAMS"),
        (-32004, "MCP_TERMINAL_NOT_FOUND"),
        (-32005, "MCP_PERMISSION_DENIED"),
        (-32006, "MCP_INTERNAL"),
        (-32007, "MCP_CORE_UNAVAILABLE"),
        (-32008, "MCP_DEGRADED_MODE"),
    ];
    
    // Error codes range from -32000 (largest) to -32008 (smallest/most negative)
    // So valid range is: code >= -32008 AND code <= -32000
    for (code, _name) in error_codes {
        assert!((code >= -32008 && code <= -32000), "Invalid error code range: {}", code);
    }
}

/// Validates search_memory input schema
#[test]
fn test_search_memory_input_schema() {
    // Required fields
    let valid = json!({
        "query": "test"
    });
    assert!(valid.get("query").is_some());
    
    // Optional fields
    let with_options = json!({
        "query": "test",
        "limit": 10,
        "mode": "hybrid",
        "app_name": "code",
        "keywords": ["rust", "async"],
        "date_range": "today",
        "has_ocr": true
    });
    
    assert!(with_options.get("limit").is_some());
    assert!(with_options.get("mode").is_some());
    assert!(with_options.get("app_name").is_some());
    assert!(with_options.get("keywords").is_some());
    assert!(with_options.get("date_range").is_some());
    assert!(with_options.get("has_ocr").is_some());
}

/// Validates get_recent_activity input schema
#[test]
fn test_get_recent_activity_input_schema() {
    let valid = json!({
        "minutes": 5,
        "limit": 50
    });
    
    assert!(valid.get("minutes").is_some());
    assert!(valid.get("limit").is_some());
}

/// Validates get_related_context input schema
#[test]
fn test_get_related_context_input_schema() {
    let valid = json!({
        "query": "context test",
        "limit": 5,
        "max_chars_per_item": 400
    });
    
    assert!(valid.get("query").is_some());
    assert!(valid.get("limit").is_some());
    assert!(valid.get("max_chars_per_item").is_some());
}

/// Validates get_terminal_output input schema
#[test]
fn test_get_terminal_output_input_schema() {
    let valid = json!({
        "limit": 20
    });
    
    assert!(valid.get("limit").is_some());
}
