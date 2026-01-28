//! Desktop context implementation for Tauri
//!
//! This module provides the Tauri-specific implementation of the RuntimeContext trait,
//! enabling the agent to use Tauri's capabilities (file paths, event emission) on the desktop.

use memflow_core::context::{RuntimeContext, AiAnalysisResult, TaskContext};
use tauri::{AppHandle, Manager, Emitter};
use std::path::PathBuf;
use std::future::Future;
use std::pin::Pin;

/// Tauri-specific implementation of RuntimeContext
/// 
/// Wraps a Tauri AppHandle to provide platform capabilities to the agent module.
pub struct TauriContext {
    pub app_handle: AppHandle,
}

impl TauriContext {
    /// Create a new TauriContext from an AppHandle
    pub fn new(app_handle: AppHandle) -> Self {
        Self { app_handle }
    }
}

impl RuntimeContext for TauriContext {
    fn app_dir(&self) -> PathBuf {
        self.app_handle
            .path()
            .app_data_dir()
            .unwrap_or_else(|_| PathBuf::from("."))
    }
    
    fn resource_dir(&self) -> PathBuf {
        self.app_handle
            .path()
            .resource_dir()
            .unwrap_or_else(|_| PathBuf::from("."))
    }
    
    fn emit(&self, event: &str, payload: serde_json::Value) -> anyhow::Result<()> {
        self.app_handle.emit(event, payload).map_err(|e| e.into())
    }
    
    fn analyze_for_proposals(
        &self,
        context_text: &str,
    ) -> Pin<Box<dyn Future<Output = anyhow::Result<AiAnalysisResult>> + Send + '_>> {
        let context_text = context_text.to_string();
        Box::pin(async move {
            // Call the existing AI analysis function from src-tauri
            let result = crate::ai::analyze_for_proposals(&context_text).await?;
            
            // Convert from src-tauri's AiAnalysisResult to memflow_core's
            Ok(AiAnalysisResult {
                tasks: result.tasks.into_iter().map(|t| TaskContext {
                    title: t.title,
                    summary: t.summary,
                    related_urls: t.related_urls,
                    related_files: t.related_files,
                    related_apps: t.related_apps,
                }).collect(),
            })
        })
    }
}
