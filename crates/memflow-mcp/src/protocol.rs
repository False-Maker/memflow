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

// MCP Specific Types (Simplified)

#[derive(Debug, Serialize)]
pub struct Tool {
    pub name: String,
    pub description: String,
    pub input_schema: Value,
}

#[derive(Debug, Serialize)]
pub struct ListToolsResult {
    pub tools: Vec<Tool>,
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
