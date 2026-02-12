use serde::{Deserialize, Serialize};
use serde_json::Value;

// JSON-RPC 2.0 Types

#[derive(Debug, Deserialize)]
pub struct JsonRpcRequest {
    pub jsonrpc: String,
    pub method: String,
    pub params: Option<Value>,
    pub id: Option<Value>,
}

#[derive(Debug, Serialize)]
pub struct JsonRpcResponse {
    pub jsonrpc: String,
    pub result: Option<Value>,
    pub error: Option<JsonRpcError>,
    pub id: Option<Value>,
}

#[derive(Debug, Serialize)]
pub struct JsonRpcError {
    pub code: i32,
    pub message: String,
    pub data: Option<Value>,
}

impl JsonRpcResponse {
    pub fn success(id: Option<Value>, result: Value) -> Self {
        Self {
            jsonrpc: "2.0".to_string(),
            result: Some(result),
            error: None,
            id,
        }
    }

    pub fn error(id: Option<Value>, code: i32, message: &str) -> Self {
        Self {
            jsonrpc: "2.0".to_string(),
            result: None,
            error: Some(JsonRpcError {
                code,
                message: message.to_string(),
                data: None,
            }),
            id,
        }
    }
}

// MCP Specific Types

/// Tool name constants for type safety
pub const TOOL_SEARCH_MEMORY: &str = "search_memory";
pub const TOOL_SEARCH_MEMORY_ALIAS: &str = "search_visual_memory";
pub const TOOL_GET_RECENT_ACTIVITY: &str = "get_recent_activity";
pub const TOOL_GET_RECENT_ACTIVITY_ALIAS: &str = "get_recent_activities";
pub const TOOL_GET_ACTIVE_WINDOW_CONTEXT: &str = "get_active_window_context";
pub const TOOL_GET_TERMINAL_OUTPUT: &str = "get_terminal_output";
pub const TOOL_GET_SYSTEM_ENVIRONMENT: &str = "get_system_environment";
pub const TOOL_GET_RELATED_CONTEXT: &str = "get_related_context";

/// Unified tool name enum with alias support
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ToolName {
    SearchMemory,
    GetRecentActivity,
    GetActiveWindowContext,
    GetTerminalOutput,
    GetSystemEnvironment,
    GetRelatedContext,
}

impl ToolName {
    /// Parse tool name from string, handling aliases
    pub fn from_str(name: &str) -> Option<Self> {
        match name {
            TOOL_SEARCH_MEMORY | TOOL_SEARCH_MEMORY_ALIAS => Some(Self::SearchMemory),
            TOOL_GET_RECENT_ACTIVITY | TOOL_GET_RECENT_ACTIVITY_ALIAS => {
                Some(Self::GetRecentActivity)
            }
            TOOL_GET_ACTIVE_WINDOW_CONTEXT => Some(Self::GetActiveWindowContext),
            TOOL_GET_TERMINAL_OUTPUT => Some(Self::GetTerminalOutput),
            TOOL_GET_SYSTEM_ENVIRONMENT => Some(Self::GetSystemEnvironment),
            TOOL_GET_RELATED_CONTEXT => Some(Self::GetRelatedContext),
            _ => None,
        }
    }

    /// Get the canonical (formal) name for this tool
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::SearchMemory => TOOL_SEARCH_MEMORY,
            Self::GetRecentActivity => TOOL_GET_RECENT_ACTIVITY,
            Self::GetActiveWindowContext => TOOL_GET_ACTIVE_WINDOW_CONTEXT,
            Self::GetTerminalOutput => TOOL_GET_TERMINAL_OUTPUT,
            Self::GetSystemEnvironment => TOOL_GET_SYSTEM_ENVIRONMENT,
            Self::GetRelatedContext => TOOL_GET_RELATED_CONTEXT,
        }
    }

    /// Check if the given name is an alias (not the canonical name)
    pub fn is_alias(name: &str) -> bool {
        matches!(
            name,
            TOOL_SEARCH_MEMORY_ALIAS | TOOL_GET_RECENT_ACTIVITY_ALIAS
        )
    }
}

/// Tool definition structure
#[derive(Debug, Serialize)]
pub struct Tool {
    pub name: String,
    pub description: String,
    pub input_schema: Value,
}

impl Tool {
    /// Create a new tool definition
    pub fn new(
        name: impl Into<String>,
        description: impl Into<String>,
        input_schema: Value,
    ) -> Self {
        Self {
            name: name.into(),
            description: description.into(),
            input_schema,
        }
    }
}

/// Result of tools/list method
#[derive(Debug, Serialize)]
pub struct ListToolsResult {
    pub tools: Vec<Tool>,
}

/// Tool call result content
#[derive(Debug, Serialize)]
pub struct ToolCallContent {
    #[serde(rename = "type")]
    pub type_: String,
    pub text: String,
}

/// Tool call result
#[derive(Debug, Serialize)]
pub struct ToolCallResult {
    pub content: Vec<ToolCallContent>,
}

impl ToolCallResult {
    /// Create a simple text result
    pub fn text(text: impl Into<String>) -> Self {
        Self {
            content: vec![ToolCallContent {
                type_: "text".to_string(),
                text: text.into(),
            }],
        }
    }
}

// Prompts

#[derive(Debug, Serialize, Deserialize)]
pub struct Prompt {
    pub name: String,
    pub description: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub arguments: Option<Vec<PromptArgument>>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct PromptArgument {
    pub name: String,
    pub description: Option<String>,
    pub required: Option<bool>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct ListPromptsResult {
    pub prompts: Vec<Prompt>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct GetPromptResult {
    pub description: Option<String>,
    pub messages: Vec<PromptMessage>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct PromptMessage {
    pub role: String,
    pub content: PromptMessageContent,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct PromptMessageContent {
    #[serde(rename = "type")]
    pub type_: String,
    pub text: String,
}

// Error Codes

/// MCP Standard Error Codes
pub const ERROR_PARSE_ERROR: i32 = -32700;
pub const ERROR_INVALID_REQUEST: i32 = -32600;
pub const ERROR_METHOD_NOT_FOUND: i32 = -32601;
pub const ERROR_INVALID_PARAMS: i32 = -32602;
pub const ERROR_INTERNAL_ERROR: i32 = -32603;

/// Memflow-Specific Error Codes
pub const ERROR_SERVER_ERROR: i32 = -32000;
pub const ERROR_UNAUTHORIZED: i32 = -32001;
pub const ERROR_READ_ONLY_MODE: i32 = -32003;
pub const ERROR_TERMINAL_NOT_FOUND: i32 = -32004;
pub const ERROR_PERMISSION_DENIED: i32 = -32005;
pub const ERROR_DATABASE_LOCKED: i32 = -32006;
pub const ERROR_NO_DATA_AVAILABLE: i32 = -32007;
pub const ERROR_OCR_FAILED: i32 = -32008;

/// Get error message for a given error code
pub fn error_message(code: i32) -> &'static str {
    match code {
        ERROR_PARSE_ERROR => "Parse error: Invalid JSON",
        ERROR_INVALID_REQUEST => "Invalid Request",
        ERROR_METHOD_NOT_FOUND => "Method not found",
        ERROR_INVALID_PARAMS => "Invalid params",
        ERROR_INTERNAL_ERROR => "Internal error",
        ERROR_SERVER_ERROR => "Server error",
        ERROR_UNAUTHORIZED => "Unauthorized",
        ERROR_READ_ONLY_MODE => "Read-only mode",
        ERROR_TERMINAL_NOT_FOUND => "Terminal not found",
        ERROR_PERMISSION_DENIED => "Permission denied",
        ERROR_DATABASE_LOCKED => "Database locked",
        ERROR_NO_DATA_AVAILABLE => "No data available",
        ERROR_OCR_FAILED => "OCR failed",
        _ => "Unknown error",
    }
}
