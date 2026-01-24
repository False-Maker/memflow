use crate::ai::nlp::{extract_keywords, extract_keywords_tfidf, KeywordOptions};
use crate::db;
use anyhow::Result;
use chrono::{DateTime, Timelike, Utc};
use once_cell::sync::Lazy;
use serde::{Deserialize, Serialize};
use sqlx::Row;
use std::collections::HashMap;
use std::sync::RwLock;

/// 图谱缓存结构
struct GraphCache {
    data: Option<GraphData>,
    last_activity_count: i64,
    last_updated: Option<std::time::Instant>,
}

/// 全局图谱缓存（避免频繁重建）
static GRAPH_CACHE: Lazy<RwLock<GraphCache>> = Lazy::new(|| {
    RwLock::new(GraphCache {
        data: None,
        last_activity_count: 0,
        last_updated: None,
    })
});

/// 缓存有效期（5 分钟）
const CACHE_TTL_SECS: u64 = 300;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GraphData {
    pub nodes: Vec<Node>,
    pub edges: Vec<Edge>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Node {
    pub id: String,
    pub name: String,
    pub group: String,
    pub size: i32,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Edge {
    pub source: String,
    pub target: String,
    pub value: i32,
}

#[derive(Debug, Clone)]
struct ActivityRecord {
    timestamp: i64,
    app_name: String,
    ocr_text: Option<String>,
}

/// 构建知识图谱（带缓存）
pub async fn build_graph() -> Result<GraphData> {
    // 检查缓存是否有效
    let current_count = db::get_activity_count().await.unwrap_or(0);
    
    {
        let cache = GRAPH_CACHE.read().unwrap();
        if let Some(ref cached_data) = cache.data {
            if let Some(last_updated) = cache.last_updated {
                let elapsed = last_updated.elapsed().as_secs();
                // 如果活动数量未变化且缓存未过期，直接返回缓存
                if cache.last_activity_count == current_count && elapsed < CACHE_TTL_SECS {
                    tracing::debug!(
                        "使用图谱缓存 (活动数: {}, 缓存时间: {}s)",
                        current_count,
                        elapsed
                    );
                    return Ok(cached_data.clone());
                }
            }
        }
    }
    
    tracing::info!("重建知识图谱 (活动数: {})", current_count);
    
    // 1. 提取实体（应用、文档、时间段）
    let activities = db::get_activities(1000).await?;
    let records: Vec<ActivityRecord> = activities
        .into_iter()
        .map(|a| ActivityRecord {
            timestamp: a.timestamp,
            app_name: a.app_name,
            ocr_text: a.ocr_text,
        })
        .collect();

    let graph = build_graph_from_records(&records);
    
    // 更新缓存
    {
        let mut cache = GRAPH_CACHE.write().unwrap();
        cache.data = Some(graph.clone());
        cache.last_activity_count = current_count;
        cache.last_updated = Some(std::time::Instant::now());
    }
    
    Ok(graph)
}

/// 强制重建图谱（忽略缓存）
pub async fn rebuild_graph_force() -> Result<GraphData> {
    tracing::info!("强制重建知识图谱");
    
    // 清除缓存
    {
        let mut cache = GRAPH_CACHE.write().unwrap();
        cache.data = None;
        cache.last_updated = None;
    }
    
    // 重新构建
    build_graph().await
}

fn build_graph_from_records(records: &[ActivityRecord]) -> GraphData {
    let mut app_nodes: HashMap<String, i32> = HashMap::new();
    let mut doc_nodes: HashMap<String, i32> = HashMap::new();
    let mut time_nodes: HashMap<String, i32> = HashMap::new();
    let mut edges: Vec<(String, String, String)> = Vec::new();

    for activity in records {
        // 应用节点
        let app_id = format!("app:{}", activity.app_name);
        *app_nodes.entry(app_id.clone()).or_insert(0) += 1;

        // 时间段节点（按小时）
        let timestamp: DateTime<Utc> =
            DateTime::from_timestamp(activity.timestamp, 0).unwrap_or_else(|| Utc::now());
        let hour_key = format!("{:02}:00", timestamp.hour());
        let time_id = format!("time:{}", hour_key);
        *time_nodes.entry(time_id.clone()).or_insert(0) += 1;

        // 创建边：应用 -> 时间段
        edges.push((app_id.clone(), time_id.clone(), "occurs_at".to_string()));

        // 如果有 OCR 文本，使用 NLP 引擎提取关键词作为文档节点
        if let Some(ref ocr_text) = activity.ocr_text {
            if ocr_text.len() > 10 {
                // 使用 NLP 引擎提取关键词
                let keywords = extract_keywords_for_graph(ocr_text);
                for keyword in keywords {
                    let doc_id = format!("doc:{}", keyword);
                    *doc_nodes.entry(doc_id.clone()).or_insert(0) += 1;
                    edges.push((app_id.clone(), doc_id.clone(), "contains".to_string()));
                }
            }
        }
    }

    // 2. 构建节点列表
    let mut nodes = Vec::new();

    for (id, count) in app_nodes {
        nodes.push(Node {
            id: id.clone(),
            name: id.strip_prefix("app:").unwrap_or(&id).to_string(),
            group: "app".to_string(),
            size: count,
        });
    }

    for (id, count) in time_nodes {
        nodes.push(Node {
            id: id.clone(),
            name: id.strip_prefix("time:").unwrap_or(&id).to_string(),
            group: "time".to_string(),
            size: count,
        });
    }

    for (id, count) in doc_nodes {
        nodes.push(Node {
            id: id.clone(),
            name: id.strip_prefix("doc:").unwrap_or(&id).to_string(),
            group: "doc".to_string(),
            size: count,
        });
    }

    // 3. 构建边列表（去重并计数）
    let mut edge_map: HashMap<(String, String), i32> = HashMap::new();
    for (source, target, _) in edges {
        *edge_map.entry((source, target)).or_insert(0) += 1;
    }

    let graph_edges: Vec<Edge> = edge_map
        .into_iter()
        .map(|((source, target), value)| Edge {
            source,
            target,
            value,
        })
        .collect();

    GraphData {
        nodes,
        edges: graph_edges,
    }
}

/// 使用 NLP 引擎提取关键词
fn extract_keywords_for_graph(text: &str) -> Vec<String> {
    // 优先使用 TF-IDF 提取高质量关键词
    let tfidf_keywords = extract_keywords_tfidf(text, 3);
    
    if !tfidf_keywords.is_empty() {
        return tfidf_keywords;
    }
    
    // 回退到普通分词提取
    let options = KeywordOptions {
        max_keywords: 5,
        min_word_len: 2,
        filter_stopwords: true,
        filter_numbers: true,
    };
    
    extract_keywords(text, Some(options))
}

/// 保存图谱数据到数据库
pub async fn save_graph(graph: &GraphData) -> Result<()> {
    let pool = db::get_pool().await?;

    // 清空旧数据
    sqlx::query("DELETE FROM knowledge_nodes")
        .execute(&pool)
        .await?;
    sqlx::query("DELETE FROM knowledge_edges")
        .execute(&pool)
        .await?;

    // 插入节点
    for node in &graph.nodes {
        sqlx::query(
            "INSERT OR REPLACE INTO knowledge_nodes (id, name, node_group) VALUES (?, ?, ?)",
        )
        .bind(&node.id)
        .bind(&node.name)
        .bind(&node.group)
        .execute(&pool)
        .await?;
    }

    // 插入边
    for edge in &graph.edges {
        sqlx::query("INSERT INTO knowledge_edges (source, target, value) VALUES (?, ?, ?)")
            .bind(&edge.source)
            .bind(&edge.target)
            .bind(edge.value)
            .execute(&pool)
            .await?;
    }

    Ok(())
}

/// 从数据库加载图谱数据
pub async fn load_graph() -> Result<GraphData> {
    let pool = db::get_pool().await?;

    // 加载节点
    let node_rows = sqlx::query("SELECT id, name, node_group FROM knowledge_nodes")
        .fetch_all(&pool)
        .await?;

    let mut nodes = Vec::new();
    for row in node_rows {
        let id: String = row.get(0);
        let name: String = row.get(1);
        let group: String = row.get(2);

        // 计算节点大小（基于边的数量）
        let size: i64 = sqlx::query_scalar::<_, i64>(
            "SELECT COUNT(*) FROM knowledge_edges WHERE source = ? OR target = ?",
        )
        .bind(&id)
        .bind(&id)
        .fetch_one(&pool)
        .await?;

        nodes.push(Node {
            id,
            name,
            group,
            size: size as i32,
        });
    }

    // 加载边
    let edge_rows = sqlx::query("SELECT source, target, value FROM knowledge_edges")
        .fetch_all(&pool)
        .await?;

    let edges: Vec<Edge> = edge_rows
        .into_iter()
        .map(|row| Edge {
            source: row.get(0),
            target: row.get(1),
            value: row.get(2),
        })
        .collect();

    Ok(GraphData { nodes, edges })
}

#[cfg(test)]
mod tests {
    use super::{build_graph_from_records, ActivityRecord};
    use std::collections::HashSet;

    #[test]
    fn build_graph_creates_nodes_for_all_edge_endpoints() {
        let records = vec![
            ActivityRecord {
                timestamp: 1_700_000_000,
                app_name: "TestApp".to_string(),
                ocr_text: Some("hello world lorem ipsum".to_string()),
            },
            ActivityRecord {
                timestamp: 1_700_000_360,
                app_name: "TestApp".to_string(),
                ocr_text: Some("another document".to_string()),
            },
        ];

        let graph = build_graph_from_records(&records);
        let node_ids: HashSet<&str> = graph.nodes.iter().map(|n| n.id.as_str()).collect();
        for edge in &graph.edges {
            assert!(node_ids.contains(edge.source.as_str()));
            assert!(node_ids.contains(edge.target.as_str()));
        }
    }
}
