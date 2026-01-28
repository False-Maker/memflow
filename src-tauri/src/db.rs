//! Database module - Tauri wrapper for memflow-core database
//!
//! This module re-exports from memflow_core::db and provides
//! Tauri-specific wrappers for initialization and path resolution.

// Re-export everything from memflow-core db module
pub use memflow_core::db::*;

use anyhow::Result;
use std::path::PathBuf;
use std::time::Duration;
use tauri::{AppHandle, Manager};

// Private Tauri-specific utilities

fn get_db_path(app_handle: &AppHandle) -> Result<PathBuf> {
    app_handle
        .path()
        .app_data_dir()
        .map(|d| d.join("memflow.db"))
        .map_err(|e| anyhow::anyhow!("无法获取应用数据目录: {}", e))
}

/// 获取数据库路径用于诊断（公开函数）
pub fn get_db_path_for_diagnostics(app_handle: &AppHandle) -> Result<PathBuf> {
    get_db_path(app_handle)
}

/// Initialize database using Tauri AppHandle to resolve paths
/// This wraps memflow_core::db::init_db_with_path
pub async fn init_db(app_handle: AppHandle) -> Result<()> {
    let db_path = get_db_path(&app_handle)?;
    let app_data = db_path.parent().unwrap().to_path_buf();
    let screenshots_dir = app_data.join("screenshots");
    
    // Call the core init function with resolved paths
    memflow_core::db::init_db_with_path(db_path, screenshots_dir).await
}

/// Force database recovery - Tauri-specific wrapper
pub async fn force_recovery(app_handle: AppHandle) -> Result<()> {
    // Close existing pool first
    if let Ok(pool) = get_pool().await {
        if sqlx::query("SELECT 1").fetch_one(&pool).await.is_ok() {
            tracing::info!("Database already healthy");
            return Ok(());
        }
    }

    tracing::warn!("Initiating database recovery...");
    tokio::time::sleep(Duration::from_millis(500)).await;

    // Re-initialize with resolved paths
    init_db(app_handle).await?;
    
    tracing::info!("Database recovery completed.");
    Ok(())
}

// Re-import sqlx for the force_recovery function
use sqlx;
