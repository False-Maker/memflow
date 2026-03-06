//! System service management module for MemFlow
//!
//! This module provides functionality to:
//! - Enable/disable auto-start on system boot
//! - Check service status
//! - Manage Windows startup entries
//!
//! Uses the tauri-plugin-autostart plugin for cross-platform auto-start support.

use tauri::AppHandle;
use tauri_plugin_autostart::ManagerExt;

/// Result type for service operations
pub type ServiceResult<T> = Result<T, ServiceError>;

/// Service error types
#[derive(Debug, thiserror::Error)]
pub enum ServiceError {
    #[error("Failed to enable auto-start: {0}")]
    EnableFailed(String),

    #[error("Failed to disable auto-start: {0}")]
    DisableFailed(String),

    #[error("Failed to get auto-start status: {0}")]
    StatusFailed(String),
}

/// Enable auto-start on system boot
pub fn enable_autostart(app: &AppHandle) -> ServiceResult<()> {
    let autostart_manager = app.autolaunch();

    match autostart_manager.enable() {
        Ok(_) => {
            tracing::info!("Auto-start enabled successfully");
            Ok(())
        }
        Err(e) => {
            let msg = format!("Failed to enable auto-start: {}", e);
            tracing::error!("{}", msg);
            Err(ServiceError::EnableFailed(msg))
        }
    }
}

/// Disable auto-start on system boot
pub fn disable_autostart(app: &AppHandle) -> ServiceResult<()> {
    let autostart_manager = app.autolaunch();

    match autostart_manager.disable() {
        Ok(_) => {
            tracing::info!("Auto-start disabled successfully");
            Ok(())
        }
        Err(e) => {
            let msg = format!("Failed to disable auto-start: {}", e);
            tracing::error!("{}", msg);
            Err(ServiceError::DisableFailed(msg))
        }
    }
}

/// Check if auto-start is enabled
pub fn is_autostart_enabled(app: &AppHandle) -> ServiceResult<bool> {
    let autostart_manager = app.autolaunch();

    match autostart_manager.is_enabled() {
        Ok(enabled) => Ok(enabled),
        Err(e) => {
            let msg = format!("Failed to get auto-start status: {}", e);
            tracing::error!("{}", msg);
            Err(ServiceError::StatusFailed(msg))
        }
    }
}

/// Get auto-start configuration info
pub fn get_autostart_info(app: &AppHandle) -> ServiceResult<AutostartInfo> {
    let autostart_manager = app.autolaunch();
    let enabled = autostart_manager.is_enabled().map_err(|e| {
        ServiceError::StatusFailed(format!("Failed to get autostart status: {}", e))
    })?;

    // Get app name from the plugin - use a default if not available
    let app_name = "MemFlow".to_string();

    Ok(AutostartInfo {
        enabled,
        app_name,
    })
}

/// Auto-start information
#[derive(Debug, Clone, serde::Serialize)]
#[serde(rename_all = "camelCase")]
pub struct AutostartInfo {
    pub enabled: bool,
    pub app_name: String,
}

/// Initialize autostart plugin
pub fn init_autostart(app: &AppHandle) {
    let autostart = app.autolaunch();

    // Enable logging of autostart status
    match autostart.is_enabled() {
        Ok(enabled) => {
            tracing::info!("Auto-start is currently {}", if enabled { "enabled" } else { "disabled" });
        }
        Err(e) => {
            tracing::warn!("Failed to check autostart status: {}", e);
        }
    }
}
