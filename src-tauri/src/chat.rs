//! 对话历史与反馈系统
//!
//! 功能：
//! - 对话会话管理（创建、查询、删除）
//! - 消息持久化
//! - 消息评价
//! - 用户反馈

use crate::db::get_pool;
use anyhow::Result;
use serde::{Deserialize, Serialize};
use sqlx::Row;

// ============================================
// 数据结构定义
// ============================================

#[derive(Debug, Serialize, Deserialize, Clone)]
#[serde(rename_all = "camelCase")]
pub struct ChatSession {
    pub id: i64,
    pub title: String,
    pub created_at: i64,
    pub updated_at: i64,
    pub message_count: i64,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
#[serde(rename_all = "camelCase")]
pub struct ChatMessage {
    pub id: i64,
    pub session_id: i64,
    pub role: String,
    pub content: String,
    pub context_ids: Option<Vec<i64>>,
    pub created_at: i64,
    pub rating: Option<i32>,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
#[serde(rename_all = "camelCase")]
pub struct UserFeedback {
    pub id: i64,
    pub category: String,
    pub title: String,
    pub content: String,
    pub screenshot_path: Option<String>,
    pub context_session_id: Option<i64>,
    pub status: String,
    pub created_at: i64,
}

// ============================================
// 对话会话操作
// ============================================

/// 创建新的对话会话
pub async fn create_session(title: &str) -> Result<i64> {
    let pool = get_pool().await?;
    let now = chrono::Utc::now().timestamp_millis();

    let id =
        sqlx::query("INSERT INTO chat_sessions (title, created_at, updated_at) VALUES (?, ?, ?)")
            .bind(title)
            .bind(now)
            .bind(now)
            .execute(&pool)
            .await?
            .last_insert_rowid();

    Ok(id)
}

/// 更新会话标题
pub async fn update_session_title(session_id: i64, title: &str) -> Result<()> {
    let pool = get_pool().await?;
    let now = chrono::Utc::now().timestamp_millis();

    sqlx::query("UPDATE chat_sessions SET title = ?, updated_at = ? WHERE id = ?")
        .bind(title)
        .bind(now)
        .bind(session_id)
        .execute(&pool)
        .await?;

    Ok(())
}

/// 更新会话的 updated_at 时间戳
async fn touch_session(session_id: i64) -> Result<()> {
    let pool = get_pool().await?;
    let now = chrono::Utc::now().timestamp_millis();

    sqlx::query("UPDATE chat_sessions SET updated_at = ? WHERE id = ?")
        .bind(now)
        .bind(session_id)
        .execute(&pool)
        .await?;

    Ok(())
}

/// 获取对话会话列表
pub async fn get_sessions(
    limit: Option<i64>,
    offset: Option<i64>,
    search: Option<&str>,
) -> Result<Vec<ChatSession>> {
    let pool = get_pool().await?;
    let limit = limit.unwrap_or(50);
    let offset = offset.unwrap_or(0);

    let rows = if let Some(keyword) = search {
        if keyword.trim().is_empty() {
            sqlx::query(
                r#"
                SELECT s.id, s.title, s.created_at, s.updated_at,
                       (SELECT COUNT(*) FROM chat_messages WHERE session_id = s.id) as message_count
                FROM chat_sessions s
                ORDER BY s.updated_at DESC
                LIMIT ? OFFSET ?
                "#,
            )
            .bind(limit)
            .bind(offset)
            .fetch_all(&pool)
            .await?
        } else {
            // 使用 FTS5 搜索消息内容，并关联到会话
            sqlx::query(
                r#"
                SELECT DISTINCT s.id, s.title, s.created_at, s.updated_at,
                       (SELECT COUNT(*) FROM chat_messages WHERE session_id = s.id) as message_count
                FROM chat_sessions s
                LEFT JOIN chat_messages m ON s.id = m.session_id
                LEFT JOIN chat_messages_fts fts ON m.id = fts.rowid
                WHERE s.title LIKE ? OR fts.content MATCH ?
                ORDER BY s.updated_at DESC
                LIMIT ? OFFSET ?
                "#,
            )
            .bind(format!("%{}%", keyword))
            .bind(keyword)
            .bind(limit)
            .bind(offset)
            .fetch_all(&pool)
            .await?
        }
    } else {
        sqlx::query(
            r#"
            SELECT s.id, s.title, s.created_at, s.updated_at,
                   (SELECT COUNT(*) FROM chat_messages WHERE session_id = s.id) as message_count
            FROM chat_sessions s
            ORDER BY s.updated_at DESC
            LIMIT ? OFFSET ?
            "#,
        )
        .bind(limit)
        .bind(offset)
        .fetch_all(&pool)
        .await?
    };

    let sessions = rows
        .into_iter()
        .map(|row| ChatSession {
            id: row.get(0),
            title: row.get(1),
            created_at: row.get(2),
            updated_at: row.get(3),
            message_count: row.get(4),
        })
        .collect();

    Ok(sessions)
}

/// 获取会话总数
pub async fn get_session_count() -> Result<i64> {
    let pool = get_pool().await?;
    let count: i64 = sqlx::query_scalar("SELECT COUNT(*) FROM chat_sessions")
        .fetch_one(&pool)
        .await?;
    Ok(count)
}

/// 删除指定会话（级联删除消息和评价）
pub async fn delete_session(session_id: i64) -> Result<()> {
    let pool = get_pool().await?;

    sqlx::query("DELETE FROM chat_sessions WHERE id = ?")
        .bind(session_id)
        .execute(&pool)
        .await?;

    Ok(())
}

/// 清空所有对话历史
pub async fn clear_all_history() -> Result<()> {
    let pool = get_pool().await?;

    // 由于设置了级联删除，删除 sessions 会自动删除相关 messages 和 ratings
    sqlx::query("DELETE FROM chat_sessions")
        .execute(&pool)
        .await?;

    Ok(())
}

// ============================================
// 消息操作
// ============================================

/// 保存聊天消息
pub async fn save_message(
    session_id: i64,
    role: &str,
    content: &str,
    context_ids: Option<Vec<i64>>,
) -> Result<i64> {
    let pool = get_pool().await?;
    let now = chrono::Utc::now().timestamp_millis();

    // 将 context_ids 序列化为 JSON
    let context_ids_json = context_ids.map(|ids| serde_json::to_string(&ids).unwrap_or_default());

    let id = sqlx::query(
        "INSERT INTO chat_messages (session_id, role, content, context_ids, created_at) VALUES (?, ?, ?, ?, ?)"
    )
    .bind(session_id)
    .bind(role)
    .bind(content)
    .bind(context_ids_json)
    .bind(now)
    .execute(&pool)
    .await?
    .last_insert_rowid();

    // 更新会话的 updated_at
    touch_session(session_id).await?;

    Ok(id)
}

/// 获取指定会话的消息列表
pub async fn get_messages(session_id: i64) -> Result<Vec<ChatMessage>> {
    let pool = get_pool().await?;

    let rows = sqlx::query(
        r#"
        SELECT m.id, m.session_id, m.role, m.content, m.context_ids, m.created_at,
               r.rating
        FROM chat_messages m
        LEFT JOIN message_ratings r ON m.id = r.message_id
        WHERE m.session_id = ?
        ORDER BY m.created_at ASC
        "#,
    )
    .bind(session_id)
    .fetch_all(&pool)
    .await?;

    let messages = rows
        .into_iter()
        .map(|row| {
            let context_ids_str: Option<String> = row.get(4);
            let context_ids = context_ids_str.and_then(|s| serde_json::from_str(&s).ok());

            ChatMessage {
                id: row.get(0),
                session_id: row.get(1),
                role: row.get(2),
                content: row.get(3),
                context_ids,
                created_at: row.get(5),
                rating: row.get(6),
            }
        })
        .collect();

    Ok(messages)
}

// ============================================
// 消息评价操作
// ============================================

/// 对消息进行评价
pub async fn rate_message(message_id: i64, rating: i32, comment: Option<&str>) -> Result<()> {
    let pool = get_pool().await?;
    let now = chrono::Utc::now().timestamp_millis();

    // 使用 UPSERT：如果已评价则更新，否则插入
    sqlx::query(
        r#"
        INSERT INTO message_ratings (message_id, rating, comment, created_at)
        VALUES (?, ?, ?, ?)
        ON CONFLICT(message_id) DO UPDATE SET
            rating = excluded.rating,
            comment = excluded.comment,
            created_at = excluded.created_at
        "#,
    )
    .bind(message_id)
    .bind(rating)
    .bind(comment)
    .bind(now)
    .execute(&pool)
    .await?;

    Ok(())
}

/// 删除消息评价
pub async fn delete_rating(message_id: i64) -> Result<()> {
    let pool = get_pool().await?;

    sqlx::query("DELETE FROM message_ratings WHERE message_id = ?")
        .bind(message_id)
        .execute(&pool)
        .await?;

    Ok(())
}

// ============================================
// 用户反馈操作
// ============================================

/// 提交用户反馈
pub async fn submit_feedback(
    category: &str,
    title: &str,
    content: &str,
    screenshot_path: Option<&str>,
    context_session_id: Option<i64>,
) -> Result<i64> {
    let pool = get_pool().await?;
    let now = chrono::Utc::now().timestamp_millis();

    let id = sqlx::query(
        r#"
        INSERT INTO user_feedbacks (category, title, content, screenshot_path, context_session_id, status, created_at)
        VALUES (?, ?, ?, ?, ?, 'pending', ?)
        "#
    )
    .bind(category)
    .bind(title)
    .bind(content)
    .bind(screenshot_path)
    .bind(context_session_id)
    .bind(now)
    .execute(&pool)
    .await?
    .last_insert_rowid();

    Ok(id)
}

/// 获取用户反馈列表
pub async fn get_feedbacks(limit: Option<i64>) -> Result<Vec<UserFeedback>> {
    let pool = get_pool().await?;
    let limit = limit.unwrap_or(50);

    let rows = sqlx::query(
        r#"
        SELECT id, category, title, content, screenshot_path, context_session_id, status, created_at
        FROM user_feedbacks
        ORDER BY created_at DESC
        LIMIT ?
        "#,
    )
    .bind(limit)
    .fetch_all(&pool)
    .await?;

    let feedbacks = rows
        .into_iter()
        .map(|row| UserFeedback {
            id: row.get(0),
            category: row.get(1),
            title: row.get(2),
            content: row.get(3),
            screenshot_path: row.get(4),
            context_session_id: row.get(5),
            status: row.get(6),
            created_at: row.get(7),
        })
        .collect();

    Ok(feedbacks)
}

/// 获取反馈总数
pub async fn get_feedback_count() -> Result<i64> {
    let pool = get_pool().await?;
    let count: i64 = sqlx::query_scalar("SELECT COUNT(*) FROM user_feedbacks")
        .fetch_one(&pool)
        .await?;
    Ok(count)
}

/// 删除反馈
pub async fn delete_feedback(feedback_id: i64) -> Result<()> {
    let pool = get_pool().await?;

    sqlx::query("DELETE FROM user_feedbacks WHERE id = ?")
        .bind(feedback_id)
        .execute(&pool)
        .await?;

    Ok(())
}










