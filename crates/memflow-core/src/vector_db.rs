//! Vector database module for embedding storage and similarity search
//!
//! This module provides Tauri-independent vector operations. The embedding
//! generation function that requires config/API keys is moved to src-tauri.

use crate::db;
use anyhow::Result;
use serde::{Deserialize, Serialize};
use sqlx::Row;

/// Vector dimension (matches embedding model output)
pub const EMBEDDING_DIM: usize = 384;

/// Calculate cosine similarity between two vectors
pub fn cosine_similarity(a: &[f32], b: &[f32]) -> f64 {
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

/// Insert vector embedding into database
pub async fn insert_embedding(activity_id: i64, embedding: Vec<f32>) -> Result<()> {
    if embedding.len() != EMBEDDING_DIM {
        return Err(anyhow::anyhow!(
            "Vector dimension mismatch: expected {}, got {}",
            EMBEDDING_DIM,
            embedding.len()
        ));
    }

    let embedding_json = serde_json::to_string(&embedding)?;
    let pool = db::get_pool().await?;

    sqlx::query("INSERT OR REPLACE INTO vector_embeddings (activity_id, embedding) VALUES (?, ?)")
        .bind(activity_id)
        .bind(embedding_json)
        .execute(&pool)
        .await?;

    Ok(())
}

/// Search result with activity ID and similarity score
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SearchResult {
    pub id: i64,
    pub score: f64,
}

/// Search similar vectors (full table scan version, kept for backward compatibility)
pub async fn search_similar(query: Vec<f32>, limit: usize) -> Result<Vec<SearchResult>> {
    search_similar_with_candidates(query, limit, None).await
}

/// Search similar vectors with optional candidate set filtering
pub async fn search_similar_with_candidates(
    query: Vec<f32>,
    limit: usize,
    candidate_ids: Option<&[i64]>,
) -> Result<Vec<SearchResult>> {
    if query.len() != EMBEDDING_DIM {
        return Err(anyhow::anyhow!(
            "Query vector dimension mismatch: expected {}, got {}",
            EMBEDDING_DIM,
            query.len()
        ));
    }

    let pool = db::get_pool().await?;

    let rows = match candidate_ids {
        Some(ids) if !ids.is_empty() => {
            let mut builder =
                sqlx::QueryBuilder::new("SELECT activity_id, embedding FROM vector_embeddings WHERE activity_id IN (");
            let mut separated = builder.separated(", ");
            for id in ids {
                separated.push_bind(*id);
            }
            separated.push_unseparated(")");
            builder.build().fetch_all(&pool).await?
        }
        _ => {
            sqlx::query("SELECT activity_id, embedding FROM vector_embeddings")
                .fetch_all(&pool)
                .await?
        }
    };

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

    results.sort_by(|a, b| {
        b.score
            .partial_cmp(&a.score)
            .unwrap_or(std::cmp::Ordering::Equal)
    });

    results.truncate(limit);

    Ok(results)
}

/// Get embedding for an activity
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

/// Generate a placeholder embedding using hash (when no API is available)
/// Note: Real embedding generation requiring API keys is in src-tauri
pub fn generate_placeholder_embedding(text: &str) -> Vec<f32> {
    use std::collections::hash_map::DefaultHasher;
    use std::hash::{Hash, Hasher};

    let mut hasher = DefaultHasher::new();
    text.hash(&mut hasher);
    let hash = hasher.finish();

    let mut embedding = vec![0.0f32; EMBEDDING_DIM];
    for i in 0..EMBEDDING_DIM {
        let seed = (hash as u64).wrapping_mul(i as u64 + 1);
        embedding[i] = ((seed % 1000) as f32 / 1000.0 - 0.5) * 2.0;
    }

    // Normalize
    let norm: f32 = embedding.iter().map(|x| x * x).sum::<f32>().sqrt();
    if norm > 0.0 {
        for e in &mut embedding {
            *e /= norm;
        }
    }

    embedding
}
