//! MemFlow Core - UI-independent business logic
//!
//! This crate contains the core logic for MemFlow that can be reused
//! across different frontends (desktop app, MCP server, etc.)

pub mod agent;
pub mod ai;
pub mod context;
pub mod db;
pub mod focus_analytics;
pub mod ipc;
pub mod ipc_client;
pub mod collection;
pub mod ocr_enhance;
pub mod redact;
pub mod vector_db;
pub mod system_env;

pub use ipc_client::IpcClient;
pub use collection::{
    CollectionConfig, CollectionState, ActivityCollector, 
    capture_screen, encode_webp,
    get_foreground_window_info, is_terminal_process, normalize_app_name,
    ActivityRecord, WindowInfo,
    UiaTextResult, extract_uia_text, is_uia_available,
    ActivityEvent,
};