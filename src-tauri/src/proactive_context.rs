use crate::ai::provider::{chat_with_anthropic, chat_with_openai, ProviderConfig};
use crate::ai::rag::HybridSearch;
use crate::{app_config, db, secure_storage};
use crate::window_info::WindowInfo;
use once_cell::sync::Lazy;
use serde::{Deserialize, Serialize};
use std::sync::Mutex;
use tauri::AppHandle;
use tokio::time::{timeout, Duration};

const MIN_TRIGGER_INTERVAL_SECS: i64 = 3;
const TITLE_DISTANCE_THRESHOLD: usize = 12;
const TITLE_DISTANCE_RATIO_THRESHOLD: f64 = 0.25;

#[derive(Clone, Debug, PartialEq, Eq)]
struct ContextKey {
    process_name: String,
    window_title: String,
}

#[derive(Clone, Debug)]
pub struct TriggerContext {
    pub triggered_at: i64,
    pub process_name: String,
    pub process_path: String,
    pub window_title: String,
}

#[derive(Debug, Serialize, Clone)]
#[serde(rename_all = "camelCase")]
pub struct ContextSuggestionPayload {
    pub context: ContextSnapshot,
    pub related_memories: Vec<RelatedMemory>,
    pub suggested_actions: Vec<SuggestedAction>,
}

#[derive(Debug, Serialize, Clone)]
#[serde(rename_all = "camelCase")]
pub struct ContextSnapshot {
    pub triggered_at: i64,
    pub app_name: String,
    pub window_title: String,
}

#[derive(Debug, Serialize, Clone)]
#[serde(rename_all = "camelCase")]
pub struct RelatedMemory {
    pub id: i64,
    pub timestamp: i64,
    pub app_name: String,
    pub window_title: String,
    pub score: Option<f64>,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
#[serde(rename_all = "camelCase")]
pub struct SuggestedAction {
    pub label: String,
    pub action: String,
    pub value: String,
}

static LAST_CONTEXT: Lazy<Mutex<Option<ContextKey>>> = Lazy::new(|| Mutex::new(None));
static LAST_TRIGGER_AT: Lazy<Mutex<Option<i64>>> = Lazy::new(|| Mutex::new(None));

pub fn maybe_trigger(window_info: &WindowInfo, app_handle: Option<AppHandle>) {
    let ctx = match evaluate_and_update(window_info) {
        Some(v) => v,
        None => return,
    };
    spawn_suggestion_task(ctx, app_handle);
}

fn evaluate_and_update(window_info: &WindowInfo) -> Option<TriggerContext> {
    let now = chrono::Utc::now().timestamp();
    let next_key = ContextKey {
        process_name: window_info.process_name.trim().to_string(),
        window_title: window_info.title.trim().to_string(),
    };

    let mut last_key_guard = LAST_CONTEXT.lock().unwrap();
    let last_key = last_key_guard.as_ref().cloned();

    let mut last_trigger_guard = LAST_TRIGGER_AT.lock().unwrap();
    let last_trigger_at = *last_trigger_guard;

    let should_trigger = should_trigger(last_key.as_ref(), &next_key);
    *last_key_guard = Some(next_key);

    if !should_trigger {
        return None;
    }

    if let Some(last_at) = last_trigger_at {
        if now.saturating_sub(last_at) < MIN_TRIGGER_INTERVAL_SECS {
            return None;
        }
    }

    *last_trigger_guard = Some(now);

    Some(TriggerContext {
        triggered_at: now,
        process_name: window_info.process_name.clone(),
        process_path: window_info.process_path.clone(),
        window_title: window_info.title.clone(),
    })
}

fn should_trigger(prev: Option<&ContextKey>, next: &ContextKey) -> bool {
    let Some(prev) = prev else {
        return false;
    };

    if prev.process_name != next.process_name {
        return true;
    }

    significant_title_change(&prev.window_title, &next.window_title)
}

fn significant_title_change(prev: &str, next: &str) -> bool {
    let a = prev.trim();
    let b = next.trim();

    if a.is_empty() || b.is_empty() {
        return false;
    }

    if a == b {
        return false;
    }

    let max_len = a.chars().count().max(b.chars().count()).max(1);
    let dist = levenshtein(a, b);
    let ratio = dist as f64 / max_len as f64;

    dist >= TITLE_DISTANCE_THRESHOLD && ratio >= TITLE_DISTANCE_RATIO_THRESHOLD
}

fn levenshtein(a: &str, b: &str) -> usize {
    let a_chars: Vec<char> = a.chars().collect();
    let b_chars: Vec<char> = b.chars().collect();

    if a_chars.is_empty() {
        return b_chars.len();
    }
    if b_chars.is_empty() {
        return a_chars.len();
    }

    let mut prev: Vec<usize> = (0..=b_chars.len()).collect();
    let mut curr = vec![0; b_chars.len() + 1];

    for (i, ca) in a_chars.iter().enumerate() {
        curr[0] = i + 1;
        for (j, cb) in b_chars.iter().enumerate() {
            let cost = if ca == cb { 0 } else { 1 };
            let insert = curr[j] + 1;
            let delete = prev[j + 1] + 1;
            let replace = prev[j] + cost;
            curr[j + 1] = insert.min(delete).min(replace);
        }
        prev.clone_from_slice(&curr);
    }

    prev[b_chars.len()]
}

pub fn spawn_suggestion_task(ctx: TriggerContext, app_handle: Option<AppHandle>) {
    let Some(app_handle) = app_handle else {
        return;
    };

    tokio::spawn(async move {
        let enabled = app_config::get_config()
            .await
            .map(|c| c.enable_proactive_assistant && c.ai_enabled && !c.privacy_mode_enabled)
            .unwrap_or(false);
        if !enabled {
            return;
        }

        let payload = match timeout(Duration::from_secs(6), build_payload(&ctx)).await {
            Ok(Ok(v)) => v,
            Ok(Err(e)) => {
                tracing::debug!("proactive context build payload failed: {}", e);
                return;
            }
            Err(_) => {
                tracing::debug!("proactive context build payload timeout");
                return;
            }
        };

        use tauri::Emitter;
        let _ = app_handle.emit("context-suggestion", payload);
    });
}

async fn build_payload(ctx: &TriggerContext) -> anyhow::Result<ContextSuggestionPayload> {
    let query = format!("{} {}", ctx.process_name, ctx.window_title);
    let (related, activities_for_actions) = load_related_memories(&query, &ctx.process_name).await?;
    let suggested_actions = build_suggested_actions(ctx, &activities_for_actions).await;

    Ok(ContextSuggestionPayload {
        context: ContextSnapshot {
            triggered_at: ctx.triggered_at,
            app_name: ctx.process_name.clone(),
            window_title: ctx.window_title.clone(),
        },
        related_memories: related,
        suggested_actions,
    })
}

async fn load_related_memories(
    query: &str,
    app_name: &str,
) -> anyhow::Result<(Vec<RelatedMemory>, Vec<crate::commands::ActivityLog>)> {
    let searcher = HybridSearch::new();
    // HybridSearch from core requires explicit embedding
    let embedding = crate::vector_db::generate_embedding(query).await?;
    let results = searcher.search_with_embedding(query, embedding, 5).await;
    match results {
        Ok(items) => {
            let mut related = Vec::new();
            let mut activities = Vec::new();
            for item in items {
                if let Ok(activity) = db::get_activity_by_id(item.id).await {
                    related.push(RelatedMemory {
                        id: activity.id,
                        timestamp: activity.timestamp,
                        app_name: activity.app_name.clone(),
                        window_title: activity.window_title.clone(),
                        score: Some(item.score),
                    });
                    activities.push(activity);
                }
            }
            Ok((related, activities))
        }
        Err(_) => {
            let (items, _) = db::search_activities(
                None,
                Some(app_name.to_string()),
                None,
                None,
                None,
                Some(5),
                Some(0),
                Some("time".to_string()),
            )
            .await?;
            let related = items
                .iter()
                .map(|a| RelatedMemory {
                    id: a.id,
                    timestamp: a.timestamp,
                    app_name: a.app_name.clone(),
                    window_title: a.window_title.clone(),
                    score: None,
                })
                .collect::<Vec<_>>();
            Ok((related, items))
        }
    }
}

async fn build_suggested_actions(
    ctx: &TriggerContext,
    related: &[crate::commands::ActivityLog],
) -> Vec<SuggestedAction> {
    let config = match app_config::get_config().await {
        Ok(c) => c,
        Err(_) => return Vec::new(),
    };

    if !config.ai_enabled {
        return Vec::new();
    }

    let model_id = &config.chat_model;
    let is_anthropic = model_id.starts_with("claude-");

    let api_key = if is_anthropic {
        match secure_storage::get_api_key("anthropic").await {
            Ok(Some(v)) => v,
            _ => return Vec::new(),
        }
    } else {
        match secure_storage::get_api_key("openai").await {
            Ok(Some(v)) => v,
            _ => return Vec::new(),
        }
    };

    let mut context_text = String::new();
    for a in related.iter().take(5) {
        context_text.push_str(&format!("应用: {} | 窗口: {}\n", a.app_name, a.window_title));
        if let Some(ref t) = a.ocr_text {
            let t = t.trim();
            if !t.is_empty() {
                let excerpt: String = t.chars().take(400).collect();
                context_text.push_str(&format!("内容: {}\n", excerpt));
            }
        }
        context_text.push('\n');
    }

    let system_prompt = r#"你是一个主动式个人工作助理。基于当前窗口上下文与相关记忆，给出最多 3 条“建议操作”。
请返回 JSON 数组，每个元素包含：
- "label": 简短的操作描述
- "action": 操作类型，必须是 "open_url" (打开链接), "search" (在MemFlow中搜索), "copy" (复制内容) 之一
- "value": 对应的链接、搜索关键词或要复制的文本

例如：
[
  { "label": "打开相关 PR", "action": "open_url", "value": "https://github.com/..." },
  { "label": "搜索 'Rust 错误处理'", "action": "search", "value": "Rust 错误处理" }
]
"#;

    let user_query = format!("当前窗口：{} | {}", ctx.process_name, ctx.window_title);

    let provider_config = ProviderConfig::new(
        api_key,
        if is_anthropic {
            config.anthropic_base_url.clone()
        } else {
            config.openai_base_url.clone()
        },
        if is_anthropic {
            "https://api.anthropic.com"
        } else {
            "https://api.openai.com/v1"
        },
    );

    let response = timeout(Duration::from_secs(8), async {
        if is_anthropic {
            chat_with_anthropic(
                &user_query,
                &context_text,
                model_id,
                &provider_config,
                Some(system_prompt),
            )
            .await
        } else {
            chat_with_openai(
                &user_query,
                &context_text,
                model_id,
                &provider_config,
                Some(system_prompt),
            )
            .await
        }
    })
    .await;

    let Ok(Ok(text)) = response else {
        return Vec::new();
    };

    let json_str = text.trim();
    let json_str = if json_str.starts_with("```json") {
        json_str
            .trim_start_matches("```json")
            .trim_end_matches("```")
            .trim()
    } else if json_str.starts_with("```") {
        json_str
            .trim_start_matches("```")
            .trim_end_matches("```")
            .trim()
    } else {
        json_str
    };

    serde_json::from_str::<Vec<SuggestedAction>>(json_str).unwrap_or_default()
}

#[cfg(test)]
mod tests {
    use super::{levenshtein, significant_title_change, should_trigger, ContextKey};

    #[test]
    fn levenshtein_basic_cases() {
        assert_eq!(levenshtein("", ""), 0);
        assert_eq!(levenshtein("", "abc"), 3);
        assert_eq!(levenshtein("abc", ""), 3);
        assert_eq!(levenshtein("kitten", "sitting"), 3);
    }

    #[test]
    fn significant_title_change_requires_large_difference() {
        assert!(!significant_title_change("a", "b"));
        assert!(!significant_title_change("Hello", "Hello"));

        let a = "aaaaaaaaaaaaaaaaaaaa";
        let b = "bbbbbbbbbbbbbbbbbbbb";
        assert!(significant_title_change(a, b));
    }

    #[test]
    fn should_trigger_on_app_change() {
        let prev = ContextKey {
            process_name: "a.exe".to_string(),
            window_title: "Title".to_string(),
        };
        let next = ContextKey {
            process_name: "b.exe".to_string(),
            window_title: "Title".to_string(),
        };
        assert!(should_trigger(Some(&prev), &next));
    }
}
