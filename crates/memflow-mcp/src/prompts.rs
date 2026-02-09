use crate::protocol::{
    GetPromptResult, ListPromptsResult, Prompt, PromptArgument, PromptMessage, PromptMessageContent,
};
use memflow_core::ai::prompt_engine::PromptTemplate;
use serde_json::Value;
use std::collections::HashMap;

fn build_vars(arguments: Option<Value>) -> HashMap<String, String> {
    let mut vars = HashMap::new();
    if let Some(Value::Object(map)) = arguments {
        for (k, v) in map {
            let value = match v {
                Value::String(s) => s,
                Value::Number(n) => n.to_string(),
                Value::Bool(b) => b.to_string(),
                Value::Null => String::new(),
                other => other.to_string(),
            };
            vars.insert(k, value);
        }
    }
    vars
}

pub fn list_prompts() -> ListPromptsResult {
    ListPromptsResult {
        prompts: vec![
            Prompt {
                name: "mcp_search_intent".to_string(),
                description: Some("Parse a user query into structured search filters for memory retrieval.".to_string()),
                arguments: Some(vec![PromptArgument {
                    name: "query".to_string(),
                    description: Some("User query to parse".to_string()),
                    required: Some(true),
                }]),
            },
            Prompt {
                name: "mcp_answer_with_memory".to_string(),
                description: Some("Answer user questions using retrieved memory context.".to_string()),
                arguments: Some(vec![
                    PromptArgument {
                        name: "query".to_string(),
                        description: Some("User question".to_string()),
                        required: Some(true),
                    },
                    PromptArgument {
                        name: "context".to_string(),
                        description: Some("Related memory context".to_string()),
                        required: Some(true),
                    },
                ]),
            },
        ],
    }
}

pub fn get_prompt(name: &str, arguments: Option<Value>) -> Option<GetPromptResult> {
    match name {
        "mcp_search_intent" => {
            let template = PromptTemplate::new(
                "You are a strict query parser for a personal memory system.\n\
Return ONLY a JSON object with fields:\n\
{ \"app_name\": string|null, \"keywords\": string[], \"date_range\": string|null, \"has_ocr\": boolean|null }\n\
Allowed date_range values: today, yesterday, this_week, last_week, this_month.\n\
If unsure, set fields to null or empty array.\n\
User query: {{query}}",
            );
            let vars = build_vars(arguments);
            let content = template.render(&vars);
            Some(GetPromptResult {
                description: Some("Parse query into search filters for MCP memory tools.".to_string()),
                messages: vec![PromptMessage {
                    role: "system".to_string(),
                    content: PromptMessageContent {
                        type_: "text".to_string(),
                        text: content,
                    },
                }],
            })
        }
        "mcp_answer_with_memory" => {
            let template = PromptTemplate::new(
                "You answer questions using the provided memory context.\n\
If the context is insufficient, say so and ask a concise follow-up question.\n\
Do not invent facts.\n\n\
## Memory Context\n{{context}}\n\n\
## User Question\n{{query}}",
            );
            let vars = build_vars(arguments);
            let content = template.render(&vars);
            Some(GetPromptResult {
                description: Some("Answer user question using memory context.".to_string()),
                messages: vec![PromptMessage {
                    role: "system".to_string(),
                    content: PromptMessageContent {
                        type_: "text".to_string(),
                        text: content,
                    },
                }],
            })
        }
        _ => None,
    }
}
