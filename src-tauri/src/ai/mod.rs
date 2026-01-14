pub mod provider;
pub mod rag;

use crate::ai::provider::{chat_with_anthropic, chat_with_openai, ProviderConfig};
use crate::ai::rag::HybridSearch;
use crate::vector_db;
use anyhow::Result;

pub async fn analyze_activity(activity_id: i64) -> Result<String> {
    // 1. 获取活动信息
    let activity = crate::db::get_activity_by_id(activity_id).await?;

    // 2. 生成嵌入
    let text = activity.ocr_text.unwrap_or_default();
    if text.is_empty() {
        return Ok("活动无 OCR 文本，无法分析".to_string());
    }

    let embedding = vector_db::generate_embedding(&text).await?;

    // 3. 保存嵌入
    vector_db::insert_embedding(activity_id, embedding).await?;

    // 4. 简单的分析（实际应该调用 LLM）
    Ok(format!(
        "活动分析：应用 {} 在窗口 {} 中进行了操作",
        activity.app_name, activity.window_title
    ))
}

pub async fn chat(query: &str, _context: Vec<i64>) -> Result<String> {
    // 1. 混合检索相关活动
    let searcher = HybridSearch::new();
    let results = searcher.search(query, 5).await?;

    // 2. 构建上下文
    let mut context_text = String::new();
    let mut context_count = 0;
    for result in results {
        if let Ok(activity) = crate::db::get_activity_by_id(result.id).await {
            let mut has_content = false;
            if let Some(ref ocr_text) = activity.ocr_text {
                if !ocr_text.trim().is_empty() {
                    context_text.push_str(&format!(
                        "应用: {} | 窗口: {}\n内容: {}\n\n",
                        activity.app_name,
                        activity.window_title,
                        ocr_text.trim()
                    ));
                    has_content = true;
                }
            }
            // 即使没有 OCR 文本，也记录应用和窗口信息
            if !has_content {
                context_text.push_str(&format!(
                    "应用: {} | 窗口: {}\n\n",
                    activity.app_name, activity.window_title
                ));
            }
            context_count += 1;
        }
    }

    tracing::info!(
        "检索到 {} 条相关活动记录，构建了 {} 字符的上下文",
        context_count,
        context_text.len()
    );

    // 3. 尝试调用 LLM API
    let config = crate::app_config::get_config().await.unwrap_or_else(|_| {
        let mut cfg: crate::commands::AppConfig = serde_json::from_str("{}").unwrap();
        cfg.ocr_enabled = true;
        cfg
    });

    let model_id = &config.chat_model;

    // 根据模型名称自动判断提供商：如果以 "claude-" 开头则是 Anthropic，否则默认 OpenAI
    let is_anthropic = model_id.starts_with("claude-");

    if is_anthropic {
        // 尝试使用 Anthropic API
        match crate::secure_storage::get_api_key("anthropic").await {
            Ok(Some(api_key)) => {
                let provider_config = ProviderConfig::new(
                    api_key,
                    config.anthropic_base_url.clone(),
                    "https://api.anthropic.com",
                );

                match chat_with_anthropic(query, &context_text, model_id, &provider_config, None).await {
                    Ok(answer) => {
                        tracing::info!("使用 Anthropic API 生成回答，模型: {}", model_id);
                        return Ok(answer);
                    }
                    Err(e) => {
                        tracing::error!("Anthropic API 调用失败: {}", e);
                        return Ok(format!(
                            "⚠️ Anthropic API 调用失败\n\n错误信息：{}\n\n请检查：\n1. API Key 是否有效\n2. 网络连接是否正常\n3. 模型名称是否正确（当前: {}）",
                            e, model_id
                        ));
                    }
                }
            }
            Ok(None) => {
                tracing::warn!("未配置 Anthropic API Key");
                return Ok(format!(
                    "⚠️ 未配置 Anthropic API Key\n\n当前选择的模型是 {}，需要 Anthropic API Key。\n\n请在「设置」中配置 Anthropic API Key 后再试。",
                    model_id
                ));
            }
            Err(e) => {
                tracing::error!("获取 Anthropic API Key 失败: {}", e);
                return Ok(format!("⚠️ 获取 API Key 失败：{}", e));
            }
        }
    } else {
        // 默认使用 OpenAI API
        match crate::secure_storage::get_api_key("openai").await {
            Ok(Some(api_key)) => {
                let provider_config = ProviderConfig::new(
                    api_key,
                    config.openai_base_url.clone(),
                    "https://api.openai.com/v1",
                );

                match chat_with_openai(query, &context_text, model_id, &provider_config, None).await {
                    Ok(answer) => {
                        tracing::info!("使用 OpenAI API 生成回答，模型: {}", model_id);
                        return Ok(answer);
                    }
                    Err(e) => {
                        tracing::error!("OpenAI API 调用失败: {}", e);
                        return Ok(format!(
                            "⚠️ OpenAI API 调用失败\n\n错误信息：{}\n\n请检查：\n1. API Key 是否有效\n2. 网络连接是否正常\n3. 模型名称是否正确（当前: {}）\n4. 如果使用自定义 Base URL，请确认地址正确",
                            e, model_id
                        ));
                    }
                }
            }
            Ok(None) => {
                tracing::warn!("未配置 OpenAI API Key");
                return Ok(format!(
                    "⚠️ 未配置 OpenAI API Key\n\n当前选择的模型是 {}，需要 OpenAI API Key。\n\n请在「设置」中配置 OpenAI API Key 后再试。",
                    model_id
                ));
            }
            Err(e) => {
                tracing::error!("获取 OpenAI API Key 失败: {}", e);
                return Ok(format!("⚠️ 获取 API Key 失败：{}", e));
            }
        }
    }
}

use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize)]
pub struct AiAnalysisResult {
    pub tasks: Vec<TaskContext>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct TaskContext {
    pub title: String,
    pub summary: String,
    pub related_urls: Vec<String>,
    pub related_files: Vec<String>,
    pub related_apps: Vec<String>,
}

pub async fn analyze_for_proposals(context_text: &str) -> Result<AiAnalysisResult> {
    let config = crate::app_config::get_config().await.unwrap_or_else(|_| {
        let mut cfg: crate::commands::AppConfig = serde_json::from_str("{}").unwrap();
        cfg.ocr_enabled = true;
        cfg
    });

    let model_id = &config.chat_model;
    let is_anthropic = model_id.starts_with("claude-");
    
    let system_prompt = r#"你是专业的个人工作助理。请分析用户的电脑活动日志，识别出用户今天的主要任务/上下文（Task Contexts）。
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
5. `related_apps`: 如果任务依赖特定应用程序（如 VS Code, Photoshop），且日志中明确记录了该应用的绝对路径（app_path），请将其路径放入此列表。忽略系统自带应用（如资源管理器）。
"#;

    let response = if is_anthropic {
        match crate::secure_storage::get_api_key("anthropic").await {
            Ok(Some(api_key)) => {
                let provider_config = ProviderConfig::new(
                    api_key,
                    config.anthropic_base_url.clone(),
                    "https://api.anthropic.com",
                );

                chat_with_anthropic(
                    "请分析活动记录并生成建议", 
                    context_text, 
                    model_id, 
                    &provider_config, 
                    Some(system_prompt)
                ).await?
            }
            Ok(None) => return Err(anyhow::anyhow!("未配置 Anthropic API Key")),
            Err(e) => return Err(anyhow::anyhow!("获取 API Key 失败: {}", e)),
        }
    } else {
        match crate::secure_storage::get_api_key("openai").await {
            Ok(Some(api_key)) => {
                let provider_config = ProviderConfig::new(
                    api_key,
                    config.openai_base_url.clone(),
                    "https://api.openai.com/v1",
                );

                chat_with_openai(
                    "请分析活动记录并生成建议", 
                    context_text, 
                    model_id, 
                    &provider_config, 
                    Some(system_prompt)
                ).await?
            }
            Ok(None) => return Err(anyhow::anyhow!("未配置 OpenAI API Key")),
            Err(e) => return Err(anyhow::anyhow!("获取 API Key 失败: {}", e)),
        }
    };

    // 尝试解析 JSON（处理可能存在的 markdown 标记）
    let json_str = response.trim();
    let json_str = if json_str.starts_with("```json") {
        json_str.trim_start_matches("```json").trim_end_matches("```").trim()
    } else if json_str.starts_with("```") {
        json_str.trim_start_matches("```").trim_end_matches("```").trim()
    } else {
        json_str
    };

    let result: AiAnalysisResult = serde_json::from_str(json_str)
        .map_err(|e| anyhow::anyhow!("JSON 解析失败: {} - 原文: {}", e, json_str))?;
    
    Ok(result)
}

