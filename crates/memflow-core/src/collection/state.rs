//! Collection state for MemFlow Core
//!
//! Defines the state management for activity collection

use serde::{Deserialize, Serialize};
use crate::ipc::server::CoreState;

/// Collection state
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum CollectionState {
    /// Collecting activities
    Running,
    /// Collection paused
    Paused,
    /// Not collecting
    Standby,
}

impl Default for CollectionState {
    fn default() -> Self {
        CollectionState::Standby
    }
}

impl From<CoreState> for CollectionState {
    fn from(state: CoreState) -> Self {
        match state {
            CoreState::Running => CollectionState::Running,
            CoreState::Paused => CollectionState::Paused,
            CoreState::Standby => CollectionState::Standby,
        }
    }
}

impl From<CollectionState> for CoreState {
    fn from(state: CollectionState) -> Self {
        match state {
            CollectionState::Running => CoreState::Running,
            CollectionState::Paused => CoreState::Paused,
            CollectionState::Standby => CoreState::Standby,
        }
    }
}

/// Activity record produced by collection
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ActivityRecord {
    /// Unique ID
    pub id: i64,
    /// Timestamp
    pub timestamp: i64,
    /// Application name
    pub app_name: String,
    /// Window title
    pub window_title: String,
    /// Screenshot file path
    pub image_path: Option<String>,
    /// OCR text
    pub ocr_text: Option<String>,
    /// Perceptual hash
    pub phash: Option<String>,
    /// Process path
    pub process_path: Option<String>,
    /// OCR CER score
    pub ocr_cer: Option<f64>,
    /// OCR WER score
    pub ocr_wer: Option<f64>,
    /// OCR quality score
    pub ocr_quality: Option<f64>,
}
