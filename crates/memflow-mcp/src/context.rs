use anyhow::Result;
use memflow_core::context::{AiAnalysisResult, RuntimeContext};
use serde_json::Value;
use std::future::Future;
use std::path::PathBuf;
use std::pin::Pin;

pub struct McpContext;

impl McpContext {
    pub fn new() -> Self {
        Self
    }
}

impl RuntimeContext for McpContext {
    fn app_dir(&self) -> PathBuf {
        dirs::data_dir()
            .map(|p| p.join("com.memflow.app"))
            .unwrap_or_else(|| PathBuf::from("."))
    }

    fn resource_dir(&self) -> PathBuf {
        let path = self.app_dir().join("memflow-resources");
        if !path.exists() {
            let _ = std::fs::create_dir_all(&path);
        }
        path
    }

    fn emit(&self, event: &str, payload: Value) -> Result<()> {
        // MCP protocol relies on JSON-RPC over stdout.
        // Asynchronous events (notifications) can be sent, but here we just log significant events to stderr
        // to avoid corrupting the stdout JSON-RPC stream.
        eprintln!("[MCP Event] {}: {}", event, payload);
        Ok(())
    }

    fn analyze_for_proposals(
        &self,
        _context_text: &str,
    ) -> Pin<Box<dyn Future<Output = Result<AiAnalysisResult>> + Send + '_>> {
        // Initially return empty results as this is just the searching interface
        Box::pin(async {
            Ok(AiAnalysisResult { tasks: vec![] })
        })
    }
}
