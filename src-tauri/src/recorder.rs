//! Recording controller for MemFlow Desktop
//!
//! This module wraps memflow-core's ActivityCollector and provides
//! a simple interface for the Tauri desktop app.

use crate::app_config;
use crate::commands::ActivityLog;
use crate::db;
use crate::proactive_context;
use crate::window_info as local_window_info;
use anyhow::Result;
use memflow_core::collection::{ActivityCollector, CollectionConfig, ActivityEvent};
use std::sync::Arc;
use tauri::{AppHandle, Emitter};
use tokio::sync::RwLock;

/// Global collector instance
static COLLECTOR: once_cell::sync::Lazy<Arc<CollectorWrapper>> =
    once_cell::sync::Lazy::new(|| Arc::new(CollectorWrapper::new()));

/// Wrapper around ActivityCollector with Tauri integration
struct CollectorWrapper {
    collector: Arc<ActivityCollector>,
    app_handle: RwLock<Option<AppHandle>>,
}

impl CollectorWrapper {
    fn new() -> Self {
        use memflow_core::ipc::server::CoreStateManager;
        
        let state = Arc::new(CoreStateManager::new());
        let collector = Arc::new(ActivityCollector::new(state));
        
        Self {
            collector,
            app_handle: RwLock::new(None),
        }
    }

    async fn init(&self, app_handle: AppHandle) {
        let app_handle_for_callback = app_handle.clone();
        let app_handle_for_events = app_handle.clone();
        
        *self.app_handle.write().await = Some(app_handle.clone());
        
        // Set screenshots directory
        if let Some(screenshots_dir) = db::get_screenshots_dir().await {
            self.collector.set_screenshots_dir(screenshots_dir).await;
        }
        
        // Set proactive context callback
        self.collector.set_proactive_callback(move |core_window_info| {
            // This callback runs in the collector's async context
            // Convert core WindowInfo to local WindowInfo
            let wi = local_window_info::WindowInfo {
                process_name: core_window_info.process_name.clone(),
                title: core_window_info.title.clone(),
                process_path: core_window_info.process_path.clone().unwrap_or_default(),
            };
            // We need to spawn a new task to interact with Tauri
            let handle = app_handle_for_callback.clone();
            tokio::spawn(async move {
                proactive_context::maybe_trigger(&wi, Some(handle));
            });
        }).await;
        
        // Subscribe to activity events
        let collector = self.collector.clone();
        let handle = app_handle_for_events.clone();
        tokio::spawn(async move {
            let mut rx = collector.subscribe();
            loop {
                match rx.recv().await {
                    Ok(event) => {
                        Self::handle_activity_event(&handle, event).await;
                    }
                    Err(e) => {
                        tracing::warn!("Error receiving activity event: {}", e);
                        break;
                    }
                }
            }
        });
        
        tracing::info!("Collector wrapper initialized");
    }

    async fn handle_activity_event(app_handle: &AppHandle, event: ActivityEvent) {
        let activity = event.activity;
        
        // Emit new-activity event to frontend
        let activity_log = ActivityLog {
            id: activity.id,
            timestamp: activity.timestamp,
            app_name: activity.app_name,
            window_title: activity.window_title,
            image_path: activity.image_path,
            ocr_text: activity.ocr_text,
            phash: activity.phash,
            ocr_cer: activity.ocr_cer,
            ocr_wer: activity.ocr_wer,
            ocr_quality: activity.ocr_quality,
        };
        
        if let Err(e) = app_handle.emit("new-activity", &activity_log) {
            tracing::warn!("Failed to emit new-activity event: {}", e);
        }
        
        // If OCR is needed, notify OCR worker
        if event.needs_ocr {
            // OCR worker will pick up from database
            tracing::debug!("OCR needed for activity {}", activity.id);
        }
    }

    async fn start(&self) -> Result<()> {
        // Get config and update collector
        if let Ok(config) = app_config::get_config().await {
            let collection_config = CollectionConfig {
                recording_interval_ms: config.recording_interval,
                ocr_enabled: config.ocr_enabled,
                ocr_engine: "rapidocr".to_string(),
                ocr_preprocess_enabled: true,
                ocr_preprocess_target_width: 1280,
                ocr_preprocess_max_pixels: 3_000_000,
                ocr_redaction_enabled: config.privacy_mode_enabled,
                ocr_redaction_level: "basic".to_string(),
                privacy_mode_enabled: config.privacy_mode_enabled,
                privacy_mode_until: config.privacy_mode_until,
                pause_recording_enabled: config.pause_recording_enabled,
                pause_until: config.pause_until,
                blocklist_enabled: config.blocklist_enabled,
                blocklist_mode: config.blocklist_mode,
                compression_quality: config.compression_quality,
                target_resolution_scale: config.target_resolution_scale,
            };
            self.collector.update_config(collection_config).await;
        }
        
        // Ensure screenshots directory is set
        if let Some(screenshots_dir) = db::get_screenshots_dir().await {
            self.collector.set_screenshots_dir(screenshots_dir).await;
        }
        
        self.collector.start().await?;
        
        // Emit recording status
        if let Some(handle) = self.app_handle.read().await.as_ref() {
            let _ = handle.emit("recording-status", true);
        }
        
        // Spawn focus analytics if enabled
        crate::focus_analytics::spawn_if_enabled();
        
        Ok(())
    }

    async fn stop(&self) -> Result<()> {
        self.collector.stop().await;
        
        // Emit recording status
        if let Some(handle) = self.app_handle.read().await.as_ref() {
            let _ = handle.emit("recording-status", false);
        }
        
        Ok(())
    }

    fn is_recording(&self) -> bool {
        self.collector.is_recording()
    }
}

/// Initialize the recorder with app handle
pub fn init(app_handle: AppHandle) {
    let app_handle_clone = app_handle.clone();
    tauri::async_runtime::spawn(async move {
        COLLECTOR.init(app_handle_clone).await;
    });
}

/// Start recording
pub fn start() -> Result<()> {
    if COLLECTOR.is_recording() {
        return Err(anyhow::anyhow!("Recording already in progress"));
    }

    tauri::async_runtime::spawn(async move {
        if let Err(e) = COLLECTOR.start().await {
            tracing::error!("Failed to start recording: {}", e);
        }
    });

    Ok(())
}

/// Stop recording
pub fn stop() -> Result<()> {
    tauri::async_runtime::spawn(async move {
        if let Err(e) = COLLECTOR.stop().await {
            tracing::error!("Failed to stop recording: {}", e);
        }
    });

    Ok(())
}

/// Check if recording is active
pub fn is_recording() -> bool {
    COLLECTOR.is_recording()
}

/// Set recording interval (for external configuration changes)
pub fn set_interval(ms: u64) {
    // This would need to be exposed by the collector
    // For now, config changes are handled via update_config
    tracing::debug!("Set interval called with {}ms", ms);
}

/// Set base interval for recording (legacy compatibility)
pub fn set_base_interval(ms: u64) {
    set_interval(ms);
}
