use crate::db;
use crate::vector_db;
use anyhow::Result;
use sqlx::Row;
use std::collections::HashMap;
use chrono;

#[derive(Debug, Clone)]
pub struct SearchResult {
    pub id: i64,
    pub score: f64,
}

pub struct HybridSearch {
    // BM25 参数
    #[allow(dead_code)]
    k1: f64,
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

    /// 混合检索：结合 BM25 关键词匹配和向量语义检索
    /// 
    /// 优化策略：
    /// 1. 先用 FTS/BM25 获取候选集（粗筛，快速）
    /// 2. 对候选集进行向量语义检索（精排，只针对候选）
    /// 3. 加权合并结果
    /// 
    /// 这样将向量检索从 O(N) 全表扫描降为 O(|candidates|)
    pub async fn search(&self, query: &str, limit: usize) -> Result<Vec<SearchResult>> {
        // 候选集大小：取 limit 的若干倍，确保有足够的候选用于精排
        let candidate_size = (limit * 4).max(50);

        // 1. BM25 关键词检索（获取候选集）
        let bm25_results = self.bm25_search(query, candidate_size).await?;
        
        // 提取候选 ID
        let candidate_ids: Vec<i64> = bm25_results.iter().map(|r| r.id).collect();
        
        // 2. 向量语义检索（只在候选集中搜索）
        let query_embedding = vector_db::generate_embedding(query).await?;
        let vector_results = if candidate_ids.is_empty() {
            // 如果 BM25 没有结果，回退到全表向量检索
            vector_db::search_similar(query_embedding, limit * 2).await?
        } else {
            // 只对候选集进行向量检索
            vector_db::search_similar_with_candidates(
                query_embedding,
                limit * 2,
                Some(&candidate_ids),
            )
            .await?
        };

        // 3. 合并结果（加权平均）
        let mut combined: HashMap<i64, (f64, usize)> = HashMap::new();

        // 向量检索结果（权重 0.6）
        for result in vector_results.iter() {
            let weight = 0.6 * result.score;
            let entry = combined.entry(result.id).or_insert((0.0, 0));
            entry.0 += weight;
            entry.1 += 1;
        }

        // BM25 结果（权重 0.4）
        for result in bm25_results.iter() {
            let weight = 0.4 * result.score;
            let entry = combined.entry(result.id).or_insert((0.0, 0));
            entry.0 += weight;
            entry.1 += 1;
        }

        // 4. 应用时间衰减因子
        let ids: Vec<i64> = combined.keys().cloned().collect();
        let timestamps = self.get_timestamps(&ids).await?;
        let now = chrono::Utc::now().timestamp();

        let mut final_results: Vec<SearchResult> = combined
            .into_iter()
            .map(|(id, (score, _))| {
                let timestamp = timestamps.get(&id).copied().unwrap_or(0);
                let new_score = Self::calculate_decayed_score(score, timestamp, now);

                SearchResult {
                    id,
                    score: new_score,
                }
            })
            .collect();

        // 5. 排序并限制结果数
        final_results.sort_by(|a, b| {
            b.score
                .partial_cmp(&a.score)
                .unwrap_or(std::cmp::Ordering::Equal)
        });
        final_results.truncate(limit);

        Ok(final_results)
    }

    /// BM25 关键词检索
    async fn bm25_search(&self, query: &str, limit: usize) -> Result<Vec<SearchResult>> {
        // 简单的关键词匹配实现
        // 实际应该使用 FTS5 全文检索

        let pool = db::get_pool().await?;

        // 使用 FTS5 搜索
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
                // 计算简单的 TF-IDF 分数
                let score = self.calculate_tf_idf(query, &text);

                results.push(SearchResult {
                    id: activity_id,
                    score,
                });
            }
        }

        Ok(results)
    }

    /// 计算 TF-IDF 分数（简化版）
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
                // 简化的 IDF（实际应该基于整个语料库）
                let idf = 1.0 + (doc_terms.len() as f64 / (tf + 1.0)).ln();
                score += tf * idf;
            }
        }

        score
    }

    /// 批量获取活动时间戳
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

    /// 计算衰减后的分数
    /// 衰减策略：每过去30天，权重降低 10% (乘以 0.9)
    fn calculate_decayed_score(original_score: f64, timestamp: i64, now: i64) -> f64 {
        if timestamp == 0 {
            return original_score * 0.5; // 无时间戳的惩罚
        }

        let diff_seconds = now - timestamp;
        if diff_seconds < 0 {
            // 未来时间，不衰减
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
        let now = 1700000000; // 假设当前时间
        let score = 100.0;

        // 1. 测试当天 (无衰减)
        let decayed = HybridSearch::calculate_decayed_score(score, now, now);
        assert!((decayed - 100.0).abs() < 0.001);

        // 2. 测试30天前 (应当约等于 90.0)
        let past_30_days = now - 30 * 24 * 3600;
        let decayed_30 = HybridSearch::calculate_decayed_score(score, past_30_days, now);
        // 0.9^1 = 0.9
        assert!((decayed_30 - 90.0).abs() < 0.001);

        // 3. 测试60天前 (应当约等于 81.0)
        let past_60_days = now - 60 * 24 * 3600;
        let decayed_60 = HybridSearch::calculate_decayed_score(score, past_60_days, now);
        // 0.9^2 = 0.81
        assert!((decayed_60 - 81.0).abs() < 0.001);

        // 4. 测试1年前 (365天)
        let past_year = now - 365 * 24 * 3600;
        let decayed_year = HybridSearch::calculate_decayed_score(score, past_year, now);
        // 0.9^(365/30) = 0.9^12.16 ≈ 0.27
        println!("Year decay: {}", decayed_year);
        assert!(decayed_year < 30.0);
    }
}
