//! Collection module for MemFlow Core
//!
//! This module handles activity collection, including:
//! - Screen capture
//! - Activity logging
//! - OCR processing

pub mod config;
pub mod state;
pub mod capture;
pub mod collector;
pub mod window;
pub mod uia;
pub mod win_event;

pub use config::CollectionConfig;
pub use state::{CollectionState, ActivityRecord};
pub use capture::{capture_screen, encode_webp, calculate_phash, hamming_distance, is_similar, CaptureResult};
pub use collector::{ActivityCollector, ActivityEvent};
pub use window::{WindowInfo, get_foreground_window_info, is_terminal_process, normalize_app_name};
pub use uia::{UiaTextResult, extract_uia_text, is_uia_available};
pub use win_event::{WindowEvent, EventLoopConfig, EventDrivenRecorder, start_event_loop};