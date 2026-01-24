// 重新导出以便其他模块使用
pub use crate::commands::AppConfig;
use anyhow::Result;
use serde_json;
use std::path::PathBuf;
use std::sync::Arc;
use tauri::{AppHandle, Manager};
use tokio::sync::RwLock;

static CONFIG: once_cell::sync::Lazy<Arc<RwLock<Option<AppConfig>>>> =
    once_cell::sync::Lazy::new(|| Arc::new(RwLock::new(None)));

pub async fn init_config(app_handle: AppHandle) -> Result<()> {
    let app_data = app_handle
        .path()
        .app_data_dir()
        .map_err(|e| anyhow::anyhow!("无法获取应用数据目录: {}", e))?;

    std::fs::create_dir_all(&app_data)?;

    let config_path = app_data.join("config.json");

    // 如果配置文件存在，加载它
    if config_path.exists() {
        let content = std::fs::read_to_string(&config_path)?;
        let config: AppConfig = serde_json::from_str(&content)?;
        *CONFIG.write().await = Some(config);
    } else {
        // 使用默认配置
        let default_config = AppConfig {
            recording_interval: 5000,
            ocr_enabled: true,
            ocr_engine: "rapidocr".to_string(),
            ai_enabled: false,
            retention_days: 30,
            chat_model: "gpt-4o-mini".to_string(),
            embedding_model: "text-embedding-3-small".to_string(),
            embedding_base_url: None,
            embedding_use_shared_key: true,
            openai_base_url: None,
            anthropic_base_url: None,
            blocklist_enabled: false,
            blocklist_mode: "blocklist".to_string(),
            privacy_mode_enabled: false,
            privacy_mode_until: None,
            intent_parse_timeout_ms: Some(20_000),
            enable_focus_analytics: true,
            enable_proactive_assistant: false,
            ocr_redaction_enabled: true,
            ocr_redaction_level: "basic".to_string(),
            ocr_preprocess_enabled: true,
            ocr_preprocess_target_width: 1280,
            ocr_preprocess_max_pixels: 3_000_000,
            agent_note_path: None,
        };
        save_config_internal(&config_path, &default_config).await?;
        *CONFIG.write().await = Some(default_config);
    }

    Ok(())
}

async fn save_config_internal(config_path: &PathBuf, config: &AppConfig) -> Result<()> {
    let content = serde_json::to_string_pretty(config)?;
    
    // 原子性写入：先写临时文件，再重命名
    let temp_path = config_path.with_extension("json.tmp");
    std::fs::write(&temp_path, &content)?;
    
    // Windows 上 rename 不会覆盖已存在文件，需要先删除
    if config_path.exists() {
        std::fs::remove_file(config_path)?;
    }
    std::fs::rename(&temp_path, config_path)?;
    
    tracing::debug!("配置已保存到: {:?}", config_path);
    Ok(())
}

pub async fn get_config() -> Result<AppConfig> {
    CONFIG
        .read()
        .await
        .clone()
        .ok_or_else(|| anyhow::anyhow!("配置未初始化"))
}

pub async fn update_config(config: AppConfig, app_handle: AppHandle) -> Result<()> {
    *CONFIG.write().await = Some(config.clone());

    // 持久化到文件
    let app_data = app_handle
        .path()
        .app_data_dir()
        .map_err(|e| anyhow::anyhow!("无法获取应用数据目录: {}", e))?;

    let config_path = app_data.join("config.json");
    save_config_internal(&config_path, &config).await?;

    Ok(())
}
