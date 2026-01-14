//! FTS5 全文检索测试
//!
//! 运行方式: cargo run --example test_fts
//!
//! 测试目标:
//! - 验证 FTS5 全文检索索引正常工作
//! - 可以根据关键词检索活动记录
//! - 检索性能良好（< 100ms）

use std::time::Instant;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // 获取数据库路径
    let app_data = dirs::data_dir()
        .ok_or("无法获取数据目录")?
        .join("com.memflow.app");
    let db_path = app_data.join("memflow.db");

    println!("数据库路径: {}", db_path.display());

    if !db_path.exists() {
        println!("❌ 数据库文件不存在！请先运行应用并录制一些活动。");
        return Ok(());
    }

    // 连接数据库
    let pool = sqlx::sqlite::SqlitePool::connect(&format!("sqlite:{}", db_path.display())).await?;

    println!("\n========== FTS5 全文检索测试 ==========\n");

    // 测试 1: 检查数据量
    println!("📊 测试 1: 检查数据量");
    let total_logs: (i64,) = sqlx::query_as("SELECT COUNT(*) FROM activity_logs")
        .fetch_one(&pool)
        .await?;
    println!("   活动记录总数: {}", total_logs.0);

    let logs_with_ocr: (i64,) =
        sqlx::query_as("SELECT COUNT(*) FROM activity_logs WHERE ocr_text IS NOT NULL")
            .fetch_one(&pool)
            .await?;
    println!("   带 OCR 文本的记录数: {}", logs_with_ocr.0);

    let fts_count: (i64,) = sqlx::query_as("SELECT COUNT(*) FROM activity_logs_fts")
        .fetch_one(&pool)
        .await?;
    println!("   FTS 索引条目数: {}", fts_count.0);

    // 如果 FTS 索引为空但有 OCR 数据，自动同步
    if fts_count.0 == 0 && logs_with_ocr.0 > 0 {
        println!("\n⚠️  FTS 索引为空，正在同步数据...");

        // 同步现有 OCR 数据到 FTS 表
        sqlx::query(
            "INSERT INTO activity_logs_fts(rowid, ocr_text) 
             SELECT id, ocr_text FROM activity_logs WHERE ocr_text IS NOT NULL",
        )
        .execute(&pool)
        .await?;

        let new_fts_count: (i64,) = sqlx::query_as("SELECT COUNT(*) FROM activity_logs_fts")
            .fetch_one(&pool)
            .await?;
        println!("   ✅ 已同步 {} 条记录到 FTS 索引", new_fts_count.0);
    } else if fts_count.0 == 0 {
        println!("\n⚠️  没有 OCR 数据可供测试！");
        return Ok(());
    }

    // 测试 2: 基础全文检索
    println!("\n🔍 测试 2: 基础全文检索");

    // 先获取一个存在的关键词用于测试
    let sample_text: Option<(String,)> =
        sqlx::query_as("SELECT ocr_text FROM activity_logs WHERE ocr_text IS NOT NULL LIMIT 1")
            .fetch_optional(&pool)
            .await?;

    let test_keyword = if let Some((text,)) = sample_text {
        // 从 OCR 文本中提取一个词作为测试关键词
        text.split_whitespace()
            .find(|w| w.len() >= 2)
            .unwrap_or("文件")
            .to_string()
    } else {
        "文件".to_string()
    };

    println!("   测试关键词: '{}'", test_keyword);

    let start = Instant::now();
    let results: Vec<(i64, Option<String>)> = sqlx::query_as(
        "SELECT rowid, ocr_text FROM activity_logs_fts WHERE activity_logs_fts MATCH ? LIMIT 10",
    )
    .bind(&test_keyword)
    .fetch_all(&pool)
    .await?;
    let elapsed = start.elapsed();

    println!("   找到 {} 条结果", results.len());
    println!("   检索耗时: {:?}", elapsed);

    if elapsed.as_millis() < 100 {
        println!("   ✅ 性能良好 (< 100ms)");
    } else {
        println!("   ⚠️  性能较慢 (>= 100ms)");
    }

    // 测试 3: 多关键词 OR 查询
    println!("\n🔍 测试 3: 多关键词 OR 查询");
    let or_query = format!("{} OR 的 OR 是", test_keyword);

    let start = Instant::now();
    let results: Vec<(i64, Option<String>)> = sqlx::query_as(
        "SELECT rowid, ocr_text FROM activity_logs_fts WHERE activity_logs_fts MATCH ? LIMIT 10",
    )
    .bind(&or_query)
    .fetch_all(&pool)
    .await?;
    let elapsed = start.elapsed();

    println!("   查询: '{}'", or_query);
    println!("   找到 {} 条结果", results.len());
    println!("   检索耗时: {:?}", elapsed);

    // 测试 4: 显示一些检索结果示例
    println!("\n📄 测试 4: 检索结果示例");
    for (i, (rowid, ocr_text)) in results.iter().take(3).enumerate() {
        println!("   结果 {}:", i + 1);
        println!("      rowid: {}", rowid);
        if let Some(text) = ocr_text {
            // 安全截取中文字符串
            let preview: String = text.chars().take(80).collect();
            let preview = if text.chars().count() > 80 {
                format!("{}...", preview)
            } else {
                preview
            };
            println!("      文本: {}", preview.replace('\n', " "));
        }
    }

    // 测试 5: FTS 完整性检查
    println!("\n🔧 测试 5: FTS 完整性检查");
    let integrity: (String,) = sqlx::query_as("PRAGMA integrity_check")
        .fetch_one(&pool)
        .await?;

    if integrity.0 == "ok" {
        println!("   ✅ 数据库完整性检查通过");
    } else {
        println!("   ❌ 数据库完整性问题: {}", integrity.0);
    }

    println!("\n========== 测试完成 ==========\n");

    // 汇总
    println!("📋 测试汇总:");
    println!(
        "   - 搜索功能: {}",
        if results.len() > 0 {
            "✅ 正常"
        } else {
            "⚠️  无结果"
        }
    );
    println!(
        "   - 检索性能: {}",
        if elapsed.as_millis() < 100 {
            "✅ 良好"
        } else {
            "⚠️  较慢"
        }
    );
    println!(
        "   - 数据库完整性: {}",
        if integrity.0 == "ok" {
            "✅ 正常"
        } else {
            "❌ 异常"
        }
    );

    Ok(())
}
