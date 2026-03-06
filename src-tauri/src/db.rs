//! Database module - Tauri wrapper for memflow-core database
//!
//! This module re-exports from memflow_core::db and provides
//! Tauri-specific wrappers for initialization and path resolution.

// Re-export everything from memflow_core db module
pub use memflow_core::db::*;

use crate::desktop_context::TauriContext;
use memflow_core::context::RuntimeContext;
use anyhow::Result;
use std::path::PathBuf;
use std::time::Duration;
use tauri::AppHandle;

// Private utilities built on top of the unified RuntimeContext

fn get_db_path_from_ctx(ctx: &impl RuntimeContext) -> PathBuf {
    // All platform-specific logic (Tauri / MCP / CLI) should be expressed
    // in the concrete RuntimeContext implementation. Here we only rely on
    // the logical app_dir and derive the DB file name from it.
    ctx.app_dir().join("memflow.db")
}

fn get_screenshots_dir_from_ctx(ctx: &impl RuntimeContext) -> PathBuf {
    ctx.app_dir().join("screenshots")
}

/// 获取数据库路径用于诊断（公开函数）
pub fn get_db_path_for_diagnostics(app_handle: &AppHandle) -> Result<PathBuf> {
    let ctx = TauriContext::new(app_handle.clone());
    Ok(get_db_path_from_ctx(&ctx))
}

/// Initialize database using RuntimeContext (Tauri wrapper)
/// This wraps memflow_core::db::init_db_with_path
pub async fn init_db(app_handle: AppHandle) -> Result<()> {
    let ctx = TauriContext::new(app_handle);
    let db_path = get_db_path_from_ctx(&ctx);
    let screenshots_dir = get_screenshots_dir_from_ctx(&ctx);

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

    // Re-initialize with resolved paths (through unified RuntimeContext)
    init_db(app_handle).await?;
    
    tracing::info!("Database recovery completed.");
    Ok(())
}

// Re-import sqlx for the force_recovery function
use sqlx;

/// 获取数据库文件大小（字节）
pub async fn get_database_size() -> u64 {
    if let Some(db_path) = memflow_core::db::get_db_path().await {
        if db_path.exists() {
            if let Ok(meta) = std::fs::metadata(&db_path) {
                return meta.len();
            }
        }
    }
    0
}

/// 获取数据库文件大小（MB）
pub async fn get_database_size_mb() -> f64 {
    get_database_size().await as f64 / 1024.0 / 1024.0
}