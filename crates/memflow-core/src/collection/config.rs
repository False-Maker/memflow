//! Collection configuration for MemFlow Core
//!
//! Defines configuration options for activity collection

use serde::{Deserialize, Serialize};

/// Configuration for activity collection
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct CollectionConfig {
    /// Recording interval in milliseconds
    #[serde(default = "default_recording_interval")]
    pub recording_interval_ms: u64,
    
    /// Enable OCR processing
    #[serde(default = "default_ocr_enabled")]
    pub ocr_enabled: bool,
    
    /// OCR engine to use
    #[serde(default = "default_ocr_engine")]
    pub ocr_engine: String,
    
    /// Enable OCR preprocessing
    #[serde(default = "default_ocr_preprocess_enabled")]
    pub ocr_preprocess_enabled: bool,
    
    /// Target width for OCR preprocessing
    #[serde(default = "default_ocr_preprocess_target_width")]
    pub ocr_preprocess_target_width: u32,
    
    /// Maximum pixels for OCR preprocessing
    #[serde(default = "default_ocr_preprocess_max_pixels")]
    pub ocr_preprocess_max_pixels: u64,
    
    /// Enable OCR redaction
    #[serde(default = "default_ocr_redaction_enabled")]
    pub ocr_redaction_enabled: bool,
    
    /// OCR redaction level
    #[serde(default = "default_ocr_redaction_level")]
    pub ocr_redaction_level: String,
    
    /// Privacy mode enabled
    #[serde(default)]
    pub privacy_mode_enabled: bool,
    
    /// Privacy mode until timestamp
    #[serde(default)]
    pub privacy_mode_until: Option<i64>,
    
    /// Pause recording enabled
    #[serde(default)]
    pub pause_recording_enabled: bool,
    
    /// Pause until timestamp
    #[serde(default)]
    pub pause_until: Option<i64>,
    
    /// Blocklist enabled
    #[serde(default)]
    pub blocklist_enabled: bool,
    
    /// Blocklist mode (blocklist/allowlist)
    #[serde(default = "default_blocklist_mode")]
    pub blocklist_mode: String,
    
    /// Compression quality for screenshots (0-100)
    #[serde(default = "default_compression_quality")]
    pub compression_quality: u8,
    
    /// Target resolution scale
    #[serde(default = "default_target_resolution_scale")]
    pub target_resolution_scale: f32,
}

fn default_recording_interval() -> u64 { 5000 }
fn default_ocr_enabled() -> bool { true }
fn default_ocr_engine() -> String { "rapidocr".to_string() }
fn default_ocr_preprocess_enabled() -> bool { true }
fn default_ocr_preprocess_target_width() -> u32 { 1280 }
fn default_ocr_preprocess_max_pixels() -> u64 { 3_000_000 }
fn default_ocr_redaction_enabled() -> bool { true }
fn default_ocr_redaction_level() -> String { "basic".to_string() }
fn default_blocklist_mode() -> String { "blocklist".to_string() }
fn default_compression_quality() -> u8 { 80 }
fn default_target_resolution_scale() -> f32 { 1.0 }

impl Default for CollectionConfig {
    fn default() -> Self {
        Self {
            recording_interval_ms: 5000,
            ocr_enabled: true,
            ocr_engine: "rapidocr".to_string(),
            ocr_preprocess_enabled: true,
            ocr_preprocess_target_width: 1280,
            ocr_preprocess_max_pixels: 3_000_000,
            ocr_redaction_enabled: true,
            ocr_redaction_level: "basic".to_string(),
            privacy_mode_enabled: false,
            privacy_mode_until: None,
            pause_recording_enabled: false,
            pause_until: None,
            blocklist_enabled: false,
            blocklist_mode: "blocklist".to_string(),
            compression_quality: 80,
            target_resolution_scale: 1.0,
        }
    }
}
