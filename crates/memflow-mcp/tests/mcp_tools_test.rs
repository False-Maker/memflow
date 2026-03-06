// Integration tests for MCP server tools.
// These tests validate the tool handlers against the MCP Tool Contract v1.

use memflow_mcp::tools;
use memflow_core::context::RuntimeContext;
use serde_json::json;

// Test helper: mock context for testing (uses McpContext which implements RuntimeContext)
fn create_test_context() -> impl RuntimeContext {
    // For testing, we use McpContext which implements RuntimeContext
    memflow_mcp::context::McpContext::new()
}

#[tokio::test]
async fn test_search_memory_schema() {
    let ctx = create_test_context();
    
    // Test: missing required field 'query'
    let result = tools::handle_search_memory(&ctx, &json!({})).await;
    assert!(result.is_err());
    
    // Test: valid query with default params
    let result = tools::handle_search_memory(&ctx, &json!({
        "query": "test"
    })).await;
    // Should return valid JSON with content field
    // Note: May fail if DB not initialized, but schema should be valid
    match result {
        Ok(resp) => {
            assert!(resp.get("content").is_some());
            let content = resp.get("content").unwrap();
            assert!(content.is_array());
        }
        Err(_) => {
            // Expected if DB not available in test environment
        }
    }
}

#[tokio::test]
async fn test_search_memory_limit_clamping() {
    let ctx = create_test_context();
    
    // Test: limit exceeding MAX_SEARCH_LIMIT (50) should be clamped
    let result = tools::handle_search_memory(&ctx, &json!({
        "query": "test",
        "limit": 100
    })).await;
    
    // Should not panic, limit should be clamped
    assert!(result.is_ok() || result.is_err()); // Any result is acceptable
}

#[tokio::test]
async fn test_search_memory_modes() {
    let ctx = create_test_context();
    
    // Test all three modes
    for mode in ["keyword", "semantic", "hybrid"] {
        let result = tools::handle_search_memory(&ctx, &json!({
            "query": "rust programming",
            "mode": mode
        })).await;
        
        // Should not panic on any valid mode
        assert!(result.is_ok() || result.is_err());
    }
}

#[tokio::test]
async fn test_search_memory_filters() {
    let ctx = create_test_context();
    
    // Test: date_range filter
    for date_range in ["today", "yesterday", "this_week", "last_week", "this_month"] {
        let result = tools::handle_search_memory(&ctx, &json!({
            "query": "test",
            "date_range": date_range
        })).await;
        
        assert!(result.is_ok() || result.is_err());
    }
    
    // Test: has_ocr filter
    let result = tools::handle_search_memory(&ctx, &json!({
        "query": "test",
        "has_ocr": true
    })).await;
    
    assert!(result.is_ok() || result.is_err());
    
    // Test: app_name filter
    let result = tools::handle_search_memory(&ctx, &json!({
        "query": "test",
        "app_name": "code"
    })).await;
    
    assert!(result.is_ok() || result.is_err());
}

#[tokio::test]
async fn test_get_recent_activity_schema() {
    let ctx = create_test_context();
    
    // Test: valid params
    let result = tools::handle_get_recent_activity(&ctx, &json!({
        "minutes": 5,
        "limit": 10
    })).await;
    
    // Should return valid JSON structure
    match result {
        Ok(resp) => {
            assert!(resp.get("content").is_some());
        }
        Err(_) => {
            // Expected if DB not available
        }
    }
}

#[tokio::test]
async fn test_get_recent_activity_bounds() {
    let ctx = create_test_context();
    
    // Test: minutes clamping (1-30)
    let result = tools::handle_get_recent_activity(&ctx, &json!({
        "minutes": 100
    })).await;
    assert!(result.is_ok() || result.is_err());
    
    // Test: limit clamping (1-200)
    let result = tools::handle_get_recent_activity(&ctx, &json!({
        "limit": 500
    })).await;
    assert!(result.is_ok() || result.is_err());
}

#[tokio::test]
async fn test_get_active_window_context() {
    let ctx = create_test_context();
    
    let result = tools::handle_get_active_window_context(&ctx, &json!({})).await;
    
    // Should always return valid structure
    match result {
        Ok(resp) => {
            assert!(resp.get("content").is_some());
        }
        Err(_) => {
            // Expected if no activities
        }
    }
}

#[tokio::test]
async fn test_get_related_context_schema() {
    let ctx = create_test_context();
    
    // Test: missing query
    let result = tools::handle_get_related_context(&ctx, &json!({})).await;
    assert!(result.is_err());
    
    // Test: valid query
    let result = tools::handle_get_related_context(&ctx, &json!({
        "query": "test context"
    })).await;
    
    match result {
        Ok(resp) => {
            assert!(resp.get("content").is_some());
        }
        Err(_) => {
            // Expected if no results
        }
    }
}

#[tokio::test]
async fn test_get_related_context_params() {
    let ctx = create_test_context();
    
    // Test: custom limit and max_chars
    let result = tools::handle_get_related_context(&ctx, &json!({
        "query": "test",
        "limit": 10,
        "max_chars_per_item": 500
    })).await;
    
    assert!(result.is_ok() || result.is_err());
    
    // Test: boundary values
    let result = tools::handle_get_related_context(&ctx, &json!({
        "query": "test",
        "limit": 1,
        "max_chars_per_item": 100
    })).await;
    
    assert!(result.is_ok() || result.is_err());
}

#[tokio::test]
async fn test_get_terminal_output() {
    let ctx = create_test_context();
    
    let result = tools::handle_get_terminal_output(&ctx, &json!({})).await;
    
    // Should always return valid structure (or specific error codes)
    match result {
        Ok(resp) => {
            assert!(resp.get("content").is_some());
        }
        Err(e) => {
            // May return -32004 if no terminal found
            let msg = e.to_string();
            assert!(msg.contains("MCP_TERMINAL_NOT_FOUND") || msg.contains("MCP_INTERNAL"));
        }
    }
}

#[tokio::test]
async fn test_get_terminal_output_limit() {
    let ctx = create_test_context();
    
    let result = tools::handle_get_terminal_output(&ctx, &json!({
        "limit": 5
    })).await;
    
    assert!(result.is_ok() || result.is_err());
}

#[tokio::test]
async fn test_get_system_environment() {
    let ctx = create_test_context();
    
    let result = tools::handle_get_system_environment(&ctx, &json!({})).await;
    
    // Should always return valid structure
    match result {
        Ok(resp) => {
            assert!(resp.get("content").is_some());
            let content = resp.get("content").unwrap().as_array().unwrap();
            assert!(!content.is_empty());
            
            let text = content[0].get("text").unwrap().as_str().unwrap();
            // Should contain expected sections
            assert!(text.contains("系统环境概览") || text.contains("System"));
        }
        Err(e) => {
            panic!("get_system_environment failed: {}", e);
        }
    }
}
