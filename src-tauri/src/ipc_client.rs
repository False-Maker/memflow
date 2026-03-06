//! IPC client wrapper for Tauri commands
//!
//! Provides a convenient interface for Tauri commands to communicate with Core daemon

use memflow_core::ipc_client::IpcClient;
use std::sync::Arc;
use tokio::sync::RwLock;

/// Global IPC client state
pub struct IpcClientState {
    /// Connection status
    connected: RwLock<bool>,
    /// IPC port
    port: u16,
}

impl IpcClientState {
    pub fn new(port: u16) -> Self {
        Self {
            connected: RwLock::new(false),
            port,
        }
    }

    /// Try to connect to Core daemon
    pub async fn connect(&self) -> bool {
        let client = IpcClient::with_port(self.port);
        if client.ping().await {
            *self.connected.write().await = true;
            tracing::info!("Connected to Core daemon via IPC on port {}", self.port);
            true
        } else {
            tracing::warn!("Failed to connect to Core daemon");
            false
        }
    }

    /// Check if connected
    pub async fn is_connected(&self) -> bool {
        *self.connected.read().await
    }

    /// Disconnect from Core daemon
    pub async fn disconnect(&self) {
        *self.connected.write().await = false;
    }

    /// Try to start recording via IPC
    pub async fn start_recording(&self) -> Result<serde_json::Value, String> {
        if !self.is_connected().await {
            // Try to connect first
            if !self.connect().await {
                return Err("Not connected to Core daemon".to_string());
            }
        }
        
        let client = IpcClient::with_port(self.port);
        match client.start_recording().await {
            Ok(result) => Ok(result),
            Err(e) => {
                *self.connected.write().await = false;
                Err(e.message.clone())
            }
        }
    }

    /// Try to stop recording via IPC
    pub async fn stop_recording(&self) -> Result<serde_json::Value, String> {
        if !self.is_connected().await {
            // Try to connect first
            if !self.connect().await {
                return Err("Not connected to Core daemon".to_string());
            }
        }
        
        let client = IpcClient::with_port(self.port);
        match client.stop_recording().await {
            Ok(result) => Ok(result),
            Err(e) => {
                *self.connected.write().await = false;
                Err(e.message.clone())
            }
        }
    }

    /// Try to get status via IPC
    pub async fn get_status(&self) -> Result<serde_json::Value, String> {
        if !self.is_connected().await {
            // Try to connect first
            if !self.connect().await {
                return Err("Not connected to Core daemon".to_string());
            }
        }
        
        let client = IpcClient::with_port(self.port);
        match client.get_status().await {
            Ok(result) => Ok(result),
            Err(e) => {
                *self.connected.write().await = false;
                Err(e.message.clone())
            }
        }
    }
}

impl Default for IpcClientState {
    fn default() -> Self {
        Self::new(11527) // Default IPC port
    }
}

/// Create a new IPC client state with default port
pub fn create_ipc_client_state() -> Arc<IpcClientState> {
    Arc::new(IpcClientState::default())
}

/// Create a new IPC client state with custom port
pub fn create_ipc_client_state_with_port(port: u16) -> Arc<IpcClientState> {
    Arc::new(IpcClientState::new(port))
}
