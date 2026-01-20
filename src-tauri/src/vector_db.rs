use crate::ai::provider::{embedding_with_openai, ProviderConfig};
use crate::db;
use anyhow::Result;
use serde::{Deserialize, Serialize};
use sqlx::Row;

// 向量维度（可以根据实际嵌入模型调整）
const EMBEDDING_DIM: usize = 384; // 使用较小的模型，如 all-MiniLM-L6-v2

/// 计算余弦相似度
fn cosine_similarity(a: &[f32], b: &[f32]) -> f64 {
    if a.len() != b.len() {
        return 0.0;
    }

    let dot_product: f32 = a.iter().zip(b.iter()).map(|(x, y)| x * y).sum();
    let norm_a: f32 = a.iter().map(|x| x * x).sum::<f32>().sqrt();
    let norm_b: f32 = b.iter().map(|x| x * x).sum::<f32>().sqrt();

    if norm_a == 0.0 || norm_b == 0.0 {
        return 0.0;
    }

    (dot_product / (norm_a * norm_b)) as f64
}

/// 插入向量嵌入
pub async fn insert_embedding(activity_id: i64, embedding: Vec<f32>) -> Result<()> {
    if embedding.len() != EMBEDDING_DIM {
        return Err(anyhow::anyhow!(
            "向量维度不匹配: 期望 {}, 实际 {}",
            EMBEDDING_DIM,
            embedding.len()
        ));
    }

    // 将向量序列化为 JSON 存储
    let embedding_json = serde_json::to_string(&embedding)?;

    // 存储到数据库
    let pool = db::get_pool().await?;

    sqlx::query("INSERT OR REPLACE INTO vector_embeddings (activity_id, embedding) VALUES (?, ?)")
        .bind(activity_id)
        .bind(embedding_json)
        .execute(&pool)
        .await?;

    Ok(())
}

/// 搜索相似向量
pub async fn search_similar(query: Vec<f32>, limit: usize) -> Result<Vec<SearchResult>> {
    if query.len() != EMBEDDING_DIM {
        return Err(anyhow::anyhow!(
            "查询向量维度不匹配: 期望 {}, 实际 {}",
            EMBEDDING_DIM,
            query.len()
        ));
    }

    let pool = db::get_pool().await?;

    // 获取所有向量
    let rows = sqlx::query("SELECT activity_id, embedding FROM vector_embeddings")
        .fetch_all(&pool)
        .await?;

    let mut results = Vec::new();

    for row in rows {
        let activity_id: i64 = row.get(0);
        let embedding_json: String = row.get(1);

        let embedding: Vec<f32> = serde_json::from_str(&embedding_json)?;

        let similarity = cosine_similarity(&query, &embedding);

        results.push(SearchResult {
            id: activity_id,
            score: similarity,
        });
    }

    // 按相似度排序
    results.sort_by(|a, b| {
        b.score
            .partial_cmp(&a.score)
            .unwrap_or(std::cmp::Ordering::Equal)
    });

    // 返回前 limit 个结果
    results.truncate(limit);

    Ok(results)
}

/// 获取活动的向量嵌入
pub async fn get_embedding(activity_id: i64) -> Result<Option<Vec<f32>>> {
    let pool = db::get_pool().await?;

    let row = sqlx::query("SELECT embedding FROM vector_embeddings WHERE activity_id = ?")
        .bind(activity_id)
        .fetch_optional(&pool)
        .await?;

    if let Some(row) = row {
        let embedding_json: String = row.get(0);
        let embedding: Vec<f32> = serde_json::from_str(&embedding_json)?;
        Ok(Some(embedding))
    } else {
        Ok(None)
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SearchResult {
    pub id: i64,
    pub score: f64,
}

/// 生成文本嵌入向量
/// 优先使用 OpenAI Embeddings API，如果未配置则使用占位实现
pub async fn generate_embedding(text: &str) -> Result<Vec<f32>> {
    // 获取配置
    let config = crate::app_config::get_config().await.unwrap_or_else(|_| {
        let mut cfg: crate::commands::AppConfig = serde_json::from_str("{}").unwrap();
        cfg.ocr_enabled = true;
        cfg
    });

    let model_id = &config.embedding_model;

    // 嵌入模型使用 OpenAI 兼容 Embeddings API（Anthropic 不支持嵌入模型）
    // - 如果 embedding_use_shared_key=true：复用 openai key
    // - 否则：使用独立的 embedding key
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

        // 使用 OpenAI Embeddings API
        match embedding_with_openai(text, model_id, &provider_config).await {
            Ok(embedding) => {
                tracing::debug!(
                    "使用 OpenAI Embeddings API 生成向量，模型: {}，维度: {}",
                    model_id,
                    embedding.len()
                );
                // 处理维度适配
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

    // 占位实现：使用简单的哈希生成向量
    use std::collections::hash_map::DefaultHasher;
    use std::hash::{Hash, Hasher};

    let mut hasher = DefaultHasher::new();
    text.hash(&mut hasher);
    let hash = hasher.finish();

    // 生成一个简单的向量（实际应该使用嵌入模型）
    let mut embedding = vec![0.0f32; EMBEDDING_DIM];
    for i in 0..EMBEDDING_DIM {
        let seed = (hash as u64).wrapping_mul(i as u64 + 1);
        embedding[i] = ((seed % 1000) as f32 / 1000.0 - 0.5) * 2.0;
    }

    // 归一化
    let norm: f32 = embedding.iter().map(|x| x * x).sum::<f32>().sqrt();
    if norm > 0.0 {
        for e in &mut embedding {
            *e /= norm;
        }
    }

    Ok(embedding)
}
