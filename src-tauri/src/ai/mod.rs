pub mod provider;
pub mod rag;

use crate::ai::provider::{chat_with_anthropic, chat_with_openai, ProviderConfig};
use crate::ai::rag::HybridSearch;
use crate::vector_db;
use anyhow::Result;
use chrono::{Duration, Local, TimeZone};

pub async fn analyze_activity(activity_id: i64) -> Result<String> {
    // 1. 获取活动信息
    let activity = crate::db::get_activity_by_id(activity_id).await?;

    // 2. 生成嵌入
    let text = activity.ocr_text.unwrap_or_default();
    if text.is_empty() {
        return Ok("活动无 OCR 文本，无法分析".to_string());
    }

    let embedding = vector_db::generate_embedding(&text).await?;

    // 3. 保存嵌入
    vector_db::insert_embedding(activity_id, embedding).await?;

    // 4. 简单的分析（实际应该调用 LLM）
    Ok(format!(
        "活动分析：应用 {} 在窗口 {} 中进行了操作",
        activity.app_name, activity.window_title
    ))
}

fn calculate_timestamps(range: &str) -> (Option<i64>, Option<i64>) {
    let now = Local::now();
    let today_start = now
        .date_naive()
        .and_hms_opt(0, 0, 0)
        .unwrap()
        .and_local_timezone(Local)
        .unwrap()
        .timestamp();
    let today_end = now
        .date_naive()
        .and_hms_opt(23, 59, 59)
        .unwrap()
        .and_local_timezone(Local)
        .unwrap()
        .timestamp();

    match range {
        "today" | "今天" => (Some(today_start), Some(today_end)),
        "yesterday" | "昨天" => {
            let y = now - Duration::days(1);
            let start = y
                .date_naive()
                .and_hms_opt(0, 0, 0)
                .unwrap()
                .and_local_timezone(Local)
                .unwrap()
                .timestamp();
            let end = y
                .date_naive()
                .and_hms_opt(23, 59, 59)
                .unwrap()
                .and_local_timezone(Local)
                .unwrap()
                .timestamp();
            (Some(start), Some(end))
        }
        "this_week" | "本周" => {
            let start = (now - Duration::days(7)).timestamp();
            (Some(start), Some(today_end))
        }
        "last_week" | "上周" => {
            let start = (now - Duration::days(14)).timestamp();
            let end = (now - Duration::days(7)).timestamp();
            (Some(start), Some(end))
        }
        "this_month" | "本月" => {
            let start = (now - Duration::days(30)).timestamp();
            (Some(start), Some(today_end))
        }
        _ => (None, None),
    }
}

async fn build_context_from_range(
    query: &str,
    intent: &FilterParams,
) -> Result<(String, usize)> {
    let (from_ts, to_ts) = if let Some(range) = &intent.date_range {
        calculate_timestamps(range)
    } else {
        (None, None)
    };

    // 如果指定了时间范围，但没有解析出时间戳（即未知的时间描述），则回退到 HybridSearch
    if intent.date_range.is_some() && from_ts.is_none() {
        return Ok((String::new(), 0));
    }

    let search_query = if !intent.keywords.is_empty() {
        Some(intent.keywords.join(" OR "))
    } else {
        None
    };

    let (activities, _) = crate::db::search_activities(
        search_query,
        intent.app_name.clone(),
        from_ts,
        to_ts,
        intent.has_ocr,
        Some(50), // 增加上下文数量以支持总结
        None,
        Some("time".to_string()),
    )
    .await?;

    let mut context_text = String::new();
    let mut context_count = 0;

    for activity in activities {
        let time_str = Local.timestamp_opt(activity.timestamp, 0)
            .unwrap()
            .format("%H:%M:%S")
            .to_string();

        if let Some(ref ocr_text) = activity.ocr_text {
            if !ocr_text.trim().is_empty() {
                context_text.push_str(&format!(
                    "[{}] 应用: {} | 窗口: {}\n内容: {}\n\n",
                    time_str,
                    activity.app_name,
                    activity.window_title,
                    ocr_text.trim()
                ));
            } else {
                 context_text.push_str(&format!(
                    "[{}] 应用: {} | 窗口: {}\n\n",
                    time_str,
                    activity.app_name,
                    activity.window_title
                ));
            }
        } else {
             context_text.push_str(&format!(
                "[{}] 应用: {} | 窗口: {}\n\n",
                time_str,
                activity.app_name,
                activity.window_title
            ));
        }
        context_count += 1;
    }

    Ok((context_text, context_count))
}

pub async fn chat(query: &str, _context: Vec<i64>) -> Result<String> {
    // 1. 解析意图
    let intent = parse_query_intent(query).await.unwrap_or_else(|_| fallback_filter_params(query));
    
    // 2. 获取上下文
    let (mut context_text, mut context_count) = if intent.date_range.is_some() {
        match build_context_from_range(query, &intent).await {
             Ok((text, count)) if count > 0 => (text, count),
             _ => (String::new(), 0),
        }
    } else {
        (String::new(), 0)
    };

    // 如果还没有上下文（意图解析未返回时间范围，或者时间范围搜索为空），则使用混合检索
    if context_count == 0 {
         let searcher = HybridSearch::new();
         let results = searcher.search(query, 5).await?;
        
         for result in results {
            if let Ok(activity) = crate::db::get_activity_by_id(result.id).await {
                // 简单起见，这里复用 formatting 逻辑
                let time_str = Local.timestamp_opt(activity.timestamp, 0)
                    .unwrap()
                    .format("%Y-%m-%d %H:%M:%S")
                    .to_string();

                let mut has_content = false;
                if let Some(ref ocr_text) = activity.ocr_text {
                    if !ocr_text.trim().is_empty() {
                        context_text.push_str(&format!(
                            "[{}] 应用: {} | 窗口: {}\n内容: {}\n\n",
                            time_str,
                            activity.app_name,
                            activity.window_title,
                            ocr_text.trim()
                        ));
                        has_content = true;
                    }
                }
                if !has_content {
                    context_text.push_str(&format!(
                        "[{}] 应用: {} | 窗口: {}\n\n",
                        time_str,
                        activity.app_name, activity.window_title
                    ));
                }
                context_count += 1;
            }
        }
    }
    
    tracing::info!(
        "Chat Context: {} items, {} chars (Intent: DateRange={:?})",
        context_count,
        context_text.len(),
        intent.date_range
    );

    // 3. 尝试调用 LLM API
    let config = crate::app_config::get_config().await.unwrap_or_else(|_| {
        let mut cfg: crate::commands::AppConfig = serde_json::from_str("{}").unwrap();
        cfg.ocr_enabled = true;
        cfg
    });

    let model_id = &config.chat_model;

    // 根据模型名称自动判断提供商：如果以 "claude-" 开头则是 Anthropic，否则默认 OpenAI
    let is_anthropic = model_id.starts_with("claude-");

    if is_anthropic {
        // 尝试使用 Anthropic API
        match crate::secure_storage::get_api_key("anthropic").await {
            Ok(Some(api_key)) => {
                let provider_config = ProviderConfig::new(
                    api_key,
                    config.anthropic_base_url.clone(),
                    "https://api.anthropic.com",
                );

                match chat_with_anthropic(query, &context_text, model_id, &provider_config, None).await {
                    Ok(answer) => {
                        tracing::info!("使用 Anthropic API 生成回答，模型: {}", model_id);
                        return Ok(answer);
                    }
                    Err(e) => {
                        tracing::error!(
                            "Anthropic API 调用失败: {}",
                            crate::redact::redact_secrets(&e.to_string())
                        );
                        return Ok(format!(
                            "⚠️ Anthropic API 调用失败\n\n错误信息：{}\n\n请检查：\n1. API Key 是否有效\n2. 网络连接是否正常\n3. 模型名称是否正确（当前: {}）",
                            crate::redact::redact_secrets(&e.to_string()),
                            model_id
                        ));
                    }
                }
            }
            Ok(None) => {
                tracing::warn!("未配置 Anthropic API Key");
                return Ok(format!(
                    "⚠️ 未配置 Anthropic API Key\n\n当前选择的模型是 {}，需要 Anthropic API Key。\n\n请在「设置」中配置 Anthropic API Key 后再试。",
                    model_id
                ));
            }
            Err(e) => {
                tracing::error!("获取 Anthropic API Key 失败: {}", e);
                return Ok(format!(
                    "⚠️ 获取 API Key 失败：{}",
                    crate::redact::redact_secrets(&e.to_string())
                ));
            }
        }
    } else {
        // 默认使用 OpenAI API
        match crate::secure_storage::get_api_key("openai").await {
            Ok(Some(api_key)) => {
                let provider_config = ProviderConfig::new(
                    api_key,
                    config.openai_base_url.clone(),
                    "https://api.openai.com/v1",
                );

                match chat_with_openai(query, &context_text, model_id, &provider_config, None).await {
                    Ok(answer) => {
                        tracing::info!("使用 OpenAI API 生成回答，模型: {}", model_id);
                        return Ok(answer);
                    }
                    Err(e) => {
                        tracing::error!(
                            "OpenAI API 调用失败: {}",
                            crate::redact::redact_secrets(&e.to_string())
                        );
                        return Ok(format!(
                            "⚠️ OpenAI API 调用失败\n\n错误信息：{}\n\n请检查：\n1. API Key 是否有效\n2. 网络连接是否正常\n3. 模型名称是否正确（当前: {}）\n4. 如果使用自定义 Base URL，请确认地址正确",
                            crate::redact::redact_secrets(&e.to_string()),
                            model_id
                        ));
                    }
                }
            }
            Ok(None) => {
                tracing::warn!("未配置 OpenAI API Key");
                return Ok(format!(
                    "⚠️ 未配置 OpenAI API Key\n\n当前选择的模型是 {}，需要 OpenAI API Key。\n\n请在「设置」中配置 OpenAI API Key 后再试。",
                    model_id
                ));
            }
            Err(e) => {
                tracing::error!("获取 OpenAI API Key 失败: {}", e);
                return Ok(format!(
                    "⚠️ 获取 API Key 失败：{}",
                    crate::redact::redact_secrets(&e.to_string())
                ));
            }
        }
    }
}


pub async fn chat_stream<F>(query: &str, _context: Vec<i64>, on_chunk: F) -> Result<()>
where
    F: Fn(String) + Send + Sync + 'static,
{
    // 1. 解析意图 (Time Awareness)
    let intent = parse_query_intent(query).await.unwrap_or_else(|_| fallback_filter_params(query));
    
    // 2. 获取上下文
    let (mut context_text, mut context_count) = if intent.date_range.is_some() {
        match build_context_from_range(query, &intent).await {
             Ok((text, count)) if count > 0 => (text, count),
             _ => (String::new(), 0),
        }
    } else {
        (String::new(), 0)
    };

    // 如果还没有上下文，回退到 HybridSearch
    if context_count == 0 {
         let searcher = HybridSearch::new();
         let results = searcher.search(query, 5).await?;
        
         for result in results {
            if let Ok(activity) = crate::db::get_activity_by_id(result.id).await {
                // 简单起见，这里复用 formatting 逻辑
                let time_str = Local.timestamp_opt(activity.timestamp, 0)
                    .unwrap()
                    .format("%Y-%m-%d %H:%M:%S")
                    .to_string();

                let mut has_content = false;
                if let Some(ref ocr_text) = activity.ocr_text {
                    if !ocr_text.trim().is_empty() {
                        context_text.push_str(&format!(
                            "[{}] 应用: {} | 窗口: {}\n内容: {}\n\n",
                            time_str,
                            activity.app_name,
                            activity.window_title,
                            ocr_text.trim()
                        ));
                        has_content = true;
                    }
                }
                if !has_content {
                    context_text.push_str(&format!(
                        "[{}] 应用: {} | 窗口: {}\n\n",
                        time_str,
                        activity.app_name, activity.window_title
                    ));
                }
                context_count += 1;
            }
        }
    }

    tracing::info!(
        "Chat Stream Context: {} items, {} chars (Intent: DateRange={:?})",
        context_count,
        context_text.len(),
        intent.date_range
    );

    // 3. 尝试调用 LLM API
    let config = crate::app_config::get_config().await.unwrap_or_else(|_| {
        let mut cfg: crate::commands::AppConfig = serde_json::from_str("{}").unwrap();
        cfg.ocr_enabled = true;
        cfg
    });

    let model_id = &config.chat_model;

    // 根据模型名称自动判断提供商：如果以 "claude-" 开头则是 Anthropic，否则默认 OpenAI
    let is_anthropic = model_id.starts_with("claude-");

    if is_anthropic {
        // 使用支持流式的 Anthropic 调用
        match crate::secure_storage::get_api_key("anthropic").await {
            Ok(Some(api_key)) => {
                let provider_config = ProviderConfig::new(
                    api_key,
                    config.anthropic_base_url.clone(),
                    "https://api.anthropic.com",
                );

                // 调用新实现的流式函数
                crate::ai::provider::chat_with_anthropic_stream(
                    query, 
                    &context_text, 
                    model_id, 
                    &provider_config, 
                    None, 
                    on_chunk
                ).await.map(|_| ())
            }
            Ok(None) => {
                on_chunk(format!("⚠️ Missing Anthropic API Key for {}", model_id));
                Ok(())
            }
            Err(e) => Err(anyhow::anyhow!("Get API Key failed: {}", e)),
        }
    } else {
        // 默认使用 OpenAI API
        match crate::secure_storage::get_api_key("openai").await {
            Ok(Some(api_key)) => {
                let provider_config = ProviderConfig::new(
                    api_key,
                    config.openai_base_url.clone(),
                    "https://api.openai.com/v1",
                );
                
                crate::ai::provider::chat_with_openai_stream(query, &context_text, model_id, &provider_config, None, on_chunk).await.map(|_| ())
            }
            Ok(None) => {
                on_chunk(format!("⚠️ Missing OpenAI API Key for {}", model_id));
                Ok(())
            }
            Err(e) => Err(anyhow::anyhow!("Get API Key failed: {}", e)),
        }
    }
}

use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize)]
pub struct AiAnalysisResult {
    pub tasks: Vec<TaskContext>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct TaskContext {
    pub title: String,
    pub summary: String,
    pub related_urls: Vec<String>,
    pub related_files: Vec<String>,
    pub related_apps: Vec<String>,
}

pub async fn analyze_for_proposals(context_text: &str) -> Result<AiAnalysisResult> {
    let config = crate::app_config::get_config().await.unwrap_or_else(|_| {
        let mut cfg: crate::commands::AppConfig = serde_json::from_str("{}").unwrap();
        cfg.ocr_enabled = true;
        cfg
    });

    let model_id = &config.chat_model;
    let is_anthropic = model_id.starts_with("claude-");
    
    // 诊断日志：显示实际读取到的配置
    tracing::info!(
        "analyze_for_proposals: model={}, openai_base_url={:?}, is_anthropic={}",
        model_id,
        config.openai_base_url,
        is_anthropic
    );
    
    let system_prompt = r#"你是专业的个人工作助理。请分析用户的电脑活动日志，识别出用户今天的主要任务/上下文（Task Contexts）。
请返回 JSON 格式，不要包含 Markdown 代码块标记。
JSON 结构如下：
{
  "tasks": [
    {
      "title": "任务名称（如：MemFlow 后端开发）",
      "summary": "该任务段的详细摘要（Markdown 格式），包含主要操作和产出",
      "related_urls": ["https://github.com/...", "https://docs.rs/..."],
      "related_files": ["D:\\Projects\\src\\main.rs", "C:\\Users\\...\\report.docx"],
      "related_apps": ["C:\\Program Files\\...\\Code.exe"]
    }
  ]
}

要求：
1. `tasks`: 将连续或相关联的活动聚类为一个任务。
2. `summary`: 必须是 Markdown 格式，结构清晰。
3. `related_urls`: 提取该任务中访问的关键文档或网页链接（最多 5 个）。
4. `related_files`: 尝试从窗口标题或 OCR 内容中提取关键的本地文件路径（如 .docx, .pdf, .rs, .py 等）。
5. `related_apps`: 如果任务依赖特定应用程序（如 VS Code, Photoshop），且日志中明确记录了该应用的绝对路径（app_path），请将其路径放入此列表。忽略系统自带应用（如资源管理器）。
"#;

    let response = if is_anthropic {
        match crate::secure_storage::get_api_key("anthropic").await {
            Ok(Some(api_key)) => {
                let provider_config = ProviderConfig::new(
                    api_key,
                    config.anthropic_base_url.clone(),
                    "https://api.anthropic.com",
                );

                chat_with_anthropic(
                    "请分析活动记录并生成建议", 
                    context_text, 
                    model_id, 
                    &provider_config, 
                    Some(system_prompt)
                ).await?
            }
            Ok(None) => return Err(anyhow::anyhow!("未配置 Anthropic API Key")),
            Err(e) => return Err(anyhow::anyhow!("获取 API Key 失败: {}", e)),
        }
    } else {
        match crate::secure_storage::get_api_key("openai").await {
            Ok(Some(api_key)) => {
                let provider_config = ProviderConfig::new(
                    api_key,
                    config.openai_base_url.clone(),
                    "https://api.openai.com/v1",
                );

                chat_with_openai(
                    "请分析活动记录并生成建议", 
                    context_text, 
                    model_id, 
                    &provider_config, 
                    Some(system_prompt)
                ).await?
            }
            Ok(None) => return Err(anyhow::anyhow!("未配置 OpenAI API Key")),
            Err(e) => return Err(anyhow::anyhow!("获取 API Key 失败: {}", e)),
        }
    };

    // 尝试解析 JSON（处理可能存在的 markdown 标记）
    let json_str = response.trim();
    let json_str = if json_str.starts_with("```json") {
        json_str.trim_start_matches("```json").trim_end_matches("```").trim()
    } else if json_str.starts_with("```") {
        json_str.trim_start_matches("```").trim_end_matches("```").trim()
    } else {
        json_str
    };

    let result: AiAnalysisResult = serde_json::from_str(json_str)
        .map_err(|e| anyhow::anyhow!("JSON 解析失败: {} - 原文: {}", e, json_str))?;
    
    Ok(result)
}

#[derive(Debug, Serialize, Deserialize)]
pub struct FilterParams {
    pub app_name: Option<String>,
    pub keywords: Vec<String>,
    pub date_range: Option<String>, // "today", "yesterday", "this_week", "last_week"
    pub has_ocr: Option<bool>,
}

fn strip_json_code_fence(input: &str) -> &str {
    let s = input.trim();
    if s.starts_with("```json") {
        return s
            .trim_start_matches("```json")
            .trim_end_matches("```")
            .trim();
    }
    if s.starts_with("```") {
        return s
            .trim_start_matches("```")
            .trim_end_matches("```")
            .trim();
    }
    s
}

fn parse_filter_params_from_llm_response(response: &str) -> Result<FilterParams> {
    let json_str = strip_json_code_fence(response);
    let params: FilterParams = serde_json::from_str(json_str)
        .map_err(|e| anyhow::anyhow!("JSON 解析失败: {} - 原文: {}", e, json_str))?;
    Ok(params)
}

fn fallback_filter_params(query: &str) -> FilterParams {
    let q = query.trim();
    let lower = q.to_lowercase();

    let mut date_range = None;
    if lower.contains("yesterday") {
        date_range = Some("yesterday".to_string());
    } else if lower.contains("today") {
        date_range = Some("today".to_string());
    } else if lower.contains("last week") || lower.contains("last_week") {
        date_range = Some("last_week".to_string());
    } else if lower.contains("this week") || lower.contains("this_week") {
        date_range = Some("this_week".to_string());
    } else if lower.contains("this month") || lower.contains("this_month") {
        date_range = Some("this_month".to_string());
    }

    let has_ocr = if lower.contains("ocr")
        || lower.contains("content")
        || lower.contains("text")
        || lower.contains("内容")
        || lower.contains("文本")
    {
        Some(true)
    } else {
        None
    };

    let keywords = q
        .split_whitespace()
        .filter_map(|w| {
            let trimmed = w.trim_matches(|c: char| !c.is_alphanumeric() && c != '_' && c != '-');
            if trimmed.is_empty() {
                return None;
            }
            Some(trimmed.to_string())
        })
        .collect::<Vec<_>>();

    FilterParams {
        app_name: None,
        keywords,
        date_range,
        has_ocr,
    }
}

pub async fn parse_query_intent(query: &str) -> Result<FilterParams> {
    let config = crate::app_config::get_config().await.unwrap_or_else(|_| {
        let mut cfg: crate::commands::AppConfig = serde_json::from_str("{}").unwrap();
        cfg.ocr_enabled = true;
        cfg
    });

    if !config.ai_enabled {
        return Ok(fallback_filter_params(query));
    }

    let model_id = &config.chat_model;
    let is_anthropic = model_id.starts_with("claude-");
    let timeout_ms = config.intent_parse_timeout_ms.unwrap_or(20_000);
    let llm_timeout = std::time::Duration::from_millis(timeout_ms);

    let system_prompt = r#"You are a smart query parser for a personal activity logger. 
Your goal is to extract search filters from the user's natural language query.

Return a JSON object with the following fields:
- "app_name": (string | null) Filter by application name (e.g., "Chrome", "VS Code"). If the user mentions "pdf", map it to a likely pdf reader or just "pdf".
- "keywords": (string[]) List of keywords to search in OCR text or window titles.
- "date_range": (string | null) One of: "today", "yesterday", "this_week", "last_week", "this_month", or null if not specified.
- "has_ocr": (boolean | null) true if user wants to search within text/content, null otherwise.

Example 1:
Input: "Show me what I did on Chrome yesterday"
Output: { "app_name": "Chrome", "keywords": [], "date_range": "yesterday", "has_ocr": null }

Example 2:
Input: "Find PDF files about rust from last week"
Output: { "app_name": "pdf", "keywords": ["rust"], "date_range": "last_week", "has_ocr": true }

Example 3:
Input: "coding session"
Output: { "app_name": "Code", "keywords": ["coding"], "date_range": null, "has_ocr": null }

Return ONLY the JSON object.
"#;

    let response = if is_anthropic {
        match crate::secure_storage::get_api_key("anthropic").await {
            Ok(Some(api_key)) => {
                let provider_config = ProviderConfig::new(
                    api_key,
                    config.anthropic_base_url.clone(),
                    "https://api.anthropic.com",
                );

                match tokio::time::timeout(
                    llm_timeout,
                    chat_with_anthropic(query, "", model_id, &provider_config, Some(system_prompt)),
                )
                .await
                {
                    Ok(Ok(v)) => Some(v),
                    Ok(Err(e)) => {
                        tracing::warn!(
                            "parse_query_intent: Anthropic 调用失败，使用回退解析: {}",
                            crate::redact::redact_secrets(&e.to_string())
                        );
                        None
                    }
                    Err(_) => {
                        tracing::warn!(
                            "parse_query_intent: Anthropic 调用超时({}ms) model={} base_url={}，使用回退解析",
                            timeout_ms,
                            model_id,
                            provider_config.base_url
                        );
                        None
                    }
                }
            }
            Ok(None) => {
                tracing::debug!("parse_query_intent: 未配置 Anthropic API Key，使用回退解析");
                None
            }
            Err(e) => {
                tracing::warn!(
                    "parse_query_intent: 获取 API Key 失败，使用回退解析: {}",
                    crate::redact::redact_secrets(&e.to_string())
                );
                None
            }
        }
    } else {
        match crate::secure_storage::get_api_key("openai").await {
            Ok(Some(api_key)) => {
                let provider_config = ProviderConfig::new(
                    api_key,
                    config.openai_base_url.clone(),
                    "https://api.openai.com/v1",
                );

                match tokio::time::timeout(
                    llm_timeout,
                    chat_with_openai(query, "", model_id, &provider_config, Some(system_prompt)),
                )
                .await
                {
                    Ok(Ok(v)) => Some(v),
                    Ok(Err(e)) => {
                        tracing::warn!(
                            "parse_query_intent: OpenAI 调用失败，使用回退解析: {}",
                            crate::redact::redact_secrets(&e.to_string())
                        );
                        None
                    }
                    Err(_) => {
                        tracing::warn!(
                            "parse_query_intent: OpenAI 调用超时({}ms) model={} base_url={}，使用回退解析",
                            timeout_ms,
                            model_id,
                            provider_config.base_url
                        );
                        None
                    }
                }
            }
            Ok(None) => {
                tracing::debug!("parse_query_intent: 未配置 OpenAI API Key，使用回退解析");
                None
            }
            Err(e) => {
                tracing::warn!(
                    "parse_query_intent: 获取 API Key 失败，使用回退解析: {}",
                    crate::redact::redact_secrets(&e.to_string())
                );
                None
            }
        }
    };

    let Some(response) = response else {
        return Ok(fallback_filter_params(query));
    };

    match parse_filter_params_from_llm_response(&response) {
        Ok(v) => Ok(v),
        Err(e) => {
            tracing::warn!(
                "parse_query_intent: 解析失败，使用回退解析: {}",
                crate::redact::redact_secrets(&e.to_string())
            );
            Ok(fallback_filter_params(query))
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn strips_code_fences_for_filter_params() {
        let input = r#"```json
{"app_name":"Chrome","keywords":[],"date_range":"yesterday","has_ocr":null}
```"#;
        let parsed = parse_filter_params_from_llm_response(input).unwrap();
        assert_eq!(parsed.app_name.as_deref(), Some("Chrome"));
        assert_eq!(parsed.date_range.as_deref(), Some("yesterday"));
        assert!(parsed.keywords.is_empty());
    }

    #[test]
    fn parses_plain_json_for_filter_params() {
        let input = r#"{"app_name":null,"keywords":["rust"],"date_range":"last_week","has_ocr":true}"#;
        let parsed = parse_filter_params_from_llm_response(input).unwrap();
        assert_eq!(parsed.app_name, None);
        assert_eq!(parsed.keywords, vec!["rust".to_string()]);
        assert_eq!(parsed.date_range.as_deref(), Some("last_week"));
        assert_eq!(parsed.has_ocr, Some(true));
    }

    #[test]
    fn fallback_never_panics_and_returns_keywords() {
        let parsed = fallback_filter_params("Find pdf last week rust ocr");
        assert!(!parsed.keywords.is_empty());
        assert_eq!(parsed.date_range.as_deref(), Some("last_week"));
        assert_eq!(parsed.has_ocr, Some(true));
    }
}
