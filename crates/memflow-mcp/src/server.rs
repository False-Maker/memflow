use crate::protocol::{JsonRpcError, JsonRpcRequest, JsonRpcResponse, ListToolsResult, Tool};
use crate::context::McpContext;
use memflow_core::context::RuntimeContext;
use memflow_core::db;
use serde_json::{json, Value};
use std::sync::Arc;
use tracing::{error, info};

pub struct McpServer {
    context: Arc<McpContext>,
}

impl McpServer {
    pub fn new(context: Arc<McpContext>) -> Self {
        Self { context }
    }

    pub async fn handle_request(&self, request: JsonRpcRequest) -> JsonRpcResponse {
        let result = match request.method.as_str() {
            "unknown" => Err(JsonRpcError {
                code: -32601,
                message: "Method not found".to_string(),
                data: None,
            }),
            
            // MCP Methods
            "tools/list" => self.list_tools().await,
            "tools/call" => self.call_tool(request.params).await,
            
            // Fallback
            _ => Err(JsonRpcError {
                code: -32601,
                message: format!("Method {} not found", request.method),
                data: None,
            }),
        };

        match result {
            Ok(value) => JsonRpcResponse::success(request.id, value),
            Err(e) => JsonRpcResponse {
                jsonrpc: "2.0".to_string(),
                result: None,
                error: Some(e),
                id: request.id,
            },
        }
    }

    async fn list_tools(&self) -> Result<Value, JsonRpcError> {
        let tools = vec![
            Tool {
                name: "memflow_search_activities".to_string(),
                description: "Search user activity history for specific keywords or time ranges".to_string(),
                input_schema: json!({
                    "type": "object",
                    "properties": {
                        "query": { "type": "string", "description": "Keywords to search for in OCR text or window titles" },
                        "limit": { "type": "integer", "description": "Max number of results (default 10)" }
                    },
                    "required": ["query"]
                }),
            },
            Tool {
                name: "memflow_get_activity".to_string(),
                description: "Get detailed information about a specific activity by ID".to_string(),
                input_schema: json!({
                    "type": "object",
                    "properties": {
                        "activity_id": { "type": "integer", "description": "The ID of the activity to retrieve" }
                    },
                    "required": ["activity_id"]
                }),
            },
        ];

        Ok(serde_json::to_value(ListToolsResult { tools }).unwrap())
    }

    async fn call_tool(&self, params: Option<Value>) -> Result<Value, JsonRpcError> {
        let params = params.ok_or_else(|| JsonRpcError {
            code: -32602,
            message: "Missing params".to_string(),
            data: None,
        })?;

        let name = params.get("name").and_then(|v| v.as_str()).ok_or_else(|| JsonRpcError {
            code: -32602,
            message: "Missing tool name".to_string(),
            data: None,
        })?;

        let args = params.get("arguments").cloned().unwrap_or(json!({}));

        match name {
            "memflow_search_activities" => {
                let query = args.get("query").and_then(|v| v.as_str()).unwrap_or("");
                let limit = args.get("limit").and_then(|v| v.as_i64()).unwrap_or(10);
                
                info!("Executing search tool: query='{}', limit={}", query, limit);

                // Initialize DB connection if needed (lazy or reusing global)
                // In memflow-core, db init passes app_handle/path. We might need to ensure DB is initialized in main.
                
                // For now, let's assume global pool is set or we error out if not
                // This is a simplified search that maps to db::search_activities
                
                // Note: db::search_activities signature inside memflow-core might need checking
                // Assuming access to memflow_core::db
                
                let results = db::search_activities(
                    Some(query.to_string()), // query
                    None, // app_name
                    None, // start_time
                    None, // end_time
                    None, // has_ocr
                    Some(limit), // limit
                    None, // offset
                    None, // order_by
                ).await.map_err(|e| JsonRpcError {
                    code: -32000,
                    message: format!("Search failed: {}", e),
                    data: None,
                })?;

                Ok(json!({ "activities": results.0 }))
            }
            "memflow_get_activity" => {
                let activity_id = args.get("activity_id").and_then(|v| v.as_i64()).ok_or_else(|| JsonRpcError {
                    code: -32602,
                    message: "Missing activity_id".to_string(),
                    data: None,
                })?;

                info!("Executing get_activity tool: id={}", activity_id);

                let activity = db::get_activity_by_id(activity_id).await.map_err(|e| JsonRpcError {
                    code: -32000,
                    message: format!("Get activity failed: {}", e),
                    data: None,
                })?;

                Ok(serde_json::to_value(activity).unwrap())
            }
            _ => Err(JsonRpcError {
                code: -32601,
                message: format!("Tool {} not found", name),
                data: None,
            }),
        }
    }
}
