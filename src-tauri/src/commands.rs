use memflow_core::agent;
use crate::desktop_context::TauriContext;
use std::sync::Arc;
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

// Re-export sqlx for database operations
use sqlx;

// ActivityLog is imported from crate::db (re-exported from memflow_core)
pub use crate::db::ActivityLog;

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
    #[serde(default = "default_ocr_preprocess_enabled", alias = "ocr_preprocess_enabled")]
    pub ocr_preprocess_enabled: bool,
    #[serde(
        default = "default_ocr_preprocess_target_width",
        alias = "ocr_preprocess_target_width"
    )]
    pub ocr_preprocess_target_width: u32,
    #[serde(
        default = "default_ocr_preprocess_max_pixels",
        alias = "ocr_preprocess_max_pixels"
    )]
    pub ocr_preprocess_max_pixels: u64,
    /// Agent 生成笔记的保存路径（可选，默认为文档目录）
    #[serde(default, alias = "agent_note_path")]
    pub agent_note_path: Option<String>,
    #[serde(default = "default_compression_quality", alias = "compression_quality")]
    pub compression_quality: u8,
    #[serde(
        default = "default_target_resolution_scale",
        alias = "target_resolution_scale"
    )]
    pub target_resolution_scale: f32,
    /// 暂停录制开关
    #[serde(default, alias = "pause_recording_enabled")]
    pub pause_recording_enabled: bool,
    /// 暂停录制到指定时间戳
    #[serde(default, alias = "pause_until")]
    pub pause_until: Option<i64>,
    /// 最大存储空间（GB）
    #[serde(default = "default_max_storage_gb", alias = "max_storage_gb")]
    pub max_storage_gb: u32,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
#[serde(rename_all = "camelCase")]
pub struct ClearResult {
    pub deleted_activities: u64,
    pub deleted_screenshots: u64,
    pub freed_bytes: u64,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
#[serde(rename_all = "camelCase")]
pub struct AutostartInfo {
    pub enabled: bool,
    pub app_name: String,
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

fn default_ocr_preprocess_enabled() -> bool {
    true
}

fn default_max_storage_gb() -> u32 {
    10
}


fn default_ocr_preprocess_target_width() -> u32 {
    1280
}

fn default_ocr_preprocess_max_pixels() -> u64 {
    3_000_000
}

fn default_compression_quality() -> u8 {
    80
}

fn default_target_resolution_scale() -> f32 {
    1.0
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
    true
}

fn default_enable_proactive_assistant() -> bool {
    false
}

// Stats is imported from crate::db (re-exported from memflow_core)
pub use crate::db::Stats;

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
            if let Some(first) = activities.first() {
                println!("[DEBUG] First activity: ID={}, App={}, ImagePath={:?}", 
                    first.id, first.app_name, first.image_path);
            }
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
pub async fn get_ocr_queue_stats() -> Result<db::OcrQueueStats, String> {
    db::get_ocr_queue_stats().await.map_err(|e| e.to_string())
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
    let interval = config.recording_interval;
    app_config::update_config(config, app_handle)
        .await
        .map_err(|e| e.to_string())?;
    
    // Notify recorder of the new interval
    recorder::set_base_interval(interval);
    Ok(())
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
    let config = app_config::get_config().await.map_err(|e| e.to_string())?;
    // Config interval is in ms, convert to seconds
    let interval = config.recording_interval as f64 / 1000.0;
    // Sanity check: ensure interval is positive, default to 5s if invalid
    let interval = if interval <= 0.0 { 5.0 } else { interval };
    
    db::get_stats(interval).await.map_err(|e| e.to_string())
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
    println!("[DEBUG] get_image_path: resolving '{}'", filename);
    println!("[DEBUG] Full path: {:?}", file_path);
    if file_path.exists() {
        println!("[DEBUG] File exists!");
    } else {
        println!("[DEBUG] File does NOT exist!");
    }

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
        assert_eq!(cfg.enable_focus_analytics, true); // 修正：默认值应为 true
        assert_eq!(cfg.enable_proactive_assistant, false);
        assert_eq!(cfg.blocklist_enabled, false);
        assert_eq!(cfg.blocklist_mode, "blocklist");
        assert_eq!(cfg.privacy_mode_enabled, false);
        assert_eq!(cfg.ocr_redaction_enabled, true);
        assert_eq!(cfg.ocr_redaction_level, "basic");
        assert_eq!(cfg.ocr_preprocess_enabled, true);
        assert_eq!(cfg.ocr_preprocess_target_width, 1280);
        assert_eq!(cfg.ocr_preprocess_max_pixels, 3_000_000);
    }

    #[test]
    fn app_config_partial_json_fills_defaults() {
        // 测试部分字段的 JSON 会正确填充缺失字段的默认值
        let json = r#"{"recordingInterval": 3000, "ocrEnabled": true}"#;
        let cfg: AppConfig = serde_json::from_str(json).unwrap();
        
        // 指定的字段应该使用提供的值
        assert_eq!(cfg.recording_interval, 3000);
        assert_eq!(cfg.ocr_enabled, true);
        
        // 未指定的字段应该使用默认值
        assert_eq!(cfg.ocr_engine, "rapidocr");
        assert_eq!(cfg.ai_enabled, false);
        assert_eq!(cfg.retention_days, 30);
        assert_eq!(cfg.enable_focus_analytics, true);
        assert_eq!(cfg.blocklist_mode, "blocklist");
        assert_eq!(cfg.ocr_preprocess_enabled, true);
        assert_eq!(cfg.ocr_preprocess_target_width, 1280);
        assert_eq!(cfg.ocr_preprocess_max_pixels, 3_000_000);
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
        assert_eq!(cfg.ocr_preprocess_enabled, true);
        assert_eq!(cfg.ocr_preprocess_target_width, 1280);
        assert_eq!(cfg.ocr_preprocess_max_pixels, 3_000_000);
    }

    #[test]
    fn test_scan_directory_with_nonexistent_path() {
        // Test that scan_directory returns zeros for non-existent directory
        let temp_dir = std::path::PathBuf::from("nonexistent_dir_12345");
        let (count, size) = super::scan_directory(&temp_dir).unwrap();
        assert_eq!(count, 0);
        assert_eq!(size, 0);
    }

    #[test]
    fn test_scan_directory_with_files() {
        // Create a temporary directory with some test files
        let temp_dir = std::env::temp_dir().join("memflow_test_scan");
        let _ = std::fs::remove_dir_all(&temp_dir); // Clean up if exists
        std::fs::create_dir_all(&temp_dir).unwrap();

        // Create test files with known sizes
        let file1 = temp_dir.join("test1.txt");
        let file2 = temp_dir.join("test2.txt");
        std::fs::write(&file1, b"hello").unwrap();
        std::fs::write(&file2, b"world world").unwrap();

        let (count, size) = super::scan_directory(&temp_dir).unwrap();
        assert_eq!(count, 2);
        assert_eq!(size, 16); // "hello" (5) + "world world" (11)

        // Clean up
        let _ = std::fs::remove_dir_all(&temp_dir);
    }

    #[test]
    fn test_scan_directory_with_subdirectory() {
        // Test that subdirectories are not counted as files
        let temp_dir = std::env::temp_dir().join("memflow_test_subdir");
        let _ = std::fs::remove_dir_all(&temp_dir);
        std::fs::create_dir_all(&temp_dir).unwrap();

        // Create a file and a subdirectory
        let file1 = temp_dir.join("test.txt");
        std::fs::write(&file1, b"content").unwrap();

        let subdir = temp_dir.join("subdir");
        std::fs::create_dir(&subdir).unwrap();

        let (count, size) = super::scan_directory(&temp_dir).unwrap();
        assert_eq!(count, 1); // Only the file, not the directory
        assert_eq!(size, 7); // "content"

        // Clean up
        let _ = std::fs::remove_dir_all(&temp_dir);
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
    app_handle: tauri::AppHandle,
) -> Result<Vec<agent::AutomationProposalDto>, String> {
    let ctx = Arc::new(TauriContext::new(app_handle));
    agent::propose_automation(params.unwrap_or_default(), ctx)
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
    let ctx = Arc::new(TauriContext::new(app_handle));
    agent::execute_automation(proposal_id, ctx)
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

#[derive(Debug, Serialize, Deserialize, Clone)]
#[serde(rename_all = "camelCase")]
pub struct StorageStatsResponse {
    pub screenshots_count: u64,
    pub screenshots_size_mb: f64,
    pub activities_count: u64,
    pub database_size_mb: f64,
    pub total_size_mb: f64,
    pub max_storage_gb: f64,
    pub usage_percent: f64,
    pub next_gc_time: Option<String>,
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

#[tauri::command]
pub async fn clear_all_data() -> Result<ClearResult, String> {
    tracing::info!("clear_all_data called");

    // Get the database pool and screenshots directory
    let pool = db::get_pool().await.map_err(|e| e.to_string())?;
    let screenshots_dir = db::get_screenshots_dir()
        .await
        .ok_or_else(|| "Screenshots directory not initialized".to_string())?;

    let mut result = ClearResult {
        deleted_activities: 0,
        deleted_screenshots: 0,
        freed_bytes: 0,
    };

    // Use transaction for database deletion (all or nothing)
    let mut tx = pool.begin().await.map_err(|e| {
        tracing::error!("Failed to begin transaction: {}", e);
        e.to_string()
    })?;

    // 1. Count activities before deletion (for accurate reporting)
    let activity_count: i64 = sqlx::query_scalar("SELECT COUNT(*) FROM activity_logs")
        .fetch_one(&mut *tx)
        .await
        .map_err(|e| {
            tracing::error!("Failed to count activities: {}", e);
            e.to_string()
        })?;
    result.deleted_activities = activity_count as u64;

    // 2. Get all image paths before deleting from database
    let image_rows: Vec<(String,)> = sqlx::query_as("SELECT image_path FROM activity_logs WHERE image_path IS NOT NULL")
        .fetch_all(&mut *tx)
        .await
        .map_err(|e| {
            tracing::error!("Failed to fetch image paths: {}", e);
            e.to_string()
        })?;

    // 3. Delete from database in proper order (respecting foreign keys)
    // FTS table will be cleaned by triggers

    // Delete from vector_embeddings (CASCADE will handle via FK)
    sqlx::query("DELETE FROM vector_embeddings")
        .execute(&mut *tx)
        .await
        .map_err(|e| {
            tracing::error!("Failed to delete vector embeddings: {}", e);
            e.to_string()
        })?;

    // Delete from knowledge_edges
    sqlx::query("DELETE FROM knowledge_edges")
        .execute(&mut *tx)
        .await
        .map_err(|e| {
            tracing::error!("Failed to delete knowledge edges: {}", e);
            e.to_string()
        })?;

    // Delete from knowledge_nodes
    sqlx::query("DELETE FROM knowledge_nodes")
        .execute(&mut *tx)
        .await
        .map_err(|e| {
            tracing::error!("Failed to delete knowledge nodes: {}", e);
            e.to_string()
        })?;

    // Delete from ocr_queue
    sqlx::query("DELETE FROM ocr_queue")
        .execute(&mut *tx)
        .await
        .map_err(|e| {
            tracing::error!("Failed to delete OCR queue: {}", e);
            e.to_string()
        })?;

    // Delete from focus_metrics
    sqlx::query("DELETE FROM focus_metrics")
        .execute(&mut *tx)
        .await
        .map_err(|e| {
            tracing::error!("Failed to delete focus metrics: {}", e);
            e.to_string()
        })?;

    // Delete from recording_stats
    sqlx::query("DELETE FROM recording_stats")
        .execute(&mut *tx)
        .await
        .map_err(|e| {
            tracing::error!("Failed to delete recording stats: {}", e);
            e.to_string()
        })?;

    // Delete from terminal_logs
    sqlx::query("DELETE FROM terminal_logs")
        .execute(&mut *tx)
        .await
        .map_err(|e| {
            tracing::error!("Failed to delete terminal logs: {}", e);
            e.to_string()
        })?;

    // Delete from activity_logs (FTS triggers will clean activity_logs_fts)
    let deleted = sqlx::query("DELETE FROM activity_logs")
        .execute(&mut *tx)
        .await
        .map_err(|e| {
            tracing::error!("Failed to delete activity logs: {}", e);
            e.to_string()
        })?;

    tracing::info!("Deleted {} activities from database", deleted.rows_affected());

    // Commit transaction
    tx.commit().await.map_err(|e| {
        tracing::error!("Failed to commit transaction: {}", e);
        e.to_string()
    })?;

    // 4. Delete screenshot files
    for (image_path,) in image_rows {
        let full_path = screenshots_dir.join(&image_path);
        if let Ok(metadata) = std::fs::metadata(&full_path) {
            result.freed_bytes += metadata.len();
        }
        if std::fs::remove_file(&full_path).is_ok() {
            result.deleted_screenshots += 1;
        } else {
            tracing::warn!("Failed to delete screenshot: {:?}", full_path);
        }
    }

    // 5. Clean up FTS table explicitly (triggers should handle this, but let's be safe)
    let _ = sqlx::query("DELETE FROM activity_logs_fts")
        .execute(&pool)
        .await;

    tracing::info!(
        "clear_all_data completed: {} activities, {} screenshots, {} bytes freed",
        result.deleted_activities,
        result.deleted_screenshots,
        result.freed_bytes
    );

    Ok(result)
}

#[tauri::command]
pub async fn enable_autostart() -> Result<(), String> {
    tracing::info!("enable_autostart called");

    #[cfg(target_os = "windows")]
    {
        use winreg::enums::{HKEY_CURRENT_USER, KEY_WRITE};
        use winreg::RegKey;

        // Get the current executable path
        let exe_path = std::env::current_exe()
            .map_err(|e| format!("Failed to get executable path: {}", e))?;

        let exe_path_str = exe_path
            .to_str()
            .ok_or_else(|| "Executable path contains invalid UTF-8 characters".to_string())?;

        // Open the registry key for autostart
        let path = r"Software\Microsoft\Windows\CurrentVersion\Run";
        let hkcu = RegKey::predef(HKEY_CURRENT_USER);
        let key = hkcu
            .open_subkey_with_flags(path, KEY_WRITE)
            .map_err(|e| format!("Failed to open registry key: {}", e))?;

        // Set the registry value
        key.set_value("MemFlow", &exe_path_str)
            .map_err(|e| format!("Failed to set registry value: {}", e))?;

        tracing::info!("Autostart enabled: {}", exe_path_str);
        Ok(())
    }

    #[cfg(not(target_os = "windows"))]
    {
        Err(format!(
            "Autostart is not supported on this platform ({}). Please use platform-specific methods.",
            std::env::consts::OS
        ))
    }
}

#[tauri::command]
pub async fn disable_autostart() -> Result<(), String> {
    tracing::info!("disable_autostart called");

    #[cfg(target_os = "windows")]
    {
        use winreg::enums::{HKEY_CURRENT_USER, KEY_WRITE};
        use winreg::RegKey;

        // Open the registry key for autostart
        let path = r"Software\Microsoft\Windows\CurrentVersion\Run";
        let hkcu = RegKey::predef(HKEY_CURRENT_USER);
        let key = hkcu
            .open_subkey_with_flags(path, KEY_WRITE)
            .map_err(|e| format!("Failed to open registry key: {}", e))?;

        // Delete the registry value if it exists
        match key.delete_value("MemFlow") {
            Ok(_) => {
                tracing::info!("Autostart disabled");
                Ok(())
            }
            Err(e) => {
                // Ignore "value not found" error - it means autostart was already disabled
                if e.raw_os_error() == Some(2) {
                    // ERROR_FILE_NOT_FOUND
                    tracing::info!("Autostart registry value does not exist (already disabled)");
                    Ok(())
                } else {
                    Err(format!("Failed to delete registry value: {}", e))
                }
            }
        }
    }

    #[cfg(not(target_os = "windows"))]
    {
        Err(format!(
            "Autostart is not supported on this platform ({}). Please use platform-specific methods.",
            std::env::consts::OS
        ))
    }
}

#[tauri::command]
pub async fn get_autostart_status() -> Result<AutostartInfo, String> {
    tracing::info!("get_autostart_status called");

    #[cfg(target_os = "windows")]
    {
        use winreg::enums::{HKEY_CURRENT_USER, KEY_READ};
        use winreg::RegKey;

        // Open the registry key for autostart
        let path = r"Software\Microsoft\Windows\CurrentVersion\Run";
        let hkcu = RegKey::predef(HKEY_CURRENT_USER);
        let key = hkcu
            .open_subkey_with_flags(path, KEY_READ)
            .map_err(|e| format!("Failed to open registry key: {}", e))?;

        // Check if the registry value exists
        let enabled = key
            .get_value::<String, _>("MemFlow")
            .ok()
            .is_some();

        tracing::info!("Autostart status: enabled={}", enabled);
        Ok(AutostartInfo {
            enabled,
            app_name: "MemFlow".to_string(),
        })
    }

    #[cfg(not(target_os = "windows"))]
    {
        // For non-Windows platforms, return disabled with platform info
        Ok(AutostartInfo {
            enabled: false,
            app_name: format!("MemFlow (not supported on {})", std::env::consts::OS),
        })
    }
}

/// Helper function to scan a directory and return (file_count, total_size_bytes)
fn scan_directory(dir_path: &std::path::Path) -> Result<(u64, u64), String> {
    if !dir_path.exists() {
        return Ok((0, 0));
    }

    let mut file_count = 0u64;
    let mut total_size = 0u64;

    let entries = std::fs::read_dir(dir_path)
        .map_err(|e| format!("Permission denied or access error reading directory '{}': {}", dir_path.display(), e))?;

    for entry in entries.flatten() {
        let path = entry.path();

        // Skip directories and special files
        if path.is_file() {
            if let Ok(metadata) = std::fs::metadata(&path) {
                file_count += 1;
                total_size += metadata.len();
            }
        }
    }

    Ok((file_count, total_size))
}

#[tauri::command]
pub async fn get_storage_stats(app_handle: tauri::AppHandle) -> Result<StorageStatsResponse, String> {
    tracing::info!("Frontend requested get_storage_stats");

    // Get config for max_storage_gb
    let config = app_config::get_config().await
        .map_err(|e| format!("Failed to read config: {}", e))?;
    let max_storage_gb = config.max_storage_gb as f64;

    // Get database size
    let database_size_bytes = db::get_database_size().await;
    let database_size_mb = database_size_bytes as f64 / 1024.0 / 1024.0;

    // Get activity count from database
    let activities_count = match db::get_pool().await {
        Ok(pool) => {
            match sqlx::query_scalar::<_, i64>("SELECT COUNT(*) FROM activity_logs")
                .fetch_one(&pool)
                .await
            {
                Ok(count) => count as u64,
                Err(e) => {
                    tracing::warn!("Failed to get activity count: {}", e);
                    0
                }
            }
        }
        Err(e) => {
            tracing::warn!("Failed to get database pool: {}", e);
            0
        }
    };

    // Scan screenshots directory for count and size
    let app_data_dir = app_handle.path().app_data_dir()
        .map_err(|e| format!("Failed to get app data directory: {}", e))?;

    let screenshots_dir = app_data_dir.join("screenshots");
    let (screenshots_count, screenshots_size_bytes) = scan_directory(&screenshots_dir)
        .unwrap_or_else(|e| {
            tracing::warn!("Failed to scan screenshots directory '{}': {}", screenshots_dir.display(), e);
            (0, 0)
        });
    let screenshots_size_mb = screenshots_size_bytes as f64 / 1024.0 / 1024.0;

    // Scan logs directory for size
    let logs_dir = app_data_dir.join("logs");
    let logs_size_bytes = scan_directory(&logs_dir)
        .unwrap_or_else(|e| {
            tracing::warn!("Failed to scan logs directory '{}': {}", logs_dir.display(), e);
            (0, 0)
        })
        .1; // Only need size

    // Calculate total size
    let total_size_bytes = database_size_bytes + screenshots_size_bytes + logs_size_bytes;
    let total_size_mb = total_size_bytes as f64 / 1024.0 / 1024.0;

    // Calculate usage percentage
    let usage_percent = if max_storage_gb > 0.0 {
        (total_size_mb / (max_storage_gb * 1024.0)) * 100.0
    } else {
        0.0
    };

    // Calculate next GC time (retention days from config)
    let next_gc_time = if config.retention_days > 0 {
        let next_gc_ts = chrono::Utc::now().timestamp() + (config.retention_days as i64 * 86400);
        Some(chrono::DateTime::from_timestamp(next_gc_ts, 0)
            .unwrap_or_else(|| chrono::Utc::now())
            .to_rfc3339())
    } else {
        None
    };

    Ok(StorageStatsResponse {
        screenshots_count,
        screenshots_size_mb,
        activities_count,
        database_size_mb,
        total_size_mb,
        max_storage_gb,
        usage_percent,
        next_gc_time,
    })
}

#[tauri::command]
pub async fn export_data_json(limit: i64) -> Result<String, String> {
    tracing::info!("Frontend requested export_data_json with limit: {}", limit);

    // Query activities from database
    let activities = db::get_activities(limit)
        .await
        .map_err(|e| {
            tracing::error!("Failed to query activities for export: {}", e);
            format!("Failed to query activities: {}", e)
        })?;

    // Format as JSON array with metadata
    let export_data = serde_json::json!({
        "exportType": "json",
        "version": "1.0",
        "timestamp": chrono::Utc::now().to_rfc3339(),
        "count": activities.len(),
        "activities": activities
    });

    serde_json::to_string_pretty(&export_data)
        .map_err(|e| format!("Failed to serialize JSON: {}", e))
}

#[tauri::command]
pub async fn export_data_markdown(limit: i64) -> Result<String, String> {
    tracing::info!("Frontend requested export_data_markdown with limit: {}", limit);

    // Query activities from database
    let activities = db::get_activities(limit)
        .await
        .map_err(|e| {
            tracing::error!("Failed to query activities for markdown export: {}", e);
            format!("Failed to query activities: {}", e)
        })?;

    if activities.is_empty() {
        return Ok(
            "# MemFlow Activity Export\n\n**No activities found**\n\nThere are no activities to export.".to_string()
        );
    }

    // Build markdown output
    let mut md = String::from("# MemFlow Activity Export\n\n");
    md.push_str(&format!("**Export Date:** {}\n", chrono::Utc::now().format("%Y-%m-%d %H:%M:%S UTC")));
    md.push_str(&format!("**Total Activities:** {}\n\n", activities.len()));
    md.push_str("---\n\n");

    for (idx, activity) in activities.iter().enumerate() {
        md.push_str(&format!("## Activity #{}\n", idx + 1));
        md.push_str(&format!("**ID:** `{}`\n", activity.id));
        md.push_str(&format!("**Timestamp:** `{}`\n", activity.timestamp));

        // Format timestamp as human-readable date
        if let Some(dt) = chrono::DateTime::from_timestamp(activity.timestamp, 0) {
            md.push_str(&format!("**Date:** {}\n", dt.format("%Y-%m-%d %H:%M:%S")));
        }

        md.push_str(&format!("**Application:** `{}`\n", activity.app_name));
        md.push_str(&format!("**Window Title:** `{}`\n", activity.window_title));

        if let Some(ref path) = activity.image_path {
            md.push_str(&format!("**Screenshot:** `{}`\n", path));
        }

        if let Some(ref text) = activity.ocr_text {
            if !text.is_empty() {
                md.push_str("**OCR Text:**\n");
                md.push_str("```\n");
                // Truncate OCR text if too long for readability
                if text.len() > 500 {
                    md.push_str(&text[..500]);
                    md.push_str("...\n");
                } else {
                    md.push_str(text);
                    md.push_str("\n");
                }
                md.push_str("```\n");
            }
        }

        md.push_str("\n---\n\n");
    }

    md.push_str("*Generated by [MemFlow](https://github.com/memflow-app/memflow)*\n");

    Ok(md)
}
