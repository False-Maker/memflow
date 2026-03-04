//! Activity collector for MemFlow Core
//!
//! Main collector that orchestrates activity collection with:
//! - Event-driven capture (window focus changes)
//! - Heartbeat mechanism (periodic sampling)
//! - Smart heartbeat interval adjustment
//! - Hybrid deduplication (visual + text)
//! - UIA text extraction (priority over OCR)
//! - Terminal output logging

use std::sync::Arc;
use std::sync::atomic::{AtomicBool, AtomicU64, Ordering};
use std::time::Instant;
use tokio::sync::{broadcast, RwLock as AsyncRwLock, Mutex as AsyncMutex};
use anyhow::Result;

use super::config::CollectionConfig;
use super::state::{CollectionState, ActivityRecord};
use super::capture;
use super::window::{self, WindowInfo};
use super::uia;
use super::win_event::{EventDrivenRecorder, EventLoopConfig, WindowEvent};
use crate::db;
use crate::ipc::server::CoreStateManager;

/// Event emitted when new activity is captured
#[derive(Debug, Clone)]
pub struct ActivityEvent {
    /// The captured activity
    pub activity: ActivityRecord,
    /// Whether OCR is needed (UIA failed)
    pub needs_ocr: bool,
}

/// Callback for proactive context triggering
pub type ProactiveContextCallback = Box<dyn Fn(&WindowInfo) + Send + Sync>;

/// Main collector orchestrator
pub struct ActivityCollector {
    /// Current configuration (wrapped in Arc for sharing)
    config: Arc<AsyncRwLock<CollectionConfig>>,
    /// Current state
    state: Arc<CoreStateManager>,
    /// Event broadcast channel
    event_tx: broadcast::Sender<ActivityEvent>,
    /// Whether recording is active
    recording: AtomicBool,
    /// Last captured hash (for deduplication)
    last_hash: AsyncMutex<Option<u64>>,
    /// Last text hash (for deduplication)
    last_text_hash: AsyncMutex<Option<u64>>,
    /// Current heartbeat interval (milliseconds)
    heartbeat_ms: AtomicU64,
    /// Base recording interval (milliseconds)
    base_interval_ms: AtomicU64,
    /// Screenshots directory path
    screenshots_dir: Arc<AsyncMutex<Option<std::path::PathBuf>>>,
    /// Event recorder (Windows-specific)
    event_recorder: AsyncMutex<Option<EventDrivenRecorder>>,
    /// Callback for proactive context
    proactive_callback: AsyncMutex<Option<Arc<ProactiveContextCallback>>>,
}

impl ActivityCollector {
    /// Create a new collector
    pub fn new(state: Arc<CoreStateManager>) -> Self {
        let (event_tx, _) = broadcast::channel(100);
        
        Self {
            config: Arc::new(AsyncRwLock::new(CollectionConfig::default())),
            state,
            event_tx,
            recording: AtomicBool::new(false),
            last_hash: AsyncMutex::new(None),
            last_text_hash: AsyncMutex::new(None),
            heartbeat_ms: AtomicU64::new(5000),
            base_interval_ms: AtomicU64::new(5000),
            screenshots_dir: Arc::new(AsyncMutex::new(None)),
            event_recorder: AsyncMutex::new(None),
            proactive_callback: AsyncMutex::new(None),
        }
    }

    /// Set screenshots directory
    pub async fn set_screenshots_dir(&self, path: std::path::PathBuf) {
        *self.screenshots_dir.lock().await = Some(path);
    }

    /// Set proactive context callback
    pub async fn set_proactive_callback<F>(&self, callback: F)
    where
        F: Fn(&WindowInfo) + Send + Sync + 'static,
    {
        *self.proactive_callback.lock().await = Some(Arc::new(Box::new(callback)));
    }

    /// Update configuration
    pub async fn update_config(&self, config: CollectionConfig) {
        let interval = config.recording_interval_ms;
        *self.config.write().await = config;
        self.set_base_interval(interval);
    }

    /// Get current configuration
    pub async fn get_config(&self) -> CollectionConfig {
        self.config.read().await.clone()
    }

    /// Get current state
    pub async fn get_state(&self) -> CollectionState {
        let core_state = self.state.get_state().await;
        CollectionState::from(core_state)
    }

    /// Subscribe to activity events
    pub fn subscribe(&self) -> broadcast::Receiver<ActivityEvent> {
        self.event_tx.subscribe()
    }

    /// Set base recording interval
    fn set_base_interval(&self, ms: u64) {
        self.base_interval_ms.store(ms, Ordering::Relaxed);
        self.heartbeat_ms.store(ms, Ordering::Relaxed);
        tracing::info!("Recording interval updated to {}ms", ms);
    }

    /// Adjust heartbeat interval based on duplicate detection
    fn adjust_heartbeat(&self, on_duplicate: bool) {
        let base = self.base_interval_ms.load(Ordering::Relaxed);
        let max = 60_000_u64; // Max 60s
        let step = 500_u64; // Step 500ms
        
        loop {
            let current = self.heartbeat_ms.load(Ordering::Relaxed);
            let next = if on_duplicate {
                (current + step).min(max)
            } else {
                if current > base {
                    current.saturating_sub(step).max(base)
                } else {
                    base
                }
            };
            
            if self.heartbeat_ms
                .compare_exchange(current, next, Ordering::Relaxed, Ordering::Relaxed)
                .is_ok()
            {
                break;
            }
        }
    }

    /// Check if app is in blocklist
    async fn is_blocked(&self, app_name: &str) -> bool {
        let config = self.config.read().await.clone();
        if !config.blocklist_enabled {
            return false;
        }

        let normalized = window::normalize_app_name(app_name);

        match db::get_blocklist().await {
            Ok(blocklist) => {
                let is_blocked = match config.blocklist_mode.as_str() {
                    "allowlist" => {
                        !blocklist.iter().any(|app| {
                            window::normalize_app_name(app) == normalized
                        })
                    }
                    _ => {
                        blocklist.iter().any(|app| {
                            window::normalize_app_name(app) == normalized
                        })
                    }
                };
                is_blocked
            }
            Err(e) => {
                tracing::warn!("Failed to load blocklist from database: {}", e);
                false
            }
        }
    }

    /// Start recording
    pub async fn start(&self) -> Result<()> {
        if self.recording.swap(true, Ordering::SeqCst) {
            return Err(anyhow::anyhow!("Recording already in progress"));
        }

        // Initialize event-driven recorder
        let event_config = EventLoopConfig {
            track_foreground: true,
            track_lifecycle: false,
            track_title_change: false,
            debounce_ms: 100,
        };
        
        let mut recorder = EventDrivenRecorder::new(event_config);
        let event_rx = recorder.start();
        
        *self.event_recorder.lock().await = Some(recorder);

        // Start the collection loop
        let collector = self.clone_for_task();
        let event_rx = event_rx;
        
        tokio::spawn(async move {
            collector.event_driven_loop(event_rx).await;
        });

        tracing::info!("Activity collector started");
        Ok(())
    }

    /// Stop recording
    pub async fn stop(&self) {
        self.recording.store(false, Ordering::SeqCst);
        
        // Stop event recorder
        if let Some(mut recorder) = self.event_recorder.lock().await.take() {
            recorder.stop();
        }
        
        tracing::info!("Activity collector stopped");
    }

    /// Check if recording is active
    pub fn is_recording(&self) -> bool {
        self.recording.load(Ordering::SeqCst)
    }

    /// Event-driven collection loop
    async fn event_driven_loop(&self, mut event_rx: tokio::sync::mpsc::Receiver<WindowEvent>) {
        let debounce_duration = tokio::time::Duration::from_millis(500);
        let mut last_capture_time = Instant::now();

        tracing::info!("Starting event-driven collection loop");

        while self.recording.load(Ordering::SeqCst) {
            let sleep_ms = self.heartbeat_ms.load(Ordering::Relaxed);
            
            tokio::select! {
                // A. Respond to system events (window focus change)
                Some(event) = event_rx.recv() => {
                    if let WindowEvent::ForegroundChanged { .. } = event {
                        // Debounce check: don't process within 500ms
                        if last_capture_time.elapsed() > debounce_duration {
                            last_capture_time = Instant::now();
                            tracing::debug!("Event-driven trigger capture");
                            match self.capture_and_save().await {
                                Ok(_) => {}
                                Err(e) => {
                                    tracing::warn!("Event-driven capture failed: {:?}", e);
                                }
                            }
                        } else {
                            tracing::debug!("Event debounce skip, elapsed: {:?}", last_capture_time.elapsed());
                        }
                    }
                }
                // B. Fallback heartbeat (periodic sampling to prevent missed captures in static scenarios)
                _ = tokio::time::sleep(tokio::time::Duration::from_millis(sleep_ms)) => {
                    if self.recording.load(Ordering::SeqCst) {
                        tracing::debug!("Heartbeat trigger capture ({}ms)", sleep_ms);
                        match self.capture_and_save().await {
                            Ok(_) => {}
                            Err(e) => {
                                tracing::warn!("Heartbeat capture failed: {:?}", e);
                            }
                        }
                        last_capture_time = Instant::now();
                    }
                }
            }
        }
        
        tracing::info!("Exited event-driven collection loop");
    }

    /// Capture and save activity
    pub async fn capture_and_save(&self) -> Result<Option<ActivityRecord>> {
        let config = self.config.read().await.clone();
        
        // Check pause mode
        let now = chrono::Utc::now().timestamp();
        
        if config.pause_recording_enabled {
            if let Some(until) = config.pause_until {
                if now > until {
                    let mut cfg = self.config.write().await;
                    cfg.pause_recording_enabled = false;
                    cfg.pause_until = None;
                } else {
                    tracing::debug!("Recording paused, skipping capture");
                    return Ok(None);
                }
            } else {
                tracing::debug!("Recording paused (indefinite), skipping capture");
                return Ok(None);
            }
        }
        
        // Check privacy mode
        if config.privacy_mode_enabled {
            if let Some(until) = config.privacy_mode_until {
                if now > until {
                    let mut cfg = self.config.write().await;
                    cfg.privacy_mode_enabled = false;
                    cfg.privacy_mode_until = None;
                } else {
                    tracing::debug!("Privacy mode active, skipping capture");
                    return Ok(None);
                }
            } else {
                tracing::debug!("Privacy mode active, skipping capture");
                return Ok(None);
            }
        }
        
        // Get window info
        let window_info = match window::get_foreground_window_info() {
            Ok(info) => info,
            Err(e) => {
                tracing::warn!("Failed to get window info: {}", e);
                WindowInfo::default()
            }
        };
        
        // Trigger proactive context callback
        if let Some(callback) = self.proactive_callback.lock().await.as_ref() {
            callback(&window_info);
        }
        
        // Check blocklist
        if self.is_blocked(&window_info.process_name).await {
            tracing::debug!("App {} is in blocklist, skipping", window_info.process_name);
            return Ok(None);
        }
        
        // Extract UIA text
        let uia_text = tokio::task::spawn_blocking({
            let hwnd = window_info.hwnd;
            move || -> Result<Option<String>> {
                match hwnd {
                    Some(h) => uia::extract_uia_text(h).map(|r| r.text),
                    None => Ok(None),
                }
            }
        }).await??;
        
        let uia_text = uia_text.and_then(|text| {
            if text.trim().is_empty() {
                None
            } else {
                Some(text)
            }
        });
        
        // Capture screen
        let capture_result = capture::capture_screen()?;
        
        // Calculate hash and encode to webp
        let current_hash = capture::calculate_phash(&capture_result.image);
        
        // Scale image if needed
        let final_image = if (config.target_resolution_scale - 1.0).abs() > 0.01 
            && config.target_resolution_scale > 0.0 
            && config.target_resolution_scale < 1.0 
        {
            let new_width = (capture_result.width as f32 * config.target_resolution_scale) as u32;
            let new_height = (capture_result.height as f32 * config.target_resolution_scale) as u32;
            capture_result.image.resize(new_width, new_height, image::imageops::FilterType::Lanczos3)
        } else {
            capture_result.image
        };
        
        // Encode to WebP
        let webp_data = capture::encode_webp(&final_image, config.compression_quality as f32)?;
        
        let phash_str = format!("{:016x}", current_hash);
        
        // Calculate text hash for deduplication
        let current_text_hash = uia_text.as_ref().map(|text| {
            use std::collections::hash_map::DefaultHasher;
            use std::hash::{Hash, Hasher};
            let mut hasher = DefaultHasher::new();
            text.hash(&mut hasher);
            hasher.finish()
        });
        
        // Hybrid deduplication: only skip if both visual and text are unchanged
        let last_hash = self.last_hash.lock().await.clone();
        let last_text_hash = self.last_text_hash.lock().await.clone();
        
        let visual_changed = if let Some(last) = last_hash {
            !capture::is_similar(current_hash, last, 5)
        } else {
            true // First run - treat as changed
        };
        
        let text_changed = match (&current_text_hash, &last_text_hash) {
            (Some(curr), Some(last)) => curr != last,
            (Some(_), None) | (None, Some(_)) => true,
            (None, None) => false,
        };
        
        if !visual_changed && !text_changed {
            tracing::debug!("Duplicate detected: visual unchanged, text unchanged, skipping");
            self.adjust_heartbeat(true);
            return Ok(None);
        }
        
        tracing::debug!("Detected change: visual_changed={}, text_changed={}", visual_changed, text_changed);
        
        // Save screenshot
        let filename = self.save_screenshot(&webp_data, current_hash).await?;
        
        // Save to database
        let activity_id = db::insert_activity(
            now,
            &window_info.process_name,
            &window_info.title,
            &filename,
            Some(&phash_str),
            window_info.process_path.as_deref(),
        ).await?;
        
        // Update last hashes
        *self.last_hash.lock().await = Some(current_hash);
        *self.last_text_hash.lock().await = current_text_hash;
        
        // Create activity record
        let activity = ActivityRecord {
            id: activity_id,
            timestamp: now,
            app_name: window_info.process_name.clone(),
            window_title: window_info.title.clone(),
            image_path: Some(filename.clone()),
            ocr_text: None,
            phash: Some(phash_str),
            process_path: window_info.process_path.clone(),
            ocr_cer: None,
            ocr_wer: None,
            ocr_quality: None,
        };
        
        // Handle terminal output logging
        if let Some(ref text) = uia_text {
            if window::is_terminal_process(&window_info.process_name) {
                let session_id = format!("{}|{}", 
                    window_info.process_path.as_deref().unwrap_or(""),
                    window_info.title
                );
                let app_name = window_info.process_name.clone();
                let window_title = window_info.title.clone();
                let text_to_log = text.clone();
                
                tokio::spawn(async move {
                    if let Err(e) = db::insert_terminal_output(
                        Some(&session_id),
                        Some(&app_name),
                        Some(&window_title),
                        &text_to_log,
                    ).await {
                        tracing::debug!("Failed to insert terminal output log: {}", e);
                    }
                });
            }
        }
        
        // Handle text: use UIA result directly, or fall back to OCR
        let needs_ocr = if let Some(ref text) = uia_text {
            // UIA success: update database directly, skip OCR
            if let Err(e) = db::update_activity_ocr(activity_id, text).await {
                tracing::error!("Failed to update UIA text: {}", e);
            }
            false
        } else {
            // UIA failed: fall back to OCR
            if config.ocr_enabled {
                tokio::spawn(async move {
                    if let Err(e) = db::enqueue_ocr_task(activity_id).await {
                        tracing::warn!("OCR enqueue failed: {}", e);
                    }
                });
            }
            true
        };
        
        // Emit event
        let event = ActivityEvent {
            activity: activity.clone(),
            needs_ocr,
        };
        
        let _ = self.event_tx.send(event);
        
        tracing::info!("Captured activity: {} - {}", activity.app_name, activity.window_title);
        
        self.adjust_heartbeat(false);
        
        Ok(Some(activity))
    }

    /// Save screenshot to disk
    async fn save_screenshot(&self, webp_data: &[u8], phash: u64) -> Result<String> {
        let dir = {
            let guard = self.screenshots_dir.lock().await;
            match guard.as_ref() {
                Some(d) => d.clone(),
                None => {
                    return Err(anyhow::anyhow!("Screenshots directory not set"));
                }
            }
        };
        
        let timestamp = chrono::Utc::now().timestamp();
        let phash_short = format!("{:016x}", phash);
        let filename = format!("{}_{}.webp", timestamp, &phash_short[..16]);
        let filepath = dir.join(&filename);
        
        tokio::fs::write(&filepath, webp_data).await?;
        
        tracing::debug!("Saved screenshot: {}", filename);
        
        Ok(filename)
    }

    /// Clone the collector for use in spawned tasks
    fn clone_for_task(&self) -> Self {
        Self {
            config: self.config.clone(),
            state: self.state.clone(),
            event_tx: self.event_tx.clone(),
            recording: AtomicBool::new(self.recording.load(Ordering::SeqCst)),
            last_hash: AsyncMutex::new(None),
            last_text_hash: AsyncMutex::new(None),
            heartbeat_ms: AtomicU64::new(self.heartbeat_ms.load(Ordering::Relaxed)),
            base_interval_ms: AtomicU64::new(self.base_interval_ms.load(Ordering::Relaxed)),
            screenshots_dir: self.screenshots_dir.clone(),
            event_recorder: AsyncMutex::new(None),
            proactive_callback: AsyncMutex::new(None),
        }
    }
}
