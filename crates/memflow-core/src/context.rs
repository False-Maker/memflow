//! Runtime context abstraction for platform-independent agent execution
//!
//! This module defines the `RuntimeContext` trait that abstracts away platform-specific
//! capabilities, allowing the agent to run on both Tauri desktop and future CLI/MCP contexts.

use std::path::PathBuf;
use std::future::Future;
use std::pin::Pin;
use serde::{Serialize, Deserialize};
use serde_json::Value;
use anyhow::Result;

/// AI analysis result containing extracted task contexts
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AiAnalysisResult {
    pub tasks: Vec<TaskContext>,
}

/// Task context extracted from activity analysis
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TaskContext {
    pub title: String,
    pub summary: String,
    pub related_urls: Vec<String>,
    pub related_files: Vec<String>,
    pub related_apps: Vec<String>,
}

/// Defines the capabilities the Agent needs from the host system (Desktop or CLI)
///
/// This trait allows the agent module to be platform-agnostic:
/// - Desktop (Tauri): Uses AppHandle for paths and event emission
/// - MCP/CLI: Can use file-based paths and log to stdout
pub trait RuntimeContext: Send + Sync {
    /// Get the application data directory
    ///
    /// Desktop: Returns the Tauri app data directory
    /// CLI/MCP: Returns a configured path or current directory
    fn app_dir(&self) -> PathBuf;
    
    /// Get the resources directory (for models, prompts, etc.)
    ///
    /// Desktop: Returns the Tauri resources directory
    /// CLI/MCP: Returns a configured path or current directory
    fn resource_dir(&self) -> PathBuf;

    /// Send an event to the UI (if applicable)
    ///
    /// Desktop: Emits a Tauri event to the frontend
    /// MCP/CLI: Logs to stdout or ignores based on verbosity settings
    fn emit(&self, event: &str, payload: Value) -> Result<()>;
    
    /// Analyze activity context using AI to generate proposals
    ///
    /// Desktop: Calls LLM API via app_config and secure_storage
    /// CLI/MCP: Can use environment variables for API keys
    fn analyze_for_proposals(
        &self,
        context_text: &str,
    ) -> Pin<Box<dyn Future<Output = Result<AiAnalysisResult>> + Send + '_>>;
}
