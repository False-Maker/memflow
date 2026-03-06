//! IPC Commands for MemFlow Core
//!
//! Defines all IPC commands that Core can handle

use serde::{Deserialize, Serialize};
use crate::ipc::protocol::IpcError;

/// Core state enumeration
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum CoreState {
    /// Core is running and collecting data
    Running,
    /// Core is paused (collecting is paused)
    Paused,
    /// Core is in standby mode (idle)
    Standby,
}

impl Default for CoreState {
    fn default() -> Self {
        CoreState::Standby
    }
}

impl std::fmt::Display for CoreState {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            CoreState::Running => write!(f, "running"),
            CoreState::Paused => write!(f, "paused"),
            CoreState::Standby => write!(f, "standby"),
        }
    }
}

/// Core status response
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct CoreStatus {
    pub state: CoreState,
    pub recording_interval_ms: u64,
    pub privacy_mode_enabled: bool,
    pub pause_recording_enabled: bool,
    pub blocklist_enabled: bool,
    pub ocr_enabled: bool,
}

impl Default for CoreStatus {
    fn default() -> Self {
        Self {
            state: CoreState::Standby,
            recording_interval_ms: 5000,
            privacy_mode_enabled: false,
            pause_recording_enabled: false,
            blocklist_enabled: false,
            ocr_enabled: true,
        }
    }
}

/// Configuration update request
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ConfigUpdateRequest {
    pub recording_interval_ms: Option<u64>,
    pub privacy_mode_enabled: Option<bool>,
    pub pause_recording_enabled: Option<bool>,
    pub blocklist_enabled: Option<bool>,
    pub ocr_enabled: Option<bool>,
}

/// Activity record structure
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ActivityRecord {
    pub id: i64,
    pub timestamp: i64,
    pub app_name: String,
    pub window_title: String,
    pub image_path: Option<String>,
    pub ocr_text: Option<String>,
    pub phash: Option<String>,
    pub ocr_cer: Option<f64>,
    pub ocr_wer: Option<f64>,
    pub ocr_quality: Option<f64>,
}

/// Search query request
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct SearchQueryRequest {
    pub query: String,
    pub mode: Option<String>,  // "hybrid", "semantic", "keyword"
    pub limit: Option<i64>,
    pub app_name: Option<String>,
    pub date_range: Option<DateRange>,
    pub has_ocr: Option<bool>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct DateRange {
    pub start: i64,
    pub end: i64,
}

/// Recent activity request
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct RecentActivityRequest {
    pub minutes: Option<i64>,  // Default: 30
    pub limit: Option<i64>,
}

/// IPC Command trait - implement this to add new commands
pub trait IpcCommand: Send + Sync {
    /// Command name
    fn name(&self) -> &str;
    
    /// Execute the command
    fn execute(&self, params: Option<serde_json::Value>) -> Result<serde_json::Value, IpcError>;
}
