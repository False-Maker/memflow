# 向量转换定时任务方案

## 背景

当前系统存在一个问题：OCR 文本生成后不会自动转换为向量数据，导致语义搜索功能无法正常工作。

### 问题分析

| 功能 | 是否需要预存向量 |
|------|------------------|
| Keyword 搜索 | ❌ 不需要 |
| Hybrid 搜索 | ⚠️ 部分需要（有 BM25 结果时可正常工作） |
| Semantic 搜索 | ✅ **完全依赖**预存向量 |
| Proactive Context | ✅ **完全依赖**预存向量 |

当前系统中：
- ✅ 新活动会进行 OCR 识别
- ✅ OCR 文本存入 `activity_logs` 表
- ❌ **OCR 文本不会自动转换为向量**
- ❌ 历史数据永远不会被转换

### 向量模型使用位置

1. **MCP 搜索工具** (`crates/memflow-mcp/src/tools.rs`)
   - Semantic 模式：纯语义搜索
   - Hybrid 模式：混合搜索（需要向量评分）
   - `get_related_context`：获取相关上下文

2. **Proactive Context** (`src-tauri/src/proactive_context.rs`)
   - 主动上下文推荐

3. **桌面端问答** (`src-tauri/src/ai/mod.rs`)
   - 问答搜索

---

## 方案设计

### 1. 实现位置

在现有的 OCR Worker 中添加定时任务，因为 OCR Worker 已经长期运行，无需额外的进程管理。

### 2. 核心配置

在 `AppConfig` 中添加以下配置项：

```rust
pub struct AppConfig {
    // ... 现有字段 ...
    
    /// 向量化任务是否启用
    #[serde(default = "default_vectorize_enabled")]
    pub vectorize_enabled: bool,
    
    /// 向量化任务间隔（秒）
    #[serde(default = "default_vectorize_interval")]
    pub vectorize_interval: u64,
    
    /// 每批转换数量
    #[serde(default = "default_vectorize_batch_size")]
    pub vectorize_batch_size: i64,
}

fn default_vectorize_enabled() -> bool { true }
fn default_vectorize_interval() -> u64 { 300 } // 5分钟
fn default_vectorize_batch_size() -> i64 { 50 }
```

### 3. 数据库查询

新增数据库查询函数，用于获取待向量化的活动：

```rust
/// 获取需要向量化的活动
/// 条件：有 OCR 文本但没有向量记录
pub async fn get_pending_vectorize_tasks(limit: i64) -> Result<Vec<i64>> {
    let pool = db::get_pool().await?;
    
    let rows = sqlx::query_scalar!(
        r#"
        SELECT a.id FROM activity_logs a
        LEFT JOIN vector_embeddings v ON a.id = v.activity_id
        WHERE a.ocr_text IS NOT NULL 
          AND a.ocr_text != ''
          AND v.id IS NULL
        ORDER BY a.timestamp DESC
        LIMIT ?
        "#,
        limit
    )
    .fetch_all(&pool)
    .await?;
    
    Ok(rows)
}
```

### 4. Worker 实现

```rust
// ocr_worker.rs

const VECTORIZE_CHECK_INTERVAL_SECS: u64 = 300; // 默认5分钟
const VECTORIZE_BATCH_SIZE: i64 = 50;

async fn run_vectorize_worker(app_handle: AppHandle) {
    let mut ticker = interval(Duration::from_secs(VECTORIZE_CHECK_INTERVAL_SECS));
    
    loop {
        ticker.tick().await;
        
        // 获取配置
        let config = match app_config::get_config().await {
            Ok(c) => c,
            Err(_) => continue,
        };
        
        // 检查是否启用
        if !config.vectorize_enabled {
            continue;
        }
        
        // 获取待转换的活动
        let pending = db::get_pending_vectorize_tasks(
            config.vectorize_batch_size
        ).await;
        
        for activity_id in pending {
            // 获取活动文本
            let activity = match db::get_activity_by_id(activity_id).await {
                Ok(a) => a,
                Err(_) => continue,
            };
            
            let text = activity.ocr_text.unwrap_or_default();
            if text.is_empty() {
                continue;
            }
            
            // 生成向量并存储
            match vector_db::generate_embedding(&text).await {
                Ok(embedding) => {
                    if let Err(e) = vector_db::insert_embedding(activity_id, embedding).await {
                        tracing::warn!("向量存储失败 activity_id={}: {}", activity_id, e);
                    }
                }
                Err(e) => {
                    tracing::warn!("向量生成失败 activity_id={}: {}", activity_id, e);
                }
            }
        }
    }
}
```

### 5. 启动入口

在 `src-tauri/src/lib.rs` 中启动 Worker：

```rust
pub fn run() {
    // 现有代码...
    
    // 启动向量转换 worker
    tauri::async_runtime::spawn(async move {
        ocr_worker::run_vectorize_worker(app_handle.clone()).await;
    });
}
```

---

## 处理流程

```
┌─────────────────────────────────────────────────────┐
│              OCR Worker 主循环                      │
│  ┌─────────────────────────────────────────────┐   │
│  │ 定时器触发（默认 5 分钟间隔）                   │   │
│  └─────────────────────────────────────────────┘   │
│                       │                             │
│                       ▼                             │
│  ┌─────────────────────────────────────────────┐   │
│  │ 检查配置：vectorize_enabled                  │   │
│  │ 如果未启用：跳过本次                           │   │
│  └─────────────────────────────────────────────┘   │
│                       │                             │
│                       ▼                             │
│  ┌─────────────────────────────────────────────┐   │
│  │ SQL: 查询未向量化的活动                        │   │
│  │ - ocr_text IS NOT NULL                      │   │
│  │ - ocr_text != ''                            │   │
│  │ - id NOT IN vector_embeddings               │   │
│  │ - ORDER BY timestamp DESC                   │   │
│  │ - LIMIT batch_size                          │   │
│  └─────────────────────────────────────────────┘   │
│                       │                             │
│           ┌───────────┴───────────┐                │
│           ▼                       ▼                │
│  ┌─────────────────┐    ┌─────────────────┐        │
│  │ 逐条处理        │    │ 并发处理         │        │
│  │ for loop       │    │ (可选优化)       │        │
│  └─────────────────┘    └─────────────────┘        │
│           │                       │                │
│           └───────────┬───────────┘                │
│                       ▼                             │
│  ┌─────────────────────────────────────────────┐   │
│  │ 1. generate_embedding(text) → Vec<f32>     │   │
│  │ 2. insert_embedding(activity_id, vector)    │   │
│  │ 3. 失败记录日志，继续处理下一个               │   │
│  └─────────────────────────────────────────────┘   │
└─────────────────────────────────────────────────────┘
```

---

## 配置说明

| 配置项 | 类型 | 默认值 | 说明 |
|--------|------|--------|------|
| `vectorize_enabled` | bool | true | 是否启用定时向量转换任务 |
| `vectorize_interval` | u64 | 300 | 任务执行间隔（秒），默认 5 分钟 |
| `vectorize_batch_size` | i64 | 50 | 每批处理的活动数量 |

### 调整建议

- **首次运行**：建议设置较小的间隔（如 60 秒）和较大的批量（如 100），快速处理历史数据
- **日常运行**：间隔 5 分钟，批量 50，足以处理新增数据
- **性能敏感场景**：可临时禁用，或设置更长的间隔

---

## 错误处理

1. **单条失败不影响整批**：使用 `for` 循环逐条处理，某条失败记录日志后继续
2. **向量生成失败**：记录警告日志，跳过该条
3. **数据库错误**：使用 `?` 传播错误，由外层捕获
4. **配置读取失败**：跳过本次，继续等待下一次触发

---

## 监控与调试

### 日志关键词

- `vectorize_worker started` - Worker 启动
- `vectorize tasks found: N` - 发现待处理任务
- `vectorization completed: M/N` - 完成任务统计
- `向量生成失败 activity_id=X` - 生成失败警告
- `向量存储失败 activity_id=X` - 存储失败警告

### 手动触发

可以通过以下方式手动触发一次转换：

```rust
// 调试命令
#[tauri::command]
async fn trigger_vectorize() -> Result<String> {
    let pending = db::get_pending_vectorize_tasks(1000).await?;
    // ... 处理逻辑
    Ok(format!("处理了 {} 条", pending.len()))
}
```

---

## 后续优化

### 1. 并发处理

当前是串行处理，可以改为并发：

```rust
// 使用 futures::future::join_all
let tasks: Vec<_> = pending
    .iter()
    .map(|id| vectorize_single(*id))
    .collect();

let results = join_all(tasks).await;
```

### 2. 进度显示

在 UI 中显示向量化进度：

- 待处理总数
- 已处理数量
- 预计剩余时间

### 3. 增量与全量模式

- **增量模式**（默认）：只处理新数据
- **全量模式**：一次性处理所有历史数据（类似 rebuild_graph）

---

## 总结

| 特性 | 说明 |
|------|------|
| 触发方式 | 定时器触发（默认 5 分钟） |
| 处理方式 | 增量转换，只处理未向量化的记录 |
| 批量大小 | 可配置，默认 50 条/批 |
| 容错机制 | 单条失败不影响整批 |
| 依赖 | 需要 OCR 文本存在 |
| 性能影响 | 低，每次只处理少量记录 |
