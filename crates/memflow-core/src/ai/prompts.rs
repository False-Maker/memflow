//! Prompt 管理模块 - 外部化 System Prompts 配置
//!
//! 支持从资源文件加载 prompts，失败时使用内置默认值

use anyhow::Result;
use once_cell::sync::Lazy;
use serde::{Deserialize, Serialize};
use std::path::PathBuf;
use std::sync::Arc;
use tokio::sync::RwLock;

/// Prompt 配置结构
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PromptsConfig {
    pub chat: ChatPrompts,
    pub intent_parser: IntentParserPrompts,
    pub analyze_for_proposals: AnalyzePrompts,
    #[serde(default)]
    pub agent: AgentConfig,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ChatPrompts {
    pub system_default: String,
    pub system_with_context: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct IntentParserPrompts {
    pub system: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AnalyzePrompts {
    pub system: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct AgentConfig {
    #[serde(default = "default_context_max_items")]
    pub context_max_items: usize,
    #[serde(default = "default_context_max_chars_per_ocr")]
    pub context_max_chars_per_ocr: usize,
    #[serde(default = "default_session_gap_minutes")]
    pub session_gap_minutes: i64,
}

fn default_context_max_items() -> usize { 40 }
fn default_context_max_chars_per_ocr() -> usize { 100 }
fn default_session_gap_minutes() -> i64 { 5 }

impl Default for PromptsConfig {
    fn default() -> Self {
        Self {
            chat: ChatPrompts {
                system_default: "你是桌面活动记录分析助手。直接回答用户的问题，简洁明了。如果用户只是测试，简单确认即可。".to_string(),
                system_with_context: "你是桌面活动记录分析助手。基于用户提供的桌面活动记录（OCR文本、应用名称等）回答问题。只回答事实，不要解释如何设计系统。".to_string(),
            },
            intent_parser: IntentParserPrompts {
                system: r#"You are a smart query parser for a personal activity logger. 
Your goal is to extract search filters from the user's natural language query.

Return a JSON object with the following fields:
- "app_name": (string | null) Filter by application name (e.g., "Chrome", "VS Code"). If the user mentions "pdf", map it to a likely pdf reader or just "pdf".
- "keywords": (string[]) List of keywords to search in OCR text or window titles.
- "date_range": (string | null) One of: "today", "yesterday", "this_week", "last_week", "this_month", or null if not specified.
- "has_ocr": (boolean | null) true if user wants to search within text/content, null otherwise.

Example 1:
Input: "Show me what I did on Chrome yesterday"
Output: { "app_name": "Chrome", "keywords": [], "date_range": "yesterday", "has_ocr": null }

Example 2:
Input: "Find PDF files about rust from last week"
Output: { "app_name": "pdf", "keywords": ["rust"], "date_range": "last_week", "has_ocr": true }

Example 3:
Input: "coding session"
Output: { "app_name": "Code", "keywords": ["coding"], "date_range": null, "has_ocr": null }

Return ONLY the JSON object."#.to_string(),
            },
            analyze_for_proposals: AnalyzePrompts {
                system: r#"你是专业的个人工作助理。请分析用户的电脑活动日志，识别出用户今天的主要任务/上下文（Task Contexts）。
请返回 JSON 格式，不要包含 Markdown 代码块标记。
JSON 结构如下：
{
  "tasks": [
    {
      "title": "任务名称（如：MemFlow 后端开发）",
      "summary": "该任务段的详细摘要（Markdown 格式），包含主要操作和产出",
      "related_urls": ["https://github.com/...", "https://docs.rs/..."],
      "related_files": ["D:\\Projects\\src\\main.rs", "C:\\Users\\...\\report.docx"],
      "related_apps": ["C:\\Program Files\\...\\Code.exe"]
    }
  ]
}

要求：
1. `tasks`: 将连续或相关联的活动聚类为一个任务。
2. `summary`: 必须是 Markdown 格式，结构清晰。
3. `related_urls`: 提取该任务中访问的关键文档或网页链接（最多 5 个）。
4. `related_files`: 尝试从窗口标题或 OCR 内容中提取关键的本地文件路径（如 .docx, .pdf, .rs, .py 等）。
5. `related_apps`: 如果任务依赖特定应用程序（如 VS Code, Photoshop），且日志中明确记录了该应用的绝对路径（app_path），请将其路径放入此列表。忽略系统自带应用（如资源管理器）。"#.to_string(),
            },
            agent: AgentConfig::default(),
        }
    }
}

static PROMPTS: Lazy<Arc<RwLock<PromptsConfig>>> = 
    Lazy::new(|| Arc::new(RwLock::new(PromptsConfig::default())));

/// 从资源目录初始化 prompts 配置
pub async fn init_prompts(resource_path: Option<PathBuf>) -> Result<()> {
    let config = if let Some(path) = resource_path {
        let prompts_path = path.join("prompts.json");
        if prompts_path.exists() {
            match std::fs::read_to_string(&prompts_path) {
                Ok(content) => {
                    match serde_json::from_str::<PromptsConfig>(&content) {
                        Ok(config) => {
                            tracing::info!("从 {:?} 加载 prompts 配置成功", prompts_path);
                            config
                        }
                        Err(e) => {
                            tracing::warn!("解析 prompts.json 失败: {}, 使用默认配置", e);
                            PromptsConfig::default()
                        }
                    }
                }
                Err(e) => {
                    tracing::warn!("读取 prompts.json 失败: {}, 使用默认配置", e);
                    PromptsConfig::default()
                }
            }
        } else {
            tracing::info!("prompts.json 不存在，使用默认配置");
            PromptsConfig::default()
        }
    } else {
        tracing::info!("未指定资源路径，使用默认 prompts 配置");
        PromptsConfig::default()
    };
    
    *PROMPTS.write().await = config;
    Ok(())
}

/// 获取当前 prompts 配置
pub async fn get_prompts() -> PromptsConfig {
    PROMPTS.read().await.clone()
}

/// 获取 chat 系统提示词（根据是否有上下文选择）
pub async fn get_chat_system_prompt(has_context: bool) -> String {
    let prompts = PROMPTS.read().await;
    if has_context {
        prompts.chat.system_with_context.clone()
    } else {
        prompts.chat.system_default.clone()
    }
}

/// 获取意图解析系统提示词
pub async fn get_intent_parser_prompt() -> String {
    PROMPTS.read().await.intent_parser.system.clone()
}

/// 获取提案分析系统提示词
pub async fn get_analyze_proposals_prompt() -> String {
    PROMPTS.read().await.analyze_for_proposals.system.clone()
}

/// 获取 Agent 配置
pub async fn get_agent_config() -> AgentConfig {
    PROMPTS.read().await.agent.clone()
}

