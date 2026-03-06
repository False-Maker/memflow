use anyhow::Result;
use memflow_core::ai::rag::HybridSearch;
use memflow_core::ai::{fallback_filter_params, FilterParams};
use memflow_core::context::RuntimeContext;
use memflow_core::{db, redact};
use serde_json::Value;
use tracing::{error, info};

/// Hard upper bound for `search_memory` result count.
/// The Tool Contract recommends small result sets (3–10); we clamp to this
/// maximum instead of erroring on large values to keep behavior robust.
const MAX_SEARCH_LIMIT: u64 = 50;

/// Handle `search_memory` tool call.
///
/// Currently implements a minimal version:
/// - Uses local embedding model via `memflow_core::ai::embedding`
/// - Runs hybrid / semantic / keyword search based on `mode`
/// - Applies basic filters (app / date_range / has_ocr)
/// - Returns a single text block suitable for MCP tool output
pub async fn handle_search_memory(
    ctx: &impl RuntimeContext,
    args: &Value,
) -> Result<Value> {
    // Validate and normalize `query`
    let raw_query = args
        .get("query")
        .and_then(|v| v.as_str())
        .ok_or_else(|| anyhow::anyhow!("MCP_INVALID_PARAMS: missing required field 'query'"))?;

    let query = raw_query.trim();
    if query.is_empty() {
        anyhow::bail!("MCP_INVALID_PARAMS: 'query' must be a non-empty string");
    }

    // Clamp `limit` to a reasonable range to avoid pathological requests
    let raw_limit = args
        .get("limit")
        .and_then(|v| v.as_u64())
        .unwrap_or(5);
    let clamped_limit = raw_limit.clamp(1, MAX_SEARCH_LIMIT);
    let limit = clamped_limit as usize;

    let mode = args
        .get("mode")
        .and_then(|v| v.as_str())
        .unwrap_or("hybrid")
        .to_lowercase();

    // Explicit filter params from arguments
    let mut filters = FilterParams {
        app_name: args
            .get("app_name")
            .and_then(|v| v.as_str())
            .map(|s| s.to_string()),
        keywords: args
            .get("keywords")
            .and_then(|v| v.as_array())
            .map(|arr| {
                arr.iter()
                    .filter_map(|v| v.as_str().map(|s| s.to_string()))
                    .collect::<Vec<_>>()
            })
            .unwrap_or_default(),
        date_range: args
            .get("date_range")
            .and_then(|v| v.as_str())
            .map(|s| s.to_string()),
        has_ocr: args.get("has_ocr").and_then(|v| v.as_bool()),
    };

    // If some filters are not explicitly provided, try to infer them from query text
    if filters.date_range.is_none() || filters.has_ocr.is_none() {
        let inferred = fallback_filter_params(query);
        if filters.date_range.is_none() {
            filters.date_range = inferred.date_range;
        }
        if filters.has_ocr.is_none() {
            filters.has_ocr = inferred.has_ocr;
        }
        if filters.keywords.is_empty() {
            filters.keywords = inferred.keywords;
        }
    }

    info!(
        "search_memory: query='{}', mode='{}', limit={} (raw_limit={}), app_name={:?}, date_range={:?}, has_ocr={:?}",
        query,
        mode,
        limit,
        raw_limit,
        filters.app_name,
        filters.date_range,
        filters.has_ocr
    );

    // Resolve date_range into timestamp bounds
    let (from_ts, to_ts) = resolve_date_range(filters.date_range.as_deref());

    // Dispatch by mode
    let text_output = match mode.as_str() {
        "keyword" => {
            run_keyword_search(query, &filters, from_ts, to_ts, limit).await?
        }
        "semantic" => {
            run_semantic_search(ctx, query, &filters, from_ts, to_ts, limit).await?
        }
        // default / "hybrid"
        _ => {
            run_hybrid_search(ctx, query, &filters, from_ts, to_ts, limit).await?
        }
    };

    Ok(serde_json::json!({
        "content": [
            {
                "type": "text",
                "text": text_output
            }
        ]
    }))
}

/// Map a logical `date_range` string to Unix timestamp bounds (UTC seconds)
fn resolve_date_range(range: Option<&str>) -> (Option<i64>, Option<i64>) {
    use chrono::{Datelike, Local};

    let Some(r) = range else {
        return (None, None);
    };

    let now = Local::now();
    let today = now.date_naive();

    match r {
        "today" => {
            let start = today.and_hms_opt(0, 0, 0).unwrap();
            (Some(start.and_utc().timestamp()), Some(now.timestamp()))
        }
        "yesterday" => {
            let y = today.pred_opt().unwrap_or(today);
            let start = y.and_hms_opt(0, 0, 0).unwrap();
            let end = today.and_hms_opt(0, 0, 0).unwrap();
            (Some(start.and_utc().timestamp()), Some(end.and_utc().timestamp()))
        }
        "last_week" => {
            let start = (now - chrono::TimeDelta::days(7)).timestamp();
            (Some(start), Some(now.timestamp()))
        }
        "this_week" => {
            let weekday = today.weekday().num_days_from_monday() as i64;
            let start_date = today - chrono::Days::new(weekday as u64);
            let start = start_date.and_hms_opt(0, 0, 0).unwrap();
            (Some(start.and_utc().timestamp()), Some(now.timestamp()))
        }
        "this_month" => {
            let start_date =
                chrono::NaiveDate::from_ymd_opt(today.year(), today.month(), 1).unwrap();
            let start = start_date.and_hms_opt(0, 0, 0).unwrap();
            (Some(start.and_utc().timestamp()), Some(now.timestamp()))
        }
        _ => (None, None),
    }
}

/// Convert a Unix timestamp (seconds, UTC) into a local `DateTime<Local>`,
/// falling back to `Local::now()` if the timestamp is invalid or ambiguous.
fn ts_to_local(ts: i64) -> chrono::DateTime<chrono::Local> {
    use chrono::{Local, LocalResult, TimeZone};

    match Local.timestamp_opt(ts, 0) {
        LocalResult::Single(dt) => dt,
        _ => Local::now(),
    }
}

/// Keyword-only search using the FTS-backed `search_activities` helper.
async fn run_keyword_search(
    query: &str,
    filters: &FilterParams,
    from_ts: Option<i64>,
    to_ts: Option<i64>,
    limit: usize,
) -> Result<String> {
    let (activities, total) = db::search_activities(
        Some(query.to_string()),
        filters.app_name.clone(),
        from_ts,
        to_ts,
        filters.has_ocr,
        Some(limit as i64),
        None,
        Some("rank".to_string()),
    )
    .await?;

    if activities.is_empty() {
        return Ok(format!("No matching results found for query: '{}'.", query));
    }

    let mut out = String::new();

    out.push_str(&format!(
        "Keyword search results for '{}' (total {}, showing up to {}):\n\n",
        query,
        total,
        activities.len()
    ));

    for act in activities {
        let dt = ts_to_local(act.timestamp);

        let title = redact::redact_secrets(&act.window_title);
        let ocr_snippet = act
            .ocr_text
            .as_deref()
            .unwrap_or("")
            .trim();
        let ocr_snippet = if ocr_snippet.is_empty() {
            String::from("(no OCR text)")
        } else {
            let redacted = redact::redact_secrets(ocr_snippet);
            let max_len = 200;
            if redacted.chars().count() > max_len {
                redacted.chars().take(max_len).collect::<String>() + "…"
            } else {
                redacted
            }
        };

        out.push_str(&format!(
            "ID: {} | Time: {} | App: {} | Title: {}\nContent: {}\n---\n",
            act.id,
            dt.format("%Y-%m-%d %H:%M:%S"),
            act.app_name,
            title,
            ocr_snippet,
        ));
    }

    Ok(out)
}

/// Semantic-only search using vector similarity; BM25 is not consulted.
async fn run_semantic_search(
    ctx: &impl RuntimeContext,
    query: &str,
    filters: &FilterParams,
    from_ts: Option<i64>,
    to_ts: Option<i64>,
    limit: usize,
) -> Result<String> {
    // Prefer local embedding model; fall back to deterministic placeholder
    let embedding = match memflow_core::ai::embedding::embed_with_local_model(ctx, query) {
        Ok(vec) => vec,
        Err(e) => {
            error!(
                "Embedding generation failed in semantic mode, falling back to placeholder: {}",
                e
            );
            memflow_core::vector_db::generate_placeholder_embedding(query)
        }
    };

    let vector_results =
        memflow_core::vector_db::search_similar(embedding, limit * 4).await?;

    if vector_results.is_empty() {
        return Ok(format!(
            "No semantic results found for query: '{}'.",
            query
        ));
    }

    // Apply filters at the activity level
    let mut out = String::new();

    out.push_str(&format!(
        "Semantic search results for '{}' (top {}):\n\n",
        query, limit
    ));

    let mut written = 0usize;
    for res in vector_results {
        if written >= limit {
            break;
        }

        let Ok(act) = db::get_activity_by_id(res.id).await else {
            continue;
        };

        if !activity_matches_filters(&act, filters, from_ts, to_ts) {
            continue;
        }

        let dt = ts_to_local(act.timestamp);

        let title = redact::redact_secrets(&act.window_title);
        let ocr_snippet = act
            .ocr_text
            .as_deref()
            .unwrap_or("")
            .trim();
        let ocr_snippet = if ocr_snippet.is_empty() {
            String::from("(no OCR text)")
        } else {
            let redacted = redact::redact_secrets(ocr_snippet);
            let max_len = 200;
            if redacted.chars().count() > max_len {
                redacted.chars().take(max_len).collect::<String>() + "…"
            } else {
                redacted
            }
        };

        out.push_str(&format!(
            "ID: {} | Score: {:.2}\nTime: {} | App: {} | Title: {}\nContent: {}\n---\n",
            act.id,
            res.score,
            dt.format("%Y-%m-%d %H:%M:%S"),
            act.app_name,
            title,
            ocr_snippet,
        ));

        written += 1;
    }

    if written == 0 {
        Ok(format!(
            "Semantic search found results, but none matched the current filters for query: '{}'.",
            query
        ))
    } else {
        Ok(out)
    }
}

/// Default hybrid search using `HybridSearch`, with filters applied after ranking.
async fn run_hybrid_search(
    ctx: &impl RuntimeContext,
    query: &str,
    filters: &FilterParams,
    from_ts: Option<i64>,
    to_ts: Option<i64>,
    limit: usize,
) -> Result<String> {
    // Prefer local embedding model; fall back to deterministic placeholder
    let embedding = match memflow_core::ai::embedding::embed_with_local_model(ctx, query) {
        Ok(vec) => vec,
        Err(e) => {
            error!(
                "Embedding generation failed in hybrid mode, falling back to placeholder: {}",
                e
            );
            memflow_core::vector_db::generate_placeholder_embedding(query)
        }
    };

    let searcher = HybridSearch::new();
    let results = searcher
        .search_with_embedding(query, embedding, limit * 4)
        .await?;

    if results.is_empty() {
        return Ok(format!(
            "No hybrid search results found for query: '{}'.",
            query
        ));
    }

    let mut out = String::new();

    out.push_str(&format!(
        "Hybrid search results for '{}' (top {}):\n\n",
        query, limit
    ));

    let mut written = 0usize;
    for res in results {
        if written >= limit {
            break;
        }

        let Ok(act) = db::get_activity_by_id(res.id).await else {
            continue;
        };

        if !activity_matches_filters(&act, filters, from_ts, to_ts) {
            continue;
        }

        let dt = ts_to_local(act.timestamp);

        let title = redact::redact_secrets(&act.window_title);
        let ocr_snippet = act
            .ocr_text
            .as_deref()
            .unwrap_or("")
            .trim();
        let ocr_snippet = if ocr_snippet.is_empty() {
            String::from("(no OCR text)")
        } else {
            let redacted = redact::redact_secrets(ocr_snippet);
            let max_len = 200;
            if redacted.chars().count() > max_len {
                redacted.chars().take(max_len).collect::<String>() + "…"
            } else {
                redacted
            }
        };

        out.push_str(&format!(
            "ID: {} | Score: {:.2}\nTime: {} | App: {} | Title: {}\nContent: {}\n---\n",
            act.id,
            res.score,
            dt.format("%Y-%m-%d %H:%M:%S"),
            act.app_name,
            title,
            ocr_snippet,
        ));

        written += 1;
    }

    if written == 0 {
        Ok(format!(
            "Hybrid search found results, but none matched the current filters for query: '{}'.",
            query
        ))
    } else {
        Ok(out)
    }
}

/// Apply app / time / OCR filters to an activity.
fn activity_matches_filters(
    act: &memflow_core::db::ActivityLog,
    filters: &FilterParams,
    from_ts: Option<i64>,
    to_ts: Option<i64>,
) -> bool {
    if let Some(app) = &filters.app_name {
        let app_lower = app.to_lowercase();
        let mut name = act.app_name.to_lowercase();
        if let Some(stripped) = name.strip_suffix(".exe") {
            name = stripped.to_string();
        }
        if !name.contains(&app_lower) {
            return false;
        }
    }

    if let Some(from) = from_ts {
        if act.timestamp < from {
            return false;
        }
    }
    if let Some(to) = to_ts {
        if act.timestamp > to {
            return false;
        }
    }

    if let Some(has_ocr) = filters.has_ocr {
        let has = act
            .ocr_text
            .as_ref()
            .map(|s| !s.trim().is_empty())
            .unwrap_or(false);
        if has_ocr != has {
            return false;
        }
    }

    true
}

/// Stub handlers for planned tools from MCP Tool Contract v1.
///
/// These are intentionally left unimplemented for now but provide a clear
/// structure and dedicated error codes so clients can distinguish them.
pub async fn handle_get_recent_activity(_ctx: &impl RuntimeContext, _args: &Value) -> Result<Value> {
    // minutes: 默认 5，最大 30，最小 1
    let minutes = _args
        .get("minutes")
        .and_then(|v| v.as_u64())
        .unwrap_or(5)
        .clamp(1, 30);

    // limit: 默认 50，最小 1，最大 200（避免一次性返回过多活动）
    let limit = _args
        .get("limit")
        .and_then(|v| v.as_u64())
        .unwrap_or(50)
        .clamp(1, 200);

    let now = chrono::Utc::now().timestamp();
    let from_ts = now - (minutes as i64) * 60;

    // 直接使用 search_activities 的时间过滤能力
    let (activities, total) = db::search_activities(
        None,
        None,
        Some(from_ts),
        Some(now),
        None,
        Some(limit as i64),
        None,
        None,
    )
    .await?;

    if activities.is_empty() {
        return Ok(serde_json::json!({
            "content": [
                {
                    "type": "text",
                    "text": format!("最近 {} 分钟内没有记录到任何活动。", minutes)
                }
            ]
        }));
    }

    let mut output = String::new();
    output.push_str(&format!(
        "最近 {} 分钟内共记录到 {} 条活动（本次返回前 {} 条）：\n\n",
        minutes,
        total,
        activities.len()
    ));

    for act in activities {
        let dt = ts_to_local(act.timestamp);

        let title = redact::redact_secrets(&act.window_title);
        let ocr_snippet = act
            .ocr_text
            .as_deref()
            .unwrap_or("")
            .trim();
        let ocr_snippet = if ocr_snippet.is_empty() {
            String::from("(无 OCR 文本)")
        } else {
            let redacted = redact::redact_secrets(ocr_snippet);
            let max_len = 200;
            if redacted.chars().count() > max_len {
                redacted.chars().take(max_len).collect::<String>() + "…"
            } else {
                redacted
            }
        };

        output.push_str(&format!(
            "- 时间：{}\n  应用：{}\n  标题：{}\n  OCR 摘要：{}\n\n",
            dt.format("%Y-%m-%d %H:%M:%S"),
            act.app_name,
            title,
            ocr_snippet,
        ));
    }

    Ok(serde_json::json!({
        "content": [
            {
                "type": "text",
                "text": output
            }
        ]
    }))
}

pub async fn handle_get_active_window_context(_ctx: &impl RuntimeContext, _args: &Value) -> Result<Value> {
    // 直接取最近一条 activity 作为“当前窗口”近似
    let activities = db::get_activities(1).await?;

    if activities.is_empty() {
        return Ok(serde_json::json!({
            "content": [
                {
                    "type": "text",
                    "text": "当前没有任何已记录的活动，无法推断活跃窗口。"
                }
            ]
        }));
    }

    let act = &activities[0];
    let dt = ts_to_local(act.timestamp);

    let title = redact::redact_secrets(&act.window_title);
    let mut text = String::new();

    text.push_str("推断的当前活跃窗口上下文：\n\n");
    text.push_str(&format!(
        "- 时间：{}\n- 应用：{}\n- 标题：{}\n\n",
        dt.format("%Y-%m-%d %H:%M:%S"),
        act.app_name,
        title,
    ));

    if let Some(ocr) = &act.ocr_text {
        let redacted = redact::redact_secrets(ocr.trim());
        let max_len = 800;
        let snippet = if redacted.chars().count() > max_len {
            redacted.chars().take(max_len).collect::<String>() + "…"
        } else {
            redacted
        };
        text.push_str("相关 OCR 文本（已脱敏，可能部分截断）：\n");
        text.push_str(&snippet);
    } else {
        text.push_str("没有可用的 OCR 文本。");
    }

    Ok(serde_json::json!({
        "content": [
            {
                "type": "text",
                "text": text
            }
        ]
    }))
}

pub async fn handle_get_related_context(_ctx: &impl RuntimeContext, _args: &Value) -> Result<Value> {
    let query = _args
        .get("query")
        .and_then(|v| v.as_str())
        .ok_or_else(|| anyhow::anyhow!("MCP_INVALID_PARAMS: missing required field 'query'"))?
        .trim()
        .to_string();

    if query.is_empty() {
        anyhow::bail!("MCP_INVALID_PARAMS: 'query' must be a non-empty string");
    }

    // limit: 默认 5，最小 1，最大 20（避免过多片段）
    let limit = _args
        .get("limit")
        .and_then(|v| v.as_u64())
        .unwrap_or(5)
        .clamp(1, 20) as usize;

    // max_chars_per_item: 默认 400，最小 100，最大 2000（防止 prompt 过长）
    let max_chars_per_item = _args
        .get("max_chars_per_item")
        .and_then(|v| v.as_u64())
        .unwrap_or(400)
        .clamp(100, 2000) as usize;

    info!(
        "get_related_context: query='{}', limit={}, max_chars_per_item={}",
        query, limit, max_chars_per_item
    );

    let embedding = match memflow_core::ai::embedding::embed_with_local_model(_ctx, &query) {
        Ok(vec) => vec,
        Err(e) => {
            error!("Embedding generation failed, falling back to placeholder: {}", e);
            memflow_core::vector_db::generate_placeholder_embedding(&query)
        }
    };

    let searcher = HybridSearch::new();
    let results = searcher.search_with_embedding(&query, embedding, limit).await?;

    if results.is_empty() {
        return Ok(serde_json::json!({
            "content": [
                {
                    "type": "text",
                    "text": "没有找到与当前 query 相关的上下文片段。"
                }
            ]
        }));
    }

    let mut out = String::new();

    out.push_str("与当前 query 最相关的上下文片段（已脱敏，按相关度排序）：\n\n");

    for (idx, res) in results.iter().enumerate() {
            if let Ok(act) = db::get_activity_by_id(res.id).await {
                let dt = ts_to_local(act.timestamp);

            let title = redact::redact_secrets(&act.window_title);
            let raw_text = act.ocr_text.unwrap_or_default();
            let redacted = redact::redact_secrets(raw_text.trim());

            let snippet = if redacted.chars().count() > max_chars_per_item {
                redacted
                    .chars()
                    .take(max_chars_per_item)
                    .collect::<String>()
                    + "…"
            } else {
                redacted
            };

            out.push_str(&format!(
                "片段 #{}  (score = {:.2})\n时间：{}\n应用：{}\n标题：{}\n内容：\n{}\n\n---\n\n",
                idx + 1,
                res.score,
                dt.format("%Y-%m-%d %H:%M:%S"),
                act.app_name,
                title,
                snippet,
            ));
        }
    }

    Ok(serde_json::json!({
        "content": [
            {
                "type": "text",
                "text": out
            }
        ]
    }))
}

pub async fn handle_get_terminal_output(_ctx: &impl RuntimeContext, _args: &Value) -> Result<Value> {
    // 可选参数：limit 控制返回的终端日志条目数（每条通常是一段文本块，而非逐行）
    let limit = _args
        .get("limit")
        .and_then(|v| v.as_u64())
        .unwrap_or(20)
        .max(1) as i64;

    // 未来如果前端 / IDE 传入当前终端会话 ID，可以在这里读取：
    // let requested_session = _args
    //     .get("terminal_session_id")
    //     .and_then(|v| v.as_str())
    //     .map(|s| s.to_string());

    let logs = match db::get_recent_terminal_output(limit).await {
        Ok(v) => v,
        Err(e) => {
            let msg = e.to_string();
            // 尝试将典型权限问题包装为标准 MCP 错误标记，交由 server.rs 映射为 -32005
            if msg.to_lowercase().contains("permission denied")
                || msg.contains("access is denied")
            {
                anyhow::bail!("MCP_PERMISSION_DENIED: failed to read terminal logs: {msg}");
            } else {
                anyhow::bail!("MCP_INTERNAL: failed to read terminal logs: {msg}");
            }
        }
    };

    if logs.is_empty() {
        // 当前实现基于最近的终端日志来近似“当前终端会话”；
        // 若完全没有日志，则视为“终端未找到”，交由 server.rs 映射为 -32004。
        anyhow::bail!(
            "MCP_TERMINAL_NOT_FOUND: no recent terminal output found in local database"
        );
    }

    let mut text = String::new();
    text.push_str("最近终端输出（已脱敏，按时间排序，最新在前）：\n\n");

    for log in logs {
        let dt = ts_to_local(log.timestamp);

        let session = log
            .terminal_session_id
            .as_deref()
            .unwrap_or("<unknown-session>");
        let app = log.app_name.as_deref().unwrap_or("<unknown-app>");
        let title = log
            .window_title
            .as_deref()
            .map(|t| redact::redact_secrets(t))
            .unwrap_or_else(|| "<no-title>".to_string());

        let raw_text = log.text.trim();
        let redacted = redact::redact_secrets(raw_text);
        let max_len = 2000usize;
        let snippet = if redacted.chars().count() > max_len {
            redacted.chars().take(max_len).collect::<String>() + "…"
        } else {
            redacted
        };

        text.push_str(&format!(
            "[{}] session={} app={} title={}\n{}\n\n---\n\n",
            dt.format("%Y-%m-%d %H:%M:%S"),
            session,
            app,
            title,
            snippet,
        ));
    }

    Ok(serde_json::json!({
        "content": [
            {
                "type": "text",
                "text": text
            }
        ]
    }))
}

pub async fn handle_get_system_environment(_ctx: &impl RuntimeContext, _args: &Value) -> Result<Value> {
    // 统一复用 memflow-core 的系统环境探测逻辑，确保 MCP 与桌面端一致。
    let text = memflow_core::system_env::get_system_environment_report().await;

    Ok(serde_json::json!({
        "content": [
            {
                "type": "text",
                "text": text
            }
        ]
    }))
}

