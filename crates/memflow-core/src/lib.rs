//! MemFlow Core - UI-independent business logic
//!
//! This crate contains the core logic for MemFlow that can be reused
//! across different frontends (desktop app, MCP server, etc.)

pub mod agent;
pub mod ai;
pub mod context;
pub mod db;
pub mod focus_analytics;
pub mod redact;
pub mod vector_db;
