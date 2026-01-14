use sqlx::sqlite::SqliteConnectOptions;
use std::str::FromStr;

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    // 假设是 com.memflow.app (从 tauri.conf.json 获取)
    let app_data = dirs::data_dir().unwrap().join("com.memflow.app");
    println!("Target Data dir: {:?}", app_data);

    if !app_data.exists() {
        println!("Error: Data directory not found! Are you sure the app has run locally?");
        return Ok(());
    }

    // 1. Clean screenshots
    let screenshots = app_data.join("screenshots");
    if screenshots.exists() {
        println!("Cleaning screenshots dir...");
        let mut count = 0;
        for entry in std::fs::read_dir(&screenshots)? {
            let entry = entry?;
            let path = entry.path();
            if path.is_file() {
                if let Err(e) = std::fs::remove_file(&path) {
                    println!("Failed to remove {:?}: {}", path, e);
                } else {
                    count += 1;
                }
            }
        }
        println!("Deleted {} screenshots", count);
    } else {
        println!("Screenshots dir not found");
    }

    // 2. Clean DB
    let db_path = app_data.join("memflow.db");
    if db_path.exists() {
        println!("Cleaning database...");
        let opts =
            SqliteConnectOptions::from_str(&format!("sqlite://{}", db_path.to_string_lossy()))?
                .create_if_missing(false)
                .journal_mode(sqlx::sqlite::SqliteJournalMode::Wal);

        let pool = sqlx::SqlitePool::connect_with(opts).await?;

        // Clears data
        sqlx::query("DELETE FROM activity_logs")
            .execute(&pool)
            .await?;
        println!("Cleared activity_logs table");

        // Optional: Reset stats if stored in DB? No, stats are COUNT(*).

        sqlx::query("VACUUM").execute(&pool).await?;
        println!("Database vacuumed");
    } else {
        println!("Database file not found at {:?}", db_path);
    }

    Ok(())
}
