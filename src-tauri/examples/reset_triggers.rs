//! 删除 FTS 触发器，用于测试 migration 是否能自动创建
//!
//! 运行方式: cargo run --example reset_triggers

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let app_data = dirs::data_dir()
        .ok_or("无法获取数据目录")?
        .join("com.memflow.app");
    let db_path = app_data.join("memflow.db");

    println!("数据库路径: {}", db_path.display());

    let pool = sqlx::sqlite::SqlitePool::connect(&format!("sqlite:{}", db_path.display())).await?;

    println!("\n========== 删除 FTS 触发器 ==========\n");

    // 删除触发器
    sqlx::query("DROP TRIGGER IF EXISTS activity_logs_fts_insert")
        .execute(&pool)
        .await?;
    sqlx::query("DROP TRIGGER IF EXISTS activity_logs_fts_update")
        .execute(&pool)
        .await?;
    sqlx::query("DROP TRIGGER IF EXISTS activity_logs_fts_delete")
        .execute(&pool)
        .await?;

    println!("✅ 已删除所有 FTS 触发器");

    // 验证
    let triggers: Vec<(String,)> = sqlx::query_as(
        "SELECT name FROM sqlite_master WHERE type = 'trigger' AND name LIKE '%fts%'",
    )
    .fetch_all(&pool)
    .await?;

    if triggers.is_empty() {
        println!("✅ 确认：没有 FTS 触发器存在");
        println!("\n现在请重启应用 (pnpm tauri dev)，migration 会自动创建触发器");
    } else {
        println!("⚠️  仍有触发器存在: {:?}", triggers);
    }

    Ok(())
}
