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
                description: Some(
                    "Parse a user query into structured search filters for memory retrieval."
                        .to_string(),
                ),
                arguments: Some(vec![PromptArgument {
                    name: "query".to_string(),
                    description: Some("User query to parse".to_string()),
                    required: Some(true),
                }]),
            },
            Prompt {
                name: "mcp_answer_with_memory".to_string(),
                description: Some(
                    "Answer user questions using retrieved memory context.".to_string(),
                ),
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
            // New Prompt Resources for Wave 3
            Prompt {
                name: "debug_context".to_string(),
                description: Some(
                    "Analyze recent error logs and terminal output to help debug issues."
                        .to_string(),
                ),
                arguments: Some(vec![
                    PromptArgument {
                        name: "time_range".to_string(),
                        description: Some("Time range to analyze: 5m, 15m, 30m, 1h".to_string()),
                        required: Some(false),
                    },
                    PromptArgument {
                        name: "error_pattern".to_string(),
                        description: Some("Optional error pattern to focus on".to_string()),
                        required: Some(false),
                    },
                ]),
            },
            Prompt {
                name: "visual_regression".to_string(),
                description: Some(
                    "Analyze UI changes and visual differences across time periods.".to_string(),
                ),
                arguments: Some(vec![
                    PromptArgument {
                        name: "app_name".to_string(),
                        description: Some("Application name to analyze".to_string()),
                        required: Some(true),
                    },
                    PromptArgument {
                        name: "compare_range".to_string(),
                        description: Some(
                            "Comparison range: today_vs_yesterday, this_week_vs_last".to_string(),
                        ),
                        required: Some(false),
                    },
                ]),
            },
            Prompt {
                name: "implicit_knowledge".to_string(),
                description: Some(
                    "Discover implicit knowledge and patterns from user's work history."
                        .to_string(),
                ),
                arguments: Some(vec![
                    PromptArgument {
                        name: "topic".to_string(),
                        description: Some("Topic area to explore".to_string()),
                        required: Some(true),
                    },
                    PromptArgument {
                        name: "depth".to_string(),
                        description: Some(
                            "Exploration depth: surface, deep, comprehensive".to_string(),
                        ),
                        required: Some(false),
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
                description: Some(
                    "Parse query into search filters for MCP memory tools.".to_string(),
                ),
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
        // New Prompt Resources for Wave 3
        "debug_context" => {
            let template = PromptTemplate::new(
                "You are a debugging assistant analyzing recent terminal output and error logs.\n\
                 Analyze the provided context and help identify:\n\
                 1. The root cause of any errors\n\
                 2. Potential solutions or next steps\n\
                 3. Related issues that might be connected\n\n\
                 Focus on the time range: {{time_range}} (default: last 15 minutes)\n\
                 Error pattern to look for: {{error_pattern}} (if specified)\n\n\
                 Provide concise, actionable insights.",
            );
            let mut vars = build_vars(arguments);
            vars.entry("time_range".to_string())
                .or_insert_with(|| "15m".to_string());
            vars.entry("error_pattern".to_string())
                .or_insert_with(|| "any".to_string());
            let content = template.render(&vars);
            Some(GetPromptResult {
                description: Some("Debug assistant for analyzing errors and logs.".to_string()),
                messages: vec![PromptMessage {
                    role: "system".to_string(),
                    content: PromptMessageContent {
                        type_: "text".to_string(),
                        text: content,
                    },
                }],
            })
        }
        "visual_regression" => {
            let template = PromptTemplate::new(
                "You are analyzing UI changes and visual differences in application usage.\n\
                 App being analyzed: {{app_name}}\n\
                 Comparison range: {{compare_range}} (default: today_vs_yesterday)\n\n\
                 Identify:\n\
                 1. Significant UI changes or new features used\n\
                 2. Workflow patterns that have changed\n\
                 3. Potential regressions or improvements\n\n\
                 Provide a summary of visual/workflow changes.",
            );
            let mut vars = build_vars(arguments);
            vars.entry("compare_range".to_string())
                .or_insert_with(|| "today_vs_yesterday".to_string());
            let content = template.render(&vars);
            Some(GetPromptResult {
                description: Some("Analyze UI changes and visual differences.".to_string()),
                messages: vec![PromptMessage {
                    role: "system".to_string(),
                    content: PromptMessageContent {
                        type_: "text".to_string(),
                        text: content,
                    },
                }],
            })
        }
        "implicit_knowledge" => {
            let template = PromptTemplate::new(
                "You are discovering implicit knowledge from the user's work history.\n\
                 Topic area: {{topic}}\n\
                 Exploration depth: {{depth}} (default: deep)\n\n\
                 Analyze patterns in the user's memory to identify:\n\
                 1. Common workflows and shortcuts they use\n\
                 2. Frequently referenced code patterns or documentation\n\
                 3. Implicit relationships between different work contexts\n\
                 4. Knowledge gaps that could be filled\n\n\
                 Provide insights about the user's implicit knowledge in this topic area.",
            );
            let mut vars = build_vars(arguments);
            vars.entry("depth".to_string())
                .or_insert_with(|| "deep".to_string());
            let content = template.render(&vars);
            Some(GetPromptResult {
                description: Some("Discover implicit knowledge from work history.".to_string()),
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
