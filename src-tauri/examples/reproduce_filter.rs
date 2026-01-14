use sqlx::{sqlite::SqlitePoolOptions, QueryBuilder, Row};
use std::error::Error;

#[tokio::main]
async fn main() -> Result<(), Box<dyn Error>> {
    // 1. 创建内存数据库
    let pool = SqlitePoolOptions::new()
        .connect("sqlite::memory:")
        .await?;

    // 2. 初始化表结构
    sqlx::query(
        r#"
        CREATE TABLE activity_logs (
            id INTEGER PRIMARY KEY AUTOINCREMENT,
            timestamp INTEGER NOT NULL,
            app_name TEXT NOT NULL,
            window_title TEXT NOT NULL,
            image_path TEXT NOT NULL,
            ocr_text TEXT,
            phash TEXT,
            created_at INTEGER DEFAULT (strftime('%s', 'now'))
        );
        CREATE VIRTUAL TABLE activity_logs_fts USING fts5(
            ocr_text,
            content='activity_logs',
            content_rowid='id'
        );
        "#,
    )
    .execute(&pool)
    .await?;

    // 3. 插入测试数据
    // 插入一条 explorer.exe 的记录，OCR 包含 "File"
    sqlx::query(
        r#"
        INSERT INTO activity_logs (timestamp, app_name, window_title, image_path, ocr_text)
        VALUES (1700000000, 'Explorer.EXE', 'File Explorer', '/tmp/1.png', 'File Edit Selection View');
        INSERT INTO activity_logs_fts(rowid, ocr_text) VALUES (1, 'File Edit Selection View');
        "#,
    )
    .execute(&pool)
    .await?;

    println!("Data inserted. Now testing search logic...");

    // 4. 模拟 search_activities 逻辑
    let query_str = Some("File".to_string());
    let app_name_filter = Some("antigravity".to_string()); // 应该过滤掉 Explorer

    // 复制 db.rs 里的逻辑
    let mut builder = QueryBuilder::new(
        "SELECT a.id, a.app_name, a.ocr_text FROM activity_logs a "
    );

    let has_query = query_str.as_ref().map(|s| !s.is_empty()).unwrap_or(false);

    if has_query {
        builder.push("JOIN activity_logs_fts f ON a.id = f.rowid ");
    }

    builder.push("WHERE 1=1 ");

    if has_query {
        builder.push("AND activity_logs_fts MATCH ");
        builder.push_bind(query_str.unwrap());
    }

    if let Some(app) = app_name_filter {
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

    // 打印 SQL
    println!("[DEBUG] SQL: {}", builder.sql());

    let query = builder.build();
    let rows = query.fetch_all(&pool).await?;

    println!("Found {} rows:", rows.len());
    for row in &rows {
        let app: String = row.get(1);
        let ocr: String = row.get(2);
        println!(" - App: {}, OCR: {}", app, ocr);
    }

    if rows.len() == 0 {
        println!("TEST PASSED: No rows returned (as expected).");
    } else {
        println!("TEST FAILED: Rows returned but should be filtered out.");
    }

    Ok(())
}
