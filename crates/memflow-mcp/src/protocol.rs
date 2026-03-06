use serde::{Deserialize, Serialize};
use serde_json::Value;

/// Custom MCP tool error codes (see MCP_TOOL_CONTRACT_v1).
///
/// NOTE: The underlying spec file is not shipped in this repo, but the
/// architecture doc pins the custom error code range to -32000 ~ -32008,
/// with the following semantics used in this implementation:
///
pub const ERR_TOOL_FAILED: i32 = -32000;
pub const ERR_INVALID_PARAMS: i32 = -32001;
#[allow(dead_code)]
pub const ERR_BACKEND_UNAVAILABLE: i32 = -32002;
#[allow(dead_code)]
pub const ERR_TIMEOUT: i32 = -32003;
pub const ERR_TERMINAL_NOT_FOUND: i32 = -32004;
pub const ERR_PERMISSION_DENIED: i32 = -32005;
#[allow(dead_code)]
pub const ERR_NOT_IMPLEMENTED: i32 = -32006;
#[allow(dead_code)]
pub const ERR_RATE_LIMITED: i32 = -32007;
pub const ERR_INTERNAL: i32 = -32008;
#[allow(dead_code)]
pub const ERR_CORE_UNAVAILABLE: i32 = -32009;
#[allow(dead_code)]
pub const ERR_DEGRADED_MODE: i32 = -32010;

/// JSON-RPC 2.0 request
#[derive(Serialize, Deserialize, Debug)]
pub struct JsonRpcRequest {
    pub jsonrpc: String,
    pub method: String,
    pub params: Option<Value>,
    pub id: Option<Value>,
}

impl JsonRpcRequest {
    /// 验证请求格式是否符合基本要求
    #[allow(dead_code)]
    pub fn is_valid(&self) -> bool {
        self.jsonrpc == "2.0" && !self.method.is_empty()
    }
}

/// JSON-RPC 2.0 response
#[derive(Serialize, Deserialize, Debug)]
pub struct JsonRpcResponse {
    pub jsonrpc: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub result: Option<Value>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub error: Option<JsonRpcError>,
    pub id: Option<Value>,
}

/// JSON-RPC 2.0 error object
#[derive(Serialize, Deserialize, Debug)]
pub struct JsonRpcError {
    pub code: i32,
    pub message: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub data: Option<Value>,
}

impl JsonRpcResponse {
    pub fn ok(id: Option<Value>, result: Value) -> Self {
        Self {
            jsonrpc: "2.0".to_string(),
            result: Some(result),
            error: None,
            id,
        }
    }

    pub fn error(id: Option<Value>, code: i32, message: impl Into<String>) -> Self {
        Self {
            jsonrpc: "2.0".to_string(),
            result: None,
            error: Some(JsonRpcError {
                code,
                message: message.into(),
                data: None,
            }),
            id,
        }
    }

    /// Build an error response with optional structured `data` field.
    ///
    /// MCP Tool Contract encourages using `data` for machine-readable context
    /// (e.g. which field is invalid). Callers can keep it `None` for simple
    /// textual errors.
    #[allow(dead_code)]
    pub fn error_with_data(
        id: Option<Value>,
        code: i32,
        message: impl Into<String>,
        data: Option<Value>,
    ) -> Self {
        Self {
            jsonrpc: "2.0".to_string(),
            result: None,
            error: Some(JsonRpcError {
                code,
                message: message.into(),
                data,
            }),
            id,
        }
    }
}

