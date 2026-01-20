use crate::agent;
use crate::ai;
use crate::ai::provider::{
    chat_with_anthropic, chat_with_openai, embedding_with_openai, ProviderConfig,
};
use crate::app_config;
use crate::chat;
use crate::db;
use crate::graph;
use crate::performance;
use crate::recorder;
use serde::{Deserialize, Serialize};
use tauri::Manager;

#[derive(Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ActivityLog {
    pub id: i64,
    pub timestamp: i64,
    pub app_name: String,
    pub window_title: String,
    pub image_path: String,
    pub ocr_text: Option<String>,
    pub phash: Option<String>,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
#[serde(rename_all = "camelCase")]
pub struct AppConfig {
    #[serde(default = "default_recording_interval", alias = "recording_interval")]
    pub recording_interval: u64,
    #[serde(default, alias = "ocr_enabled")]
    pub ocr_enabled: bool,
    #[serde(default = "default_ocr_engine", alias = "ocr_engine")]
    pub ocr_engine: String,
    #[serde(default, alias = "ai_enabled")]
    pub ai_enabled: bool,
    #[serde(default = "default_retention_days", alias = "retention_days")]
    pub retention_days: u32,
    #[serde(default = "default_chat_model", alias = "chat_model")]
    pub chat_model: String,
    #[serde(default = "default_embedding_model", alias = "embedding_model")]
    pub embedding_model: String,
    #[serde(default, alias = "embedding_base_url")]
    pub embedding_base_url: Option<String>,
    #[serde(
        default = "default_embedding_use_shared_key",
        alias = "embedding_use_shared_key"
    )]
    pub embedding_use_shared_key: bool,
    // API 配置
    #[serde(default, alias = "openai_base_url")]
    pub openai_base_url: Option<String>,
    #[serde(default, alias = "anthropic_base_url")]
    pub anthropic_base_url: Option<String>,
    #[serde(default, alias = "blocklist_enabled")]
    pub blocklist_enabled: bool,
    #[serde(default = "default_blocklist_mode", alias = "blocklist_mode")]
    pub blocklist_mode: String,
    #[serde(default, alias = "privacy_mode_enabled")]
    pub privacy_mode_enabled: bool,
    #[serde(default, alias = "privacy_mode_until")]
    pub privacy_mode_until: Option<i64>,
    #[serde(default, alias = "intent_parse_timeout_ms")]
    pub intent_parse_timeout_ms: Option<u64>,
    #[serde(
        default = "default_enable_focus_analytics",
        alias = "enable_focus_analytics"
    )]
    pub enable_focus_analytics: bool,
    #[serde(
        default = "default_enable_proactive_assistant",
        alias = "enable_proactive_assistant"
    )]
    pub enable_proactive_assistant: bool,
    #[serde(
        default = "default_ocr_redaction_enabled",
        alias = "ocr_redaction_enabled"
    )]
    pub ocr_redaction_enabled: bool,
    #[serde(default = "default_ocr_redaction_level", alias = "ocr_redaction_level")]
    pub ocr_redaction_level: String,
}

fn default_recording_interval() -> u64 {
    5000
}

fn default_blocklist_mode() -> String {
    "blocklist".to_string()
}

fn default_ocr_redaction_enabled() -> bool {
    true
}

fn default_ocr_redaction_level() -> String {
    "basic".to_string()
}

fn default_ocr_engine() -> String {
    "rapidocr".to_string()
}

fn default_retention_days() -> u32 {
    30
}

fn default_chat_model() -> String {
    "gpt-4o-mini".to_string()
}

fn default_embedding_model() -> String {
    "text-embedding-3-small".to_string()
}

fn default_embedding_use_shared_key() -> bool {
    true
}

fn default_enable_focus_analytics() -> bool {
    false
}

fn default_enable_proactive_assistant() -> bool {
    false
}

#[derive(Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Stats {
    pub total_activities: i64,
    pub total_hours: f64,
    pub top_app: String,
}

#[tauri::command]
pub async fn start_recording() -> Result<(), String> {
    tracing::info!("Frontend requested start_recording");
    println!("[DEBUG] Frontend requested start_recording");
    match recorder::start() {
        Ok(_) => {
            tracing::info!("Recorder started successfully");
            Ok(())
        }
        Err(e) => {
            tracing::error!("Failed to start recorder: {}", e);
            Err(e.to_string())
        }
    }
}

#[tauri::command]
pub async fn stop_recording() -> Result<(), String> {
    recorder::stop().map_err(|e| e.to_string())?;
    Ok(())
}

#[tauri::command]
pub async fn get_activities(limit: Option<i64>) -> Result<Vec<ActivityLog>, String> {
    let limit = limit.unwrap_or(100);
    tracing::info!("Frontend requested get_activities, limit: {}", limit);
    match db::get_activities(limit).await {
        Ok(activities) => {
            tracing::info!("Returning {} activities", activities.len());
            Ok(activities)
        }
        Err(e) => {
            tracing::error!("Failed to get activities: {}", e);
            Err(e.to_string())
        }
    }
}

#[tauri::command]
pub async fn search_activities(
    query: Option<String>,
    app_name: Option<String>,
    from_ts: Option<i64>,
    to_ts: Option<i64>,
    has_ocr: Option<bool>,
    limit: Option<i64>,
    offset: Option<i64>,
    order_by: Option<String>,
) -> Result<serde_json::Value, String> {
    let (items, total) = db::search_activities(
        query, app_name, from_ts, to_ts, has_ocr, limit, offset, order_by,
    )
    .await
    .map_err(|e| e.to_string())?;

    Ok(serde_json::json!({
        "items": items,
        "total": total
    }))
}

#[tauri::command]
pub async fn get_recording_stats(limit: Option<i64>) -> Result<Vec<db::RecordingStat>, String> {
    db::get_recording_stats(limit.unwrap_or(30))
        .await
        .map_err(|e| e.to_string())
}

#[tauri::command]
pub async fn get_blocklist() -> Result<Vec<String>, String> {
    db::get_blocklist().await.map_err(|e| e.to_string())
}

#[tauri::command]
pub async fn add_blocklist_item(app_name: String) -> Result<(), String> {
    db::add_blocklist_item(app_name)
        .await
        .map_err(|e| e.to_string())
}

#[tauri::command]
pub async fn remove_blocklist_item(app_name: String) -> Result<(), String> {
    db::remove_blocklist_item(app_name)
        .await
        .map_err(|e| e.to_string())
}

#[tauri::command]
pub async fn clear_blocklist() -> Result<(), String> {
    db::clear_blocklist().await.map_err(|e| e.to_string())
}

#[tauri::command]
pub async fn get_activity_by_id(id: i64) -> Result<ActivityLog, String> {
    db::get_activity_by_id(id).await.map_err(|e| e.to_string())
}

#[tauri::command]
pub async fn get_config() -> Result<AppConfig, String> {
    app_config::get_config().await.map_err(|e| e.to_string())
}

#[tauri::command]
pub async fn update_config(config: AppConfig, app_handle: tauri::AppHandle) -> Result<(), String> {
    app_config::update_config(config, app_handle)
        .await
        .map_err(|e| e.to_string())
}

#[tauri::command]
pub async fn set_privacy_mode(
    enabled: bool,
    until_ts: Option<i64>,
    app_handle: tauri::AppHandle,
) -> Result<(), String> {
    let mut config = app_config::get_config().await.map_err(|e| e.to_string())?;
    config.privacy_mode_enabled = enabled;
    config.privacy_mode_until = until_ts;
    app_config::update_config(config, app_handle)
        .await
        .map_err(|e| e.to_string())
}

#[tauri::command]
pub async fn get_stats() -> Result<Stats, String> {
    db::get_stats().await.map_err(|e| e.to_string())
}

#[tauri::command]
pub async fn get_image_path(
    filename: String,
    app_handle: tauri::AppHandle,
) -> Result<String, String> {
    // 获取应用数据目录
    let app_data = app_handle
        .path()
        .app_data_dir()
        .map_err(|e| format!("无法获取应用数据目录: {}", e))?;

    let screenshots_dir = app_data.join("screenshots");
    let file_path = screenshots_dir.join(&filename);

    // 返回完整路径
    file_path
        .to_str()
        .map(|s| s.to_string())
        .ok_or_else(|| "路径转换失败".to_string())
}

#[tauri::command]
pub async fn get_image_paths(
    filenames: Vec<String>,
    app_handle: tauri::AppHandle,
) -> Result<Vec<String>, String> {
    let app_data = app_handle
        .path()
        .app_data_dir()
        .map_err(|e| format!("无法获取应用数据目录: {}", e))?;

    let screenshots_dir = app_data.join("screenshots");

    let mut results = Vec::with_capacity(filenames.len());
    for filename in filenames {
        let file_path = screenshots_dir.join(&filename);
        let s = file_path
            .to_str()
            .map(|s| s.to_string())
            .ok_or_else(|| "路径转换失败".to_string())?;
        results.push(s);
    }

    Ok(results)
}

#[tauri::command]
pub async fn get_graph_data() -> Result<graph::GraphData, String> {
    graph::load_graph().await.map_err(|e| e.to_string())
}

#[tauri::command]
pub async fn rebuild_graph() -> Result<graph::GraphData, String> {
    tracing::info!("rebuild_graph started");
    let graph_data = graph::build_graph().await.map_err(|e| {
        tracing::error!("build_graph failed: {}", e);
        e.to_string()
    })?;
    tracing::info!(
        nodes = graph_data.nodes.len(),
        edges = graph_data.edges.len(),
        "rebuild_graph built graph"
    );
    graph::save_graph(&graph_data).await.map_err(|e| {
        tracing::error!("save_graph failed: {}", e);
        e.to_string()
    })?;
    tracing::info!("rebuild_graph completed");
    Ok(graph_data)
}

#[tauri::command]
pub async fn get_performance_metrics() -> Result<performance::PerformanceMetrics, String> {
    let monitor = performance::PerformanceMonitor::new();
    monitor.get_metrics().await.map_err(|e| e.to_string())
}

#[tauri::command]
pub async fn trigger_gc() -> Result<(), String> {
    let monitor = performance::PerformanceMonitor::new();
    monitor.trigger_gc().await.map_err(|e| e.to_string())
}

#[tauri::command]
pub async fn ai_chat(query: String) -> Result<String, String> {
    ai::chat(&query, vec![]).await.map_err(|e| e.to_string())
}

#[tauri::command]
pub async fn ai_chat_stream(query: String, app_handle: tauri::AppHandle) -> Result<(), String> {
    use tauri::Emitter;
    
    let handle = app_handle.clone();
    let res = ai::chat_stream(&query, vec![], move |chunk| {
        if let Err(e) = handle.emit("ai-chat-chunk", chunk) {
            tracing::error!("Failed to emit ai-chat-chunk: {}", e);
        }
    })
    .await;

    if let Err(e) = res {
        let _ = app_handle.emit("ai-chat-chunk", format!("Error: {}", e)); // Emit error as chunk or separate event? Plan says "handle error" implicitly. Sticking to simple error reporting.
        return Err(e.to_string());
    }

    let _ = app_handle.emit("ai-chat-done", ());
    Ok(())
}

#[tauri::command]
pub async fn parse_query_intent(query: String) -> Result<ai::FilterParams, String> {
    ai::parse_query_intent(&query).await.map_err(|e| e.to_string())
}


#[tauri::command]
pub async fn get_activity_heatmap_stats(year: Option<i32>) -> Result<Vec<db::HeatmapData>, String> {
    db::get_activity_heatmap_stats(year)
        .await
        .map_err(|e| e.to_string())
}

#[tauri::command]
pub async fn get_app_usage_stats(limit: Option<i64>) -> Result<Vec<db::AppUsageStat>, String> {
    db::get_app_usage_stats(limit.unwrap_or(5))
        .await
        .map_err(|e| e.to_string())
}

#[tauri::command]
pub async fn get_hourly_activity_stats() -> Result<Vec<db::HourlyStat>, String> {
    db::get_hourly_activity_stats()
        .await
        .map_err(|e| e.to_string())
}

#[tauri::command]
pub async fn get_focus_metrics(
    from_ts: Option<i64>,
    to_ts: Option<i64>,
    limit: Option<i64>,
) -> Result<Vec<db::FocusMetric>, String> {
    db::get_focus_metrics(from_ts, to_ts, limit.unwrap_or(24 * 60))
        .await
        .map_err(|e| e.to_string())
}

#[tauri::command]
pub async fn open_external_url(url: String) -> Result<(), String> {
    #[cfg(target_os = "windows")]
    {
        std::process::Command::new("cmd")
            .args(["/C", "start", "", &url])
            .spawn()
            .map_err(|e| e.to_string())?;
    }
    #[cfg(target_os = "macos")]
    {
        std::process::Command::new("open")
            .arg(&url)
            .spawn()
            .map_err(|e| e.to_string())?;
    }
    #[cfg(target_os = "linux")]
    {
        std::process::Command::new("xdg-open")
            .arg(&url)
            .spawn()
            .map_err(|e| e.to_string())?;
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::AppConfig;

    #[test]
    fn app_config_defaults_work() {
        let cfg: AppConfig = serde_json::from_str("{}").unwrap();
        assert_eq!(cfg.recording_interval, 5000);
        assert_eq!(cfg.ocr_enabled, false);
        assert_eq!(cfg.ocr_engine, "rapidocr");
        assert_eq!(cfg.ai_enabled, false);
        assert_eq!(cfg.retention_days, 30);
        assert_eq!(cfg.chat_model, "gpt-4o-mini");
        assert_eq!(cfg.embedding_model, "text-embedding-3-small");
        assert_eq!(cfg.embedding_base_url, None);
        assert_eq!(cfg.embedding_use_shared_key, true);
        assert_eq!(cfg.openai_base_url, None);
        assert_eq!(cfg.anthropic_base_url, None);
        assert_eq!(cfg.enable_focus_analytics, false);
    }

    #[test]
    fn app_config_accepts_legacy_snake_case_aliases() {
        let json = r#"
        {
          "recording_interval": 1234,
          "ocr_enabled": true,
          "ocr_engine": "rapidocr",
          "ai_enabled": true,
          "retention_days": 7,
          "chat_model": "gpt-4o-mini",
          "embedding_model": "text-embedding-3-small",
          "embedding_base_url": "http://localhost:11434/v1",
          "embedding_use_shared_key": false,
          "openai_base_url": "https://api.openai.com/v1",
          "anthropic_base_url": "https://api.anthropic.com",
          "enable_focus_analytics": true
        }
        "#;
        let cfg: AppConfig = serde_json::from_str(json).unwrap();
        assert_eq!(cfg.recording_interval, 1234);
        assert_eq!(cfg.ocr_enabled, true);
        assert_eq!(cfg.ocr_engine, "rapidocr");
        assert_eq!(cfg.ai_enabled, true);
        assert_eq!(cfg.retention_days, 7);
        assert_eq!(cfg.chat_model, "gpt-4o-mini");
        assert_eq!(cfg.embedding_model, "text-embedding-3-small");
        assert_eq!(
            cfg.embedding_base_url.as_deref(),
            Some("http://localhost:11434/v1")
        );
        assert_eq!(cfg.embedding_use_shared_key, false);
        assert_eq!(
            cfg.openai_base_url.as_deref(),
            Some("https://api.openai.com/v1")
        );
        assert_eq!(
            cfg.anthropic_base_url.as_deref(),
            Some("https://api.anthropic.com")
        );
        assert_eq!(cfg.enable_focus_analytics, true);
    }
}

// ============================================
// 连接测试命令（真实调用 API）
// ============================================

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct TestChatConnectionParams {
    pub provider: String, // openai | anthropic | custom
    pub model: String,
    pub api_key: Option<String>,  // 如果前端传了，优先用；否则走安全存储
    pub base_url: Option<String>, // 可选覆盖
}

#[tauri::command]
pub async fn test_chat_connection(params: TestChatConnectionParams) -> Result<(), String> {
    let provider = params.provider;
    let model = params.model;

    let api_key = if let Some(k) = params.api_key.filter(|s| !s.trim().is_empty()) {
        k
    } else {
        let service = if provider == "anthropic" {
            "anthropic"
        } else {
            "openai"
        };
        crate::secure_storage::get_api_key(service)
            .await
            .map_err(|e| crate::redact::redact_secrets(&e.to_string()))?
            .ok_or_else(|| format!("未配置 {} API Key", service))?
    };

    if provider == "anthropic" {
        let cfg = ProviderConfig::new(api_key, params.base_url, "https://api.anthropic.com");
        // 真实调用一次 messages
        chat_with_anthropic("ping", "", &model, &cfg, None)
            .await
            .map(|_| ())
            .map_err(|e| crate::redact::redact_secrets(&e.to_string()))
    } else {
        let cfg = ProviderConfig::new(api_key, params.base_url, "https://api.openai.com/v1");
        // 真实调用一次 chat/completions
        chat_with_openai("ping", "", &model, &cfg, None)
            .await
            .map(|_| ())
            .map_err(|e| crate::redact::redact_secrets(&e.to_string()))
    }
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct TestEmbeddingConnectionParams {
    pub provider: String, // openai | custom
    pub model: String,
    pub api_key: Option<String>,  // 如果前端传了，优先用；否则走安全存储
    pub base_url: Option<String>, // 可选覆盖（自定义端点）
    pub use_shared_key: bool,
}

#[tauri::command]
pub async fn test_embedding_connection(
    params: TestEmbeddingConnectionParams,
) -> Result<(), String> {
    let provider = params.provider;
    let model = params.model;

    // 当前实现仅支持 OpenAI 兼容 embeddings
    let api_key = if let Some(k) = params.api_key.filter(|s| !s.trim().is_empty()) {
        k
    } else {
        let service = if params.use_shared_key {
            "openai"
        } else {
            "embedding"
        };
        crate::secure_storage::get_api_key(service)
            .await
            .map_err(|e| crate::redact::redact_secrets(&e.to_string()))?
            .ok_or_else(|| format!("未配置 {} API Key", service))?
    };

    let cfg = ProviderConfig::new(api_key, params.base_url, "https://api.openai.com/v1");

    // 真实调用一次 embeddings
    let vec = embedding_with_openai("ping", &model, &cfg)
        .await
        .map_err(|e| crate::redact::redact_secrets(&e.to_string()))?;

    if vec.is_empty() {
        return Err("Embeddings API 返回空向量".to_string());
    }

    // provider 仅用于参数合法性（保留扩展空间）
    if provider != "openai" && provider != "custom" {
        return Err("未知 embedding provider".to_string());
    }

    Ok(())
}

#[tauri::command]
pub async fn save_api_key(service: String, key: String) -> Result<(), String> {
    crate::secure_storage::save_api_key(&service, &key)
        .await
        .map_err(|e| crate::redact::redact_secrets(&e.to_string()))
}

#[tauri::command]
pub async fn get_api_key(service: String) -> Result<Option<String>, String> {
    crate::secure_storage::get_api_key(&service)
        .await
        .map(|v| v.map(|_| "configured".to_string()))
        .map_err(|e| crate::redact::redact_secrets(&e.to_string()))
}

#[tauri::command]
pub async fn delete_api_key(service: String) -> Result<(), String> {
    crate::secure_storage::delete_api_key(&service)
        .await
        .map_err(|e| crate::redact::redact_secrets(&e.to_string()))
}

// ============================================
// 对话历史相关命令
// ============================================

/// 创建新的对话会话
#[tauri::command]
pub async fn create_chat_session(title: String) -> Result<i64, String> {
    chat::create_session(&title)
        .await
        .map_err(|e| e.to_string())
}

/// 保存聊天消息
#[tauri::command]
pub async fn save_chat_message(
    session_id: i64,
    role: String,
    content: String,
    context_ids: Option<Vec<i64>>,
) -> Result<i64, String> {
    chat::save_message(session_id, &role, &content, context_ids)
        .await
        .map_err(|e| e.to_string())
}

/// 更新会话标题
#[tauri::command]
pub async fn update_session_title(session_id: i64, title: String) -> Result<(), String> {
    chat::update_session_title(session_id, &title)
        .await
        .map_err(|e| e.to_string())
}

/// 获取对话会话列表
#[tauri::command]
pub async fn get_chat_sessions(
    limit: Option<i64>,
    offset: Option<i64>,
    search: Option<String>,
) -> Result<Vec<chat::ChatSession>, String> {
    chat::get_sessions(limit, offset, search.as_deref())
        .await
        .map_err(|e| e.to_string())
}

/// 获取指定会话的消息列表
#[tauri::command]
pub async fn get_chat_messages(session_id: i64) -> Result<Vec<chat::ChatMessage>, String> {
    chat::get_messages(session_id)
        .await
        .map_err(|e| e.to_string())
}

/// 删除指定会话
#[tauri::command]
pub async fn delete_chat_session(session_id: i64) -> Result<(), String> {
    chat::delete_session(session_id)
        .await
        .map_err(|e| e.to_string())
}

/// 清空所有对话历史
#[tauri::command]
pub async fn clear_all_chat_history() -> Result<(), String> {
    chat::clear_all_history().await.map_err(|e| e.to_string())
}

// ============================================
// 反馈相关命令
// ============================================

/// 对消息进行评价
#[tauri::command]
pub async fn rate_message(
    message_id: i64,
    rating: i32,
    comment: Option<String>,
) -> Result<(), String> {
    chat::rate_message(message_id, rating, comment.as_deref())
        .await
        .map_err(|e| e.to_string())
}

/// 提交用户反馈
#[tauri::command]
pub async fn submit_feedback(
    category: String,
    title: String,
    content: String,
    screenshot_path: Option<String>,
    context_session_id: Option<i64>,
) -> Result<i64, String> {
    chat::submit_feedback(
        &category,
        &title,
        &content,
        screenshot_path.as_deref(),
        context_session_id,
    )
    .await
    .map_err(|e| e.to_string())
}

/// 获取用户反馈列表
#[tauri::command]
pub async fn get_user_feedbacks(limit: Option<i64>) -> Result<Vec<chat::UserFeedback>, String> {
    chat::get_feedbacks(limit).await.map_err(|e| e.to_string())
}

// ============================================
// 智能代理（自动化提案/执行/审计）相关命令
// ============================================

#[tauri::command]
pub async fn agent_propose_automation(
    params: Option<agent::AgentProposeParams>,
) -> Result<Vec<agent::AutomationProposalDto>, String> {
    agent::propose_automation(params.unwrap_or_default())
        .await
        .map_err(|e| e.to_string())
}

#[tauri::command]
pub async fn agent_execute_automation(
    proposal_id: i64,
    app_handle: tauri::AppHandle,
) -> Result<agent::ExecutionResultDto, String> {
    tracing::info!(
        "agent_execute_automation called: proposal_id={}",
        proposal_id
    );
    agent::execute_automation(proposal_id, app_handle)
        .await
        .map_err(|e| e.to_string())
}

#[tauri::command]
pub async fn agent_list_executions(
    limit: Option<i64>,
    offset: Option<i64>,
) -> Result<Vec<agent::ExecutionDto>, String> {
    agent::list_executions(limit.unwrap_or(50), offset.unwrap_or(0))
        .await
        .map_err(|e| e.to_string())
}

#[tauri::command]
pub async fn run_retention_cleanup(dry_run: Option<bool>) -> Result<db::CleanupStats, String> {
    let config = app_config::get_config().await.map_err(|e| e.to_string())?;
    let days = config.retention_days;
    db::cleanup_old_activities(days, dry_run.unwrap_or(false))
        .await
        .map_err(|e| e.to_string())
}

#[tauri::command]
pub async fn agent_cancel_execution(execution_id: i64) -> Result<(), String> {
    tracing::info!(
        "agent_cancel_execution called: execution_id={}",
        execution_id
    );
    agent::cancel_execution(execution_id)
        .await
        .map_err(|e| e.to_string())
}
