use anyhow::{Context, Result};
use serde::{Deserialize, Serialize};
use std::time::Duration;

/// 安全地截取字符串到指定字符数，确保在字符边界处分割
fn safe_truncate(s: &str, max_chars: usize) -> &str {
    if s.chars().count() <= max_chars {
        return s;
    }
    s.char_indices()
        .take(max_chars)
        .last()
        .map(|(idx, _)| &s[..idx])
        .unwrap_or("")
}

/// 提供商配置
pub struct ProviderConfig {
    pub api_key: String,
    pub base_url: String,
}

impl ProviderConfig {
    pub fn new(api_key: String, base_url: Option<String>, default_url: &str) -> Self {
        Self {
            api_key,
            base_url: base_url.unwrap_or_else(|| default_url.to_string()),
        }
    }
}

/// 使用 OpenAI API 进行对话
pub async fn chat_with_openai(
    query: &str,
    context: &str,
    model: &str,
    config: &ProviderConfig,
    custom_system_prompt: Option<&str>,
) -> Result<String> {
    fn extract_final_answer_from_reasoning(reasoning: &str) -> Option<String> {
        let s = reasoning.trim();
        if s.is_empty() {
            return None;
        }

        // 跳过明显的"设计说明"部分（包含这些关键词的段落）
        let design_keywords = [
            "分析用户输入",
            "分析上下文",
            "确定目标",
            "起草回复",
            "润色回复",
            "自我修正",
            "更好的版本",
        ];
        let lines: Vec<&str> = s.lines().collect();
        let mut found_final = false;
        let mut final_lines = Vec::new();

        for line in lines {
            let line_lower = line.to_lowercase();
            // 如果遇到"最终"相关的标记，开始收集
            if line.contains("最终")
                || line.contains("更好的版本")
                || line.contains("回复（中文）")
                || line.contains("最终润色")
                || line.contains("最终回复")
            {
                found_final = true;
                // 跳过标记行本身
                continue;
            }

            // 如果包含设计关键词但不是最终部分，跳过
            if design_keywords.iter().any(|&kw| line_lower.contains(kw)) && !found_final {
                continue;
            }

            // 收集最终部分或所有非设计说明的内容
            if found_final || !design_keywords.iter().any(|&kw| line_lower.contains(kw)) {
                let cleaned = line.trim();
                if !cleaned.is_empty() && !cleaned.starts_with("**") && !cleaned.starts_with("##") {
                    // 跳过明显的标题行
                    if cleaned.len() < 100
                        || cleaned.chars().filter(|c| c.is_alphanumeric()).count() > 20
                    {
                        final_lines.push(cleaned);
                    }
                }
            }
        }

        // 如果找到了最终部分，返回它；否则返回所有非设计说明的内容
        if !final_lines.is_empty() {
            let joined = final_lines.join("\n").trim().to_string();
            if joined.len() > 10 {
                // 至少要有一定长度
                return Some(joined);
            }
        }

        // 回退：尝试找最后一个引号或对话风格的内容
        if let Some(quote_idx) = s.rfind('"') {
            if let Some(start_quote) = s[..quote_idx].rfind('"') {
                let extracted = &s[start_quote + 1..quote_idx];
                if extracted.len() > 10 {
                    return Some(extracted.trim().to_string());
                }
            }
        }

        // 最后回退：返回整个 reasoning，但过滤掉明显的设计说明段落
        let filtered: Vec<&str> = s
            .lines()
            .filter(|l| {
                let lower = l.to_lowercase();
                !design_keywords.iter().any(|&kw| lower.contains(kw))
            })
            .filter(|l| l.trim().len() > 5)
            .collect();

        if !filtered.is_empty() {
            Some(filtered.join("\n").trim().to_string())
        } else {
            None
        }
    }

    #[derive(Serialize)]
    struct ChatRequest {
        model: String,
        messages: Vec<Message>,
        max_tokens: u32,
        temperature: f32,
    }

    #[derive(Serialize)]
    struct Message {
        role: String,
        content: String,
    }

    #[derive(Deserialize, Debug)]
    struct ChatResponse {
        choices: Vec<Choice>,
    }

    #[derive(Deserialize, Debug)]
    struct Choice {
        message: Option<MessageResponse>,
        // 某些 API 使用 text 字段（旧版 OpenAI 格式）
        text: Option<String>,
        // 流式响应中的 delta
        delta: Option<MessageResponse>,
    }

    #[derive(Deserialize, Debug)]
    struct MessageResponse {
        // 某些 API 可能返回 null，使用 Option 处理
        content: Option<String>,
        // 部分 OpenAI 兼容实现会把“推理/思考”放到该字段
        reasoning_content: Option<String>,
    }

    // 构建系统提示词
    let default_system_prompt = if context.is_empty() {
        "你是桌面活动记录分析助手。直接回答用户的问题，简洁明了。如果用户只是测试，简单确认即可。"
            .to_string()
    } else {
        "你是桌面活动记录分析助手。基于用户提供的桌面活动记录（OCR文本、应用名称等）回答问题。只回答事实，不要解释如何设计系统。".to_string()
    };
    
    let system_prompt = custom_system_prompt
        .map(|s| s.to_string())
        .unwrap_or(default_system_prompt);

    // 构建用户消息
    let user_content = if context.is_empty() {
        query.to_string()
    } else {
        format!("{}\n\n--- 相关桌面活动记录 ---\n{}", query, context)
    };

    tracing::debug!("发送给模型的上下文长度: {} 字符", context.len());
    if !context.is_empty() {
        tracing::debug!("上下文预览: {}", safe_truncate(context, 200));
    }

    let request = ChatRequest {
        model: model.to_string(),
        messages: vec![
            Message {
                role: "system".to_string(),
                content: system_prompt,
            },
            Message {
                role: "user".to_string(),
                content: user_content,
            },
        ],
        max_tokens: 4096,
        temperature: 0.7,
    };

    let client = reqwest::Client::builder()
        .timeout(Duration::from_secs(300))
        .build()
        .context("创建 HTTP 客户端失败")?;
    // 智能处理 URL：如果已经包含 /chat/completions 则不再追加
    let base = config.base_url.trim_end_matches('/');
    let url = if base.ends_with("/chat/completions") {
        base.to_string()
    } else {
        format!("{}/chat/completions", base)
    };

    tracing::info!("chat_with_openai: 发送请求到 {}", url);
    let start = std::time::Instant::now();
    
    let response = client
        .post(&url)
        .header("Authorization", format!("Bearer {}", config.api_key))
        .header("Content-Type", "application/json")
        .json(&request)
        .send()
        .await
        .context("OpenAI Chat API 请求失败")?;
    
    tracing::info!("chat_with_openai: 收到响应, 耗时 {}ms, status={}", start.elapsed().as_millis(), response.status());

    if !response.status().is_success() {
        let status = response.status();
        let error_text = crate::redact::redact_secrets(&response.text().await.unwrap_or_default());
        return Err(anyhow::anyhow!(
            "OpenAI Chat API 返回错误: {} - {}",
            status,
            safe_truncate(&error_text, 800)
        ));
    }

    let response_text = response.text().await.context("读取响应体失败")?;
    let response_preview = crate::redact::redact_secrets(&response_text);
    tracing::debug!(
        "OpenAI API 原始响应: {}",
        safe_truncate(&response_preview, 1000)
    );

    let result: ChatResponse =
        serde_json::from_str(&response_text).context("解析 OpenAI Chat API 响应失败")?;

    if result.choices.is_empty() {
        return Err(anyhow::anyhow!("OpenAI API 返回空选择"));
    }

    let choice = &result.choices[0];

    // 尝试从多种可能的字段中提取内容
    let mut content = choice
        .message
        .as_ref()
        .and_then(|m| m.content.clone())
        .or_else(|| choice.delta.as_ref().and_then(|d| d.content.clone()))
        .or_else(|| choice.text.clone())
        .unwrap_or_default();

    // BigModel/部分推理模型：content 可能为空，但 reasoning_content 里包含最终答复草稿
    if content.trim().is_empty() {
        if let Some(reasoning) = choice
            .message
            .as_ref()
            .and_then(|m| m.reasoning_content.clone())
        {
            tracing::debug!("检测到 reasoning_content，长度: {}", reasoning.len());
            // 优先尝试智能提取
            if let Some(extracted) = extract_final_answer_from_reasoning(&reasoning) {
                tracing::debug!(
                    "从 reasoning_content 中提取到最终答案，长度: {}",
                    extracted.len()
                );
                content = extracted;
            } else {
                // 如果提取失败，尝试找最后一个引号内的内容（通常是最终回答）
                if let Some(last_quote) = reasoning.rfind('"') {
                    if let Some(start_quote) = reasoning[..last_quote].rfind('"') {
                        let candidate = &reasoning[start_quote + 1..last_quote];
                        if candidate.len() > 10 && candidate.len() < 500 {
                            content = candidate.trim().to_string();
                            tracing::debug!("从引号中提取到内容");
                        }
                    }
                }
                // 最后的回退：使用最后一段非空内容
                if content.trim().is_empty() {
                    let lines: Vec<&str> = reasoning.lines().collect();
                    for line in lines.iter().rev() {
                        let trimmed = line.trim();
                        if trimmed.len() > 10
                            && !trimmed.starts_with("*")
                            && !trimmed.starts_with("##")
                        {
                            content = trimmed.to_string();
                            tracing::debug!("使用最后一段作为答案");
                            break;
                        }
                    }
                }
            }
        }
    }

    if content.is_empty() {
        tracing::warn!("API 返回了空内容，完整响应: {:?}", result);
        return Err(anyhow::anyhow!("API 返回了空内容，请检查模型配置"));
    }

    Ok(content)
}

/// 使用 OpenAI API 进行流式对话
pub async fn chat_with_openai_stream<F>(
    query: &str,
    context: &str,
    model: &str,
    config: &ProviderConfig,
    custom_system_prompt: Option<&str>,
    on_chunk: F,
) -> Result<String>
where
    F: Fn(String) + Send + Sync + 'static,
{
    #[derive(Serialize)]
    struct ChatRequestStream {
        model: String,
        messages: Vec<Message>,
        max_tokens: u32,
        temperature: f32,
        stream: bool,
    }

    #[derive(Serialize)]
    struct Message {
        role: String,
        content: String,
    }

    #[derive(Deserialize, Debug)]
    struct StreamResponse {
        choices: Vec<StreamChoice>,
    }

    #[derive(Deserialize, Debug)]
    struct StreamChoice {
        delta: StreamDelta,
        finish_reason: Option<String>,
    }

    #[derive(Deserialize, Debug)]
    struct StreamDelta {
        content: Option<String>,
    }

    // 构建系统提示词
    let default_system_prompt = if context.is_empty() {
        "你是桌面活动记录分析助手。直接回答用户的问题，简洁明了。如果用户只是测试，简单确认即可。"
            .to_string()
    } else {
        "你是桌面活动记录分析助手。基于用户提供的桌面活动记录（OCR文本、应用名称等）回答问题。只回答事实，不要解释如何设计系统。".to_string()
    };
    
    let system_prompt = custom_system_prompt
        .map(|s| s.to_string())
        .unwrap_or(default_system_prompt);

    // 构建用户消息
    let user_content = if context.is_empty() {
        query.to_string()
    } else {
        format!("{}\n\n--- 相关桌面活动记录 ---\n{}", query, context)
    };

    let request = ChatRequestStream {
        model: model.to_string(),
        messages: vec![
            Message {
                role: "system".to_string(),
                content: system_prompt,
            },
            Message {
                role: "user".to_string(),
                content: user_content,
            },
        ],
        max_tokens: 2000,
        temperature: 0.7,
        stream: true,
    };

    let client = reqwest::Client::builder()
        .timeout(Duration::from_secs(300))
        .build()
        .context("创建 HTTP 客户端失败")?;

    let base = config.base_url.trim_end_matches('/');
    let url = if base.ends_with("/chat/completions") {
        base.to_string()
    } else {
        format!("{}/chat/completions", base)
    };

    let mut response = client
        .post(&url)
        .header("Authorization", format!("Bearer {}", config.api_key))
        .header("Content-Type", "application/json")
        .json(&request)
        .send()
        .await
        .context("OpenAI Chat API 请求失败")?;

    if !response.status().is_success() {
        let status = response.status();
        let error_text = crate::redact::redact_secrets(&response.text().await.unwrap_or_default());
        return Err(anyhow::anyhow!(
            "OpenAI Chat API 返回错误: {} - {}",
            status,
            safe_truncate(&error_text, 800)
        ));
    }

    let mut full_response = String::new();
    let mut buffer = String::new();

    while let Some(chunk) = response.chunk().await? {
        let chunk_str = String::from_utf8_lossy(&chunk);
        buffer.push_str(&chunk_str);

        while let Some(line_end) = buffer.find('\n') {
            let line = buffer[..line_end].trim();
            let line_content = line.to_string(); // copy for processing
            buffer.drain(..line_end + 1); // remove line + newline

            if line_content.is_empty() {
                continue;
            }

            if line_content.starts_with("data: ") {
                let json_str = &line_content[6..];
                if json_str == "[DONE]" {
                    break;
                }

                if let Ok(stream_resp) = serde_json::from_str::<StreamResponse>(json_str) {
                    if let Some(choice) = stream_resp.choices.first() {
                        if let Some(content) = &choice.delta.content {
                            if !content.is_empty() {
                                on_chunk(content.clone());
                                full_response.push_str(content);
                            }
                        }
                    }
                } else {
                    tracing::warn!("解析流式响应行失败: {}", safe_truncate(json_str, 100));
                }
            }
        }
    }

    Ok(full_response)
}

/// 使用 Anthropic API 进行对话
pub async fn chat_with_anthropic(
    query: &str,
    context: &str,
    model: &str,
    config: &ProviderConfig,
    custom_system_prompt: Option<&str>,
) -> Result<String> {
    #[derive(Serialize)]
    struct ChatRequest {
        model: String,
        max_tokens: u32,
        messages: Vec<Message>,
        system: Option<String>,
    }

    #[derive(Serialize)]
    struct Message {
        role: String,
        content: String,
    }

    #[derive(Deserialize)]
    struct ChatResponse {
        content: Vec<ContentBlock>,
    }

    #[derive(Deserialize)]
    struct ContentBlock {
        text: String,
    }

    // 构建系统提示词
    let default_system_prompt = if context.is_empty() {
        "你是桌面活动记录分析助手。直接回答用户的问题，简洁明了。如果用户只是测试，简单确认即可。"
            .to_string()
    } else {
        "你是桌面活动记录分析助手。基于用户提供的桌面活动记录（OCR文本、应用名称等）回答问题。只回答事实，不要解释如何设计系统。".to_string()
    };
    
    let system_prompt = custom_system_prompt
        .map(|s| s.to_string())
        .unwrap_or(default_system_prompt);

    // 构建用户消息
    let user_content = if context.is_empty() {
        query.to_string()
    } else {
        format!("{}\n\n--- 相关桌面活动记录 ---\n{}", query, context)
    };

    tracing::debug!("发送给 Anthropic 的上下文长度: {} 字符", context.len());

    let request = ChatRequest {
        model: model.to_string(),
        max_tokens: 2000,
        messages: vec![Message {
            role: "user".to_string(),
            content: user_content,
        }],
        system: Some(system_prompt),
    };

    let client = reqwest::Client::builder()
        .timeout(Duration::from_secs(300))
        .build()
        .context("创建 HTTP 客户端失败")?;
    // 智能处理 URL：如果已经包含 /v1/messages 则不再追加
    let base = config.base_url.trim_end_matches('/');
    let url = if base.ends_with("/v1/messages") || base.ends_with("/messages") {
        base.to_string()
    } else if base.ends_with("/v1") {
        format!("{}/messages", base)
    } else {
        format!("{}/v1/messages", base)
    };

    let response = client
        .post(&url)
        .header("x-api-key", &config.api_key)
        .header("anthropic-version", "2023-06-01")
        .header("Content-Type", "application/json")
        .json(&request)
        .send()
        .await
        .context("Anthropic Chat API 请求失败")?;

    if !response.status().is_success() {
        let status = response.status();
        let error_text = crate::redact::redact_secrets(&response.text().await.unwrap_or_default());
        return Err(anyhow::anyhow!(
            "Anthropic Chat API 返回错误: {} - {}",
            status,
            safe_truncate(&error_text, 800)
        ));
    }

    let result: ChatResponse = response
        .json()
        .await
        .context("解析 Anthropic Chat API 响应失败")?;

    if result.content.is_empty() {
        return Err(anyhow::anyhow!("Anthropic API 返回空内容"));
    }

    Ok(result.content[0].text.clone())
}

/// 使用 Anthropic API 进行流式对话
pub async fn chat_with_anthropic_stream<F>(
    query: &str,
    context: &str,
    model: &str,
    config: &ProviderConfig,
    custom_system_prompt: Option<&str>,
    on_chunk: F,
) -> Result<String>
where
    F: Fn(String) + Send + Sync + 'static,
{
    #[derive(Serialize)]
    struct ChatRequestStream {
        model: String,
        max_tokens: u32,
        messages: Vec<Message>,
        system: Option<String>,
        stream: bool,
    }

    #[derive(Serialize)]
    struct Message {
        role: String,
        content: String,
    }

    // 这些结构体用于解析 SSE 事件数据
    #[derive(Deserialize, Debug)]
    struct StreamEvent {
        #[serde(rename = "type")]
        event_type: String,
        delta: Option<StreamDelta>,
    }

    #[derive(Deserialize, Debug)]
    struct StreamDelta {
        #[serde(rename = "type")]
        delta_type: String,
        text: Option<String>,
    }

    // 构建系统提示词
    let default_system_prompt = if context.is_empty() {
        "你是桌面活动记录分析助手。直接回答用户的问题，简洁明了。如果用户只是测试，简单确认即可。"
            .to_string()
    } else {
        "你是桌面活动记录分析助手。基于用户提供的桌面活动记录（OCR文本、应用名称等）回答问题。只回答事实，不要解释如何设计系统。".to_string()
    };
    
    let system_prompt = custom_system_prompt
        .map(|s| s.to_string())
        .unwrap_or(default_system_prompt);

    // 构建用户消息
    let user_content = if context.is_empty() {
        query.to_string()
    } else {
        format!("{}\n\n--- 相关桌面活动记录 ---\n{}", query, context)
    };

    let request = ChatRequestStream {
        model: model.to_string(),
        max_tokens: 4096,
        messages: vec![Message {
            role: "user".to_string(),
            content: user_content,
        }],
        system: Some(system_prompt),
        stream: true,
    };

    let client = reqwest::Client::builder()
        .timeout(Duration::from_secs(300))
        .build()
        .context("创建 HTTP 客户端失败")?;

    // 智能处理 URL
    let base = config.base_url.trim_end_matches('/');
    let url = if base.ends_with("/v1/messages") || base.ends_with("/messages") {
        base.to_string()
    } else if base.ends_with("/v1") {
        format!("{}/messages", base)
    } else {
        format!("{}/v1/messages", base)
    };

    let mut response = client
        .post(&url)
        .header("x-api-key", &config.api_key)
        .header("anthropic-version", "2023-06-01")
        .header("Content-Type", "application/json")
        .json(&request)
        .send()
        .await
        .context("Anthropic Chat API 请求失败")?;

    if !response.status().is_success() {
        let status = response.status();
        let error_text = crate::redact::redact_secrets(&response.text().await.unwrap_or_default());
        return Err(anyhow::anyhow!(
            "Anthropic Chat API 返回错误: {} - {}",
            status,
            safe_truncate(&error_text, 800)
        ));
    }

    let mut full_response = String::new();
    let mut buffer = String::new();

    while let Some(chunk) = response.chunk().await? {
        let chunk_str = String::from_utf8_lossy(&chunk);
        buffer.push_str(&chunk_str);

        while let Some(line_end) = buffer.find('\n') {
            let line = buffer[..line_end].trim();
            let line_content = line.to_string(); 
            buffer.drain(..line_end + 1);

            if line_content.is_empty() {
                continue;
            }

            if line_content.starts_with("data: ") {
                let json_str = &line_content[6..];
                // Anthropic SSE 结束事件通常是 event: message_stop，data 里可能不包含 update
                // 这里我们解析每个 event data
                
                if let Ok(event) = serde_json::from_str::<StreamEvent>(json_str) {
                     if event.event_type == "content_block_delta" {
                         if let Some(delta) = event.delta {
                             if delta.delta_type == "text_delta" {
                                 if let Some(text) = delta.text {
                                     on_chunk(text.clone());
                                     full_response.push_str(&text);
                                 }
                             }
                         }
                     }
                }
            }
        }
    }

    Ok(full_response)
}

/// 使用 OpenAI API 生成嵌入向量
pub async fn embedding_with_openai(
    text: &str,
    model: &str,
    config: &ProviderConfig,
) -> Result<Vec<f32>> {
    #[derive(Serialize)]
    struct EmbeddingRequest {
        model: String,
        input: String,
    }

    #[derive(Deserialize)]
    struct EmbeddingResponse {
        data: Vec<EmbeddingData>,
    }

    #[derive(Deserialize)]
    struct EmbeddingData {
        embedding: Vec<f32>,
    }

    let client = reqwest::Client::builder()
        .timeout(Duration::from_secs(300))
        .build()
        .context("创建 HTTP 客户端失败")?;
    let request = EmbeddingRequest {
        model: model.to_string(),
        input: text.to_string(),
    };

    // 智能处理 URL：如果已经包含 /embeddings 则不再追加
    let base = config.base_url.trim_end_matches('/');
    let url = if base.ends_with("/embeddings") {
        base.to_string()
    } else {
        format!("{}/embeddings", base)
    };

    let response = client
        .post(&url)
        .header("Authorization", format!("Bearer {}", config.api_key))
        .header("Content-Type", "application/json")
        .json(&request)
        .send()
        .await
        .context("OpenAI Embeddings API 请求失败")?;

    if !response.status().is_success() {
        let status = response.status();
        let error_text = crate::redact::redact_secrets(&response.text().await.unwrap_or_default());
        return Err(anyhow::anyhow!(
            "OpenAI Embeddings API 返回错误: {} - {}",
            status,
            safe_truncate(&error_text, 800)
        ));
    }

    let result: EmbeddingResponse = response
        .json()
        .await
        .context("解析 OpenAI Embeddings API 响应失败")?;

    if result.data.is_empty() {
        return Err(anyhow::anyhow!("OpenAI API 返回空数据"));
    }

    Ok(result.data[0].embedding.clone())
}
