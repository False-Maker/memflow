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
use context::McpContext;

// Global model instance
static EMBEDDING_MODEL: OnceLock<TextEmbedding> = OnceLock::new();

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

    // Initialize Embedding Model
    info!("Initializing Embedding Model (BGESmallENV15)...");
    let model_opts = InitOptions::new(EmbeddingModel::BGESmallENV15)
        .with_cache_dir(resource_dir.join("models"))
        .with_show_download_progress(false);

    match TextEmbedding::try_new(model_opts) {
        Ok(model) => {
            if EMBEDDING_MODEL.set(model).is_err() {
                error!("Failed to set global embedding model");
            } else {
                info!("Embedding Model initialized successfully.");
            }
        },
        Err(e) => {
            error!("Failed to initialize Embedding Model: {}", e);
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
                    "tools": {}
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
                        "name": "search_memory",
                        "description": "Search semantic memory for relevant information based on a query.",
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
                    }
                ]
            });
            Ok(Some(JsonRpcResponse::ok(id, tools)))
        }
        "tools/call" => {
            let params = req.params.context("Missing params")?;
            let name = params["name"].as_str().context("Missing tool name")?;
            let args = &params["arguments"];

            if name == "search_memory" {
                let query = args["query"].as_str().context("Missing query argument")?;
                let limit = args["limit"].as_u64().unwrap_or(5) as usize;

                match call_search_memory(query, limit).await {
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
            } else {
                 Ok(Some(JsonRpcResponse::error(id, -32601, format!("Tool not found: {}", name))))
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

async fn call_search_memory(query: &str, limit: usize) -> Result<String> {
    info!("Searching for: {} (limit: {})", query, limit);

    // Check if model is available
    let embedding = if let Some(model) = EMBEDDING_MODEL.get() {
        info!("Generating embedding for query...");
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
    let results = searcher.search_with_embedding(query, embedding, limit).await?;

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
