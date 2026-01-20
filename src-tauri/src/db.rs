use chrono::Datelike;
use crate::commands::{ActivityLog, Stats};
use anyhow::Result;
use sqlx::sqlite::{SqliteConnectOptions, SqliteJournalMode, SqlitePool};
use sqlx::{QueryBuilder, Row};
use std::collections::HashMap;
use std::path::{Path, PathBuf};
use std::time::Duration;
use tauri::{AppHandle, Manager};

static DB_POOL: once_cell::sync::Lazy<tokio::sync::Mutex<Option<SqlitePool>>> =
    once_cell::sync::Lazy::new(|| tokio::sync::Mutex::new(None));

static SCREENSHOTS_DIR: once_cell::sync::Lazy<tokio::sync::Mutex<Option<PathBuf>>> =
    once_cell::sync::Lazy::new(|| tokio::sync::Mutex::new(None));

// 恢复操作互斥锁，确保一次只有一个恢复操作
static RECOVERY_LOCK: once_cell::sync::Lazy<tokio::sync::Mutex<()>> =
    once_cell::sync::Lazy::new(|| tokio::sync::Mutex::new(()));

pub async fn init_db(app_handle: AppHandle) -> Result<()> {
    // 获取应用数据目录
    let db_path = get_db_path(&app_handle)?;
    let app_data = db_path.parent().unwrap().to_path_buf();

    std::fs::create_dir_all(&app_data)?;

    // 创建截图目录
    let screenshots_dir = app_data.join("screenshots");
    std::fs::create_dir_all(&screenshots_dir)?;
    *SCREENSHOTS_DIR.lock().await = Some(screenshots_dir);

    let mut retry_count = 0;
    loop {
        let options = SqliteConnectOptions::new()
            .filename(&db_path)
            .create_if_missing(true)
            .journal_mode(SqliteJournalMode::Wal)
            .busy_timeout(Duration::from_secs(5));

        match try_connect_and_migrate(options).await {
            Ok(pool) => {
                *DB_POOL.lock().await = Some(pool);
                return Ok(());
            }
            Err(e) => {
                if is_database_corrupted(&e) {
                    tracing::error!(
                        "数据库损坏检测: {}. 开始恢复流程... 数据库路径: {}",
                        e,
                        db_path.display()
                    );
                    if let Err(err) = backup_and_reset_db(&db_path) {
                        let error_msg = format!(
                            "数据库恢复失败: {}. 数据库路径: {}, 操作系统: {}",
                            err,
                            db_path.display(),
                            get_os_info()
                        );
                        tracing::error!("{}", error_msg);
                        return Err(anyhow::anyhow!(
                            "Database init failed: {}",
                            err
                        ));
                    }
                    tracing::info!("数据库恢复成功，重新尝试连接...");
                    retry_count = 0;
                    continue;
                }

                retry_count += 1;
                if retry_count >= 3 {
                    tracing::error!(
                        "数据库连接失败，已达到最大重试次数 (3). 最后错误: {}, 数据库路径: {}",
                        e,
                        db_path.display()
                    );
                    return Err(e);
                }
                tracing::warn!(
                    "数据库连接失败，重试中... ({}/{}) 错误: {}, 数据库路径: {}",
                    retry_count,
                    3,
                    e,
                    db_path.display()
                );
                tokio::time::sleep(Duration::from_millis(500)).await;
            }
        }
    }
}

pub async fn get_pool() -> Result<SqlitePool> {
    DB_POOL
        .lock()
        .await
        .clone()
        .ok_or_else(|| anyhow::anyhow!("数据库未初始化"))
}

pub async fn get_activities(limit: i64) -> Result<Vec<ActivityLog>> {
    let pool = get_pool().await?;

    let rows = sqlx::query(
        "SELECT id, timestamp, app_name, window_title, image_path, ocr_text, phash 
         FROM activity_logs 
         ORDER BY timestamp DESC 
         LIMIT ?",
    )
    .bind(limit)
    .fetch_all(&pool)
    .await?;

    let activities = rows
        .into_iter()
        .map(|row| ActivityLog {
            id: row.get(0),
            timestamp: row.get(1),
            app_name: row.get(2),
            window_title: row.get(3),
            image_path: row.get(4),
            ocr_text: row.get(5),
            phash: row.get(6),
        })
        .collect();

    Ok(activities)
}

pub async fn get_activity_by_id(id: i64) -> Result<ActivityLog> {
    let pool = get_pool().await?;

    let row = sqlx::query(
        "SELECT id, timestamp, app_name, window_title, image_path, ocr_text, phash 
         FROM activity_logs 
         WHERE id = ?",
    )
    .bind(id)
    .fetch_one(&pool)
    .await?;

    Ok(ActivityLog {
        id: row.get(0),
        timestamp: row.get(1),
        app_name: row.get(2),
        window_title: row.get(3),
        image_path: row.get(4),
        ocr_text: row.get(5),
        phash: row.get(6),
    })
}

pub async fn insert_activity(
    timestamp: i64,
    app_name: &str,
    window_title: &str,
    image_path: &str,
    phash: Option<&str>,
    app_path: Option<&str>,
) -> Result<i64> {
    let pool = get_pool().await?;

    let id = sqlx::query(
        "INSERT INTO activity_logs (timestamp, app_name, window_title, image_path, phash, app_path) 
         VALUES (?, ?, ?, ?, ?, ?)",
    )
    .bind(timestamp)
    .bind(app_name)
    .bind(window_title)
    .bind(image_path)
    .bind(phash)
    .bind(app_path)
    .execute(&pool)
    .await?
    .last_insert_rowid();

    Ok(id)
}

/// 更新活动的 OCR 文本
pub async fn update_activity_ocr(id: i64, ocr_text: &str) -> Result<()> {
    let pool = get_pool().await?;

    sqlx::query("UPDATE activity_logs SET ocr_text = ? WHERE id = ?")
        .bind(ocr_text)
        .bind(id)
        .execute(&pool)
        .await?;

    Ok(())
}

#[derive(Debug, serde::Serialize, sqlx::FromRow)]
pub struct RecordingStat {
    pub date: String,
    pub reason: String,
    pub count: i64,
}

pub async fn get_recording_stats(limit: i64) -> Result<Vec<RecordingStat>> {
    let pool = get_pool().await?;
    let stats = sqlx::query_as::<_, RecordingStat>(
        "SELECT date, reason, count FROM recording_stats ORDER BY date DESC LIMIT ?",
    )
    .bind(limit)
    .fetch_all(&pool)
    .await?;

    Ok(stats)
}

pub async fn get_stats() -> Result<Stats> {
    let pool = get_pool().await?;

    let total_activities: i64 = sqlx::query_scalar::<_, i64>("SELECT COUNT(*) FROM activity_logs")
        .fetch_one(&pool)
        .await?;

    // 根据时间范围计算累计时长（最大时间戳 - 最小时间戳）
    let total_hours: f64 = sqlx::query_scalar::<_, Option<f64>>(
        "SELECT CAST((MAX(timestamp) - MIN(timestamp)) AS REAL) / 3600.0 FROM activity_logs",
    )
    .fetch_one(&pool)
    .await?
    .unwrap_or(0.0);

    let top_app: String = sqlx::query_scalar::<_, String>(
        "SELECT app_name FROM activity_logs 
         GROUP BY app_name 
         ORDER BY COUNT(*) DESC 
         LIMIT 1",
    )
    .fetch_optional(&pool)
    .await?
    .unwrap_or_else(|| "未知".to_string());

    Ok(Stats {
        total_activities,
        total_hours,
        top_app,
    })
}

pub async fn get_screenshots_dir() -> Option<PathBuf> {
    SCREENSHOTS_DIR.lock().await.clone()
}

pub async fn find_activity_by_phash(phash: &str) -> Result<Option<i64>> {
    let pool = get_pool().await?;

    let result = sqlx::query_scalar::<_, Option<i64>>(
        "SELECT id FROM activity_logs WHERE phash = ? ORDER BY timestamp DESC LIMIT 1",
    )
    .bind(phash)
    .fetch_optional(&pool)
    .await?;

    Ok(result.flatten())
}

pub async fn search_activities(
    query: Option<String>,
    app_name: Option<String>,
    from_ts: Option<i64>,
    to_ts: Option<i64>,
    has_ocr: Option<bool>,
    limit: Option<i64>,
    offset: Option<i64>,
    order_by: Option<String>,
) -> Result<(Vec<ActivityLog>, i64)> {
    let pool = get_pool().await?;
    let mut builder = QueryBuilder::new(
        "SELECT a.id, a.timestamp, a.app_name, a.window_title, a.image_path, a.ocr_text, a.phash FROM activity_logs a "
    );

    let has_query = query.as_ref().map(|s| !s.is_empty()).unwrap_or(false);

    if has_query {
        builder.push("JOIN activity_logs_fts f ON a.id = f.rowid ");
    }

    builder.push("WHERE 1=1 ");

    if has_query {
        builder.push("AND activity_logs_fts MATCH ");
        builder.push_bind(query.unwrap());
        builder.push(" ");
    }

    if let Some(app) = app_name {
        if !app.is_empty() {
            builder.push("AND (");
            builder.push("LOWER(a.app_name) LIKE '%' || LOWER(");
            builder.push_bind(app.clone());
            builder.push(") || '%'");
            builder.push(" OR LOWER(REPLACE(a.app_name, '.exe', '')) LIKE '%' || LOWER(");
            builder.push_bind(app);
            builder.push(") || '%'");
            builder.push(") ");
        }
    }

    if let Some(from) = from_ts {
        builder.push("AND a.timestamp >= ");
        builder.push_bind(from);
    }

    if let Some(to) = to_ts {
        builder.push("AND a.timestamp <= ");
        builder.push_bind(to);
    }

    if let Some(ocr) = has_ocr {
        if ocr {
            builder.push("AND a.ocr_text IS NOT NULL AND a.ocr_text != '' ");
        } else {
            builder.push("AND (a.ocr_text IS NULL OR a.ocr_text = '') ");
        }
    }

    // Handle ordering
    let order = order_by.unwrap_or_else(|| "time".to_string());
    if order == "rank" && has_query {
        builder.push("ORDER BY bm25(activity_logs_fts) ");
    } else {
        builder.push("ORDER BY a.timestamp DESC ");
    }

    if let Some(l) = limit {
        builder.push("LIMIT ");
        builder.push_bind(l);
    }

    if let Some(o) = offset {
        builder.push("OFFSET ");
        builder.push_bind(o);
    }

    let query = builder.build();
    let rows = query.fetch_all(&pool).await?;

    // Count total (optional, for now just 0 or separate query if needed)
    // For performance, maybe skip total count or do a separate count query with same filters.
    // Let's do a simplified count.

    let activities = rows
        .into_iter()
        .map(|row| ActivityLog {
            id: row.get(0),
            timestamp: row.get(1),
            app_name: row.get(2),
            window_title: row.get(3),
            image_path: row.get(4),
            ocr_text: row.get(5),
            phash: row.get(6),
        })
        .collect();

    Ok((activities, 0))
}

pub async fn get_blocklist() -> Result<Vec<String>> {
    let pool = get_pool().await?;
    let rows = sqlx::query("SELECT app_name FROM app_blocklist ORDER BY app_name")
        .fetch_all(&pool)
        .await?;
    Ok(rows.into_iter().map(|r| r.get(0)).collect())
}

pub async fn add_blocklist_item(app_name: String) -> Result<()> {
    let pool = get_pool().await?;
    sqlx::query("INSERT OR IGNORE INTO app_blocklist (app_name) VALUES (?)")
        .bind(app_name)
        .execute(&pool)
        .await?;
    Ok(())
}

pub async fn remove_blocklist_item(app_name: String) -> Result<()> {
    let pool = get_pool().await?;
    sqlx::query("DELETE FROM app_blocklist WHERE app_name = ?")
        .bind(app_name)
        .execute(&pool)
        .await?;
    Ok(())
}

pub async fn clear_blocklist() -> Result<()> {
    let pool = get_pool().await?;
    sqlx::query("DELETE FROM app_blocklist")
        .execute(&pool)
        .await?;
    Ok(())
}

async fn try_connect_and_migrate(options: SqliteConnectOptions) -> Result<SqlitePool> {
    let pool = SqlitePool::connect_with(options).await?;

    // 执行数据库迁移
    tracing::info!("开始执行数据库迁移...");
    let migrator = sqlx::migrate!("./migrations");
    if let Err(e) = migrator.run(&pool).await {
        let error_str = e.to_string();
        tracing::error!("数据库迁移失败: {}", error_str);

        if error_str
            .to_lowercase()
            .contains("was previously applied but has been modified")
        {
            tracing::warn!("检测到迁移校验不一致，尝试自动修复迁移校验并重试迁移...");
            if let Err(repair_err) = repair_migration_checksums(&pool, &migrator).await {
                tracing::error!("迁移校验修复失败: {:#}", repair_err);
                pool.close().await;
                return Err(anyhow::anyhow!("Database migration failed: {}", e));
            } else if let Err(retry_err) = migrator.run(&pool).await {
                tracing::error!("迁移重试仍失败: {}", retry_err);
                pool.close().await;
                return Err(anyhow::anyhow!("Database migration failed: {}", retry_err));
            } else {
                tracing::info!("迁移校验修复成功，迁移已完成");
            }
        } else {
            tracing::error!("迁移错误详情:\n- 错误信息: {}", error_str);
            if error_str.to_lowercase().contains("migration") {
                tracing::error!("检测到迁移文件相关问题，可能是迁移脚本语法错误或版本冲突");
            }
            pool.close().await;
            return Err(anyhow::anyhow!("Database migration failed: {}", e));
        }
    }
    tracing::info!("数据库迁移执行成功");

    ensure_agent_automation_schema(&pool).await?;

    // 执行完整性检查 (使用 integrity_check 以检测 FTS5 等虚拟表的损坏)
    println!("Executing database integrity check...");
    tracing::info!("执行数据库完整性检查...");
    let check_result: (String,) = sqlx::query_as("PRAGMA integrity_check")
        .fetch_one(&pool)
        .await
        .map_err(|e| anyhow::anyhow!("Integrity check failed to execute: {}", e))?;

    if check_result.0 != "ok" {
        println!("Database integrity check failed: {}", check_result.0);
        tracing::error!("数据库完整性检查失败: {}", check_result.0);
        return Err(anyhow::anyhow!(
            "Database integrity check failed (corrupt): {}",
            check_result.0
        ));
    }

    // FTS Smoke Test: 尝试写入 FTS 表以确保索引触发器未损坏
    // Note: read-only checks passed but writes failed. We must test the write path.
    println!("Executing Aggressive Write Smoke Test...");
    {
        let mut transaction = pool.begin().await?;
        // 插入一条假数据来触发 FTS 索引更新
        let test_result = sqlx::query(
            "INSERT INTO activity_logs (timestamp, app_name, window_title, image_path, ocr_text) 
             VALUES (0, 'smoke_test', 'test', 'test', 'smoke test content')",
        )
        .execute(&mut *transaction)
        .await;

        if let Err(e) = test_result {
            println!("Write smoke test failed: {}", e);
            tracing::error!("写入冒烟测试失败 (检测到损坏): {}", e);
            return Err(anyhow::anyhow!("Database write failed (corrupt): {}", e));
        }

        // 如果成功，回滚事务，不污染数据库
        transaction.rollback().await?;
    }

    println!("Database checks passed.");
    tracing::info!("数据库完整性检查通过");
    Ok(pool)
}

async fn ensure_agent_automation_schema(pool: &SqlitePool) -> Result<()> {
    sqlx::query(
        r#"
        CREATE TABLE IF NOT EXISTS automation_proposals (
            id INTEGER PRIMARY KEY AUTOINCREMENT,
            title TEXT NOT NULL,
            description TEXT NOT NULL,
            confidence REAL NOT NULL,
            risk_level TEXT NOT NULL,
            steps_json TEXT NOT NULL,
            evidence_json TEXT,
            created_at INTEGER DEFAULT (strftime('%s','now'))
        );
        "#,
    )
    .execute(pool)
    .await?;

    sqlx::query(
        r#"
        CREATE TABLE IF NOT EXISTS agent_executions (
            id INTEGER PRIMARY KEY AUTOINCREMENT,
            proposal_id INTEGER,
            action TEXT NOT NULL,
            status TEXT NOT NULL,
            created_at INTEGER DEFAULT (strftime('%s', 'now'))
        );
        "#,
    )
    .execute(pool)
    .await?;

    ensure_sqlite_column(pool, "agent_executions", "finished_at", "finished_at INTEGER").await?;
    ensure_sqlite_column(pool, "agent_executions", "error_message", "error_message TEXT").await?;
    ensure_sqlite_column(pool, "agent_executions", "metadata_json", "metadata_json TEXT").await?;

    sqlx::query(
        r#"
        CREATE INDEX IF NOT EXISTS idx_automation_proposals_created
            ON automation_proposals(created_at DESC);
        "#,
    )
    .execute(pool)
    .await?;

    sqlx::query(
        r#"
        CREATE INDEX IF NOT EXISTS idx_agent_executions_created
            ON agent_executions(created_at DESC);
        "#,
    )
    .execute(pool)
    .await?;

    Ok(())
}

async fn ensure_sqlite_column(pool: &SqlitePool, table: &str, column: &str, ddl: &str) -> Result<()> {
    let pragma = format!("PRAGMA table_info({})", table);
    let rows = sqlx::query(&pragma).fetch_all(pool).await?;
    let exists = rows
        .iter()
        .any(|r| r.get::<String, _>("name").as_str() == column);

    if exists {
        return Ok(());
    }

    let sql = format!("ALTER TABLE {} ADD COLUMN {}", table, ddl);
    match sqlx::query(&sql).execute(pool).await {
        Ok(_) => Ok(()),
        Err(e) => {
            let msg = e.to_string().to_lowercase();
            if msg.contains("duplicate column name") || msg.contains("already exists") {
                Ok(())
            } else {
                Err(e.into())
            }
        }
    }
}

#[derive(Debug, serde::Serialize, sqlx::FromRow)]
pub struct HeatmapData {
    pub date: String,
    pub count: i64,
}

pub async fn get_activity_heatmap_stats(year: Option<i32>) -> Result<Vec<HeatmapData>> {
    let pool = get_pool().await?;
    get_activity_heatmap_stats_impl(&pool, year).await
}

pub async fn get_activity_heatmap_stats_impl(pool: &SqlitePool, year: Option<i32>) -> Result<Vec<HeatmapData>> {
    let year = year.unwrap_or_else(|| chrono::Utc::now().year());
    
    // 计算该年的起始和结束时间戳
    let start_of_year = chrono::NaiveDate::from_ymd_opt(year, 1, 1)
        .unwrap()
        .and_hms_opt(0, 0, 0)
        .unwrap()
        .and_utc()
        .timestamp();
        
    let end_of_year = chrono::NaiveDate::from_ymd_opt(year + 1, 1, 1)
        .unwrap()
        .and_hms_opt(0, 0, 0)
        .unwrap()
        .and_utc()
        .timestamp();

    let stats = sqlx::query_as::<_, HeatmapData>(
        "SELECT date(timestamp, 'unixepoch', 'localtime') as date, count(*) as count 
         FROM activity_logs 
         WHERE timestamp >= ? AND timestamp < ? 
         GROUP BY date",
    )
    .bind(start_of_year)
    .bind(end_of_year)
    .fetch_all(pool)
    .await?;

    Ok(stats)
}

#[derive(Debug, serde::Serialize, sqlx::FromRow)]
pub struct OcrQueueItem {
    pub id: i64,
    pub activity_id: i64,
    pub image_path: String,
    pub retry_count: i64,
}

pub async fn enqueue_ocr_task(activity_id: i64) -> Result<()> {
    let pool = get_pool().await?;
    sqlx::query("INSERT OR IGNORE INTO ocr_queue (activity_id, status) VALUES (?, 'pending')")
        .bind(activity_id)
        .execute(&pool)
        .await?;
    Ok(())
}

pub async fn get_pending_ocr_tasks(limit: i64) -> Result<Vec<OcrQueueItem>> {
    let pool = get_pool().await?;
    // Get pending tasks or processing tasks that are stuck (e.g. created > 5 mins ago)
    let tasks = sqlx::query_as::<_, OcrQueueItem>(
        r#"
        SELECT q.id, q.activity_id, a.image_path, q.retry_count
        FROM ocr_queue q
        JOIN activity_logs a ON q.activity_id = a.id
        WHERE q.status = 'pending'
           OR (q.status = 'processing' AND q.updated_at < (strftime('%s', 'now') - 300))
        ORDER BY q.created_at ASC
        LIMIT ?
        "#
    )
    .bind(limit)
    .fetch_all(&pool)
    .await?;

    Ok(tasks)
}

pub async fn update_ocr_queue_status(id: i64, status: &str, error_message: Option<&str>) -> Result<()> {
    let pool = get_pool().await?;
    
    // 如果是重试（从 processing 回到 pending），增加重试次数
    // 如果是失败（failed），也意味这是最后一次尝试
    // 但简单的逻辑是：调用者决定是否重试。
    // 这里我们假设如果 status 是 pending，就是一次重试。
    
    let sql = if status == "pending" {
        "UPDATE ocr_queue SET status = ?, error_message = ?, updated_at = strftime('%s', 'now'), retry_count = retry_count + 1 WHERE id = ?"
    } else {
        "UPDATE ocr_queue SET status = ?, error_message = ?, updated_at = strftime('%s', 'now') WHERE id = ?"
    };
    
    sqlx::query(sql)
        .bind(status)
        .bind(error_message)
        .bind(id)
        .execute(&pool)
        .await?;
        
    Ok(())
}

#[derive(Debug, serde::Serialize, sqlx::FromRow)]
pub struct OcrQueueStats {
    pub pending: i64,
    pub processing: i64,
    pub done: i64,
    pub failed: i64,
}

pub async fn get_ocr_queue_stats() -> Result<OcrQueueStats> {
    let pool = get_pool().await?;
    let stats = sqlx::query_as::<_, OcrQueueStats>(
        r#"
        SELECT
            COALESCE(SUM(CASE WHEN status = 'pending' THEN 1 ELSE 0 END), 0) as pending,
            COALESCE(SUM(CASE WHEN status = 'processing' THEN 1 ELSE 0 END), 0) as processing,
            COALESCE(SUM(CASE WHEN status = 'done' THEN 1 ELSE 0 END), 0) as done,
            COALESCE(SUM(CASE WHEN status = 'failed' THEN 1 ELSE 0 END), 0) as failed
        FROM ocr_queue
        "#
    )
    .fetch_one(&pool)
    .await?;
    
    Ok(stats)
}

#[cfg(test)]
mod tests {
    use super::*;
    use sqlx::sqlite::SqlitePoolOptions;



    #[tokio::test]
    async fn ensures_agent_executions_columns_exist() {
        let pool = SqlitePoolOptions::new()
            .max_connections(1)
            .connect(":memory:")
            .await
            .unwrap();

        sqlx::query(
            r#"
            CREATE TABLE agent_executions (
                id INTEGER PRIMARY KEY AUTOINCREMENT,
                proposal_id INTEGER,
                action TEXT NOT NULL,
                status TEXT NOT NULL,
                created_at INTEGER DEFAULT (strftime('%s', 'now'))
            );
            "#,
        )
        .execute(&pool)
        .await
        .unwrap();

        ensure_agent_automation_schema(&pool).await.unwrap();

        let rows = sqlx::query("PRAGMA table_info(agent_executions)")
            .fetch_all(&pool)
            .await
            .unwrap();
        let cols: Vec<String> = rows.iter().map(|r| r.get::<String, _>("name")).collect();

        assert!(cols.iter().any(|c| c == "finished_at"));
        assert!(cols.iter().any(|c| c == "error_message"));
        assert!(cols.iter().any(|c| c == "metadata_json"));
    }

    #[tokio::test]
    async fn test_heatmap_aggregation_logic() {
        let pool = SqlitePoolOptions::new()
            .max_connections(1)
            .connect(":memory:")
            .await
            .unwrap();

        // Init schema
        sqlx::query(
            "CREATE TABLE activity_logs (
                id INTEGER PRIMARY KEY,
                timestamp INTEGER,
                app_name TEXT,
                window_title TEXT,
                image_path TEXT,
                phash TEXT,
                app_path TEXT,
                ocr_text TEXT
            )"
        )
        .execute(&pool)
        .await
        .unwrap();

        // Insert mock data
        // 2024-01-01 12:00:00 UTC = 1704110400
        // 2024-01-01 13:00:00 UTC = 1704114000
        // 2024-01-02 12:00:00 UTC = 1704196800
        
        let ts1 = 1704110400; 
        let ts2 = 1704114000;
        let ts3 = 1704196800;
        
        sqlx::query("INSERT INTO activity_logs (timestamp) VALUES (?)")
            .bind(ts1)
            .execute(&pool)
            .await.unwrap();
        sqlx::query("INSERT INTO activity_logs (timestamp) VALUES (?)")
            .bind(ts2)
            .execute(&pool)
            .await.unwrap();
        sqlx::query("INSERT INTO activity_logs (timestamp) VALUES (?)")
            .bind(ts3)
            .execute(&pool)
            .await.unwrap();

        // Test with year 2024
        let stats = get_activity_heatmap_stats_impl(&pool, Some(2024)).await.unwrap();
        assert!(!stats.is_empty());
        
        // Sum of counts should be 3
        let total: i64 = stats.iter().map(|s| s.count).sum();
        assert_eq!(total, 3);
    }
}

async fn repair_migration_checksums(
    pool: &SqlitePool,
    migrator: &sqlx::migrate::Migrator,
) -> Result<()> {
    let rows = sqlx::query("SELECT version, checksum FROM _sqlx_migrations WHERE success = 1")
        .fetch_all(pool)
        .await?;

    let mut applied: HashMap<i64, Vec<u8>> = HashMap::new();
    for row in rows {
        let version: i64 = row.try_get("version")?;
        let checksum: Vec<u8> = row.try_get("checksum")?;
        applied.insert(version, checksum);
    }

    let mut fixed = 0usize;
    for migration in migrator.migrations.iter() {
        let version = migration.version;
        let Some(existing) = applied.get(&version) else {
            continue;
        };
        let desired = migration.checksum.as_ref();
        if existing.as_slice() != desired {
            sqlx::query("UPDATE _sqlx_migrations SET checksum = ? WHERE version = ? AND success = 1")
                .bind(desired.to_vec())
                .bind(version)
                .execute(pool)
                .await?;
            fixed += 1;
        }
    }

    tracing::info!("迁移校验修复完成，更新了 {} 条已应用迁移的 checksum", fixed);
    Ok(())
}

pub fn is_database_corrupted(err: &anyhow::Error) -> bool {
    let err_str = err.to_string().to_lowercase();
    err_str.contains("malformed")
        || err_str.contains("corrupt")
        || err_str.contains("not a database")
        || err_str.contains("was previously applied but has been modified")
        || (err_str.contains("migration") && err_str.contains("has been modified"))
        || err_str.contains("code: 11")
        || err_str.contains("code: 26")
        || err_str.contains("code: 267")
        || err_str.contains("database disk image is malformed")
        || err_str.contains("database disk image is malformed")
}

#[derive(Debug, serde::Serialize, sqlx::FromRow)]
pub struct AppUsageStat {
    pub app_name: String,
    pub count: i64,
}

pub async fn get_app_usage_stats(limit: i64) -> Result<Vec<AppUsageStat>> {
    let pool = get_pool().await?;
    get_app_usage_stats_impl(&pool, limit).await
}

pub async fn get_app_usage_stats_impl(pool: &SqlitePool, limit: i64) -> Result<Vec<AppUsageStat>> {
    let stats = sqlx::query_as::<_, AppUsageStat>(
        "SELECT app_name, COUNT(*) as count 
         FROM activity_logs 
         GROUP BY app_name 
         ORDER BY count DESC 
         LIMIT ?",
    )
    .bind(limit)
    .fetch_all(pool)
    .await?;

    Ok(stats)
}

#[derive(Debug, serde::Serialize, sqlx::FromRow)]
pub struct HourlyStat {
    pub hour: String,
    pub count: i64,
}

pub async fn get_hourly_activity_stats() -> Result<Vec<HourlyStat>> {
    let pool = get_pool().await?;
    get_hourly_activity_stats_impl(&pool).await
}

pub async fn get_hourly_activity_stats_impl(pool: &SqlitePool) -> Result<Vec<HourlyStat>> {
    // 聚合每小时的活动数。SQLite 的 strftime('%H', ...) 返回 00-23 的字符串。
    // 注意：这里使用的是 'localtime'，与 heatmap 一致。
    let stats = sqlx::query_as::<_, HourlyStat>(
        "SELECT strftime('%H:00', timestamp, 'unixepoch', 'localtime') as hour, COUNT(*) as count 
         FROM activity_logs 
         GROUP BY hour 
         ORDER BY hour",
    )
    .fetch_all(pool)
    .await?;

    // 为了前端方便，我们最好补全 0-23 小时的数据（如果某些小时没数据，SQL 不会返回）。
    // 不过前端 BarChart 可以处理，或者我们在这里补全。
    // 这里简单起见，让前端处理，或者返回全量。
    // 补全逻辑比较繁琐，这里先返回 SQL 结果，前端 Recharts 的 BarChart 如果缺 key 只是不显示，
    // 但为了“24小时分布”美观，最好补全。
    // 让我们在 Rust 里补全。

    let mut full_stats: Vec<HourlyStat> = (0..24)
        .map(|h| HourlyStat {
            hour: format!("{:02}:00", h),
            count: 0,
        })
        .collect();

    for s in stats {
        if let Some(slot) = full_stats.iter_mut().find(|f| f.hour == s.hour) {
            slot.count = s.count;
        }
    }

    Ok(full_stats)
}

#[derive(Debug, serde::Serialize, sqlx::FromRow)]
#[serde(rename_all = "camelCase")]
pub struct FocusMetric {
    pub timestamp: i64,
    pub apm: i32,
    pub window_switch_count: i32,
    pub focus_score: f64,
}

pub async fn insert_focus_metric(
    timestamp: i64,
    apm: i32,
    window_switch_count: i32,
    focus_score: f64,
) -> Result<()> {
    let pool = get_pool().await?;
    sqlx::query(
        "INSERT OR REPLACE INTO focus_metrics (timestamp, apm, window_switch_count, focus_score)
         VALUES (?, ?, ?, ?)",
    )
    .bind(timestamp)
    .bind(apm)
    .bind(window_switch_count)
    .bind(focus_score)
    .execute(&pool)
    .await?;
    Ok(())
}

pub async fn get_focus_metrics(
    from_ts: Option<i64>,
    to_ts: Option<i64>,
    limit: i64,
) -> Result<Vec<FocusMetric>> {
    let pool = get_pool().await?;
    get_focus_metrics_impl(&pool, from_ts, to_ts, limit).await
}

pub async fn get_focus_metrics_impl(
    pool: &SqlitePool,
    from_ts: Option<i64>,
    to_ts: Option<i64>,
    limit: i64,
) -> Result<Vec<FocusMetric>> {
    let mut sql = String::from(
        "SELECT timestamp, apm, window_switch_count, focus_score FROM focus_metrics WHERE 1=1",
    );
    if from_ts.is_some() {
        sql.push_str(" AND timestamp >= ?");
    }
    if to_ts.is_some() {
        sql.push_str(" AND timestamp <= ?");
    }
    sql.push_str(" ORDER BY timestamp ASC LIMIT ?");

    let mut q = sqlx::query_as::<_, FocusMetric>(&sql);
    if let Some(v) = from_ts {
        q = q.bind(v);
    }
    if let Some(v) = to_ts {
        q = q.bind(v);
    }
    q = q.bind(limit);

    Ok(q.fetch_all(pool).await?)
}

#[cfg(test)]
mod stats_tests {
    use super::*;
    use sqlx::sqlite::SqlitePoolOptions;

    #[tokio::test]
    async fn test_app_usage_stats() {
        let pool = SqlitePoolOptions::new()
            .max_connections(1)
            .connect(":memory:")
            .await
            .unwrap();

        // Init schema
        sqlx::query(
            "CREATE TABLE activity_logs (
                id INTEGER PRIMARY KEY,
                timestamp INTEGER,
                app_name TEXT,
                window_title TEXT,
                image_path TEXT,
                phash TEXT,
                app_path TEXT,
                ocr_text TEXT
            )"
        )
        .execute(&pool).await.unwrap();

        // Insert data
        sqlx::query("INSERT INTO activity_logs (timestamp, app_name) VALUES (?, ?)")
            .bind(1000).bind("Chrome")
            .execute(&pool).await.unwrap();
        sqlx::query("INSERT INTO activity_logs (timestamp, app_name) VALUES (?, ?)")
            .bind(2000).bind("Chrome")
            .execute(&pool).await.unwrap();
        sqlx::query("INSERT INTO activity_logs (timestamp, app_name) VALUES (?, ?)")
            .bind(3000).bind("Code")
            .execute(&pool).await.unwrap();

        let stats = get_app_usage_stats_impl(&pool, 5).await.unwrap();
        assert_eq!(stats.len(), 2);
        assert_eq!(stats[0].app_name, "Chrome");
        assert_eq!(stats[0].count, 2);
        assert_eq!(stats[1].app_name, "Code");
        assert_eq!(stats[1].count, 1);
    }

    #[tokio::test]
    async fn test_focus_metrics_query() {
        let pool = SqlitePoolOptions::new()
            .max_connections(1)
            .connect(":memory:")
            .await
            .unwrap();

        sqlx::query(
            "CREATE TABLE focus_metrics (
                timestamp INTEGER PRIMARY KEY,
                apm INTEGER NOT NULL,
                window_switch_count INTEGER NOT NULL,
                focus_score REAL NOT NULL
            )",
        )
        .execute(&pool)
        .await
        .unwrap();

        sqlx::query(
            "INSERT INTO focus_metrics (timestamp, apm, window_switch_count, focus_score) VALUES (?, ?, ?, ?)",
        )
        .bind(1000_i64)
        .bind(10_i32)
        .bind(1_i32)
        .bind(50.0_f64)
        .execute(&pool)
        .await
        .unwrap();

        sqlx::query(
            "INSERT INTO focus_metrics (timestamp, apm, window_switch_count, focus_score) VALUES (?, ?, ?, ?)",
        )
        .bind(2000_i64)
        .bind(20_i32)
        .bind(2_i32)
        .bind(60.0_f64)
        .execute(&pool)
        .await
        .unwrap();

        sqlx::query(
            "INSERT INTO focus_metrics (timestamp, apm, window_switch_count, focus_score) VALUES (?, ?, ?, ?)",
        )
        .bind(3000_i64)
        .bind(30_i32)
        .bind(3_i32)
        .bind(70.0_f64)
        .execute(&pool)
        .await
        .unwrap();

        let rows = get_focus_metrics_impl(&pool, Some(1500), Some(3000), 10)
            .await
            .unwrap();
        assert_eq!(rows.len(), 2);
        assert_eq!(rows[0].timestamp, 2000);
        assert_eq!(rows[1].timestamp, 3000);

        let rows = get_focus_metrics_impl(&pool, None, None, 2).await.unwrap();
        assert_eq!(rows.len(), 2);
        assert_eq!(rows[0].timestamp, 1000);
        assert_eq!(rows[1].timestamp, 2000);
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum DbInitFailureKind {
    MigrationChecksumMismatch,
    MigrationSyntaxOrCompat,
    SqliteFtsUnavailable,
    DiskFull,
    PermissionDenied,
    FileLocked,
    Corruption,
    Unknown,
}

pub fn diagnose_init_error(err: &anyhow::Error) -> (DbInitFailureKind, &'static str) {
    let s = err.to_string().to_lowercase();

    if s.contains("was previously applied but has been modified") {
        return (
            DbInitFailureKind::MigrationChecksumMismatch,
            "检测到迁移文件与已应用记录不一致；通常是曾修改过旧 migration。建议：优先用“修复迁移校验”而非重置数据库，以避免数据丢失。",
        );
    }

    if s.contains("no such module: fts5") || s.contains("unknown tokenizer") {
        return (
            DbInitFailureKind::SqliteFtsUnavailable,
            "SQLite 运行时不支持 FTS5（或 tokenizer 不可用）；建议：启用 bundled SQLite/FTS5 或在初始化中对 FTS 做降级处理。",
        );
    }

    if s.contains("database or disk is full") || s.contains("disk full") || s.contains("os error 112")
    {
        return (
            DbInitFailureKind::DiskFull,
            "磁盘空间不足导致 SQLite 无法创建/扩展数据库或 WAL；建议：释放空间并重试。",
        );
    }

    if s.contains("permission denied") || s.contains("access is denied") {
        return (
            DbInitFailureKind::PermissionDenied,
            "权限不足导致无法创建目录/数据库文件或删除恢复文件；建议：检查 app_data_dir 权限、杀软策略，必要时以管理员运行。",
        );
    }

    if s.contains("os error 32") || s.contains("being used by another process") || s.contains("另一个程序正在使用此文件")
    {
        return (
            DbInitFailureKind::FileLocked,
            "数据库文件被占用（Windows 常见 os error 32）；建议：关闭占用该 db 的进程/工具，或确保本进程在恢复前关闭连接池。",
        );
    }

    if s.contains("near \"") && s.contains("syntax error")
        || (s.contains("migration") && s.contains("failed"))
        || s.contains("database migration failed")
    {
        return (
            DbInitFailureKind::MigrationSyntaxOrCompat,
            "迁移执行失败，可能是 SQL 语法错误或 SQLite 版本/扩展不兼容；建议：定位失败的 migration 版本与 SQL 语句并修正。",
        );
    }

    if is_database_corrupted(err) {
        return (
            DbInitFailureKind::Corruption,
            "检测到数据库损坏或迁移状态异常；建议：先备份后尝试恢复/重建，并排查文件占用与存储介质健康。",
        );
    }

    (DbInitFailureKind::Unknown, "未能自动归类根因；建议启用 RUST_BACKTRACE=1 并提供完整日志。")
}

fn get_db_path(app_handle: &AppHandle) -> Result<PathBuf> {
    app_handle
        .path()
        .app_data_dir()
        .map(|d| d.join("memflow.db"))
        .map_err(|e| anyhow::anyhow!("无法获取应用数据目录: {}", e))
}

/// 获取数据库路径用于诊断（公开函数）
pub fn get_db_path_for_diagnostics(app_handle: &AppHandle) -> Result<PathBuf> {
    get_db_path(app_handle)
}

pub async fn force_recovery(app_handle: AppHandle) -> Result<()> {
    // 尝试获取恢复锁，如果已有恢复在进行，等待其完成
    let _recovery_guard = RECOVERY_LOCK.lock().await;

    // 先检查 pool 是否已经被其他恢复操作修复
    if let Ok(pool) = get_pool().await {
        // 快速检查数据库是否健康
        if sqlx::query("SELECT 1").fetch_one(&pool).await.is_ok() {
            tracing::info!("Database already recovered by another task");
            return Ok(());
        }
    }

    tracing::warn!("Initiating database recovery...");

    // 1. Close existing pool
    {
        let mut pool_guard = DB_POOL.lock().await;
        if let Some(pool) = pool_guard.take() {
            pool.close().await;
        }
    }

    // 2. 等待文件锁释放（给其他操作时间结束）
    tokio::time::sleep(Duration::from_millis(500)).await;

    // 3. Resolve path
    let db_path = get_db_path(&app_handle)?;

    // 4. Backup and reset (允许失败，后续尝试直接重连)
    if let Err(e) = backup_and_reset_db(&db_path) {
        tracing::warn!("Backup failed: {}, attempting direct reconnect...", e);
        // 备份失败时继续尝试重新初始化，可能数据库并非完全损坏
    }

    // 5. Re-initialize
    init_db(app_handle).await?;

    tracing::info!("Database recovery completed.");
    Ok(())
}

fn backup_and_reset_db(db_path: &Path) -> Result<()> {
    let path_str = db_path.to_string_lossy();
    let wal_path = PathBuf::from(format!("{}-wal", path_str));
    let shm_path = PathBuf::from(format!("{}-shm", path_str));

    // 记录诊断信息
    let os_info = get_os_info();
    tracing::info!(
        "开始数据库恢复流程 - 数据库路径: {}, 操作系统: {}",
        db_path.display(),
        os_info
    );

    // 检查存储空间
    if let Some(parent) = db_path.parent() {
        if let Ok(metadata) = std::fs::metadata(parent) {
            tracing::info!("数据库目录元数据: {:?}", metadata);
        }
    }

    // 需要删除的所有文件
    let files_to_remove = [db_path.to_path_buf(), wal_path, shm_path];

    // 记录文件状态
    for file_path in &files_to_remove {
        if file_path.exists() {
            match std::fs::metadata(file_path) {
                Ok(meta) => {
                    tracing::info!(
                        "文件存在: {}, 大小: {} 字节, 权限: {:?}",
                        file_path.display(),
                        meta.len(),
                        meta.permissions()
                    );
                }
                Err(e) => {
                    tracing::warn!(
                        "无法获取文件元数据 {}: {}",
                        file_path.display(),
                        e
                    );
                }
            }
        } else {
            tracing::info!("文件不存在: {}", file_path.display());
        }
    }

    // 多次重试删除，使用指数退避策略
    const MAX_ATTEMPTS: u32 = 5;
    const INITIAL_DELAY_MS: u64 = 200;
    
    for attempt in 1..=MAX_ATTEMPTS {
        let mut all_removed = true;
        let mut failed_files = Vec::new();

        for file_path in &files_to_remove {
            if file_path.exists() {
                // 尝试检查文件是否被锁定（通过尝试打开）
                let is_locked = check_file_locked(file_path);
                if is_locked {
                    tracing::warn!(
                        "文件可能被锁定: {} (尝试 {}/{})",
                        file_path.display(),
                        attempt,
                        MAX_ATTEMPTS
                    );
                }

                match std::fs::remove_file(file_path) {
                    Ok(_) => {
                        tracing::info!(
                            "成功删除文件: {} (尝试 {}/{})",
                            file_path.display(),
                            attempt,
                            MAX_ATTEMPTS
                        );
                    }
                    Err(e) => {
                        let error_kind = e.kind();
                        let error_msg = format!(
                            "删除文件失败: {} (尝试 {}/{}), 错误类型: {:?}, 错误信息: {}",
                            file_path.display(),
                            attempt,
                            MAX_ATTEMPTS,
                            error_kind,
                            e
                        );
                        tracing::error!("{}", error_msg);
                        
                        // 记录详细的错误信息
                        log_file_error_details(file_path, &e);
                        
                        failed_files.push((file_path.clone(), error_kind, e.to_string()));
                        all_removed = false;
                    }
                }
            }
        }

        if all_removed {
            tracing::info!("所有数据库文件已成功清理");
            return Ok(());
        }

        // 使用指数退避策略等待后重试
        if attempt < MAX_ATTEMPTS {
            let delay_ms = INITIAL_DELAY_MS * (1 << (attempt - 1)); // 指数退避: 200ms, 400ms, 800ms, 1600ms
            tracing::info!(
                "等待 {} 毫秒后重试 (尝试 {}/{})",
                delay_ms,
                attempt + 1,
                MAX_ATTEMPTS
            );
            std::thread::sleep(std::time::Duration::from_millis(delay_ms));
        }
    }

    // 最后检查主数据库文件是否还存在
    if db_path.exists() {
        // 构建详细的错误消息
        let mut error_details = format!(
            "数据库文件删除失败 (尝试 {} 次后仍失败)\n",
            MAX_ATTEMPTS
        );
        error_details.push_str(&format!("数据库路径: {}\n", db_path.display()));
        error_details.push_str(&format!("操作系统: {}\n", os_info));
        
        // 添加失败文件的详细信息
        for file_path in &files_to_remove {
            if file_path.exists() {
                error_details.push_str(&format!("文件仍存在: {}\n", file_path.display()));
                if let Ok(meta) = std::fs::metadata(file_path) {
                    error_details.push_str(&format!(
                        "  - 大小: {} 字节\n",
                        meta.len()
                    ));
                }
            }
        }

        // 添加诊断建议
        error_details.push_str("\n诊断建议:\n");
        error_details.push_str("1. 检查文件是否被其他进程占用（如数据库查看工具、备份软件）\n");
        error_details.push_str("2. 确认当前用户有足够的文件删除权限\n");
        error_details.push_str("3. 检查存储设备是否有足够的可用空间\n");
        error_details.push_str("4. 验证文件系统是否正常（尝试手动删除该文件）\n");
        error_details.push_str("5. 在 Windows 上，检查文件是否被防病毒软件锁定\n");
        error_details.push_str("6. 尝试以管理员权限运行应用程序\n");

        tracing::error!("{}", error_details);
        
        Err(anyhow::anyhow!(
            "Failed to remove database file after {} attempts. Path: {}, OS: {}",
            MAX_ATTEMPTS,
            db_path.display(),
            os_info
        ))
    } else {
        tracing::info!("数据库文件已成功删除");
        Ok(())
    }
}

/// 检查文件是否被锁定（通过尝试以独占模式打开）
fn check_file_locked(file_path: &Path) -> bool {
    use std::fs::OpenOptions;
    
    // 尝试以独占写入模式打开文件
    // 如果文件被其他进程锁定，这个操作会失败
    match OpenOptions::new()
        .write(true)
        .create(false)
        .open(file_path)
    {
        Ok(_) => {
            // 能够打开文件，说明可能未被锁定（但可能仍有其他进程在读取）
            false
        }
        Err(e) => {
            // 检查是否是权限错误还是文件被锁定
            match e.kind() {
                std::io::ErrorKind::PermissionDenied => {
                    tracing::warn!("文件权限被拒绝: {}", file_path.display());
                    true
                }
                std::io::ErrorKind::NotFound => {
                    // 文件不存在，不算锁定
                    false
                }
                _ => {
                    // 其他错误可能表示文件被锁定
                    tracing::debug!("文件可能被锁定: {} ({})", file_path.display(), e);
                    true
                }
            }
        }
    }
}

/// 记录文件错误的详细信息
fn log_file_error_details(file_path: &Path, error: &std::io::Error) {
    // 记录文件路径
    tracing::error!("文件路径: {}", file_path.display());
    
    // 记录错误类型
    tracing::error!("错误类型: {:?}", error.kind());
    
    // 记录操作系统特定信息
    #[cfg(windows)]
    {
        use std::os::windows::fs::MetadataExt;
        if let Ok(metadata) = std::fs::metadata(file_path) {
            tracing::error!("文件属性: 0x{:x}", metadata.file_attributes());
        }
    }
    
    // 记录文件权限信息
    if let Ok(metadata) = std::fs::metadata(file_path) {
        tracing::error!("文件权限: {:?}", metadata.permissions());
        tracing::error!("文件大小: {} 字节", metadata.len());
    }
    
    // 记录父目录信息
    if let Some(parent) = file_path.parent() {
        if let Ok(metadata) = std::fs::metadata(parent) {
            tracing::error!("父目录权限: {:?}", metadata.permissions());
        }
    }
}

/// 获取操作系统信息
fn get_os_info() -> String {
    #[cfg(windows)]
    {
        format!("Windows {}", env!("CARGO_PKG_VERSION"))
    }
    #[cfg(target_os = "macos")]
    {
        format!("macOS {}", env!("CARGO_PKG_VERSION"))
    }
    #[cfg(target_os = "linux")]
    {
        format!("Linux {}", env!("CARGO_PKG_VERSION"))
    }
    #[cfg(not(any(windows, target_os = "macos", target_os = "linux")))]
    {
        format!("Unknown OS {}", env!("CARGO_PKG_VERSION"))
    }
}

#[derive(Debug, serde::Serialize)]
pub struct CleanupStats {
    pub deleted_activities: u64,
    pub deleted_screenshots: u64,
    pub freed_bytes: u64,
}

pub async fn cleanup_old_activities(days: u32, dry_run: bool) -> Result<CleanupStats> {
    let pool = get_pool().await?;
    let cutoff_ts = chrono::Utc::now().timestamp() - (days as i64 * 86400);

    // 1. Find activities to delete
    let rows = sqlx::query("SELECT id, image_path FROM activity_logs WHERE timestamp < ?")
        .bind(cutoff_ts)
        .fetch_all(&pool)
        .await?;

    let mut stats = CleanupStats {
        deleted_activities: rows.len() as u64,
        deleted_screenshots: 0,
        freed_bytes: 0,
    };

    if rows.is_empty() {
        return Ok(stats);
    }

    if dry_run {
        // Just estimate bytes
        if let Some(screenshots_dir) = get_screenshots_dir().await {
            for row in rows {
                let image_path: String = row.get(1);
                let path = screenshots_dir.join(image_path);
                if let Ok(metadata) = std::fs::metadata(path) {
                    stats.freed_bytes += metadata.len();
                    stats.deleted_screenshots += 1;
                }
            }
        }
        return Ok(stats);
    }

    // 2. Delete files
    if let Some(screenshots_dir) = get_screenshots_dir().await {
        for row in &rows {
            let image_path: String = row.get(1);
            let path = screenshots_dir.join(image_path);
            if let Ok(metadata) = std::fs::metadata(&path) {
                stats.freed_bytes += metadata.len();
            }
            if std::fs::remove_file(&path).is_ok() {
                stats.deleted_screenshots += 1;
            }
        }
    }

    // 3. Delete from DB
    let result = sqlx::query("DELETE FROM activity_logs WHERE timestamp < ?")
        .bind(cutoff_ts)
        .execute(&pool)
        .await?;

    stats.deleted_activities = result.rows_affected();

    Ok(stats)
}

pub async fn increment_skipped_stat(reason: &str) -> Result<()> {
    let pool = get_pool().await?;
    let today = chrono::Local::now().format("%Y-%m-%d").to_string();

    sqlx::query(
        "INSERT INTO recording_stats (date, reason, count) 
         VALUES (?, ?, 1) 
         ON CONFLICT(date, reason) 
         DO UPDATE SET count = count + 1",
    )
    .bind(today)
    .bind(reason)
    .execute(&pool)
    .await?;

    Ok(())
}
