use crate::db;
use crate::vector_db;
use anyhow::Result;
use sqlx::Row;
use std::collections::HashMap;

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
    pub async fn search(&self, query: &str, limit: usize) -> Result<Vec<SearchResult>> {
        // 1. 向量语义检索
        let query_embedding = vector_db::generate_embedding(query).await?;
        let vector_results = vector_db::search_similar(query_embedding, limit * 2).await?;

        // 2. BM25 关键词检索
        let bm25_results = self.bm25_search(query, limit * 2).await?;

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
        let mut final_results: Vec<SearchResult> = combined
            .into_iter()
            .map(|(id, (score, _))| {
                // 获取活动时间戳（简化实现，避免阻塞）
                // TODO: 异步获取活动信息

                // 时间衰减：越新的活动权重越高（简化实现，暂时使用固定值）
                let time_decay = 1.0;

                SearchResult {
                    id,
                    score: score * time_decay,
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
}
