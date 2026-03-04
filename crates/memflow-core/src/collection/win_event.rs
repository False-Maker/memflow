//! Windows event-driven sampling module for MemFlow Core
//!
//! Uses SetWinEventHook to listen for window focus change events,
//! enabling capture only when screen content changes.

use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::Arc;
use tokio::sync::mpsc;

/// Window event types
#[derive(Debug, Clone)]
pub enum WindowEvent {
    /// Foreground window changed
    ForegroundChanged { hwnd: isize },
    /// Window created
    WindowCreated { hwnd: isize },
    /// Window destroyed
    WindowDestroyed { hwnd: isize },
    /// Window title changed
    TitleChanged { hwnd: isize },
}

/// Event listener configuration
#[derive(Debug, Clone)]
pub struct EventLoopConfig {
    /// Whether to listen for foreground window changes
    pub track_foreground: bool,
    /// Whether to listen for window create/destroy
    pub track_lifecycle: bool,
    /// Whether to listen for title changes
    pub track_title_change: bool,
    /// Event debounce interval (milliseconds)
    pub debounce_ms: u64,
}

impl Default for EventLoopConfig {
    fn default() -> Self {
        Self {
            track_foreground: true,
            track_lifecycle: false,
            track_title_change: false,
            debounce_ms: 100,
        }
    }
}

/// Event loop handle
pub struct EventLoopHandle {
    stop_flag: Arc<AtomicBool>,
}

impl EventLoopHandle {
    /// Stop the event loop
    pub fn stop(&self) {
        self.stop_flag.store(true, Ordering::SeqCst);
    }

    /// Check if stopped
    pub fn is_stopped(&self) -> bool {
        self.stop_flag.load(Ordering::SeqCst)
    }
}

/// Start event-driven window listener
///
/// Returns event receiver channel and control handle
pub fn start_event_loop(config: EventLoopConfig) -> (mpsc::Receiver<WindowEvent>, EventLoopHandle) {
    let (tx, rx) = mpsc::channel::<WindowEvent>(100);
    let stop_flag = Arc::new(AtomicBool::new(false));
    let handle = EventLoopHandle {
        stop_flag: stop_flag.clone(),
    };

    // Start background thread to listen for window events
    let config_clone = config.clone();
    std::thread::spawn(move || {
        run_event_loop_internal(tx, stop_flag, config_clone);
    });

    (rx, handle)
}

/// Internal event loop implementation
fn run_event_loop_internal(
    tx: mpsc::Sender<WindowEvent>,
    stop_flag: Arc<AtomicBool>,
    config: EventLoopConfig,
) {
    use windows::Win32::UI::WindowsAndMessaging::GetForegroundWindow;
    
    let mut last_hwnd: isize = 0;
    let debounce_duration = std::time::Duration::from_millis(config.debounce_ms);

    tracing::info!("Event-driven sampling loop started");

    while !stop_flag.load(Ordering::SeqCst) {
        if config.track_foreground {
            unsafe {
                let hwnd = GetForegroundWindow();
                let current_hwnd = hwnd.0 as isize;

                if current_hwnd != last_hwnd && current_hwnd != 0 {
                    last_hwnd = current_hwnd;

                    // Send foreground window change event
                    if let Err(e) = tx.blocking_send(WindowEvent::ForegroundChanged {
                        hwnd: current_hwnd,
                    }) {
                        tracing::warn!("Failed to send window event: {}", e);
                        break;
                    }
                }
            }
        }

        std::thread::sleep(debounce_duration);
    }

    tracing::info!("Event-driven sampling loop stopped");
}

/// Event-driven recorder
///
/// Integrates event loop with existing recording logic
pub struct EventDrivenRecorder {
    handle: Option<EventLoopHandle>,
    config: EventLoopConfig,
}

impl EventDrivenRecorder {
    /// Create a new recorder
    pub fn new(config: EventLoopConfig) -> Self {
        Self {
            handle: None,
            config,
        }

    }

    /// Start event-driven recording
    pub fn start(&mut self) -> mpsc::Receiver<WindowEvent> {
        let (rx, handle) = start_event_loop(self.config.clone());
        self.handle = Some(handle);
        rx
    }

    /// Stop recording
    pub fn stop(&mut self) {
        if let Some(ref handle) = self.handle {
            handle.stop();
        }
        self.handle = None;
    }

    /// Check if running
    pub fn is_running(&self) -> bool {
        self.handle.as_ref().map(|h| !h.is_stopped()).unwrap_or(false)
    }
}

impl Drop for EventDrivenRecorder {
    fn drop(&mut self) {
        self.stop();
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_default_config() {
        let config = EventLoopConfig::default();
        assert!(config.track_foreground);
        assert!(!config.track_lifecycle);
        assert!(!config.track_title_change);
        assert_eq!(config.debounce_ms, 100);
    }

    #[test]
    fn test_event_loop_handle() {
        let stop_flag = Arc::new(AtomicBool::new(false));
        let handle = EventLoopHandle { stop_flag };

        assert!(!handle.is_stopped());
        handle.stop();
        assert!(handle.is_stopped());
    }
}
