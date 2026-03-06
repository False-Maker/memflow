use anyhow::Result;
use memflow_core::context::{AiAnalysisResult, RuntimeContext};
use memflow_core::db;
use memflow_core::ipc_client::IpcClient;
use serde_json::Value;
use std::future::Future;
use std::path::PathBuf;
use std::pin::Pin;
use tracing::{info, warn};

pub struct McpContext {
    /// IPC client for communicating with Core daemon (if available)
    ipc_client: Option<IpcClient>,
    /// Whether Core is available via IPC
    #[allow(dead_code)]
    ipc_available: bool,
}

impl McpContext {
    pub fn new() -> Self {
        // Try to create IPC client
        let ipc_client = IpcClient::new().into();
        
        Self {
            ipc_client,
            ipc_available: false,
        }
    }

    /// Check if IPC connection to Core daemon is available
    #[allow(dead_code)]
    pub fn is_ipc_available(&self) -> bool {
        self.ipc_available
    }

    /// Get IPC client (if available)
    #[allow(dead_code)]
    pub fn ipc(&self) -> Option<&IpcClient> {
        self.ipc_client.as_ref()
    }

    /// Check if the Core/Daemon is available via IPC or database.
    ///
    /// Returns Ok if Core is healthy, Err with specific message if unavailable.
    pub async fn check_core_health(&self) -> Result<()> {
        // First try IPC
        if let Some(client) = &self.ipc_client {
            if client.ping().await {
                info!("Core health check passed via IPC");
                return Ok(());
            }
        }
        
        // Fall back to database check
        match db::check_core_health().await {
            Ok(_) => {
                info!("Core health check passed via database");
                Ok(())
            }
            Err(e) => {
                warn!("Core health check failed: {}", e);
                Err(anyhow::anyhow!("MCP_CORE_UNAVAILABLE: Core is not accessible. Please ensure the MemFlow desktop app or daemon is running. Error: {}", e))
            }
        }
    }
}

impl Default for McpContext {
    fn default() -> Self {
        Self::new()
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
