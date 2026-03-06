//! IPC Server implementation for MemFlow Core
//!
//! Provides a TCP-based IPC server for communication with Desktop UI and MCP

use std::net::SocketAddr;
use std::sync::Arc;
use tokio::net::{TcpListener, TcpStream};
use tokio::io::{AsyncReadExt, AsyncWriteExt};
use tokio::sync::{broadcast, RwLock, Mutex};
use serde_json::Value;

use crate::ipc::protocol::{IpcRequest, IpcResponse, IpcError};

/// Default IPC port
pub const DEFAULT_IPC_PORT: u16 = 11527;

/// Core state management
pub struct CoreStateManager {
    /// Current state (using Mutex for easier sync access)
    state: Mutex<CoreState>,
    /// Broadcast channel for state changes
    state_broadcast: broadcast::Sender<CoreState>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum CoreState {
    Running,
    Paused,
    Standby,
}

impl Default for CoreState {
    fn default() -> Self {
        CoreState::Standby
    }
}

impl std::fmt::Display for CoreState {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            CoreState::Running => write!(f, "running"),
            CoreState::Paused => write!(f, "paused"),
            CoreState::Standby => write!(f, "standby"),
        }
    }
}

impl CoreStateManager {
    pub fn new() -> Self {
        let (tx, _) = broadcast::channel(16);
        Self {
            state: Mutex::new(CoreState::Standby),
            state_broadcast: tx,
        }
    }

    pub async fn get_state(&self) -> CoreState {
        *self.state.lock().await
    }

    /// Synchronous state getter for use in spawn_blocking
    pub fn get_state_sync(&self) -> CoreState {
        // Use try_lock to avoid blocking forever
        self.state.try_lock().map(|s| *s).unwrap_or(CoreState::Standby)
    }

    pub async fn set_state(&self, new_state: CoreState) {
        let old_state = *self.state.lock().await;
        if old_state != new_state {
            *self.state.lock().await = new_state;
            let _ = self.state_broadcast.send(new_state);
            tracing::info!("Core state changed: {} -> {}", old_state, new_state);
        }
    }

    /// Synchronous state setter for use in spawn_blocking
    pub fn set_state_sync(&self, new_state: CoreState) {
        if let Ok(mut state) = self.state.try_lock() {
            let old_state = *state;
            if old_state != new_state {
                *state = new_state;
                let _ = self.state_broadcast.send(new_state);
                tracing::info!("Core state changed: {} -> {}", old_state, new_state);
            }
        }
    }

    pub fn subscribe_state(&self) -> broadcast::Receiver<CoreState> {
        self.state_broadcast.subscribe()
    }
}

/// IPC Server for MemFlow Core
pub struct IpcServer {
    /// Server address
    addr: SocketAddr,
    /// Core state manager
    state_manager: Arc<CoreStateManager>,
    /// Command handlers (method name -> handler function)
    handlers: Arc<RwLock<std::collections::HashMap<String, Box<dyn IpcCommandHandler>>>>,
    /// Shutdown signal
    shutdown_tx: Option<tokio::sync::oneshot::Sender<()>>,
}

pub trait IpcCommandHandler: Send + Sync {
    fn handle(&self, params: Option<Value>) -> Result<Value, IpcError>;
}

impl<F> IpcCommandHandler for F
where
    F: Fn(Option<Value>) -> Result<Value, IpcError> + Send + Sync,
{
    fn handle(&self, params: Option<Value>) -> Result<Value, IpcError> {
        self(params)
    }
}

impl IpcServer {
    /// Create a new IPC server
    pub fn new(port: u16) -> Self {
        let addr = SocketAddr::from(([127, 0, 0, 1], port));
        Self {
            addr,
            state_manager: Arc::new(CoreStateManager::new()),
            handlers: Arc::new(RwLock::new(std::collections::HashMap::new())),
            shutdown_tx: None,
        }
    }

    /// Get the state manager
    pub fn state_manager(&self) -> Arc<CoreStateManager> {
        self.state_manager.clone()
    }

    /// Register a command handler
    pub async fn register_handler<H>(&self, method: &str, handler: H)
    where
        H: IpcCommandHandler + 'static,
    {
        let mut handlers = self.handlers.write().await;
        handlers.insert(method.to_string(), Box::new(handler));
        tracing::debug!("Registered IPC handler: {}", method);
    }

    /// Register default handlers
    pub async fn register_default_handlers(&self) {
        let state_manager = self.state_manager.clone();

        // Core status handler
        self.register_handler("core_get_status", move |_| {
            let state = state_manager.get_state_sync();
            Ok(serde_json::json!({
                "state": format!("{:?}", state),
                "status": "ok"
            }))
        }).await;

        // Core start recording
        let state_manager = self.state_manager.clone();
        self.register_handler("core_start_recording", move |_| {
            let current = state_manager.get_state_sync();
            if current == CoreState::Running {
                return Err(IpcError::already_recording());
            }
            state_manager.set_state_sync(CoreState::Running);
            tracing::info!("Core recording started via IPC");
            Ok(serde_json::json!({"success": true}))
        }).await;

        // Core stop recording
        let state_manager = self.state_manager.clone();
        self.register_handler("core_stop_recording", move |_| {
            let current = state_manager.get_state_sync();
            if current == CoreState::Standby {
                return Err(IpcError::not_recording());
            }
            state_manager.set_state_sync(CoreState::Standby);
            tracing::info!("Core recording stopped via IPC");
            Ok(serde_json::json!({"success": true}))
        }).await;

        // Core pause recording
        let state_manager = self.state_manager.clone();
        self.register_handler("core_pause_recording", move |_| {
            let current = state_manager.get_state_sync();
            if current != CoreState::Running {
                return Err(IpcError::invalid_state(&format!("{:?}", current), "paused"));
            }
            state_manager.set_state_sync(CoreState::Paused);
            tracing::info!("Core recording paused via IPC");
            Ok(serde_json::json!({"success": true}))
        }).await;

        // Core resume recording
        let state_manager = self.state_manager.clone();
        self.register_handler("core_resume_recording", move |_| {
            let current = state_manager.get_state_sync();
            if current != CoreState::Paused {
                return Err(IpcError::invalid_state(&format!("{:?}", current), "running"));
            }
            state_manager.set_state_sync(CoreState::Running);
            tracing::info!("Core recording resumed via IPC");
            Ok(serde_json::json!({"success": true}))
        }).await;

        // Ping handler
        self.register_handler("ping", |_| {
            Ok(serde_json::json!({"pong": true}))
        }).await;
    }

    /// Start the IPC server
    pub async fn start(&mut self) -> Result<(), Box<dyn std::error::Error>> {
        let listener = TcpListener::bind(self.addr).await?;
        tracing::info!("IPC server listening on {}", self.addr);

        let (shutdown_tx, mut shutdown_rx) = tokio::sync::oneshot::channel();
        self.shutdown_tx = Some(shutdown_tx);

        let handlers = self.handlers.clone();

        tokio::spawn(async move {
            loop {
                tokio::select! {
                    result = listener.accept() => {
                        match result {
                            Ok((stream, addr)) => {
                                let handlers = handlers.clone();
                                tokio::spawn(async move {
                                    if let Err(e) = Self::handle_connection(stream, handlers).await {
                                        tracing::error!("Error handling IPC connection from {}: {}", addr, e);
                                    }
                                });
                            }
                            Err(e) => {
                                tracing::error!("Failed to accept IPC connection: {}", e);
                            }
                        }
                    }
                    _ = &mut shutdown_rx => {
                        tracing::info!("IPC server shutting down");
                        break;
                    }
                }
            }
        });

        Ok(())
    }

    /// Handle a single connection
    async fn handle_connection(
        stream: TcpStream,
        handlers: Arc<RwLock<std::collections::HashMap<String, Box<dyn IpcCommandHandler>>>>,
    ) -> Result<(), Box<dyn std::error::Error>> {
        let mut stream = stream;
        let mut buf = vec![0u8; 8192];

        loop {
            let n = stream.read(&mut buf).await?;
            if n == 0 {
                break;
            }

            // Parse and handle request
            let request: IpcRequest = match serde_json::from_slice(&buf[..n]) {
                Ok(req) => req,
                Err(_e) => {
                    let error = IpcError::parse_error();
                    let response = IpcResponse::error(error, None);
                    let response_bytes = serde_json::to_vec(&response)?;
                    stream.write_all(&response_bytes).await?;
                    continue;
                }
            };

            let response = Self::handle_request(request, &handlers).await;
            let response_bytes = serde_json::to_vec(&response)?;
            stream.write_all(&response_bytes).await?;
        }

        Ok(())
    }

    /// Handle a single request
    async fn handle_request(
        request: IpcRequest,
        handlers: &Arc<RwLock<std::collections::HashMap<String, Box<dyn IpcCommandHandler>>>>,
    ) -> IpcResponse {
        let method = request.method.clone();
        let params = request.params.clone();
        let id = request.id.clone();

        let handlers = handlers.read().await;
        
        match handlers.get(&method) {
            Some(handler) => {
                match handler.handle(params) {
                    Ok(result) => IpcResponse::success(result, id),
                    Err(error) => IpcResponse::error(error, id),
                }
            }
            None => IpcResponse::error(IpcError::method_not_found(&method), id),
        }
    }

    /// Stop the server
    pub fn stop(&mut self) {
        if let Some(tx) = self.shutdown_tx.take() {
            let _ = tx.send(());
        }
    }
}
