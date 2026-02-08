use anyhow::{Context, Result};
use clap::Parser;
use fastembed::{InitOptions, TextEmbedding, EmbeddingModel};
use memflow_core::ai::rag::HybridSearch;
use memflow_core::context::RuntimeContext;
use memflow_core::db;
use serde::{Deserialize, Serialize};
use serde_json::Value;
use std::io::{self};
use tokio::io::{AsyncBufReadExt, BufReader};
use tracing::{error, info};
use tracing_subscriber::{fmt, EnvFilter};
use std::sync::OnceLock;

mod context;
use memflow_mcp::prompts;
use context::McpContext;

// Global model instance
static EMBEDDING_MODEL: OnceLock<std::sync::Mutex<TextEmbedding>> = OnceLock::new();

#[derive(Parser, Debug)]
#[command(author, version, about, long_about = None)]
struct Args {}

#[derive(Serialize, Deserialize, Debug)]
struct JsonRpcRequest {
    jsonrpc: String,
    method: String,
    params: Option<Value>,
    id: Option<Value>,
}

#[derive(Serialize, Deserialize, Debug)]
struct JsonRpcResponse {
    jsonrpc: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    result: Option<Value>,
    #[serde(skip_serializing_if = "Option::is_none")]
    error: Option<JsonRpcError>,
    id: Option<Value>,
}

#[derive(Serialize, Deserialize, Debug)]
struct JsonRpcError {
    code: i32,
    message: String,
    data: Option<Value>,
}

impl JsonRpcResponse {
    fn ok(id: Option<Value>, result: Value) -> Self {
        Self {
            jsonrpc: "2.0".to_string(),
            result: Some(result),
            error: None,
            id,
        }
    }

    fn error(id: Option<Value>, code: i32, message: String) -> Self {
        Self {
            jsonrpc: "2.0".to_string(),
            result: None,
            error: Some(JsonRpcError {
                code,
                message,
                data: None,
            }),
            id,
        }
    }
}

#[tokio::main]
async fn main() -> Result<()> {
    // 🛑 关键修复：强制日志输出到 Stderr，绝对不能污染 Stdout！
    let env_filter = EnvFilter::try_from_default_env()
        .unwrap_or_else(|_| "warn,sqlx=warn".into()); // 默认设为 warn，避免 INFO 日志被客户端标记为 [error]

    fmt()
        .with_env_filter(env_filter)
        .with_ansi(false) // <--- 禁止 ANSI 颜色，防止某些客户端解析出错
        .with_writer(io::stderr) // <--- 就是这一行！把日志赶到 Stderr 去
        .init();

    let _args = Args::parse();
    
    // Initialize context and DB
    let ctx = McpContext::new();
    let app_dir = ctx.app_dir();
    let db_path = app_dir.join("memflow.db");
    let screenshots_dir = app_dir.join("screenshots");
    let resource_dir = ctx.resource_dir();

    info!("memflow-mcp server starting...");
    info!("Resource dir: {:?}", resource_dir);

    // Initialize Embedding Model (with panic protection for ONNX Runtime conflicts)
    info!("Initializing Embedding Model (BGESmallENV15)...");
    let resource_dir_clone = resource_dir.clone();
    
    // Use catch_unwind to handle ONNX Runtime version conflicts that panic
    let init_result = std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| {
        let model_opts = InitOptions::new(EmbeddingModel::BGESmallENV15)
            .with_cache_dir(resource_dir_clone.join("models"))
            .with_show_download_progress(false);
        TextEmbedding::try_new(model_opts)
    }));

    match init_result {
        Ok(Ok(model)) => {
            if EMBEDDING_MODEL.set(std::sync::Mutex::new(model)).is_err() {
                error!("Failed to set global embedding model");
            } else {
                info!("Embedding Model initialized successfully.");
            }
        },
        Ok(Err(e)) => {
            error!("Failed to initialize Embedding Model: {}. Using placeholder embeddings.", e);
        },
        Err(_) => {
            error!("Embedding Model initialization panicked (likely ONNX Runtime version conflict). Using placeholder embeddings.");
        }
    }

    // 我们不再在主线程启动时阻塞数据库初始化，防止启动过慢导致 MCP 客户端超时
    let db_path_clone = db_path.clone();
    let screenshots_dir_clone = screenshots_dir.clone();
    tokio::spawn(async move {
        info!("Initializing database in background...");
        if let Err(e) = db::init_db_with_path(db_path_clone, screenshots_dir_clone).await {
            error!("Background database initialization failed: {}", e);
        } else {
            info!("Background database initialization successful.");
        }
    });

    info!("memflow-mcp server loop ready.");

    {
        let hb_dir = app_dir.clone();
        tokio::spawn(async move {
            let hb_path = hb_dir.join("mcp_heartbeat.json");
            loop {
                let now = chrono::Local::now().timestamp();
                let payload = serde_json::json!({
                    "status": "online",
                    "ts": now
                });
                if let Ok(s) = serde_json::to_string(&payload) {
                    if let Err(e) = std::fs::write(&hb_path, s) {
                        error!("Failed to write MCP heartbeat: {}", e);
                    }
                }
                tokio::time::sleep(std::time::Duration::from_secs(5)).await;
            }
        });
    }

    let stdin = tokio::io::stdin();
    // We don't use stdout wrapper, just println! is fine as long as we are careful.

    let mut reader = BufReader::new(stdin);
    let mut line = String::new();

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

        match process_line(trimmed).await {
            Ok(Some(response)) => {
                let json_str = serde_json::to_string(&response)?;
                println!("{}", json_str);
            }
            Ok(None) => {}
            Err(e) => {
                error!("Error processing request: {}", e);
                let err_res = JsonRpcResponse::error(None, -32700, format!("Parse error: {}", e));
                match serde_json::to_string(&err_res) {
                    Ok(s) => println!("{}", s),
                    Err(_) => eprintln!("Failed to serialize error response"),
                }
            }
        }
    }

    Ok(())
}

async fn process_line(line: &str) -> Result<Option<JsonRpcResponse>> {
    let req: JsonRpcRequest = serde_json::from_str(line)?;
    let id = req.id.clone();

    match req.method.as_str() {
        "initialize" => {
            let capabilities = serde_json::json!({
                "protocolVersion": "2024-11-05",
                "capabilities": {
                    "tools": {},
                    "prompts": {}
                },
                "serverInfo": {
                    "name": "memflow-mcp",
                    "version": "0.1.0"
                }
            });
            Ok(Some(JsonRpcResponse::ok(id, capabilities)))
        }
        "notifications/initialized" => {
            Ok(None)
        }
        "tools/list" => {
            let tools = serde_json::json!({
                "tools": [
                    {
                        "name": "search_visual_memory",
                        "description": "Search user's recorded screen history for relevant visual context. Returns OCR text, app names, and timestamps from past activities.",
                        "inputSchema": {
                            "type": "object",
                            "properties": {
                                "query": {
                                    "type": "string",
                                    "description": "The search query to match against memory."
                                },
                                "limit": {
                                    "type": "integer",
                                    "description": "Maximum number of results to return (default 5)."
                                }
                            },
                            "required": ["query"]
                        }
                    },
                    {
                        "name": "get_active_window_context",
                        "description": "Get the current/latest screen context including window title, app name, and OCR text. Use this to understand what the user is currently looking at (e.g., 'help me fix this error on screen').",
                        "inputSchema": {
                            "type": "object",
                            "properties": {}
                        }
                    },
                    {
                        "name": "get_recent_activities",
                        "description": "Get the user's recent activity timeline. Use this to understand 'what did I just do in the last few minutes'. Returns a chronological list of apps and windows the user interacted with.",
                        "inputSchema": {
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

            match name {
                "search_visual_memory" => {
                    let query = args["query"].as_str().context("Missing query argument")?;
                    let limit = args["limit"].as_u64().unwrap_or(5) as usize;

                    match call_search_visual_memory(query, limit).await {
                        Ok(result_text) => {
                             Ok(Some(JsonRpcResponse::ok(id, serde_json::json!({
                                "content": [
                                    {
                                        "type": "text",
                                        "text": result_text
                                    }
                                ]
                            }))))
                        },
                        Err(e) => {
                            error!("Search failed: {}", e);
                            Ok(Some(JsonRpcResponse::error(id, -32000, e.to_string())))
                        }
                    }
                }
                "get_active_window_context" => {
                    match call_get_active_window_context().await {
                        Ok(result_text) => {
                            Ok(Some(JsonRpcResponse::ok(id, serde_json::json!({
                                "content": [
                                    {
                                        "type": "text",
                                        "text": result_text
                                    }
                                ]
                            }))))
                        },
                        Err(e) => {
                            error!("Get active window context failed: {}", e);
                            Ok(Some(JsonRpcResponse::error(id, -32000, e.to_string())))
                        }
                    }
                }
                "get_recent_activities" => {
                    let minutes = args["minutes"].as_i64().unwrap_or(5);
                    let limit = args["limit"].as_i64().unwrap_or(20);

                    match call_get_recent_activities(minutes, limit).await {
                        Ok(result_text) => {
                            Ok(Some(JsonRpcResponse::ok(id, serde_json::json!({
                                "content": [
                                    {
                                        "type": "text",
                                        "text": result_text
                                    }
                                ]
                            }))))
                        },
                        Err(e) => {
                            error!("Get recent activities failed: {}", e);
                            Ok(Some(JsonRpcResponse::error(id, -32000, e.to_string())))
                        }
                    }
                }
                _ => {
                    Ok(Some(JsonRpcResponse::error(id, -32601, format!("Tool not found: {}", name))))
                }
            }
        }
        "prompts/list" => {
            let result = prompts::list_prompts();
            Ok(Some(JsonRpcResponse::ok(id, serde_json::to_value(result)?)))
        }
        "prompts/get" => {
            let params = req.params.context("Missing params")?;
            let name = params["name"].as_str().context("Missing prompt name")?;
            let arguments = params.get("arguments").cloned();

            match prompts::get_prompt(name, arguments) {
                Some(result) => {
                    Ok(Some(JsonRpcResponse::ok(id, serde_json::to_value(result)?)))
                }
                None => {
                    Ok(Some(JsonRpcResponse::error(id, -32601, format!("Prompt not found: {}", name))))
                }
            }
        }
        _ => {
            if id.is_none() {
                Ok(None)
            } else {
                Ok(Some(JsonRpcResponse::error(id, -32601, format!("Method not found: {}", req.method))))
            }
        }
    }
}

async fn call_search_visual_memory(query: &str, limit: usize) -> Result<String> {
    info!("Searching visual memory for: {} (limit: {})", query, limit);

    // Check if model is available
    let embedding = if let Some(model_lock) = EMBEDDING_MODEL.get() {
        info!("Generating embedding for query...");
        // Mutex lock
        let mut model = model_lock.lock().map_err(|_| anyhow::anyhow!("Embedding model is busy. Please retry."))?;
        let embeddings = model.embed(vec![query], None)?;
        // fastembed returns Vec<Vec<f32>>, we take the first one
        if let Some(vec) = embeddings.into_iter().next() {
            info!("Embedding generated (dim: {})", vec.len());
            vec
        } else {
            return Err(anyhow::anyhow!("Failed to generate embedding: empty result"));
        }
    } else {
        error!("Embedding model not initialized, falling back to placeholder.");
        memflow_core::vector_db::generate_placeholder_embedding(query)
    };

    let searcher = HybridSearch::new();
    let results = searcher.search_with_embedding(query, embedding, limit).await
        .map_err(|e| {
            let err_str = e.to_string();
            if err_str.contains("database is locked") {
                anyhow::anyhow!("Database is locked by another process (Memflow app may be recording). Please retry in a moment.")
            } else if err_str.contains("no such table") || err_str.contains("unable to open") {
                anyhow::anyhow!("Database not found or not initialized. Please ensure Memflow app has run at least once.")
            } else {
                e
            }
        })?;

    if results.is_empty() {
        return Ok("No matching results found.".to_string());
    }

    let mut output = String::new();
    for res in results {
        let activity = db::get_activity_by_id(res.id).await;
        if let Ok(act) = activity {
            use chrono::TimeZone;
            let dt = chrono::Local.timestamp_opt(act.timestamp, 0).unwrap();
            
            output.push_str(&format!(
                "ID: {} | Time: {} | App: {} | Title: {}\nScore: {:.2}\nContent: {}\n---\n",
                act.id,
                dt.format("%Y-%m-%d %H:%M:%S"),
                act.app_name,
                act.window_title,
                res.score,
                act.ocr_text.unwrap_or_default().trim()
            ));
        }
    }

    Ok(output)
}

/// Get the current/latest screen context (Phase 2: Real-time Perception)
async fn call_get_active_window_context() -> Result<String> {
    info!("Getting active window context...");

    // Get the single most recent activity
    let activities = db::get_activities(1).await
        .map_err(|e| {
            let err_str = e.to_string();
            if err_str.contains("database is locked") {
                anyhow::anyhow!("Database is locked by another process. Please retry.")
            } else if err_str.contains("no such table") || err_str.contains("unable to open") {
                anyhow::anyhow!("Database not initialized. Please ensure Memflow app has run at least once.")
            } else {
                e
            }
        })?;

    if activities.is_empty() {
        return Ok("No screen activity recorded yet. Please ensure Memflow is recording.".to_string());
    }

    let act = &activities[0];
    use chrono::TimeZone;
    let dt = chrono::Local.timestamp_opt(act.timestamp, 0).unwrap();

    let ocr_content = act.ocr_text.as_ref()
        .map(|t| t.trim())
        .filter(|t| !t.is_empty())
        .unwrap_or("[No OCR text available]");

    // Truncate OCR content if too long (keep first 2000 chars)
    let ocr_display = if ocr_content.len() > 2000 {
        format!("{}... [truncated]", &ocr_content[..2000])
    } else {
        ocr_content.to_string()
    };

    let output = format!(
        "[Current Context @ {}]\nApp: {}\nTitle: {}\n---\nOCR Content:\n{}",
        dt.format("%Y-%m-%d %H:%M:%S"),
        act.app_name,
        act.window_title,
        ocr_display
    );

    Ok(output)
}

/// Get recent activity timeline (Phase 2: Real-time Perception)
async fn call_get_recent_activities(minutes: i64, limit: i64) -> Result<String> {
    info!("Getting recent activities: {} minutes, limit {}", minutes, limit);

    let activities = db::get_recent_activities_by_time(minutes, limit).await
        .map_err(|e| {
            let err_str = e.to_string();
            if err_str.contains("database is locked") {
                anyhow::anyhow!("Database is locked by another process. Please retry.")
            } else if err_str.contains("no such table") || err_str.contains("unable to open") {
                anyhow::anyhow!("Database not initialized. Please ensure Memflow app has run at least once.")
            } else {
                e
            }
        })?;

    if activities.is_empty() {
        return Ok(format!("No activities recorded in the last {} minutes.", minutes));
    }

    use chrono::TimeZone;
    let mut output = format!("[Activity Timeline - Last {} minutes]\n\n", minutes);

    for (idx, act) in activities.iter().enumerate() {
        let dt = chrono::Local.timestamp_opt(act.timestamp, 0).unwrap();
        
        // Get a short preview of OCR content (first 100 chars)
        let ocr_preview = act.ocr_text.as_ref()
            .map(|t| t.trim())
            .filter(|t| !t.is_empty())
            .map(|t| {
                if t.len() > 100 {
                    format!("{}...", &t[..100])
                } else {
                    t.to_string()
                }
            })
            .unwrap_or_else(|| "[No OCR]".to_string());

        output.push_str(&format!(
            "{}. [{}] {} - {}\n   OCR: {}\n\n",
            idx + 1,
            dt.format("%H:%M:%S"),
            act.app_name,
            act.window_title,
            ocr_preview.replace('\n', " ")
        ));
    }

    Ok(output)
}
