# MemFlow 核心接入与填补详细执行计划 (Technical Specification)

本计划旨在将已搭建的基础设施（UIA、事件驱动、Prompt 引擎）与核心业务逻辑（Recorder、Agent）进行物理连接，完成系统的“通电”工作。

## Phase 1: UIA 文本提取引擎实现 (Priority: High)
**目标**: 填充 `src-tauri/src/uia.rs` 中的空函数，实现真正的 Windows 原生文本提取。

### 1.1 COM 环境初始化与资源管理
- **逻辑**: 使用 `windows::Win32::System::Com`。
- **实现**: 
    - 在 `get_window_text_content` 入口处调用 `CoInitializeEx(None, COINIT_MULTITHREADED)`。
    - 使用 `defer!` 或 `Drop` trait 确保 `CoUninitialize` 被调用，防止内存泄漏。

### 1.2 遍历控件树 (The Tree Walker)
- **逻辑**: 
    1. 创建 `CUIAutomation` 实例 (`IUIAutomation`)。
    2. 调用 `ElementFromHandle(hwnd)` 获取目标窗口的根元素。
    3. 创建 `Condition`：仅匹配 `ControlType` 为 `Text`, `Edit`, `Document` 的元素（过滤按钮、滚动条等噪音）。
    4. 使用 `FindAll(TreeScope_Descendants, condition)` 批量获取所有文本节点。
- **输出**: 将所有节点的 `Name` 或 `Value` 属性拼接为结构化字符串（保留换行符）。

## Phase 2: 录制循环重构 (Priority: High)
**目标**: 将 `recorder.rs` 从“死板轮询”升级为“事件驱动 + 混合采集”。

### 2.1 引入事件流
- **修改**: `recorder.rs` 的 `recording_loop`。
- **逻辑**:
    ```rust
    // 初始化事件监听
    let mut event_recorder = win_event::EventDrivenRecorder::new(config);
    let mut event_rx = event_recorder.start();
    
    loop {
        tokio::select! {
            // A. 响应系统事件 (窗口切换/标题变化)
            Some(event) = event_rx.recv() => {
                // 防抖: 如果 500ms 内连续收到事件，只处理最后一个
                if debounce_timer.elapsed() > Duration::from_millis(500) {
                    capture_and_save(event.hwnd).await;
                }
            }
            // B. 兜底心跳 (每 30s)
            _ = tokio::time::sleep(Duration::from_secs(30)) => {
                capture_and_save(get_foreground_window()).await;
            }
        }
    }
    ```

### 2.2 混合采集管线 (Hybrid Pipeline)
- **修改**: `capture_and_save` 函数。
- **流程**:
    1. **Attempt UIA**: 调用 `uia::get_window_text_content(hwnd)`。
    2. **Branching**:
        - **Success**: 获取到文本 -> 存入数据库 `ocr_text` 字段 -> **跳过截图 OCR 步骤** (节省 90% CPU)。
        - **Failure** (返回 None): 回退到原有的 `xcap` 截图 -> `calculate_phash` -> `queue_ocr` 流程。

## Phase 3: Agent 智能架构升级 (Priority: Medium)
**目标**: 让 Agent 使用 `prompt_engine` 和 `tools`。

### 3.1 提示词模板集成
- **修改**: `src-tauri/src/agent/mod.rs`。
- **逻辑**: 
    - 废弃代码中的 `format!("...{}...", context)`。
    - 引入 `ai::prompt_engine::PromptTemplate`。
    - 在 `propose_automation` 中构建变量 Map (`context`, `user_query`, `time`) 并渲染模板。

### 3.2 工具注册与执行
- **修改**: `src-tauri/src/agent/mod.rs`。
- **逻辑**:
    - 在 Agent 初始化时创建 `ToolRegistry` 并加载所有默认工具。
    - 解析 LLM 返回的 JSON 提案。
    - 遍历 `steps`，通过 `registry.get(step.action).execute(step.args)` 动态调用工具，而非硬编码的 `match` 语句。

## Phase 4: 验证与测试
1.  **UIA 测试**: 打开记事本输入一段话，运行 MemFlow，检查数据库 `ocr_text` 是否准确包含该段话且无 OCR 错字。
2.  **事件测试**: 快速切换窗口，观察日志是否立即触发录制（无延迟）。
3.  **静止测试**: 保持鼠标不动，观察日志是否停止刷屏（无冗余）。

**此计划已就绪，是否立即开始 Phase 1 (UIA 实现)？**
