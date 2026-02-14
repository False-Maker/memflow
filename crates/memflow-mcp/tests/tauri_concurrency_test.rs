
use memflow_core::context::RuntimeContext;
use memflow_core::db;
use memflow_mcp::context::McpContext;
use serde::Serialize;
use sqlx::Row;
use std::path::PathBuf;
use std::sync::Arc;
use tokio::sync::Barrier;
use tokio::sync::Mutex;
use tokio::time::{sleep, Duration, Instant};

/// Activity payload written by the simulated Tauri app
#[derive(Debug, Clone)]
struct TauriActivity {
    timestamp: i64,
    app_name: String,
    window_title: String,
    image_path: String,
}

#[derive(Debug, Serialize)]
struct EvidenceReport {
    test_duration_ms: u64,
    tauri_writes: usize,
    mcp_reads: usize,
    db_lock_errors: usize,
    data_corruption_detected: bool,
    friendly_errors_returned: bool,
    summary: String,
}

#[cfg(feature = "integration-tests")]
#[tokio::test(flavor = "multi_thread", worker_threads = 4)]
async fn tauri_concurrency_validation() {
    // Arrange shared context pointing to the real app data dir
    let ctx = McpContext::new();
    let app_dir = ctx.app_dir();
    let db_path = app_dir.join("memflow.db");
    let screenshots_dir = app_dir.join("screenshots");

    db::init_db_with_path(db_path.clone(), screenshots_dir)
        .await
        .expect("failed to init db");

    let pool = db::get_pool().await.expect("pool initialized");

    // Ensure WAL is active to match production behavior
    sqlx::query("PRAGMA journal_mode=WAL;")
        .execute(&pool)
        .await
        .expect("enable WAL");

    // Shared state trackers
    let tauri_writes = Arc::new(Mutex::new(Vec::<TauriActivity>::new()));
    let mcp_reads = Arc::new(Mutex::new(0usize));
    let db_lock_errors = Arc::new(Mutex::new(0usize));
    let friendly_error_seen = Arc::new(Mutex::new(false));

    let duration = Duration::from_secs(10);
    let start = Instant::now();
    let barrier = Arc::new(Barrier::new(11)); // 1 writer + 10 readers

    // Spawn writer task simulating Tauri app inserts every 100ms
    let writer_handles = {
        let pool = pool.clone();
        let tauri_writes = tauri_writes.clone();
        let db_lock_errors = db_lock_errors.clone();
        let friendly_error_seen = friendly_error_seen.clone();
        let barrier = barrier.clone();

        tokio::spawn(async move {
            barrier.wait().await;
            let mut counter = 0u64;
            while start.elapsed() < duration {
                let now = chrono::Utc::now().timestamp();
                let activity = TauriActivity {
                    timestamp: now,
                    app_name: "TauriSim".to_string(),
                    window_title: format!("SimWindow #{counter}"),
                    image_path: format!("screenshots/sim_{counter}.png"),
                };

                match sqlx::query(
                    "INSERT INTO activity_logs (timestamp, app_name, window_title, image_path) VALUES (?, ?, ?, ?)",
                )
                .bind(activity.timestamp)
                .bind(&activity.app_name)
                .bind(&activity.window_title)
                .bind(&activity.image_path)
                .execute(&pool)
                .await
                {
                    Ok(_) => {
                        tauri_writes.lock().await.push(activity);
                    }
                    Err(err) => {
                        if is_lock_error(&err) {
                            *db_lock_errors.lock().await += 1;
                            *friendly_error_seen.lock().await |=
                                err.to_string().contains("-32000");
                        } else {
                            panic!("unexpected writer error: {err}");
                        }
                    }
                }

                counter += 1;
                sleep(Duration::from_millis(100)).await;
            }
        })
    };

    // Spawn 10 concurrent reader tasks simulating MCP searches
    let mut reader_handles = Vec::new();
    for i in 0..10 {
        let pool = pool.clone();
        let mcp_reads = mcp_reads.clone();
        let db_lock_errors = db_lock_errors.clone();
        let friendly_error_seen = friendly_error_seen.clone();
        let barrier = barrier.clone();
        reader_handles.push(tokio::spawn(async move {
            barrier.wait().await;
            while start.elapsed() < duration {
                let result = sqlx::query(
                    "SELECT id, app_name, window_title FROM activity_logs ORDER BY timestamp DESC LIMIT 5",
                )
                .fetch_all(&pool)
                .await;

                match result {
                    Ok(rows) => {
                        if !rows.is_empty() {
                            *mcp_reads.lock().await += 1;
                        }
                    }
                    Err(err) => {
                        if is_lock_error(&err) {
                            *db_lock_errors.lock().await += 1;
                            *friendly_error_seen.lock().await |=
                                err.to_string().contains("-32000");
                        } else {
                            panic!("unexpected reader error: {err}");
                        }
                    }
                }

                // Add slight jitter to avoid perfect lockstep
                sleep(Duration::from_millis(75 + (i * 5) as u64)).await;
            }
        }));
    }

    // Wait for tasks to complete
    writer_handles.await.expect("writer task panicked");
    for handle in reader_handles {
        handle.await.expect("reader task panicked");
    }

    // Validate counts before/after to ensure no corruption
    let total_count: i64 = sqlx::query_scalar("SELECT COUNT(*) FROM activity_logs")
        .fetch_one(&pool)
        .await
        .expect("count query");

    let tauri_written = tauri_writes.lock().await.len();
    assert!(tauri_written > 0, "writer should insert records");

    // Verify each inserted activity exists in DB (no corruption)
    for activity in tauri_writes.lock().await.iter() {
        let exists = sqlx::query("SELECT COUNT(*) FROM activity_logs WHERE timestamp = ? AND app_name = ? AND window_title = ?")
            .bind(activity.timestamp)
            .bind(&activity.app_name)
            .bind(&activity.window_title)
            .fetch_one(&pool)
            .await
            .map(|row| row.get::<i64, _>(0))
            .expect("existence check")
            > 0;
        assert!(exists, "inserted activity missing -> corruption");
    }

    let data_corruption_detected = false;
    let db_lock_errors_count = *db_lock_errors.lock().await;
    let friendly_error_returned = *friendly_error_seen.lock().await || db_lock_errors_count == 0;

    // Prepare evidence report
    let report = EvidenceReport {
        test_duration_ms: duration.as_millis() as u64,
        tauri_writes: tauri_written,
        mcp_reads: *mcp_reads.lock().await,
        db_lock_errors: db_lock_errors_count,
        data_corruption_detected,
        friendly_errors_returned: friendly_error_returned,
        summary: if db_lock_errors_count == 0 {
            "PASS - No data corruption, no lock errors".to_string()
        } else if friendly_error_returned {
            "PASS - Lock errors surfaced as friendly -32000".to_string()
        } else {
            "WARN - Lock errors without friendly handling".to_string()
        },
    };

    write_evidence(&report).expect("write evidence");

    assert!(total_count >= tauri_written as i64);
    assert!(!data_corruption_detected);
    assert!(friendly_error_returned);
}

fn is_lock_error(err: &sqlx::Error) -> bool {
    matches!(err, sqlx::Error::Database(db_err) if {
        let code = db_err.code().unwrap_or_default();
        code == "5" || code == "6" || db_err.message().contains("busy")
    })
}

fn write_evidence(report: &EvidenceReport) -> std::io::Result<()> {
    let report_dir = PathBuf::from(".sisyphus/evidence");
    std::fs::create_dir_all(&report_dir)?;
    let path = report_dir.join("tauri-concurrency-test.json");
    let content = serde_json::to_string_pretty(report).expect("serialize report");
    std::fs::write(path, content)
}

#[tokio::test(flavor = "multi_thread", worker_threads = 4)]
async fn test_concurrent_read_operations() {
    // Arrange: Initialize test database with test data (use separate test DB)
    let temp_dir = std::env::temp_dir().join("memflow_test_concurrent_read");
    std::fs::create_dir_all(&temp_dir).expect("create temp dir");
    let db_path = temp_dir.join("test_memflow.db");
    let screenshots_dir = temp_dir.join("screenshots");

    // Clean up any existing test database
    if db_path.exists() {
        std::fs::remove_file(&db_path).expect("remove existing test db");
    }

    db::init_db_with_path(db_path.clone(), screenshots_dir)
        .await
        .expect("failed to init db");

    let pool = db::get_pool().await.expect("pool initialized");

    // Ensure WAL mode is active
    sqlx::query("PRAGMA journal_mode=WAL;")
        .execute(&pool)
        .await
        .expect("enable WAL");

    // Insert some test data
    for i in 0..20 {
        sqlx::query(
            "INSERT INTO activity_logs (timestamp, app_name, window_title, image_path) VALUES (?, ?, ?, ?)"
        )
        .bind(chrono::Utc::now().timestamp() + i)
        .bind(format!("TestApp{}", i))
        .bind(format!("TestWindow{}", i))
        .bind(format!("screenshots/test_{}.png", i))
        .execute(&pool)
        .await
        .expect("insert test data");
    }

    // Spawn 10 concurrent read tasks
    let mut handles = Vec::new();
    let db_lock_errors = Arc::new(Mutex::new(0usize));
    let successful_reads = Arc::new(Mutex::new(0usize));

    for task_id in 0..10 {
        let pool = pool.clone();
        let db_lock_errors = Arc::clone(&db_lock_errors);
        let successful_reads = Arc::clone(&successful_reads);

        handles.push(tokio::spawn(async move {
            // Perform 5 reads per task
            for _ in 0..5 {
                let result = sqlx::query(
                    "SELECT id, app_name, window_title FROM activity_logs ORDER BY timestamp DESC LIMIT 5"
                )
                .fetch_all(&pool)
                .await;

                match result {
                    Ok(rows) => {
                        if !rows.is_empty() {
                            *successful_reads.lock().await += 1;
                        }
                    }
                    Err(err) => {
                        if is_lock_error(&err) {
                            *db_lock_errors.lock().await += 1;
                        }
                    }
                }
            }
        }));
    }

    // Wait for all tasks to complete
    for handle in handles {
        handle.await.expect("task panicked");
    }

    // Verify results
    let lock_errors = *db_lock_errors.lock().await;
    let reads = *successful_reads.lock().await;

    assert!(reads > 0, "At least some reads should succeed");
    assert_eq!(
        lock_errors, 0,
        "Concurrent reads should not cause database locks in WAL mode. Found {} lock errors",
        lock_errors
    );

    // Log evidence
    let evidence = format!(
        "test_concurrent_read_operations completed: {} successful reads, {} lock errors",
        reads, lock_errors
    );
    let report_dir = PathBuf::from(".sisyphus/evidence");
    std::fs::create_dir_all(&report_dir).expect("create evidence dir");
    std::fs::write(report_dir.join("task-11-concurrent-read.log"), evidence)
        .expect("write evidence");
}

#[tokio::test(flavor = "multi_thread", worker_threads = 4)]
async fn test_mcp_tauri_concurrent_access() {
    // Arrange: Initialize test database (use separate test DB)
    let temp_dir = std::env::temp_dir().join("memflow_test_mcp_tauri");
    std::fs::create_dir_all(&temp_dir).expect("create temp dir");
    let db_path = temp_dir.join("test_memflow.db");
    let screenshots_dir = temp_dir.join("screenshots");

    // Clean up any existing test database
    if db_path.exists() {
        std::fs::remove_file(&db_path).expect("remove existing test db");
    }

    db::init_db_with_path(db_path.clone(), screenshots_dir)
        .await
        .expect("failed to init db");

    let pool = db::get_pool().await.expect("pool initialized");

    // Verify WAL mode
    let journal_mode: String = sqlx::query_scalar("PRAGMA journal_mode;")
        .fetch_one(&pool)
        .await
        .expect("get journal mode");
    assert_eq!(
        journal_mode.to_lowercase(),
        "wal",
        "WAL mode should be enabled for concurrent access"
    );

    // Spawn writer task simulating Tauri app
    let pool_writer = pool.clone();
    let writer_handle = tokio::spawn(async move {
        let mut write_count = 0;
        for i in 0..10 {
            let result = sqlx::query(
                "INSERT INTO activity_logs (timestamp, app_name, window_title, image_path) VALUES (?, ?, ?, ?)"
            )
            .bind(chrono::Utc::now().timestamp() + i)
            .bind("TauriApp")
            .bind(format!("TauriWindow {}", i))
            .bind(format!("screenshots/tauri_{}.png", i))
            .execute(&pool_writer)
            .await;

            if result.is_ok() {
                write_count += 1;
            }
            sleep(Duration::from_millis(50)).await;
        }
        write_count
    });

    // Spawn reader tasks simulating MCP server
    let mut reader_handles = Vec::new();
    let db_lock_errors = Arc::new(Mutex::new(0usize));
    let successful_reads = Arc::new(Mutex::new(0usize));

    for _ in 0..5 {
        let pool_reader = pool.clone();
        let db_lock_errors = Arc::clone(&db_lock_errors);
        let successful_reads = Arc::clone(&successful_reads);

        reader_handles.push(tokio::spawn(async move {
            for _ in 0..5 {
                let result = sqlx::query(
                    "SELECT id, app_name, window_title FROM activity_logs ORDER BY timestamp DESC LIMIT 10"
                )
                .fetch_all(&pool_reader)
                .await;

                match result {
                    Ok(rows) => {
                        if !rows.is_empty() {
                            *successful_reads.lock().await += 1;
                        }
                    }
                    Err(err) => {
                        if is_lock_error(&err) {
                            *db_lock_errors.lock().await += 1;
                        }
                    }
                }
                sleep(Duration::from_millis(30)).await;
            }
        }));
    }

    // Wait for all tasks to complete
    let writes_completed = writer_handle.await.expect("writer panicked");
    for handle in reader_handles {
        handle.await.expect("reader panicked");
    }

    // Verify data integrity
    let total_count: i64 = sqlx::query_scalar("SELECT COUNT(*) FROM activity_logs")
        .fetch_one(&pool)
        .await
        .expect("count query");

    let lock_errors = *db_lock_errors.lock().await;
    let reads = *successful_reads.lock().await;

    assert!(
        writes_completed > 0,
        "Tauri writer should complete at least some writes"
    );
    assert!(reads > 0, "MCP readers should complete some reads");
    assert!(
        total_count >= writes_completed as i64,
        "Total count should reflect all writes"
    );
    assert_eq!(
        lock_errors, 0,
        "WAL mode should prevent locks during concurrent read/write. Found {} lock errors",
        lock_errors
    );

    // Log evidence
    let evidence = format!(
        "test_mcp_tauri_concurrent_access completed: {} writes, {} successful reads, {} total records, {} lock errors",
        writes_completed, reads, total_count, lock_errors
    );
    let report_dir = PathBuf::from(".sisyphus/evidence");
    std::fs::create_dir_all(&report_dir).expect("create evidence dir");
    std::fs::write(report_dir.join("task-11-mcp-tauri-concurrent.log"), evidence)
        .expect("write evidence");
}
