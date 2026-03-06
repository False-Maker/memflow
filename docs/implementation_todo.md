### MemFlow 实施待办清单（面向 `main` 最终形态）

本清单基于 `docs/architecture.md`，用于指导从当前状态迁移到目标架构与功能集。建议以「阶段」为单位安排 Milestone，每个阶段内部的任务可以按优先级拆分到具体迭代中。

---

### Phase 1 – `memflow-core`：统一 RuntimeContext 与 Embedding 管理

- **P1-1 统一 RuntimeContext 使用方式**
  - 梳理 `crates/memflow-core::context` 模块现有结构（`RuntimeContext` 字段、初始化路径、运行模式等）。
  - 搜索 MCP / Tauri / 其他调用中所有直接使用路径、配置或全局变量的代码（如直接拼接 DB 路径、资源目录）。
  - 将这些分散的路径/配置访问改为通过 `RuntimeContext` 统一获取，保证后续可以轻松切换运行模式（本机 / 只读 / 测试）。
  - 补充或更新单元测试，确保在不同运行模式下的路径解析正确。

- **P1-2 把 Embedding 管理下沉到 `memflow-core::ai`**
  - 找出 MCP 中当前管理 Embedding 的代码（例如全局 `EMBEDDING_MODEL OnceLock`、`embed_with_local_model` 等）。
  - 在 `memflow-core::ai` 中新增统一的 Embedding Provider 抽象：
    - `get_global_embedding_model(ctx: &RuntimeContext) -> impl EmbeddingProvider`。
    - 负责模型下载缓存（`resource_dir/models`）与并发访问锁。
  - 将 MCP / Tauri 等所有调用 Embedding 的位置改为通过 `memflow-core::ai` 访问，而不是自行管理模型。
  - 为 Embedding 模块补充基础测试（模型加载失败、路径错误、并发访问等场景）。

---

### Phase 2 – OCR 增强模块与桌面 OCR 流水线

- **P2-1 核对 `ocr_enhance` 模块实现**
  - 检查 `crates/memflow-core/src/ocr_enhance.rs` 是否与 `docs/ocr_enhancement.md` 一致。
  - 核心能力需要包括：
    - 图像预处理：灰度化、对比度增强、二值化等。
    - 文本后处理：符号纠错、括号修复、空白规范。
    - 代码检测与语言识别：`is_likely_code` + `detect_language`。
    - 质量评估：`calculate_cer` / `calculate_wer` / `evaluate_ocr_quality`。
  - 若有缺少或实现偏差，按文档补齐并在模块内部添加注释标明用途和适用场景。

- **P2-2 集成到 `src-tauri` 的 OCR 流水线**
  - 在 `src-tauri` 中定位 OCR 流水线核心模块（例如 `ocr_worker.rs`）。
  - 调整处理步骤，确保统一走：
    1. 加载截图。
    2. 调用 `ocr_enhance::preprocess_terminal_image`（或其他预处理入口）。
    3. 调用 OCR 引擎获取原始文本。
    4. 对疑似代码内容调用 `ocr_enhance::postprocess_terminal_text` 等后处理函数。
    5. 计算 CER/WER，并记录到日志/DB（便于回归分析）。
    6. 通过 `redact` 模块做隐私脱敏。
  - 确保最终写入 `activity_logs` 的 OCR 文本已经过增强与脱敏处理。

- **P2-3 测试与性能基准** ✅ 已完成
  - 已新增 OCR 增强单元测试：
    - `crates/memflow-core/tests/ocr_enhance_test.rs`
    - 测试覆盖：预处理、后处理、CER/WER 计算、语言检测、代码识别等
  - 已新增性能基准：
    - `crates/memflow-core/benches/cer_benchmark.rs`
    - 度量：CER/WER 计算、postprocess、is_likely_code、detect_language、evaluate_ocr_quality
  - 可通过 `cargo bench` 运行基准测试，通过 `cargo test` 运行单元测试

---

### Phase 3 – MCP Server：对齐 Tool Contract v1

- **P3-1 完整实现并增强 `search_memory` 工具**
  - 对照 `doc/MCP_TOOL_CONTRACT_v1.md`，梳理当前 `search_memory` 实现：
    - 支持的入参：`query`（必填）、`limit`、`mode`（`hybrid` / `semantic` / `keyword`）、`app_name`、`keywords`、`date_range`、`has_ocr` 等。
    - 输出结构：`content: [{ type: "text", text: "..." }]`。
  - 将搜索逻辑统一委托给 `memflow-core::ai::rag::HybridSearch`：
    - 根据 `mode` 决定具体策略（仅语义、仅关键词、混合）。
    - 对 filters（app / 时间范围 / 是否有 OCR 文本）进行组合过滤。
  - 增加对边界条件的处理（空 query、limit 过大、无结果等），返回稳定、易读的摘要文本。

- **P3-2 梳理 `get_recent_activity` / `get_active_window_context` / `get_related_context`**
  - `get_recent_activity`：
    - 使用 `db::list_activities_by_time` 从 `activity_logs` 中拉取最近 N 分钟的活动。
    - 统一通过 `redact` 做脱敏。
    - 将结果拼成一段便于 LLM 理解的时间线文本。
  - `get_active_window_context`：
    - 从最近的活动记录与终端日志缓存中查找当前前台窗口相关的 OCR 文本与终端输出。
    - 组合为结构紧凑的上下文说明，限制长度，避免 prompt 爆炸。
  - `get_related_context`：
    - 调用 RAG（向量搜索 + 可能的关键词过滤）找到与给定 query 最相关的片段。
    - 对每条结果按照 `max_chars_per_item` 截断，并标注来源（时间 / app）。
  - 为上述工具补充或完善测试，覆盖典型使用场景和错误情况。

- **P3-3 MCP 结构与测试套件**
  - 恢复并规范 `crates/memflow-mcp` 的模块结构：
    - `context.rs`：`McpContext`，负责推断 app 目录、DB 路径、资源目录。
    - `protocol.rs`：JSON-RPC 解析 / 响应构建 / 错误码表。
    - `server.rs`：主循环与分发逻辑。
    - `tools.rs`：每个工具一个独立 handler，内部调用 `memflow-core`。
  - 检查并补齐 `tests/` 中的关键测试：
    - `schema_validation_test`：请求/响应结构符合 Tool Contract。
    - `perf_benchmark`：在典型负载下的延迟与吞吐。
    - `tauri_concurrency_test`（如存在）：验证与桌面端并发访问时的稳定性。
  - 根据 `MCP_TOOL_CONTRACT_v1.md` 实现或确认所有错误码（`-32000` ~ `-32008`），并处理别名工具名（如 `search_visual_memory` / `get_recent_activities`）的兼容和 deprecate 日志。

- **P3-4 Core 探活与降级策略（为 MCP / IDE 形态做铺垫）** ✅ 已完成
  - 已在 `memflow-core::db` 中实现 `check_core_health()` 函数，通过数据库连接检查 Core 可用性。
  - MCP 启动时会先探活 Core：
    - 如果 Core 可用，记录 "running in full mode"
    - 如果 Core 不可用，记录警告但继续运行在降级模式（允许 schema 查询等基本功能）
  - 新增错误码 `-32009` (CORE_UNAVAILABLE) 和 `-32010` (DEGRADED_MODE)。

---

### Phase 4 – 终端输出捕获与系统环境检测

- **P4-1 终端输出捕获与 DB 集成**
  - 回顾 `crates/memflow-core/src/terminal.rs` 及 `src-tauri` 中相关采集逻辑：
    - 在 Windows 上通过 UIA / 控件树读取终端窗口文本。
    - 按窗口/进程维度拆分不同终端 session。
  - 确认终端日志表结构：
    - 字段示例：`id`, `timestamp`, `terminal_session_id`, `app_name`, `window_title`, `text`。
    - 针对常用查询字段建立索引（如 `terminal_session_id` + `timestamp`）。
  - 将采集到的终端输出统一写入该表，并通过 `redact` 做脱敏。

- **P4-2 MCP `get_terminal_output` 工具**
  - 实现 `get_terminal_output`：
    - 根据当前会话/窗口 ID，从终端日志表中读取最近 N 行输出。
    - 若找不到终端，返回错误码 `-32004`。
    - 若因权限问题失败，返回错误码 `-32005`。
  - 输出为纯文本，适合直接拼接进 LLM prompt。
  - 为该工具添加测试，覆盖：
    - 有正常输出、无输出、终端已关闭、权限不足等情况。

- **P4-3 系统环境检测（桌面端 + MCP）**
  - 在 `src-tauri/src/system_helpers.rs` 等模块中：
    - 收集 OS 版本、硬件信息（CPU/内存）、关键开发工具版本（Node、Rust、Python 等），以及常见端口占用信息（如可行）。
    - 提供对外的结构化 API，供 MCP / 前端调用。
  - 实现 MCP `get_system_environment` 工具：
    - 调用桌面端系统信息 API。
    - 将结果组装成简洁易读的文本摘要（按类别分段：OS / 硬件 / 开发环境 / 端口等）。
  - 为系统环境工具添加测试，至少覆盖：
    - 正常返回。
    - 部分信息采集失败时的降级与错误提示（例如无法访问某些硬件信息时仍保持整体可用）。

---

### Phase 5 – 桌面前端 UI 与 Agent 能力

- **P5-1 时间线 & 搜索视图**
  - 完善 `Timeline` 组件：
    - 使用 Tauri 命令 `list_activities(time_range, paging)` 加载数据。
    - 支持按 app / 时间范围过滤，与搜索结果联动高亮。
  - 完善搜索视图（`QnA` / 搜索区块）：
    - 中央搜索输入框调用 Tauri `search_memory(query, mode, filters, limit)`。
    - UI 层支持选择 `hybrid` / `semantic` / `keyword` 模式。
    - 在结果列表中展示 app / 时间 / OCR 摘要。
    - 为“搜索闭环”预留交互：搜索 → 命中结果 → 一键跳转时间轴那一刻。

- **P5-2 上下文侧边栏与聊天**
  - 打磨 `ContextSidebar` / `ChatHistoryModal`：
    - 展示最近活动 / 当前上下文摘要。
    - 至少接入本地 `get_recent_activity` 或 RAG 的“问当前上下文”入口。
    - 预留将来与 MCP `get_related_context` 集成的接口（例如在配置中切换来源）。

- **P5-3 Agent 提案与执行 UI**
  - 完成 `AgentProposalModal`：
    - 从 `memflow-core::agent::propose_automation` 拉取提案列表。
    - 展示每条提案的描述、风险等级、预期动作（打开 URL/文件/应用、生成笔记等）。
    - 为每条提案提供明确的确认执行/取消入口。
  - 完成 `AgentHistoryModal`：
    - 展示 `agent_executions` 历史记录，包括状态、耗时、错误信息。
    - 支持按时间/状态过滤与手动刷新。

- **P5-4 Agent 后端命令与安全配置**
  - 在 `src-tauri` 中暴露并实现以下命令：
    - `agent_propose(params)`
    - `agent_execute(proposal_id)`
    - `agent_cancel(execution_id)`
    - `agent_list_executions()`
  - 确保所有执行动作都遵守安全策略：
    - 仅当 `risk_level = "low"` 时允许执行。
    - 所有动作经过 allowlist 校验并记录审计日志。
  - 补齐前端设置页：
    - `get_config` / `update_config` 对接 Tauri。
    - 提供 OCR 增强、隐私策略、Agent 执行权限等开关的 UI。

---

### Phase 5.5 – 隐私与控制（产品级体验）

- **P5.5-1 采集控制：托盘 / 快捷键暂停与恢复** ✅ 已完成
  - 桌面端已实现常驻托盘图标，提供「暂停录制 / 恢复录制」入口。
  - 已实现全局快捷键 `Ctrl+Shift+P`，一键切换录制状态，切换时在 UI / 日志中清晰标记。
  - Core 侧明确"暂停状态"的含义（停止入库 / 停止截图 / 仅保留 MCP 查询等），并在配置中记录。

- **P5.5-2 应用黑名单 / 白名单**
  - 提供以 app 名称 / 进程名 / 窗口标题为维度的黑名单 / 白名单配置。
  - 采集流水线在写入 `activity_logs` 前统一检查该配置，对被屏蔽对象完全不入库或仅保留元数据（视策略而定）。
  - 在设置页中提供可视化管理与一键添加（例如从最近活动中选中 app 加入黑名单）。

- **P5.5-3 保留策略与自动清理**
  - 支持按时间（仅保留最近 N 天）与空间（数据库 / 资源目录不超过 N GB）两种保留策略。
  - 在 Core 中实现定期 GC 任务，按策略删除或压缩旧记录与相关截图 / 向量等资源。
  - 在设置页中展示当前磁盘占用估算与下一次 GC 计划时间。

- **P5.5-4 一键清理与数据导出**
  - 提供“一键清理所有数据”的入口，明确提示影响范围（数据库 / 截图 / 向量文件等）。
  - 支持将历史记录导出为 JSON / Markdown（带时间轴结构），方便用户迁移或自备份。
  - 完成导出 / 清理后提供结果摘要（条目数 / 文件大小），并在日志中记录操作。

---

### Phase 6 – 收尾与清理

- **P6-1 对照 `dev` 分支功能清单收口** ✅ 已完成
  - 已逐项核对 `architecture.md` 第 9 节清单：
    - ✅ OCR 增强模块已完整集成 (`crates/memflow-core/src/ocr_enhance.rs`)
    - ✅ MCP Tool Contract v1 已全部实现（含错误码与别名工具）
    - ✅ 终端输出捕获与系统环境检测已打通：桌面端 + MCP
    - ✅ Proactive Context 与 Agent 相关 UI/后端已与当前架构统一
  - 补充文档：`docs/MCP_TOOL_CONTRACT_v1.md`
  - 补充测试：`crates/memflow-mcp/tests/`

- **P6-2 清理遗留代码与文档更新** ✅ 已完成
  - 已清理临时日志文件（5个 .log 文件）
  - 已创建文档：
    - `docs/MCP_TOOL_CONTRACT_v1.md` - MCP 工具契约 v1 文档
  - 已补充测试：
    - `crates/memflow-mcp/tests/mcp_tools_test.rs` - MCP 工具集成测试
    - `crates/memflow-mcp/tests/schema_validation_test.rs` - JSON-RPC 协议验证测试
  - 已更新：`crates/memflow-mcp/Cargo.toml` 添加测试依赖

---

> 建议：可以将本清单中的任务同步到 issue tracker（如 GitHub Issues），按「Phase + 编号」作为标签或前缀，便于多人协作和追踪进度。

