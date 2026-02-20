//! MemFlow Core - UI-independent business logic
//!
//! This crate contains the core logic for MemFlow that can be reused
//! across different frontends (desktop app, MCP server, etc.)

pub mod ai;
pub mod audit;
pub mod context;
pub mod db;
pub mod focus_analytics;
pub mod ocr_enhance;
pub mod redact;
pub mod terminal;
pub mod vector_db;
