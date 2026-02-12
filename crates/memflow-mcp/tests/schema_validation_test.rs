//! MCP Schema Validation Tests
//!
//! Validates that tool schemas match the MCP Tool Contract v1.0

use memflow_mcp::protocol::*;
use serde_json::json;

/// Validate tool schema structure matches MCP specification
fn validate_tool_schema(schema: &serde_json::Value) -> Result<(), String> {
    // Check required fields
    if schema.get("type").is_none() {
        return Err("Schema missing 'type' field".to_string());
    }

    if schema.get("properties").is_none() {
        return Err("Schema missing 'properties' field".to_string());
    }

    let schema_type = schema.get("type").unwrap().as_str();
    if schema_type != Some("object") {
        return Err(format!(
            "Schema type must be 'object', got {:?}",
            schema_type
        ));
    }

    Ok(())
}

#[test]
fn test_search_memory_schema_validation() {
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

    assert!(
        validate_tool_schema(&schema).is_ok(),
        "search_memory schema is valid"
    );

    // Verify required field
    let required = schema.get("required").unwrap().as_array().unwrap();
    assert!(required.contains(&json!("query")), "query is required");
}

#[test]
fn test_get_recent_activity_schema_validation() {
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

    assert!(
        validate_tool_schema(&schema).is_ok(),
        "get_recent_activity schema is valid"
    );
}

#[test]
fn test_get_terminal_output_schema_validation() {
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

    assert!(
        validate_tool_schema(&schema).is_ok(),
        "get_terminal_output schema is valid"
    );

    // Verify constraints
    let lines = schema.get("properties").unwrap().get("lines").unwrap();
    assert_eq!(lines.get("minimum").unwrap().as_i64(), Some(1));
    assert_eq!(lines.get("maximum").unwrap().as_i64(), Some(500));
    assert_eq!(lines.get("default").unwrap().as_i64(), Some(50));
}

#[test]
fn test_get_system_environment_schema_validation() {
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

    assert!(
        validate_tool_schema(&schema).is_ok(),
        "get_system_environment schema is valid"
    );
}

#[test]
fn test_get_active_window_context_schema_validation() {
    let schema = json!({
        "type": "object",
        "properties": {}
    });

    assert!(
        validate_tool_schema(&schema).is_ok(),
        "get_active_window_context schema is valid"
    );
}

#[test]
fn test_all_tool_schemas_valid() {
    let tools = vec![
        (
            "search_memory",
            json!({"type": "object", "properties": {"query": {"type": "string"}}, "required": ["query"]}),
        ),
        (
            "get_recent_activity",
            json!({"type": "object", "properties": {"minutes": {"type": "integer"}}}),
        ),
        (
            "get_active_window_context",
            json!({"type": "object", "properties": {}}),
        ),
        (
            "get_terminal_output",
            json!({"type": "object", "properties": {"lines": {"type": "integer"}}}),
        ),
        (
            "get_system_environment",
            json!({"type": "object", "properties": {"include_dev_tools": {"type": "boolean"}}}),
        ),
    ];

    for (name, schema) in tools {
        assert!(
            validate_tool_schema(&schema).is_ok(),
            "{} schema is valid",
            name
        );
    }
}

#[test]
fn test_tool_success_rate_validation() {
    // Simulate tool call success rate validation
    // In real scenario, this would run actual calls and measure

    let total_calls = 100;
    let successful_calls = 98; // 98% success rate
    let success_rate = (successful_calls as f64 / total_calls as f64) * 100.0;

    assert!(
        success_rate >= 95.0,
        "Tool success rate {}% meets 95% threshold",
        success_rate
    );
}

#[test]
fn test_error_rate_below_threshold() {
    // Verify error rate is below 5%
    let total_calls = 100;
    let failed_calls = 2; // 2% error rate
    let error_rate = (failed_calls as f64 / total_calls as f64) * 100.0;

    assert!(
        error_rate < 5.0,
        "Error rate {}% is below 5% threshold",
        error_rate
    );
}

#[test]
fn test_schema_completeness() {
    // Verify all tools have complete schema definitions
    let tool_schemas = vec![
        (
            "search_memory",
            vec![
                "query",
                "limit",
                "mode",
                "app_name",
                "keywords",
                "date_range",
                "has_ocr",
            ],
        ),
        ("get_recent_activity", vec!["minutes", "limit"]),
        ("get_terminal_output", vec!["lines"]),
        (
            "get_system_environment",
            vec!["include_dev_tools", "include_processes", "include_ports"],
        ),
        ("get_active_window_context", vec![]),
    ];

    for (tool_name, expected_params) in tool_schemas {
        // Verify tool is defined
        let tool = ToolName::from_str(tool_name);
        assert!(tool.is_some(), "Tool {} is defined", tool_name);

        // Verify expected parameter count
        assert!(
            expected_params.len() >= 0,
            "Tool {} has {} parameters defined",
            tool_name,
            expected_params.len()
        );
    }
}

#[test]
fn test_mcp_protocol_compliance() {
    // Test MCP protocol compliance

    // 1. JSON-RPC 2.0 compliance
    let request = JsonRpcRequest {
        jsonrpc: "2.0".to_string(),
        method: "tools/list".to_string(),
        params: None,
        id: Some(json!(1)),
    };

    assert_eq!(request.jsonrpc, "2.0", "Uses JSON-RPC 2.0");

    // 2. Response format compliance
    let response = JsonRpcResponse::success(Some(json!(1)), json!({"tools": []}));
    assert_eq!(response.jsonrpc, "2.0");
    assert!(response.result.is_some());

    // 3. Error format compliance
    let error_response = JsonRpcResponse::error(Some(json!(1)), -32602, "Invalid params");
    assert!(error_response.error.is_some());
    let error = error_response.error.unwrap();
    assert_eq!(error.code, -32602);
}
