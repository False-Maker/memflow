//! RAG (Retrieval-Augmented Generation) module
//! 
//! Implements hybrid search combining BM25 keyword matching and vector similarity.
//! This is the Tauri-independent core - embedding generation is passed in.

use crate::db;
use crate::vector_db;
use anyhow::Result;
use serde::{Deserialize, Serialize};
use sqlx::Row;
use std::collections::HashMap;

/// Hybrid search result with activity ID and combined score
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HybridSearchResult {
    pub id: i64,
    pub score: f64,
}

/// Hybrid search engine combining BM25 and vector similarity
pub struct HybridSearch {
    /// BM25 parameter k1 (term frequency saturation)
    #[allow(dead_code)]
    k1: f64,
    /// BM25 parameter b (document length normalization)
    #[allow(dead_code)]
    b: f64,
}

impl Default for HybridSearch {
    fn default() -> Self {
        Self { k1: 1.5, b: 0.75 }
    }
}

impl HybridSearch {
    pub fn new() -> Self {
        Self::default()
    }

    /// Hybrid search with pre-computed query embedding
    /// 
    /// # Arguments
    /// * `query` - The search query text
    /// * `query_embedding` - Pre-computed embedding vector for semantic search
    /// * `limit` - Maximum number of results
    pub async fn search_with_embedding(
        &self,
        query: &str,
        query_embedding: Vec<f32>,
        limit: usize,
    ) -> Result<Vec<HybridSearchResult>> {
        let candidate_size = (limit * 4).max(50);

        // 1. BM25 keyword search (get candidates)
        let bm25_results = self.bm25_search(query, candidate_size).await?;
        
        let candidate_ids: Vec<i64> = bm25_results.iter().map(|r| r.id).collect();
        
        // 2. Vector semantic search (only on candidates)
        let vector_results = if candidate_ids.is_empty() {
            vector_db::search_similar(query_embedding, limit * 2).await?
        } else {
            vector_db::search_similar_with_candidates(
                query_embedding,
                limit * 2,
                Some(&candidate_ids),
            )
            .await?
        };

        // 3. Merge results (weighted average)
        let mut combined: HashMap<i64, (f64, usize)> = HashMap::new();

        // Vector results (weight 0.6)
        for result in vector_results.iter() {
            let weight = 0.6 * result.score;
            let entry = combined.entry(result.id).or_insert((0.0, 0));
            entry.0 += weight;
            entry.1 += 1;
        }

        // BM25 results (weight 0.4)
        for result in bm25_results.iter() {
            let weight = 0.4 * result.score;
            let entry = combined.entry(result.id).or_insert((0.0, 0));
            entry.0 += weight;
            entry.1 += 1;
        }

        // 4. Apply time decay
        let ids: Vec<i64> = combined.keys().cloned().collect();
        let timestamps = self.get_timestamps(&ids).await?;
        let now = chrono::Utc::now().timestamp();

        let mut final_results: Vec<HybridSearchResult> = combined
            .into_iter()
            .map(|(id, (score, _))| {
                let timestamp = timestamps.get(&id).copied().unwrap_or(0);
                let new_score = Self::calculate_decayed_score(score, timestamp, now);
                HybridSearchResult { id, score: new_score }
            })
            .collect();

        // 5. Sort and limit
        final_results.sort_by(|a, b| {
            b.score.partial_cmp(&a.score).unwrap_or(std::cmp::Ordering::Equal)
        });
        final_results.truncate(limit);

        Ok(final_results)
    }

    /// Convenience method using placeholder embedding (for testing or fallback)
    pub async fn search_with_placeholder(
        &self,
        query: &str,
        limit: usize,
    ) -> Result<Vec<HybridSearchResult>> {
        let embedding = vector_db::generate_placeholder_embedding(query);
        self.search_with_embedding(query, embedding, limit).await
    }

    /// BM25 keyword search using FTS5
    async fn bm25_search(&self, query: &str, limit: usize) -> Result<Vec<HybridSearchResult>> {
        let pool = db::get_pool().await?;

        let query_terms: Vec<&str> = query.split_whitespace().collect();
        let fts_query = query_terms.join(" OR ");

        let rows = sqlx::query(
            "SELECT rowid, ocr_text FROM activity_logs_fts 
             WHERE activity_logs_fts MATCH ? 
             LIMIT ?",
        )
        .bind(&fts_query)
        .bind(limit as i64)
        .fetch_all(&pool)
        .await
        .unwrap_or_default();

        let mut results = Vec::new();

        for row in rows {
            let activity_id: i64 = row.get(0);
            let ocr_text: Option<String> = row.get(1);

            if let Some(ref text) = ocr_text {
                let score = self.calculate_tf_idf(query, text);
                results.push(HybridSearchResult {
                    id: activity_id,
                    score,
                });
            }
        }

        Ok(results)
    }

    /// Calculate simplified TF-IDF score
    fn calculate_tf_idf(&self, query: &str, document: &str) -> f64 {
        let query_terms: Vec<&str> = query.split_whitespace().collect();
        let doc_terms: Vec<&str> = document.split_whitespace().collect();

        let mut score = 0.0;

        for term in query_terms {
            let term_lower = term.to_lowercase();
            let tf = doc_terms
                .iter()
                .filter(|t| t.to_lowercase() == term_lower)
                .count() as f64;

            if tf > 0.0 {
                let idf = 1.0 + (doc_terms.len() as f64 / (tf + 1.0)).ln();
                score += tf * idf;
            }
        }

        score
    }

    /// Batch fetch activity timestamps
    async fn get_timestamps(&self, ids: &[i64]) -> Result<HashMap<i64, i64>> {
        if ids.is_empty() {
            return Ok(HashMap::new());
        }

        let pool = db::get_pool().await?;
        let mut builder = sqlx::QueryBuilder::new("SELECT id, timestamp FROM activity_logs WHERE id IN (");
        
        let mut separated = builder.separated(", ");
        for id in ids {
            separated.push_bind(id);
        }
        separated.push_unseparated(")");

        let rows = builder.build().fetch_all(&pool).await?;

        let mut timestamps = HashMap::new();
        for row in rows {
            let id: i64 = row.get(0);
            let timestamp: i64 = row.get(1);
            timestamps.insert(id, timestamp);
        }

        Ok(timestamps)
    }

    /// Calculate decayed score based on age
    /// Decay: 10% reduction per 30 days (multiply by 0.9)
    fn calculate_decayed_score(original_score: f64, timestamp: i64, now: i64) -> f64 {
        if timestamp == 0 {
            return original_score * 0.5;
        }

        let diff_seconds = now - timestamp;
        if diff_seconds < 0 {
            return original_score;
        }

        let days_diff = diff_seconds as f64 / 86400.0;
        let decay_factor = 0.9f64.powf(days_diff / 30.0);
        
        original_score * decay_factor
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_time_decay_logic() {
        let now = 1700000000;
        let score = 100.0;

        // Same day (no decay)
        let decayed = HybridSearch::calculate_decayed_score(score, now, now);
        assert!((decayed - 100.0).abs() < 0.001);

        // 30 days ago (should be ~90.0)
        let past_30_days = now - 30 * 24 * 3600;
        let decayed_30 = HybridSearch::calculate_decayed_score(score, past_30_days, now);
        assert!((decayed_30 - 90.0).abs() < 0.001);

        // 60 days ago (should be ~81.0)
        let past_60_days = now - 60 * 24 * 3600;
        let decayed_60 = HybridSearch::calculate_decayed_score(score, past_60_days, now);
        assert!((decayed_60 - 81.0).abs() < 0.001);
    }
}
