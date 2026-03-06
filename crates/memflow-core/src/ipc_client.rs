//! IPC Client for MemFlow Core
//!
//! Provides a client for connecting to Core IPC server
use std::net::SocketAddr;
use tokio::net::TcpStream;
use tokio::io::{AsyncReadExt, AsyncWriteExt};
use serde_json::Value;
use crate::ipc::protocol::{IpcRequest, IpcResponse, IpcError};
use crate::ipc::server::DEFAULT_IPC_PORT;

/// IPC Client for connecting to MemFlow Core
pub struct IpcClient {
    addr: SocketAddr,
}

impl IpcClient {
    /// Create a new IPC client connecting to localhost on default port
    pub fn new() -> Self {
        Self {
            addr: SocketAddr::from(([127, 0, 0, 1], DEFAULT_IPC_PORT)),
        }
    }

    /// Create a new IPC client with custom port
    pub fn with_port(port: u16) -> Self {
        Self {
            addr: SocketAddr::from(([127, 0, 0, 1], port)),
        }
    }

    /// Send a request and get response
    pub async fn send(&self, method: &str, params: Option<Value>) -> Result<Value, IpcError> {
        let mut stream = TcpStream::connect(self.addr).await
            .map_err(|_e| IpcError::core_unavailable())?;

        let request = IpcRequest {
            jsonrpc: "2.0".to_string(),
            method: method.to_string(),
            params,
            id: Some(Value::Number(1.into())),
        };

        let request_bytes = serde_json::to_vec(&request)?;
        stream.write_all(&request_bytes).await.map_err(|e| IpcError::internal_error(&e.to_string()))?;

        let mut response_buf = vec![0u8; 8192];
        let n = stream.read(&mut response_buf).await.map_err(|e| IpcError::internal_error(&e.to_string()))?;

        let response: IpcResponse = serde_json::from_slice(&response_buf[..n])
            .map_err(|_e| IpcError::parse_error())?;

        match response {
            IpcResponse::Success(success) => Ok(success.result),
            IpcResponse::Error(err) => Err(err.error),
        }
    }

    /// Check if Core is available
    pub async fn ping(&self) -> bool {
        self.send("ping", None).await.is_ok()
    }

    /// Get Core status
    pub async fn get_status(&self) -> Result<Value, IpcError> {
        self.send("core_get_status", None).await
    }

    /// Start recording
    pub async fn start_recording(&self) -> Result<Value, IpcError> {
        self.send("core_start_recording", None).await
    }

    /// Stop recording
    pub async fn stop_recording(&self) -> Result<Value, IpcError> {
        self.send("core_stop_recording", None).await
    }

    /// Pause recording
    pub async fn pause_recording(&self) -> Result<Value, IpcError> {
        self.send("core_pause_recording", None).await
    }

    /// Resume recording
    pub async fn resume_recording(&self) -> Result<Value, IpcError> {
        self.send("core_resume_recording", None).await
    }

    /// Search memory
    pub async fn search_memory(&self, query: &str, mode: Option<&str>, limit: Option<i64>) -> Result<Value, IpcError> {
        let params = serde_json::json!({
            "query": query,
            "mode": mode,
            "limit": limit
        });
        self.send("core_search_memory", Some(params)).await
    }

    /// Get recent activity
    pub async fn get_recent_activity(&self, minutes: Option<i64>) -> Result<Value, IpcError> {
        let params = serde_json::json!({
            "minutes": minutes
        });
        self.send("core_get_recent_activity", Some(params)).await
    }
}

impl Default for IpcClient {
    fn default() -> Self {
        Self::new()
    }
}
