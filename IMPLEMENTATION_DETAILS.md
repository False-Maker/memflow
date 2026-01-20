# MemFlow 优化与新功能技术实现方案

本文档为 `FUNCTIONAL_OPTIMIZATION_PROPOSAL.md` 中提出的优化建议与新功能提供具体的**技术设计与实施方案**。

---

## 1. 体验优化 (UX Improvements)

### 1.1 日历热力图 (Calendar Heatmap)

**目标**：在首页直观展示每日活动密度（类似 GitHub Contribution Graph）。

**状态**：✅ 已实现（后端聚合 API + 前端热力图 + 点击联动跳转时间轴）

**技术实现**：

*   **后端 (Rust)**:
    *   **API**: `db::get_activity_heatmap_stats(year: Option<i32>) -> Result<Vec<HeatmapData>>`
    *   **SQL**:
        ```sql
        SELECT date(timestamp, 'unixepoch', 'localtime') as day, count(*) as value 
        FROM activity_logs 
        WHERE timestamp BETWEEN ? AND ? 
        GROUP BY day
        ```
    *   **Cache**: 这种聚合查询可缓存 5-10 分钟。

*   **前端 (React)**:
    *   **库**: 引入 `react-calendar-heatmap`。
    *   **组件**: 新建 `ActivityHeatmap.tsx`，在 `FlowState` 或首页顶部加载。
    *   **交互**: 点击某个格子（日期），联动 `Timeline` 组件跳转到该日期的 `from_ts` / `to_ts`。

### 1.2 自然语言语义过滤器 (Semantic Filters)

**目标**：支持 "Show me PDF files I read last week" 这样的查询。

**状态**：✅ 已实现（后端 LLM 意图解析 + 前端 AI 搜索入口；当前为按钮触发，不是自动切换模式）

**技术实现**：

*   **后端 (Rust)**:
    *   **Intent Parser**: 在 `ai` 模块新增 `parse_query_intent(user_query: &str) -> FilterParams`。
    *   **LLM Prompt**:
        ```text
        You are a query parser. Convert user input into JSON filters.
        Input: "Show me PDF files I read last week"
        Output: { "app_name": "pdf", "keywords": [], "date_range": "last_week" }
        ```
    *   **Dynamic Query**: 将解析出的 JSON 映射到 `db::search_activities` 的 SQL `WHERE` 子句。

*   **前端**:
    *   复用现有搜索框。当用户输入长句时，自动切换为“AI 语义模式”（UI上显示一个小 Sparkle 图标）。

### 1.3 沉浸式画廊 (Gallery View)

**目标**：纯图模式快速回溯。

**状态**：✅ 已实现（VirtuosoGrid + 复用 activities 数据源）

**技术实现**：

*   **前端**:
    *   **组件**: `GalleryView.tsx`。
    *   **布局**: 使用 CSS Grid (`grid-template-columns: repeat(auto-fill, minmax(200px, 1fr))`)。
    *   **性能**: 必须结合 `react-virtuoso` 的 `VirtuosoGrid` 组件，因为图片数量巨大。
    *   **数据源**: 复用 `useApp().activities`，但 `itemContent` 渲染为纯 `<img />` + 悬停显示元数据。

---

## 2. 新功能规划 (New Features)

### 2.1 情感与专注度分析 (Focus Analytics)

**目标**：分析专注度（APM）与应用切换频率。

**状态**：✅ 已实现（device_query 采样 + focus_metrics 表 + 统计页趋势图；支持开关）

**技术实现**：

*   **输入监控 (Backend)**:
    *   **Crate**: 引入 `device_query` (跨平台输入状态查询)。
    *   **Privacy**: **绝不记录具体按键**。只记录 `key_press_count` 和 `mouse_move_distance`。
    *   **Loop**: 在 `recorder.rs` 的循环中，每秒采样一次输入状态，累加计数器。
    *   **安全隔离**: 必须在 `tokio::spawn` 中执行，避免阻塞主录制循环。

*   **数据模型**:
    *   新表 `focus_metrics`:
        ```sql
        CREATE TABLE focus_metrics (
            timestamp INTEGER PRIMARY KEY,
            apm INTEGER,             -- Actions Per Minute
            window_switch_count INTEGER,
            focus_score REAL         -- 计算出的专注分数 (0-100)
        );
        ```

*   **算法**:
    *   `Focus Score = (APM * w1) - (SwitchCount * w2)`。
    *   高频切换窗口通常意味着注意力分散或多任务处理。

### 2.2 主动式 AI 助理 (Proactive Context Assistant)

**目标**：根据当前窗口自动推送相关信息。

**状态**：✅ 已实现（后台触发 + emit 推送 + 右侧栏 UI + 4 秒防抖显示；受 AI/隐私/开关控制）

**技术实现**：

*   **事件驱动 (Backend)**:
    *   **Trigger**: 在 `recorder.rs` 中，当检测到 `window_title` 发生显著变化（Levenshtein 距离 > 阈值）时。
    *   **Action**: 触发 `ai::proactive_search(current_window_context)`。
    *   **Event**: 通过 `emit("context-suggestion", results)` 推送给前端。
    *   **安全隔离**: AI 请求必须异步执行，并设置超时，防止挂起。

*   **UI (Frontend)**:
    *   **Sidebar**: 新增右侧边栏 `ContextSidebar`。
    *   **Content**: 显示 "Related Memories"（相关记忆）和 "Suggested Actions"（建议操作）。
    *   **防抖**: 避免频繁切换窗口导致 UI 闪烁，前端需做 3-5秒 的防抖显示。

### 2.3 沉浸式时光机 (Immersive Replay)

**目标**：视频化回放工作流。

**状态**：✅ 已实现（requestAnimationFrame 播放 + 倍速/暂停/进度；预取采用批量取路径 + URL 缓存）

**技术实现**：

*   **原理**: 并不是真的存储视频（太占空间），而是以高帧率播放截图序列。
*   **前端**:
    *   **Player**: 使用 `requestAnimationFrame` 快速切换 `img.src`。
    *   **Prefetch**: 预加载前后 10 张截图到内存 (`new Image()`)，保证播放流畅。
    *   **UI**: 提供播放/暂停按钮，以及 1x/2x/5x 速度控制。

---

## 3. 实施路线图 (Roadmap)

### Phase 1: 基础体验 (Week 1-2)
1.  [x] 实现 `db::get_activity_heatmap_stats` API。
2.  [x] 前端集成 `react-calendar-heatmap`。
3.  [x] 优化搜索框，接入简易的规则解析（如识别 "app:chrome"）。

### Phase 2: 专注度分析 (Week 3-4)
1.  [x] 引入 `device_query`，在 recorder 中打通输入计数。
2.  [x] 创建 `focus_metrics` 表。
3.  [x] 前端 `FlowState` 增加专注度趋势图。

### Phase 3: AI 增强 (Week 5-8)
1.  [x] 实现 `ai::parse_query_intent` (需接入 LLM)。
2.  [x] 开发 `Proactive Context` 后台触发逻辑。
3.  [x] 右侧边栏 UI 开发。

---

## 4. 质量保障与测试方案 (Quality Assurance)

为了确保新功能的可靠性，并不影响现有核心功能，我们将为每个新模块引入严格的单元测试。

### 4.1 单元测试策略

#### 日历热力图 (Heatmap)
*   **测试目标**: 确保 SQL 聚合逻辑正确，特别是跨年、空数据和时区处理。
*   **当前**: ✅ 已有后端测试覆盖（含跨年样例）。
*   **测试代码示例**:
    ```rust
    #[tokio::test]
    async fn test_heatmap_aggregation_logic() {
        // Setup: 创建内存数据库并插入模拟数据
        let pool = setup_memory_db().await;
        insert_mock_activity(&pool, "2023-12-31 23:59:59").await;
        insert_mock_activity(&pool, "2024-01-01 00:00:01").await;
        
        // Act: 调用聚合 API
        let stats = db::get_activity_heatmap_stats(&pool, Some(2024)).await.unwrap();
        
        // Assert: 验证结果
        assert_eq!(stats.len(), 1);
        assert_eq!(stats[0].count, 1);
        assert_eq!(stats[0].date, "2024-01-01");
    }
    ```

#### 语义过滤器 (Intent Parser)
*   **测试目标**: 验证 Intent Parser 能正确将自然语言解析为 JSON，且在 LLM 失败时有回退机制。
*   **当前**: ✅ 已补齐 JSON 解析（含代码围栏）单元测试，并在 LLM 失败时启用本地回退解析。
*   **测试代码示例**:
    ```rust
    #[test]
    fn test_query_intent_parsing() {
        let input = "Show me chrome usage last week";
        let intent = ai::parse_intent_mock(input); // Mock LLM response
        
        assert_eq!(intent.app_name, Some("chrome".to_string()));
        assert_eq!(intent.date_range, DateRange::LastWeek);
    }
    ```

#### 专注度评分 (Focus Score)
*   **测试目标**: 验证算法在边界条件下的表现（如 APM=0, APM极高）。
*   **当前**: ✅ 已有后端测试覆盖（clamp 与 switches 影响）。
*   **测试代码示例**:
    ```rust
    #[test]
    fn test_focus_score_calculation() {
        let score_high = calculate_focus_score(120 /*apm*/, 2 /*switches*/);
        let score_low = calculate_focus_score(10 /*apm*/, 20 /*switches*/);
        
        assert!(score_high > 80.0);
        assert!(score_low < 30.0);
    }
    ```

---

## 5. 核心代码变更风险评估与防护 (Core Impact Analysis)

实现 **“专注度分析”** 和 **“主动 AI 助理”** 需要在 `src-tauri/src/recorder.rs` 的主循环中注入代码。这是系统的心脏，任何阻塞或 Panic 都会导致录制停止。

### 5.1 风险点
1.  **阻塞风险**: 采集输入或 AI 请求耗时过长，拖慢截图频率。
2.  **崩溃风险**: 新引入的 crate (如 `device_query`) 可能在特定系统状态下 Panic。

### 5.2 安全变更范式 (Safety Pattern)

我们将采用 **“三层防御”** 模式来修改 `recorder.rs`，确保核心功能不受影响：

```rust
// src-tauri/src/recorder.rs - 变更示意

async fn recording_loop() {
    let mut interval = tokio::time::interval(Duration::from_millis(5000));
    
    loop {
        interval.tick().await;
        
        // --- 核心功能 (原有逻辑，保持不变) ---
        let capture_result = capture_and_save().await;
        // ... 处理截图结果 ...

        // --- 新功能注入点 (三层防御) ---
        
        // 防御层 3: Feature Flag (运行时开关)
        if config.enable_focus_analytics {
            // 防御层 1: 异步隔离 (不阻塞主循环)
            tokio::spawn(async move {
                // 防御层 2: 异常捕获 (防止 Panic 扩散)
                let result = std::panic::catch_unwind(|| {
                    // 执行输入采集或 AI 分析
                    input_monitor::tick(); 
                });
                
                if let Err(_) = result {
                    tracing::error!("Focus analytics module panicked! Recovered.");
                }
            });
        }
    }
}
```

通过这种方式，即使新功能模块完全崩溃，Recorder 主循环依然会继续运行，仅在日志中记录错误，**实现了对原功能的零负面影响**。
