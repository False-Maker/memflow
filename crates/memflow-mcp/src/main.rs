use anyhow::{Context, Result};
use clap::Parser;
use fastembed::{InitOptions, TextEmbedding, EmbeddingModel};
use memflow_core::ai::{fallback_filter_params, FilterParams};
use memflow_core::ai::nlp;
use memflow_core::ai::rag::HybridSearch;
use memflow_core::context::RuntimeContext;
use memflow_core::db;
use memflow_core::vector_db;
use serde::{Deserialize, Serialize};
use serde_json::Value;
use std::io::{self};
use std::collections::HashMap;
use tokio::io::{AsyncBufReadExt, BufReader};
use tracing::{error, info};
use tracing_subscriber::{fmt, EnvFilter};
use std::sync::OnceLock;

mod context;
use memflow_mcp::prompts;
use context::McpContext;

// Global model instance
static EMBEDDING_MODEL: OnceLock<std::sync::Mutex<TextEmbedding>> = OnceLock::new();
static MCP_AUTH_TOKEN: OnceLock<Option<String>> = OnceLock::new();
static MCP_READ_ONLY: OnceLock<bool> = OnceLock::new();

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
    let auth_token = std::env::var("MEMFLOW_MCP_TOKEN").ok();
    let read_only = std::env::var("MEMFLOW_MCP_READ_ONLY")
        .ok()
        .map(|v| v != "0" && v.to_lowercase() != "false")
        .unwrap_or(true);
    let _ = MCP_AUTH_TOKEN.set(auth_token);
    let _ = MCP_READ_ONLY.set(read_only);
    
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
                    "version": "0.1.0",
                    "authRequired": MCP_AUTH_TOKEN.get().and_then(|v| v.as_ref()).is_some(),
                    "readOnly": *MCP_READ_ONLY.get().unwrap_or(&true)
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
                        "description": "Search user's recorded memory with keyword/semantic/hybrid strategies. Returns OCR text, app names, and timestamps from past activities.",
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
                        }
                    },
                    {
                        "name": "get_recent_activity",
                        "description": "Get the user's recent activity timeline. Use this to understand what happened in the last few minutes.",
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
                    },
                    {
                        "name": "get_related_context",
                        "description": "Return compact context chunks related to the query for downstream LLM reasoning.",
                        "inputSchema": {
                            "type": "object",
                            "properties": {
                                "query": {
                                    "type": "string",
                                    "description": "User query to find related context."
                                },
                                "limit": {
                                    "type": "integer",
                                    "description": "Max number of context items (default: 5)."
                                },
                                "max_chars_per_item": {
                                    "type": "integer",
                                    "description": "Max chars of OCR per item (default: 1200)."
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
            if !is_authorized(&req.params) {
                return Ok(Some(JsonRpcResponse::error(id, -32001, "Unauthorized".to_string())));
            }
            let params = req.params.context("Missing params")?;
            let name = params["name"].as_str().context("Missing tool name")?;
            if is_read_only() && is_write_tool(name) {
                return Ok(Some(JsonRpcResponse::error(id, -32003, "Read-only mode".to_string())));
            }
            let args = &params["arguments"];

            match name {
                "search_memory" => {
                    let parsed: SearchMemoryArgs = serde_json::from_value(args.clone())
                        .map_err(|e| anyhow::anyhow!("Invalid arguments: {}", e))?;
                    match call_search_memory(parsed).await {
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
                "get_recent_activity" => {
                    let parsed: RecentActivityArgs = serde_json::from_value(args.clone())
                        .unwrap_or_default();
                    let minutes = parsed.minutes.unwrap_or(5);
                    let limit = parsed.limit.unwrap_or(20);

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
                "get_related_context" => {
                    let parsed: RelatedContextArgs = serde_json::from_value(args.clone())
                        .map_err(|e| anyhow::anyhow!("Invalid arguments: {}", e))?;
                    match call_get_related_context(parsed).await {
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
                            error!("Get related context failed: {}", e);
                            Ok(Some(JsonRpcResponse::error(id, -32000, e.to_string())))
                        }
                    }
                }
                "search_visual_memory" => {
                    let mut parsed: SearchMemoryArgs = serde_json::from_value(args.clone())
                        .map_err(|e| anyhow::anyhow!("Invalid arguments: {}", e))?;
                    if parsed.mode.is_none() {
                        parsed.mode = Some("hybrid".to_string());
                    }
                    match call_search_memory(parsed).await {
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
                "get_recent_activities" => {
                    let parsed: RecentActivityArgs = serde_json::from_value(args.clone())
                        .unwrap_or_default();
                    let minutes = parsed.minutes.unwrap_or(5);
                    let limit = parsed.limit.unwrap_or(20);
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

#[derive(Debug, Default, Deserialize)]
struct SearchMemoryArgs {
    query: Option<String>,
    limit: Option<usize>,
    mode: Option<String>,
    app_name: Option<String>,
    keywords: Option<Vec<String>>,
    date_range: Option<String>,
    has_ocr: Option<bool>,
}

#[derive(Debug, Default, Deserialize)]
struct RecentActivityArgs {
    minutes: Option<i64>,
    limit: Option<i64>,
}

#[derive(Debug, Deserialize)]
struct RelatedContextArgs {
    query: String,
    limit: Option<usize>,
    max_chars_per_item: Option<usize>,
}

#[derive(Clone)]
struct ActivityHit {
    activity: db::ActivityLog,
    score: Option<f64>,
}

fn is_authorized(params: &Option<Value>) -> bool {
    let required = MCP_AUTH_TOKEN
        .get()
        .and_then(|v| v.as_ref())
        .is_some();
    if !required {
        return true;
    }
    let token = MCP_AUTH_TOKEN.get().and_then(|v| v.clone()).flatten();
    let provided = params.as_ref().and_then(|v| {
        v.get("authToken")
            .and_then(|t| t.as_str())
            .map(|s| s.to_string())
            .or_else(|| {
                v.get("meta")
                    .and_then(|m| m.get("authToken"))
                    .and_then(|t| t.as_str())
                    .map(|s| s.to_string())
            })
    });
    token.is_some() && token == provided
}

fn is_read_only() -> bool {
    *MCP_READ_ONLY.get().unwrap_or(&true)
}

fn is_write_tool(name: &str) -> bool {
    matches!(name, "write_memory" | "upsert_memory" | "delete_memory" | "create_memory")
        || name.starts_with("write_")
        || name.starts_with("delete_")
        || name.starts_with("update_")
}

fn normalize_limit(limit: Option<usize>, default: usize, max: usize) -> usize {
    let val = limit.unwrap_or(default).max(1);
    val.min(max)
}

fn parse_date_range(range: Option<&str>) -> (Option<i64>, Option<i64>) {
    use chrono::{Datelike, Duration, Local, TimeZone, Weekday};
    let now = Local::now();
    let today = now.date_naive();
    let (start, end) = match range {
        Some("today") => {
            let start = today.and_hms_opt(0, 0, 0);
            let end = today.and_hms_opt(23, 59, 59);
            (start, end)
        }
        Some("yesterday") => {
            let day = today - Duration::days(1);
            let start = day.and_hms_opt(0, 0, 0);
            let end = day.and_hms_opt(23, 59, 59);
            (start, end)
        }
        Some("this_week") => {
            let weekday = today.weekday();
            let days_from_monday = match weekday {
                Weekday::Mon => 0,
                Weekday::Tue => 1,
                Weekday::Wed => 2,
                Weekday::Thu => 3,
                Weekday::Fri => 4,
                Weekday::Sat => 5,
                Weekday::Sun => 6,
            };
            let start_day = today - Duration::days(days_from_monday);
            let start = start_day.and_hms_opt(0, 0, 0);
            (start, Some(now.naive_local()))
        }
        Some("last_week") => {
            let weekday = today.weekday();
            let days_from_monday = match weekday {
                Weekday::Mon => 0,
                Weekday::Tue => 1,
                Weekday::Wed => 2,
                Weekday::Thu => 3,
                Weekday::Fri => 4,
                Weekday::Sat => 5,
                Weekday::Sun => 6,
            };
            let this_week_start = today - Duration::days(days_from_monday);
            let last_week_start = this_week_start - Duration::days(7);
            let last_week_end = this_week_start - Duration::seconds(1);
            let start = last_week_start.and_hms_opt(0, 0, 0);
            let end = last_week_end.and_hms_opt(23, 59, 59);
            (start, end)
        }
        Some("this_month") => {
            let start = today.with_day(1).and_then(|d| d.and_hms_opt(0, 0, 0));
            (start, Some(now.naive_local()))
        }
        _ => (None, None),
    };
    let start_ts = start.and_then(|s| Local.from_local_datetime(&s).single()).map(|dt| dt.timestamp());
    let end_ts = end.and_then(|e| Local.from_local_datetime(&e).single()).map(|dt| dt.timestamp());
    (start_ts, end_ts)
}

fn truncate_text(text: &str, max_chars: usize) -> String {
    let count = text.chars().count();
    if count <= max_chars {
        return text.to_string();
    }
    text.chars().take(max_chars).collect::<String>() + "..."
}

fn build_filter_params(args: &SearchMemoryArgs) -> FilterParams {
    let mut params = if let Some(ref query) = args.query {
        fallback_filter_params(query)
    } else {
        FilterParams::default()
    };
    if let Some(ref app) = args.app_name {
        params.app_name = Some(app.clone());
    }
    if let Some(ref keywords) = args.keywords {
        params.keywords = keywords.clone();
    }
    if let Some(ref range) = args.date_range {
        params.date_range = Some(range.clone());
    }
    if let Some(has_ocr) = args.has_ocr {
        params.has_ocr = Some(has_ocr);
    }
    params
}

async fn generate_query_embedding(query: &str) -> Result<Vec<f32>> {
    if let Some(model_lock) = EMBEDDING_MODEL.get() {
        let mut model = model_lock.lock().map_err(|_| anyhow::anyhow!("Embedding model is busy. Please retry."))?;
        let embeddings = model.embed(vec![query], None)?;
        if let Some(vec) = embeddings.into_iter().next() {
            return Ok(vec);
        }
        return Err(anyhow::anyhow!("Failed to generate embedding: empty result"));
    }
    Ok(vector_db::generate_placeholder_embedding(query))
}

async fn fetch_candidate_ids(
    search_query: Option<String>,
    params: &FilterParams,
    from_ts: Option<i64>,
    to_ts: Option<i64>,
    limit: usize,
) -> Result<Vec<i64>> {
    let order_by = if search_query.is_some() { Some("rank".to_string()) } else { Some("time".to_string()) };
    let (activities, _) = db::search_activities(
        search_query,
        params.app_name.clone(),
        from_ts,
        to_ts,
        params.has_ocr,
        Some(limit as i64),
        Some(0),
        order_by,
    )
    .await?;
    Ok(activities.into_iter().map(|a| a.id).collect())
}

async fn load_hits_from_vector(results: Vec<vector_db::SearchResult>) -> Vec<ActivityHit> {
    let mut hits = Vec::new();
    for res in results {
        if let Ok(act) = db::get_activity_by_id(res.id).await {
            hits.push(ActivityHit {
                activity: act,
                score: Some(res.score),
            });
        }
    }
    hits
}

async fn load_hits_from_hybrid(results: Vec<memflow_core::ai::rag::HybridSearchResult>) -> Vec<ActivityHit> {
    let mut hits = Vec::new();
    for res in results {
        if let Ok(act) = db::get_activity_by_id(res.id).await {
            hits.push(ActivityHit {
                activity: act,
                score: Some(res.score),
            });
        }
    }
    hits
}

async fn search_memory_internal(args: SearchMemoryArgs) -> Result<Vec<ActivityHit>> {
    let limit = normalize_limit(args.limit, 5, 50);
    let params = build_filter_params(&args);
    let query_text = args.query.clone().unwrap_or_default();
    let search_query = if !params.keywords.is_empty() {
        params.keywords.join(" OR ")
    } else {
        query_text.clone()
    };
    if search_query.trim().is_empty() {
        return Err(anyhow::anyhow!("Query is required."));
    }
    let (from_ts, to_ts) = parse_date_range(params.date_range.as_deref());
    let has_filters = params.app_name.is_some() || params.has_ocr.is_some() || params.date_range.is_some();
    let mode = args.mode.clone().unwrap_or_else(|| "hybrid".to_string()).to_lowercase();

    if mode == "keyword" {
        let (activities, _) = db::search_activities(
            Some(search_query),
            params.app_name.clone(),
            from_ts,
            to_ts,
            params.has_ocr,
            Some(limit as i64),
            Some(0),
            Some("rank".to_string()),
        )
        .await?;
        return Ok(activities.into_iter().map(|a| ActivityHit { activity: a, score: None }).collect());
    }

    let embedding = generate_query_embedding(&search_query).await?;

    if mode == "semantic" {
        let results = if has_filters {
            let candidate_ids = fetch_candidate_ids(Some(search_query), &params, from_ts, to_ts, (limit * 10).max(50)).await?;
            if candidate_ids.is_empty() {
                return Ok(Vec::new());
            }
            vector_db::search_similar_with_candidates(embedding, limit * 2, Some(&candidate_ids)).await?
        } else {
            vector_db::search_similar(embedding, limit * 2).await?
        };
        let mut hits = load_hits_from_vector(results).await;
        hits.truncate(limit);
        return Ok(hits);
    }

    if has_filters {
        let candidate_ids = fetch_candidate_ids(Some(search_query), &params, from_ts, to_ts, (limit * 10).max(50)).await?;
        if candidate_ids.is_empty() {
            return Ok(Vec::new());
        }
        let results = vector_db::search_similar_with_candidates(embedding, limit * 2, Some(&candidate_ids)).await?;
        let mut hits = load_hits_from_vector(results).await;
        hits.truncate(limit);
        return Ok(hits);
    }

    let searcher = HybridSearch::new();
    let results = searcher.search_with_embedding(&search_query, embedding, limit).await
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
    let mut hits = load_hits_from_hybrid(results).await;
    hits.truncate(limit);
    Ok(hits)
}

async fn call_search_memory(args: SearchMemoryArgs) -> Result<String> {
    info!("Searching memory with args: {:?}", args);
    let hits = search_memory_internal(args).await?;
    if hits.is_empty() {
        return Ok("No matching results found.".to_string());
    }

    let mut output = String::new();
    for hit in hits {
        let act = hit.activity;
        use chrono::TimeZone;
        let dt = chrono::Local.timestamp_opt(act.timestamp, 0).unwrap();
        if let Some(score) = hit.score {
            output.push_str(&format!(
                "ID: {} | Time: {} | App: {} | Title: {}\nScore: {:.2}\nContent: {}\n---\n",
                act.id,
                dt.format("%Y-%m-%d %H:%M:%S"),
                act.app_name,
                act.window_title,
                score,
                act.ocr_text.unwrap_or_default().trim()
            ));
        } else {
            output.push_str(&format!(
                "ID: {} | Time: {} | App: {} | Title: {}\nContent: {}\n---\n",
                act.id,
                dt.format("%Y-%m-%d %H:%M:%S"),
                act.app_name,
                act.window_title,
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

async fn call_get_related_context(args: RelatedContextArgs) -> Result<String> {
    let limit = normalize_limit(args.limit, 5, 20);
    let max_chars = args.max_chars_per_item.unwrap_or(1200).max(200);
    let query = args.query.trim().to_string();
    if query.is_empty() {
        return Err(anyhow::anyhow!("Query is required."));
    }
    let mut search_args = SearchMemoryArgs::default();
    search_args.query = Some(query.clone());
    search_args.limit = Some(limit);
    search_args.mode = Some("hybrid".to_string());

    let hits = search_memory_internal(search_args).await?;
    if hits.is_empty() {
        return Ok("No related context found.".to_string());
    }

    let keywords = nlp::extract_keywords(&query, None);
    let entities = nlp::extract_named_entities(&query);

    let mut output = String::new();
    output.push_str("[Related Context]\n");
    output.push_str(&format!("Query: {}\n", query));
    if !keywords.is_empty() {
        output.push_str(&format!("Keywords: {}\n", keywords.join(", ")));
    }
    if !entities.is_empty() {
        output.push_str(&format!("Entities: {}\n", entities.join(", ")));
    }
    output.push('\n');

    for (idx, hit) in hits.into_iter().enumerate() {
        let act = hit.activity;
        use chrono::TimeZone;
        let dt = chrono::Local.timestamp_opt(act.timestamp, 0).unwrap();
        output.push_str(&format!(
            "{}. [{}] {} - {}\n",
            idx + 1,
            dt.format("%Y-%m-%d %H:%M:%S"),
            act.app_name,
            act.window_title
        ));
        if let Some(score) = hit.score {
            output.push_str(&format!("Score: {:.2}\n", score));
        }
        let ocr_content = act.ocr_text.unwrap_or_default();
        let cleaned = ocr_content.trim();
        let display = if cleaned.is_empty() {
            "[No OCR]".to_string()
        } else {
            truncate_text(cleaned, max_chars)
        };
        output.push_str(&format!("Content: {}\n\n", display.replace('\n', " ")));
    }

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
