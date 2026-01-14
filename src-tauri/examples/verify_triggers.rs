//! 验证 FTS 触发器是否存在
//!
//! 运行方式: cargo run --example verify_triggers

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let app_data = dirs::data_dir()
        .ok_or("无法获取数据目录")?
        .join("com.memflow.app");
    let db_path = app_data.join("memflow.db");

    println!("数据库路径: {}", db_path.display());

    let pool = sqlx::sqlite::SqlitePool::connect(&format!("sqlite:{}", db_path.display())).await?;

    println!("\n========== 检查 FTS 触发器 ==========\n");

    // 查询所有触发器
    let triggers: Vec<(String, String)> = sqlx::query_as(
        "SELECT name, sql FROM sqlite_master WHERE type = 'trigger' AND name LIKE '%fts%'",
    )
    .fetch_all(&pool)
    .await?;

    if triggers.is_empty() {
        println!("⚠️  未找到 FTS 触发器！正在创建...\n");

        // 创建触发器
        sqlx::query(
            r#"
            CREATE TRIGGER IF NOT EXISTS activity_logs_fts_insert 
            AFTER INSERT ON activity_logs 
            WHEN NEW.ocr_text IS NOT NULL
            BEGIN
                INSERT INTO activity_logs_fts(rowid, ocr_text) VALUES (NEW.id, NEW.ocr_text);
            END
        "#,
        )
        .execute(&pool)
        .await?;

        sqlx::query(
            r#"
            CREATE TRIGGER IF NOT EXISTS activity_logs_fts_update 
            AFTER UPDATE OF ocr_text ON activity_logs 
            BEGIN
                DELETE FROM activity_logs_fts WHERE rowid = OLD.id;
                INSERT INTO activity_logs_fts(rowid, ocr_text) 
                SELECT NEW.id, NEW.ocr_text WHERE NEW.ocr_text IS NOT NULL;
            END
        "#,
        )
        .execute(&pool)
        .await?;

        sqlx::query(
            r#"
            CREATE TRIGGER IF NOT EXISTS activity_logs_fts_delete 
            AFTER DELETE ON activity_logs 
            BEGIN
                DELETE FROM activity_logs_fts WHERE rowid = OLD.id;
            END
        "#,
        )
        .execute(&pool)
        .await?;

        println!("✅ 触发器创建成功！\n");

        // 重新查询
        let triggers: Vec<(String, String)> = sqlx::query_as(
            "SELECT name, sql FROM sqlite_master WHERE type = 'trigger' AND name LIKE '%fts%'",
        )
        .fetch_all(&pool)
        .await?;

        println!("✅ 找到 {} 个 FTS 触发器:\n", triggers.len());
        for (name, _sql) in &triggers {
            println!("   📌 {}", name);
        }
        println!();
    } else {
        println!("✅ 找到 {} 个 FTS 触发器:\n", triggers.len());
        for (name, sql) in &triggers {
            println!("📌 {}", name);
            println!("   {}\n", sql.replace('\n', "\n   "));
        }
    }

    // 测试触发器是否工作（插入一条测试数据然后删除）
    println!("========== 测试触发器功能 ==========\n");

    if !triggers.is_empty() {
        // 获取当前 FTS 数量
        let before: (i64,) = sqlx::query_as("SELECT COUNT(*) FROM activity_logs_fts")
            .fetch_one(&pool)
            .await?;

        // 插入测试数据
        let result = sqlx::query(
            "INSERT INTO activity_logs (timestamp, app_name, window_title, image_path, ocr_text) 
             VALUES (0, 'trigger_test', 'test', 'test.png', '触发器测试文本')",
        )
        .execute(&pool)
        .await?;

        let test_id = result.last_insert_rowid();

        // 检查 FTS 是否自动更新
        let after: (i64,) = sqlx::query_as("SELECT COUNT(*) FROM activity_logs_fts")
            .fetch_one(&pool)
            .await?;

        // 清理测试数据
        sqlx::query("DELETE FROM activity_logs WHERE id = ?")
            .bind(test_id)
            .execute(&pool)
            .await?;

        let final_count: (i64,) = sqlx::query_as("SELECT COUNT(*) FROM activity_logs_fts")
            .fetch_one(&pool)
            .await?;

        println!("   插入前 FTS 记录数: {}", before.0);
        println!("   插入后 FTS 记录数: {}", after.0);
        println!("   删除后 FTS 记录数: {}", final_count.0);

        if after.0 == before.0 + 1 && final_count.0 == before.0 {
            println!("\n   ✅ 触发器工作正常！INSERT 和 DELETE 触发器都已生效。");
        } else if after.0 == before.0 + 1 {
            println!("\n   ⚠️  INSERT 触发器正常，DELETE 触发器可能有问题");
        } else {
            println!("\n   ❌ 触发器未正常工作");
        }
    }

    Ok(())
}
