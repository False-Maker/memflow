//! IPC Protocol definitions
//!
//! Defines the request/response structures for Core IPC communication

use serde::{Deserialize, Serialize};

/// IPC Request structure (JSON-RPC 2.0 style)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct IpcRequest {
    /// JSON-RPC version (always "2.0")
    #[serde(default = "default_version")]
    pub jsonrpc: String,
    /// Request method name
    pub method: String,
    /// Request parameters
    #[serde(default)]
    pub params: Option<serde_json::Value>,
    /// Request ID for tracking responses
    #[serde(default)]
    pub id: Option<serde_json::Value>,
}

fn default_version() -> String {
    "2.0".to_string()
}

/// IPC Response structure
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(untagged)]
pub enum IpcResponse {
    /// Success response
    Success(IpcResponseSuccess),
    /// Error response
    Error(IpcResponseError),
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct IpcResponseSuccess {
    #[serde(default = "default_version")]
    pub jsonrpc: String,
    pub result: serde_json::Value,
    pub id: Option<serde_json::Value>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct IpcResponseError {
    #[serde(default = "default_version")]
    pub jsonrpc: String,
    pub error: IpcError,
    pub id: Option<serde_json::Value>,
}

/// IPC Error codes
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct IpcError {
    pub code: i32,
    pub message: String,
    #[serde(default)]
    pub data: Option<serde_json::Value>,
}

impl std::fmt::Display for IpcError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "IPC Error {}: {}", self.code, self.message)
    }
}

impl std::error::Error for IpcError {}

impl IpcError {
    /// Parse error (-32700)
    pub fn parse_error() -> Self {
        Self {
            code: -32700,
            message: "Parse error".to_string(),
            data: None,
        }
    }

    /// Invalid request (-32600)
    pub fn invalid_request(msg: &str) -> Self {
        Self {
            code: -32600,
            message: msg.to_string(),
            data: None,
        }
    }

    /// Method not found (-32601)
    pub fn method_not_found(method: &str) -> Self {
        Self {
            code: -32601,
            message: format!("Method not found: {}", method),
            data: None,
        }
    }

    /// Invalid params (-32602)
    pub fn invalid_params(msg: &str) -> Self {
        Self {
            code: -32602,
            message: msg.to_string(),
            data: None,
        }
    }

    /// Internal error (-32603)
    pub fn internal_error(msg: &str) -> Self {
        Self {
            code: -32603,
            message: msg.to_string(),
            data: None,
        }
    }

    /// Core unavailable (-32001)
    pub fn core_unavailable() -> Self {
        Self {
            code: -32001,
            message: "Core service unavailable".to_string(),
            data: None,
        }
    }

    /// Core not recording (-32002)
    pub fn not_recording() -> Self {
        Self {
            code: -32002,
            message: "Core is not recording".to_string(),
            data: None,
        }
    }

    /// Core already recording (-32003)
    pub fn already_recording() -> Self {
        Self {
            code: -32003,
            message: "Core is already recording".to_string(),
            data: None,
        }
    }

    /// Invalid state transition (-32004)
    pub fn invalid_state(current: &str, target: &str) -> Self {
        Self {
            code: -32004,
            message: format!("Invalid state transition from {} to {}", current, target),
            data: None,
        }
    }
}

impl From<serde_json::Error> for IpcError {
    fn from(err: serde_json::Error) -> Self {
        Self {
            code: -32700,
            message: format!("Parse error: {}", err),
            data: None,
        }
    }
}

impl IpcResponse {
    pub fn success(result: serde_json::Value, id: Option<serde_json::Value>) -> Self {
        IpcResponse::Success(IpcResponseSuccess {
            jsonrpc: "2.0".to_string(),
            result,
            id,
        })
    }

    pub fn error(error: IpcError, id: Option<serde_json::Value>) -> Self {
        IpcResponse::Error(IpcResponseError {
            jsonrpc: "2.0".to_string(),
            error,
            id,
        })
    }
}
