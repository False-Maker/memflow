use crate::context::McpContext;
use crate::protocol::{
    JsonRpcRequest, JsonRpcResponse, ERR_INTERNAL, ERR_INVALID_PARAMS, ERR_TOOL_FAILED,
    ERR_TERMINAL_NOT_FOUND, ERR_PERMISSION_DENIED,
};
use crate::tools;
use anyhow::{Context, Result};
use memflow_core::context::RuntimeContext;
use tokio::io::{AsyncBufReadExt, BufReader};
use tracing::{error, info};

/// Run the MCP server main loop: read JSON-RPC requests from stdin,
/// dispatch to handlers, and write responses to stdout.
pub async fn run_server(ctx: McpContext) -> Result<()> {
    let stdin = tokio::io::stdin();
    let mut reader = BufReader::new(stdin);
    let mut line = String::new();

    info!("memflow-mcp server loop ready.");

    loop {
        line.clear();
        let bytes_read = reader.read_line(&mut line).await?;
        if bytes_read == 0 {
            break; // EOF
        }

        let trimmed = line.trim();
        if trimmed.is_empty() {
            continue;
        }

        match process_line(trimmed, &ctx).await {
            Ok(Some(response)) => {
                let json_str = serde_json::to_string(&response)?;
                println!("{}", json_str);
            }
            Ok(None) => {}
            Err(e) => {
                error!("Error processing request: {}", e);
                let err_res =
                    JsonRpcResponse::error(None, -32700, format!("Parse error: {}", e));
                match serde_json::to_string(&err_res) {
                    Ok(s) => println!("{}", s),
                    Err(_) => eprintln!("Failed to serialize error response"),
                }
            }
        }
    }

    Ok(())
}

async fn process_line(
    line: &str,
    ctx: &impl RuntimeContext,
) -> Result<Option<JsonRpcResponse>> {
    let req: JsonRpcRequest = serde_json::from_str(line)?;
    let id = req.id.clone();

    match req.method.as_str() {
        "initialize" => {
            let capabilities = serde_json::json!({
                "protocolVersion": "2024-11-05",
                "capabilities": {
                    "tools": {}
                },
                "serverInfo": {
                    "name": "memflow-mcp",
                    "version": "0.1.0"
                }
            });
            Ok(Some(JsonRpcResponse::ok(id, capabilities)))
        }
        "notifications/initialized" => Ok(None),
        "tools/list" => {
            // List all tools defined in the MCP Tool Contract v1.
            let tools = serde_json::json!({
                "tools": [
                    {
                        "name": "search_memory",
                        "description": "Search local MemFlow memory (activities + OCR) using hybrid/semantic/keyword modes.",
                        "inputSchema": {
                            "type": "object",
                            "properties": {
                                "query": {
                                    "type": "string",
                                    "description": "The natural language query to search for."
                                },
                                "limit": {
                                    "type": "integer",
                                    "description": "Maximum number of results to return (default 5, typical 3-10)."
                                },
                                "mode": {
                                    "type": "string",
                                    "enum": ["hybrid", "semantic", "keyword"],
                                    "description": "Search mode: hybrid (default), semantic-only, or keyword-only."
                                },
                                "app_name": {
                                    "type": "string",
                                    "description": "Optional app name filter (e.g. 'Chrome', 'code', 'terminal')."
                                },
                                "keywords": {
                                    "type": "array",
                                    "items": { "type": "string" },
                                    "description": "Optional keyword list to emphasize; usually inferred from query."
                                },
                                "date_range": {
                                    "type": "string",
                                    "enum": ["today", "yesterday", "last_week", "this_week", "this_month"],
                                    "description": "Optional logical date range filter."
                                },
                                "has_ocr": {
                                    "type": "boolean",
                                    "description": "Whether to require OCR text presence (true) or absence (false)."
                                }
                            },
                            "required": ["query"]
                        }
                    },
                    {
                        "name": "get_recent_activity",
                        "description": "Return recent activity timeline within the last N minutes.",
                        "inputSchema": {
                            "type": "object",
                            "properties": {
                                "minutes": {
                                    "type": "integer",
                                    "description": "How many minutes back to look (default 5, max 30)."
                                },
                                "limit": {
                                    "type": "integer",
                                    "description": "Maximum number of activities to include (default 50)."
                                }
                            }
                        }
                    },
                    {
                        "name": "get_active_window_context",
                        "description": "Summarize the current active window and its recent OCR text.",
                        "inputSchema": {
                            "type": "object",
                            "properties": {}
                        }
                    },
                    {
                        "name": "get_related_context",
                        "description": "Return concise context snippets related to the given query.",
                        "inputSchema": {
                            "type": "object",
                            "properties": {
                                "query": {
                                    "type": "string",
                                    "description": "Question or topic to search related context for."
                                },
                                "limit": {
                                    "type": "integer",
                                    "description": "Maximum number of snippets to return (default 5)."
                                },
                                "max_chars_per_item": {
                                    "type": "integer",
                                    "description": "Maximum characters per snippet (default 400)."
                                }
                            },
                            "required": ["query"]
                        }
                    },
                    {
                        "name": "get_terminal_output",
                        "description": "Return recent terminal output captured by MemFlow.",
                        "inputSchema": {
                            "type": "object",
                            "properties": {
                                "limit": {
                                    "type": "integer",
                                    "description": "Maximum number of terminal log entries to include (default 20)."
                                }
                            }
                        }
                    },
                    {
                        "name": "get_system_environment",
                        "description": "Return a short summary of the local system environment (OS, user, shell, cwd).",
                        "inputSchema": {
                            "type": "object",
                            "properties": {}
                        }
                    },
                    {
                        "name": "search_visual_memory",
                        "description": "[DEPRECATED] Alias for search_memory kept for backward compatibility.",
                        "inputSchema": {
                            "type": "object",
                            "properties": {
                                "query": { "type": "string" },
                                "limit": { "type": "integer" }
                            },
                            "required": ["query"]
                        }
                    },
                    {
                        "name": "get_recent_activities",
                        "description": "[DEPRECATED] Alias for get_recent_activity kept for backward compatibility.",
                        "inputSchema": {
                            "type": "object",
                            "properties": {
                                "minutes": { "type": "integer" },
                                "limit": { "type": "integer" }
                            }
                        }
                    }
                ]
            });
            Ok(Some(JsonRpcResponse::ok(id, tools)))
        }
        "tools/call" => {
            let params = req.params.context("Missing params")?;
            let name = params["name"].as_str().context("Missing tool name")?;
            let args = &params["arguments"];

            let result = match name {
                "search_memory" | "search_visual_memory" => {
                    tools::handle_search_memory(ctx, args).await
                }
                "get_recent_activity" | "get_recent_activities" => {
                    tools::handle_get_recent_activity(ctx, args).await
                }
                "get_active_window_context" => {
                    tools::handle_get_active_window_context(ctx, args).await
                }
                "get_related_context" => tools::handle_get_related_context(ctx, args).await,
                "get_terminal_output" => tools::handle_get_terminal_output(ctx, args).await,
                "get_system_environment" => {
                    tools::handle_get_system_environment(ctx, args).await
                }
                _ => {
                    return Ok(Some(JsonRpcResponse::error(
                        id,
                        -32601,
                        format!("Tool not found: {}", name),
                    )));
                }
            };

            match result {
                Ok(value) => Ok(Some(JsonRpcResponse::ok(id, value))),
                Err(e) => {
                    let msg = e.to_string();
                    error!("Tool call failed ({}): {}", name, msg);

                    // Best-effort mapping from error text to MCP tool error codes.
                    // Tools can tag their errors with well-known markers which we
                    // translate here into the standard error range.
                    let (code, message) = if msg.contains("MCP_INVALID_PARAMS") {
                        (ERR_INVALID_PARAMS, msg)
                    } else if msg.contains("MCP_TERMINAL_NOT_FOUND") {
                        (ERR_TERMINAL_NOT_FOUND, msg)
                    } else if msg.contains("MCP_PERMISSION_DENIED") {
                        (ERR_PERMISSION_DENIED, msg)
                    } else if msg.contains("MCP_INTERNAL") {
                        (ERR_INTERNAL, msg)
                    } else {
                        (ERR_TOOL_FAILED, msg)
                    };

                    Ok(Some(JsonRpcResponse::error(id, code, message)))
                }
            }
        }
        _ => {
            if id.is_none() {
                Ok(None)
            } else {
                Ok(Some(JsonRpcResponse::error(
                    id,
                    -32601,
                    format!("Method not found: {}", req.method),
                )))
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::context::McpContext;
    use serde_json::Value;

    /// Helper to build a simple JSON-RPC request as a string.
    fn build_request(method: &str, params: Option<Value>) -> String {
        let req = serde_json::json!({
            "jsonrpc": "2.0",
            "method": method,
            "params": params,
            "id": 1
        });
        req.to_string()
    }

    #[tokio::test]
    async fn tools_list_includes_core_tools() {
        let ctx = McpContext::new();
        let line = build_request("tools/list", None);

        let resp = process_line(&line, &ctx)
            .await
            .expect("process_line should succeed")
            .expect("response should not be None");

        // tools/list should return a result with a "tools" array that includes
        // at least the primary tools defined in the Tool Contract.
        let result = resp.result.expect("result must be present");
        let tools = result
            .get("tools")
            .and_then(|v| v.as_array())
            .expect("tools must be an array");

        let mut names: Vec<String> = tools
            .iter()
            .filter_map(|t| t.get("name").and_then(|n| n.as_str()).map(|s| s.to_string()))
            .collect();
        names.sort();

        assert!(
            names.contains(&"search_memory".to_string()),
            "tools/list must expose search_memory"
        );
        assert!(
            names.contains(&"get_recent_activity".to_string()),
            "tools/list must expose get_recent_activity"
        );
        assert!(
            names.contains(&"get_related_context".to_string()),
            "tools/list must expose get_related_context"
        );
    }

    #[tokio::test]
    async fn invalid_params_are_mapped_to_err_invalid_params() {
        let ctx = McpContext::new();

        // Call search_memory without required 'query' field to trigger MCP_INVALID_PARAMS.
        let params = serde_json::json!({
            "name": "search_memory",
            "arguments": {}
        });
        let line = build_request("tools/call", Some(params));        let resp = process_line(&line, &ctx)
            .await
            .expect("process_line should succeed")
            .expect("response should not be None");

        let error = resp.error.expect("error must be present for invalid params");
        assert_eq!(
            error.code, ERR_INVALID_PARAMS,
            "MCP_INVALID_PARAMS should be mapped to ERR_INVALID_PARAMS"
        );
        assert!(
            error.message.contains("MCP_INVALID_PARAMS"),
            "error message should carry MCP_INVALID_PARAMS marker for debugging"
        );
    }
}
