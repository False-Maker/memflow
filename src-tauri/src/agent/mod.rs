//! 智能代理（Agent）- 自动化提案与执行（MVP）
//!
//! 目标：基于 activity_logs 生成低风险自动化提案，并在用户确认后执行；全过程可审计并支持取消。

use anyhow::{anyhow, Result};
use once_cell::sync::Lazy;
use serde::{Deserialize, Serialize};
use serde_json::Value;
use sqlx::Row;
use std::collections::HashMap;
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::Arc;
use std::time::Duration;
use tauri::Manager;
use tauri_plugin_opener::OpenerExt;
use tokio::sync::Mutex;

use crate::db::get_pool;

static EXECUTION_CANCEL_FLAGS: Lazy<Mutex<HashMap<i64, Arc<AtomicBool>>>> =
    Lazy::new(|| Mutex::new(HashMap::new()));

fn truncate_chars(s: &str, max_chars: usize) -> String {
    if s.chars().count() <= max_chars {
        return s.to_string();
    }
    s.chars().take(max_chars).collect()
}

// ============================================
// DTO / Params
// ============================================

#[derive(Debug, Default, Serialize, Deserialize, Clone)]
#[serde(rename_all = "camelCase")]
pub struct AgentProposeParams {
    pub time_window_hours: Option<i64>,
    pub limit: Option<i64>,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
#[serde(rename_all = "camelCase")]
pub struct AutomationEvidence {
    pub activity_id: i64,
    pub timestamp: i64,
    pub app_name: String,
    pub window_title: String,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
#[serde(tag = "type", rename_all = "snake_case")]
pub enum AutomationStep {
    OpenUrl { url: String },
    OpenFile { path: String },
    OpenApp { path: String },
    CopyToClipboard { text: String },
    CreateNote { content: String },
}

#[derive(Debug, Serialize, Deserialize, Clone)]
#[serde(rename_all = "camelCase")]
pub struct AutomationProposalDto {
    pub id: i64,
    pub title: String,
    pub description: String,
    pub confidence: f64,
    pub risk_level: String, // low | medium | high
    pub steps: Vec<AutomationStep>,
    pub evidence: Vec<AutomationEvidence>,
    pub created_at: i64,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
#[serde(rename_all = "camelCase")]
pub struct ExecutionDto {
    pub id: i64,
    pub proposal_id: Option<i64>,
    pub action: String,
    pub status: String, // running | success | failed | cancelled
    pub created_at: i64,
    pub finished_at: Option<i64>,
    pub error_message: Option<String>,
    pub metadata: Option<Value>,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
#[serde(rename_all = "camelCase")]
pub struct ExecutionResultDto {
    pub execution_id: i64,
    pub status: String,
}

// ============================================
// Public APIs
// ============================================

pub async fn propose_automation(params: AgentProposeParams) -> Result<Vec<AutomationProposalDto>> {
    let pool = get_pool().await?;
    let now = chrono::Utc::now().timestamp();
    let time_window_hours = params.time_window_hours.unwrap_or(24).max(1).min(24 * 30);
    let limit = params.limit.unwrap_or(10).max(1).min(50);
    let since_ts = now - time_window_hours * 3600;

    tracing::info!(
        "agent propose start: time_window_hours={}, limit={}, since_ts={}",
        time_window_hours,
        limit,
        since_ts
    );

    // 取样：最近窗口内最多 500 条活动，用于生成摘要与证据
    let rows = tokio::time::timeout(Duration::from_secs(10), async {
        sqlx::query(
            r#"
            SELECT id, timestamp, app_name, window_title, ocr_text
            FROM activity_logs
            WHERE timestamp >= ?
            ORDER BY timestamp DESC
            LIMIT 500
            "#,
        )
        .bind(since_ts)
        .fetch_all(&pool)
        .await
    })
    .await
    .map_err(|_| anyhow!("AGENT_TIMEOUT: fetch activity logs"))??;

    if rows.is_empty() {
        tracing::info!("agent propose: no activity rows");
        return Ok(vec![]);
    }

    tracing::info!("agent propose: fetched {} rows", rows.len());

    // 统计 top apps/window titles（规则化摘要）
    let mut app_counts: HashMap<String, i64> = HashMap::new();
    let mut title_counts: HashMap<String, i64> = HashMap::new();
    let mut evidence: Vec<AutomationEvidence> = Vec::new();

    for (idx, row) in rows.iter().enumerate() {
        let id: i64 = row.get(0);
        let timestamp: i64 = row.get(1);
        let app_name: String = row.get(2);
        let window_title: String = row.get(3);
        let _ocr: Option<String> = row.get(4); // 暂未直接使用，但已获取供后续扩展

        *app_counts.entry(app_name.clone()).or_insert(0) += 1;
        *title_counts.entry(window_title.clone()).or_insert(0) += 1;

        if idx < 5 {
            evidence.push(AutomationEvidence {
                activity_id: id,
                timestamp,
                app_name,
                window_title,
            });
        }
    }

    let mut top_apps: Vec<(String, i64)> = app_counts.into_iter().collect();
    top_apps.sort_by(|a, b| b.1.cmp(&a.1));
    top_apps.truncate(5);

    let mut top_titles: Vec<(String, i64)> = title_counts.into_iter().collect();
    top_titles.sort_by(|a, b| b.1.cmp(&a.1));
    top_titles.truncate(5);

    let rule_based_summary =
        build_activity_summary(time_window_hours, rows.len() as i64, &top_apps, &top_titles);

    // 尝试使用 LLM 生成摘要
    // 构建上下文（取最近 100 条，避免 Token 过多）
    let context_items: Vec<String> = rows.iter().take(100).map(|row| {
        let timestamp: i64 = row.get(1);
        let app_name: String = row.get(2);
        let window_title: String = row.get(3);
        let ocr_text: Option<String> = row.get(4);
        
        let mut line = if let Some(dt) = chrono::DateTime::from_timestamp(timestamp, 0) {
             let local: chrono::DateTime<chrono::Local> = chrono::DateTime::from(dt);
             format!("[{}] {}: {}", local.format("%H:%M"), app_name, window_title)
        } else {
             format!("{}: {}", app_name, window_title)
        };

        if let Some(text) = ocr_text {
             if !text.trim().is_empty() {
                 let truncated = truncate_chars(&text, 100);
                 line.push_str(&format!(" | 内容: {}", truncated.replace("\n", " ")));
             }
        }
        line
    }).collect();
    let context_text = context_items.join("\n");

    let mut proposals: Vec<AutomationProposalDto> = Vec::new();

    tracing::info!("agent propose: start ai analysis (context chars={})", context_text.len());
    match tokio::time::timeout(Duration::from_secs(25), crate::ai::analyze_for_proposals(&context_text)).await {
        Ok(Ok(analysis)) => {
            tracing::info!("agent propose: ai analysis ok, tasks={}", analysis.tasks.len());
            
            for task in analysis.tasks {
                let mut steps = Vec::new();
                
                // 1. 笔记步骤：包含摘要和链接列表
                let mut note_content = format!("# {}\n\n{}\n\n### 相关资源\n", task.title, task.summary);
                for url in &task.related_urls {
                    note_content.push_str(&format!("- 链接: {}\n", url));
                }
                for path in &task.related_files {
                    note_content.push_str(&format!("- 文件: {}\n", path));
                }
                for path in &task.related_apps {
                    note_content.push_str(&format!("- 应用: {}\n", path));
                }
                // 额外加上“一键恢复”说明
                note_content.push_str("\n*(此笔记由 MemFlow 智能回顾自动生成)*");

                steps.push(AutomationStep::CreateNote { content: note_content });

                // 2. 恢复步骤：打开链接 (限制数量，防止炸浏览器)
                for url in task.related_urls.iter().take(5) {
                    steps.push(AutomationStep::OpenUrl { url: url.clone() });
                }

                // 3. 恢复步骤：打开文件 (限制数量)
                for path in task.related_files.iter().take(5) {
                    // 简单的路径过滤（必须是绝对路径）
                    let path = path.trim();
                    if !path.is_empty() && (path.contains(":/") || path.contains(":\\") || path.starts_with("/")) {
                         steps.push(AutomationStep::OpenFile { path: path.to_string() });
                    }
                }

                // 4. 恢复步骤：打开应用 (限制数量)
                for path in task.related_apps.iter().take(3) {
                    let path = path.trim();
                    if !path.is_empty() && (path.contains(":/") || path.contains(":\\") || path.starts_with("/")) {
                         steps.push(AutomationStep::OpenApp { path: path.to_string() });
                    }
                }

                let proposal = AutomationProposalDto {
                    id: 0,
                    title: task.title,
                    description: task.summary.clone(),
                    confidence: 0.85,
                    risk_level: "low".to_string(),
                    steps,
                    evidence: vec![], // 简化处理，暂不绑定特定证据
                    created_at: now,
                };
                proposals.push(proposal);
            }

            // 如果没有生成任何任务（比如活动太少），则回退到规则摘要
            if proposals.is_empty() {
                 let fallback_proposal = AutomationProposalDto {
                    id: 0,
                    title: format!("生成最近 {} 小时活动摘要（规则）", time_window_hours),
                    description: "AI 未识别出明确任务，生成基础活动摘要。".to_string(),
                    confidence: 0.60,
                    risk_level: "low".to_string(),
                    steps: vec![AutomationStep::CreateNote { content: rule_based_summary.clone() }],
                    evidence: evidence.clone(),
                    created_at: now,
                };
                proposals.push(fallback_proposal);
            }
        }
        Ok(Err(e)) => {
            tracing::warn!("agent propose: ai analysis failed, fallback: {:?}", e);
            // 回退提案
            let fallback_proposal = AutomationProposalDto {
                id: 0,
                title: format!("生成最近 {} 小时活动摘要（规则）", time_window_hours),
                description: "AI 分析失败，生成基础活动摘要。".to_string(),
                confidence: 0.60,
                risk_level: "low".to_string(),
                steps: vec![AutomationStep::CreateNote { content: rule_based_summary }],
                evidence: evidence.clone(),
                created_at: now,
            };
            proposals.push(fallback_proposal);
        }
        Err(_) => {
            tracing::warn!("agent propose: ai analysis timeout, fallback to rule summary");
            let fallback_proposal = AutomationProposalDto {
                id: 0,
                title: format!("生成最近 {} 小时活动摘要（规则）", time_window_hours),
                description: "AI 分析超时，生成基础活动摘要。".to_string(),
                confidence: 0.55,
                risk_level: "low".to_string(),
                steps: vec![AutomationStep::CreateNote { content: rule_based_summary }],
                evidence: evidence.clone(),
                created_at: now,
            };
            proposals.push(fallback_proposal);
        }
    };

    // 批量插入数据库并更新 ID
    let mut saved_proposals = Vec::new();
    for mut p in proposals {
        let steps_json = serde_json::to_string(&p.steps)?;
        let evidence_json = serde_json::to_string(&p.evidence)?;
        
        let id = tokio::time::timeout(Duration::from_secs(10), async {
            sqlx::query(
                r#"
                INSERT INTO automation_proposals (title, description, confidence, risk_level, steps_json, evidence_json, created_at)
                VALUES (?, ?, ?, ?, ?, ?, ?)
                "#,
            )
            .bind(&p.title)
            .bind(&p.description)
            .bind(p.confidence)
            .bind(&p.risk_level)
            .bind(&steps_json)
            .bind(&evidence_json)
            .bind(now)
            .execute(&pool)
            .await
        })
        .await
        .map_err(|_| anyhow!("AGENT_TIMEOUT: insert automation proposal"))??
        .last_insert_rowid();

        p.id = id;
        saved_proposals.push(p);
    }

    tracing::info!("agent propose done: {} proposals", saved_proposals.len());

    Ok(saved_proposals.into_iter().take(limit as usize).collect())
}


pub async fn list_executions(limit: i64, offset: i64) -> Result<Vec<ExecutionDto>> {
    let pool = get_pool().await?;
    let limit = limit.max(1).min(200);
    let offset = offset.max(0);

    let rows = sqlx::query(
        r#"
        SELECT id, proposal_id, action, status, created_at, finished_at, error_message, metadata_json
        FROM agent_executions
        ORDER BY created_at DESC
        LIMIT ? OFFSET ?
        "#,
    )
    .bind(limit)
    .bind(offset)
    .fetch_all(&pool)
    .await?;

    let executions = rows
        .into_iter()
        .map(|row| {
            let metadata_str: Option<String> = row.get(7);
            let metadata = metadata_str.and_then(|s| serde_json::from_str::<Value>(&s).ok());
            ExecutionDto {
                id: row.get(0),
                proposal_id: row.get(1),
                action: row.get(2),
                status: row.get(3),
                created_at: row.get(4),
                finished_at: row.get(5),
                error_message: row.get(6),
                metadata,
            }
        })
        .collect();

    Ok(executions)
}

pub async fn execute_automation(
    proposal_id: i64,
    app_handle: tauri::AppHandle,
) -> Result<ExecutionResultDto> {
    let pool = get_pool().await?;
    let now = chrono::Utc::now().timestamp();

    // 读取 proposal
    let row = sqlx::query(
        r#"
        SELECT id, title, risk_level, steps_json
        FROM automation_proposals
        WHERE id = ?
        "#,
    )
    .bind(proposal_id)
    .fetch_optional(&pool)
    .await?;

    let row = row.ok_or_else(|| anyhow!("AGENT_NOT_FOUND: proposal {}", proposal_id))?;
    let risk_level: String = row.get(2);

    if risk_level != "low" {
        return Err(anyhow!("AGENT_RISK_BLOCKED"));
    }

    let steps_json: String = row.get(3);
    let steps: Vec<AutomationStep> =
        serde_json::from_str(&steps_json).map_err(|_| anyhow!("AGENT_STEP_NOT_ALLOWED"))?;

    // allowlist 校验（MVP：仅允许定义的 step 类型，且字段非空）
    validate_steps(&steps)?;

    // 创建执行记录
    let action = steps_action_summary(&steps);
    let execution_id = sqlx::query(
        r#"
        INSERT INTO agent_executions (proposal_id, action, status, created_at)
        VALUES (?, ?, ?, ?)
        "#,
    )
    .bind(proposal_id)
    .bind(&action)
    .bind("running")
    .bind(now)
    .execute(&pool)
    .await?
    .last_insert_rowid();

    // 注册取消标记（后台任务会清理）
    let cancel_flag = Arc::new(AtomicBool::new(false));
    EXECUTION_CANCEL_FLAGS
        .lock()
        .await
        .insert(execution_id, cancel_flag.clone());

    // 后台执行（命令立即返回 running，便于前端取消/轮询）
    let steps_total = steps.len() as i64;
    let pool_bg = pool.clone();
    let app_handle_bg = app_handle.clone();
    let steps_bg = steps.clone();
    let cancel_flag_bg = cancel_flag.clone();

    tokio::spawn(async move {
        let mut steps_success = 0_i64;

        let result: Result<()> = async {
            for step in &steps_bg {
                if cancel_flag_bg.load(Ordering::Relaxed) {
                    return Err(anyhow!("AGENT_EXECUTION_CANCELLED"));
                }

                if let Err(e) = execute_step(step, &app_handle_bg).await {
                    return Err(e);
                }
                steps_success += 1;
            }
            Ok(())
        }
        .await;

        // 清理取消标记
        EXECUTION_CANCEL_FLAGS.lock().await.remove(&execution_id);

        // 落审计
        let finished_at = chrono::Utc::now().timestamp();
        let metadata = serde_json::json!({
            "steps_total": steps_total,
            "steps_success": steps_success,
            "duration_s": (finished_at - now).max(0),
        });
        let metadata_json = serde_json::to_string(&metadata).ok();

        match result {
            Ok(_) => {
                let _ = sqlx::query(
                    r#"
                    UPDATE agent_executions
                    SET status = ?, finished_at = ?, error_message = NULL, metadata_json = ?
                    WHERE id = ?
                    "#,
                )
                .bind("success")
                .bind(finished_at)
                .bind(metadata_json)
                .bind(execution_id)
                .execute(&pool_bg)
                .await;
            }
            Err(e) => {
                let msg = e.to_string();
                let status = if msg.contains("AGENT_EXECUTION_CANCELLED") {
                    "cancelled"
                } else {
                    "failed"
                };

                let _ = sqlx::query(
                    r#"
                    UPDATE agent_executions
                    SET status = ?, finished_at = ?, error_message = ?, metadata_json = ?
                    WHERE id = ?
                    "#,
                )
                .bind(status)
                .bind(finished_at)
                .bind(&msg)
                .bind(metadata_json)
                .bind(execution_id)
                .execute(&pool_bg)
                .await;
            }
        }
    });

    Ok(ExecutionResultDto {
        execution_id,
        status: "running".to_string(),
    })
}

pub async fn cancel_execution(execution_id: i64) -> Result<()> {
    let map = EXECUTION_CANCEL_FLAGS.lock().await;
    if let Some(flag) = map.get(&execution_id) {
        flag.store(true, Ordering::Relaxed);
    }
    Ok(())
}

// ============================================
// Helpers
// ============================================

fn build_activity_summary(
    time_window_hours: i64,
    total: i64,
    top_apps: &[(String, i64)],
    top_titles: &[(String, i64)],
) -> String {
    let mut lines: Vec<String> = Vec::new();
    lines.push(format!(
        "【MemFlow 活动摘要】最近 {} 小时",
        time_window_hours
    ));
    lines.push(format!("- 记录条数：{}", total));

    if !top_apps.is_empty() {
        lines.push("- Top 应用：".to_string());
        for (app, cnt) in top_apps.iter().take(3) {
            lines.push(format!("  - {}（{}）", app, cnt));
        }
    }

    if !top_titles.is_empty() {
        lines.push("- Top 窗口：".to_string());
        for (title, cnt) in top_titles.iter().take(3) {
            let t = if title.len() > 60 {
                format!("{}...", truncate_chars(title, 60))
            } else {
                title.clone()
            };
            lines.push(format!("  - {}（{}）", t, cnt));
        }
    }

    lines.push("（提示：这是规则化摘要，可在后续版本用 LLM 进一步润色。）".to_string());
    lines.join("\n")
}

fn validate_steps(steps: &[AutomationStep]) -> Result<()> {
    for s in steps {
        match s {
            AutomationStep::OpenUrl { url } => {
                if url.trim().is_empty() {
                    return Err(anyhow!("AGENT_STEP_NOT_ALLOWED"));
                }
            }
            AutomationStep::OpenFile { path } => {
                if path.trim().is_empty() {
                    return Err(anyhow!("AGENT_STEP_NOT_ALLOWED"));
                }
            }
            AutomationStep::OpenApp { path } => {
                if path.trim().is_empty() {
                    return Err(anyhow!("AGENT_STEP_NOT_ALLOWED"));
                }
            }
            AutomationStep::CopyToClipboard { text } => {
                if text.trim().is_empty() {
                    return Err(anyhow!("AGENT_STEP_NOT_ALLOWED"));
                }
            }
            AutomationStep::CreateNote { content } => {
                if content.trim().is_empty() {
                    return Err(anyhow!("AGENT_STEP_NOT_ALLOWED"));
                }
            }
        }
    }
    Ok(())
}

fn steps_action_summary(steps: &[AutomationStep]) -> String {
    let parts: Vec<&str> = steps
        .iter()
        .map(|s| match s {
            AutomationStep::OpenUrl { .. } => "open_url",
            AutomationStep::OpenFile { .. } => "open_file",
            AutomationStep::OpenApp { .. } => "open_app",
            AutomationStep::CopyToClipboard { .. } => "copy_to_clipboard",
            AutomationStep::CreateNote { .. } => "create_note",
        })
        .collect();
    parts.join(" + ")
}

async fn execute_step(step: &AutomationStep, app_handle: &tauri::AppHandle) -> Result<()> {
    match step {
        AutomationStep::OpenUrl { url } => {
            app_handle
                .opener()
                .open_url(url, None::<&str>)
                .map_err(|e| anyhow!(e))?;
            Ok(())
        }
        AutomationStep::OpenFile { path } => {
            app_handle
                .opener()
                .open_path(path, None::<&str>)
                .map_err(|e| anyhow!(e))?;
            Ok(())
        }
        AutomationStep::OpenApp { path } => {
            app_handle
                .opener()
                .open_path(path, None::<&str>)
                .map_err(|e| anyhow!(e))?;
            Ok(())
        }
        AutomationStep::CopyToClipboard { text } => {
            let mut clipboard =
                arboard::Clipboard::new().map_err(|e| anyhow!("clipboard init failed: {}", e))?;
            clipboard
                .set_text(text.clone())
                .map_err(|e| anyhow!("clipboard write failed: {}", e))?;
            Ok(())
        }
        AutomationStep::CreateNote { content } => {
            // 实现：写入到 documents 目录下的 memflow_notes.md
            use std::io::Write;
            
            let data_dir = app_handle
                .path()
                .document_dir()
                .map_err(|e| anyhow!("failed to get document dir: {}", e))?;
            
            let notes_path = data_dir.join("memflow_notes.md");
            
            // 确保目录存在
            if let Some(parent) = notes_path.parent() {
                std::fs::create_dir_all(parent)
                    .map_err(|e| anyhow!("failed to create notes dir: {}", e))?;
            }

            let mut file = std::fs::OpenOptions::new()
                .create(true)
                .append(true)
                .open(&notes_path)
                .map_err(|e| anyhow!("failed to open notes file: {}", e))?;
            
            let now = chrono::Local::now();
            let header = format!("\n\n## 📝 自动记录 ({})\n\n", now.format("%Y-%m-%d %H:%M:%S"));
            
            file.write_all(header.as_bytes())?;
            file.write_all(content.as_bytes())?;
            
            tracing::info!("笔记已保存到: {:?}", notes_path);
            Ok(())
        }
    }
}
