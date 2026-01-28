//! AI module - Core AI functionality for MemFlow
//!
//! This module provides pure, Tauri-independent AI utilities:
//! - NLP: Keyword extraction and text analysis
//! - Prompt Engine: Template-based prompt generation
//! - Prompts: Prompt configuration management
//! - Provider: LLM API client implementations
//! - RAG: Hybrid search combining BM25 and vector similarity
//!
//! Note: High-level chat/analysis functions that require config/API keys
//! are in src-tauri/src/ai.rs which wraps these core functions.

pub mod nlp;
pub mod prompt_engine;
pub mod prompts;
pub mod provider;
pub mod rag;

// Re-export commonly used types
pub use prompt_engine::PromptTemplate;
pub use prompts::{PromptsConfig, AgentConfig};
pub use provider::ProviderConfig;
pub use rag::{HybridSearch, HybridSearchResult};

use serde::{Deserialize, Serialize};

/// AI analysis result containing extracted task contexts
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AiAnalysisResult {
    pub tasks: Vec<TaskContext>,
}

/// Task context extracted from activity analysis
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TaskContext {
    pub title: String,
    pub summary: String,
    pub related_urls: Vec<String>,
    pub related_files: Vec<String>,
    pub related_apps: Vec<String>,
}

/// Filter parameters for activity search (parsed from user query)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FilterParams {
    pub app_name: Option<String>,
    pub keywords: Vec<String>,
    pub date_range: Option<String>,
    pub has_ocr: Option<bool>,
}

impl Default for FilterParams {
    fn default() -> Self {
        Self {
            app_name: None,
            keywords: Vec::new(),
            date_range: None,
            has_ocr: None,
        }
    }
}

/// Strip JSON code fence markers from LLM response
pub fn strip_json_code_fence(input: &str) -> &str {
    let s = input.trim();
    if s.starts_with("```json") {
        return s
            .trim_start_matches("```json")
            .trim_end_matches("```")
            .trim();
    }
    if s.starts_with("```") {
        return s
            .trim_start_matches("```")
            .trim_end_matches("```")
            .trim();
    }
    s
}

/// Parse filter params from LLM JSON response
pub fn parse_filter_params_from_response(response: &str) -> anyhow::Result<FilterParams> {
    let json_str = strip_json_code_fence(response);
    let params: FilterParams = serde_json::from_str(json_str)
        .map_err(|e| anyhow::anyhow!("JSON parse failed: {} - input: {}", e, json_str))?;
    Ok(params)
}

/// Fallback filter params extraction (regex-based, no LLM)
pub fn fallback_filter_params(query: &str) -> FilterParams {
    let q = query.trim();
    let lower = q.to_lowercase();

    let mut date_range = None;
    if lower.contains("yesterday") {
        date_range = Some("yesterday".to_string());
    } else if lower.contains("today") {
        date_range = Some("today".to_string());
    } else if lower.contains("last week") || lower.contains("last_week") {
        date_range = Some("last_week".to_string());
    } else if lower.contains("this week") || lower.contains("this_week") {
        date_range = Some("this_week".to_string());
    } else if lower.contains("this month") || lower.contains("this_month") {
        date_range = Some("this_month".to_string());
    }

    let has_ocr = if lower.contains("ocr")
        || lower.contains("content")
        || lower.contains("text")
        || lower.contains("内容")
        || lower.contains("文本")
    {
        Some(true)
    } else {
        None
    };

    let keywords = q
        .split_whitespace()
        .filter_map(|w| {
            let trimmed = w.trim_matches(|c: char| !c.is_alphanumeric() && c != '_' && c != '-');
            if trimmed.is_empty() {
                return None;
            }
            Some(trimmed.to_string())
        })
        .collect::<Vec<_>>();

    FilterParams {
        app_name: None,
        keywords,
        date_range,
        has_ocr,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn strips_code_fences_for_filter_params() {
        let input = r#"```json
{"app_name":"Chrome","keywords":[],"date_range":"yesterday","has_ocr":null}
```"#;
        let parsed = parse_filter_params_from_response(input).unwrap();
        assert_eq!(parsed.app_name.as_deref(), Some("Chrome"));
        assert_eq!(parsed.date_range.as_deref(), Some("yesterday"));
        assert!(parsed.keywords.is_empty());
    }

    #[test]
    fn parses_plain_json_for_filter_params() {
        let input = r#"{"app_name":null,"keywords":["rust"],"date_range":"last_week","has_ocr":true}"#;
        let parsed = parse_filter_params_from_response(input).unwrap();
        assert_eq!(parsed.app_name, None);
        assert_eq!(parsed.keywords, vec!["rust".to_string()]);
        assert_eq!(parsed.date_range.as_deref(), Some("last_week"));
        assert_eq!(parsed.has_ocr, Some(true));
    }

    #[test]
    fn fallback_never_panics_and_returns_keywords() {
        let parsed = fallback_filter_params("Find pdf last week rust ocr");
        assert!(!parsed.keywords.is_empty());
        assert_eq!(parsed.date_range.as_deref(), Some("last_week"));
        assert_eq!(parsed.has_ocr, Some(true));
    }
}
