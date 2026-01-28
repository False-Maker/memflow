//! Vector database module - Tauri wrapper for memflow-core vector_db
//!
//! Re-exports from memflow_core::vector_db and provides the
//! generate_embedding function that requires config/API keys.

// Re-export everything from memflow-core vector_db
pub use memflow_core::vector_db::*;

use crate::ai::provider::{embedding_with_openai, ProviderConfig};
use anyhow::Result;

/// Generate embedding using configured AI provider
/// This is Tauri-specific as it uses app_config and secure_storage
pub async fn generate_embedding(text: &str) -> Result<Vec<f32>> {
    // Get config
    let config = crate::app_config::get_config().await.unwrap_or_else(|_| {
        let mut cfg: crate::commands::AppConfig = serde_json::from_str("{}").unwrap();
        cfg.ocr_enabled = true;
        cfg
    });

    let model_id = &config.embedding_model;

    // Embedding uses OpenAI-compatible API (Anthropic doesn't support embeddings)
    // - If embedding_use_shared_key=true: reuse openai key
    // - Otherwise: use dedicated embedding key
    let key_service = if config.embedding_use_shared_key {
        "openai"
    } else {
        "embedding"
    };

    if let Ok(Some(api_key)) = crate::secure_storage::get_api_key(key_service).await {
        let provider_config = ProviderConfig::new(
            api_key,
            config
                .embedding_base_url
                .clone()
                .or_else(|| config.openai_base_url.clone()),
            "https://api.openai.com/v1",
        );

        // Use OpenAI Embeddings API
        match embedding_with_openai(text, model_id, &provider_config).await {
            Ok(embedding) => {
                tracing::debug!(
                    "使用 OpenAI Embeddings API 生成向量，模型: {}，维度: {}",
                    model_id,
                    embedding.len()
                );
                // Handle dimension adaptation
                if embedding.len() != EMBEDDING_DIM {
                    if embedding.len() > EMBEDDING_DIM {
                        return Ok(embedding[..EMBEDDING_DIM].to_vec());
                    } else {
                        let mut result = embedding;
                        result.resize(EMBEDDING_DIM, 0.0);
                        return Ok(result);
                    }
                }
                return Ok(embedding);
            }
            Err(e) => {
                tracing::warn!(
                    "OpenAI Embeddings API 调用失败，使用占位实现: {}",
                    crate::redact::redact_secrets(&e.to_string())
                );
            }
        }
    } else {
        tracing::debug!(
            "未配置 Embeddings API Key(service={})，使用占位实现",
            key_service
        );
    }

    // Fallback to placeholder implementation from core
    Ok(generate_placeholder_embedding(text))
}
