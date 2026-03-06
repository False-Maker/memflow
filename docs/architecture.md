## MemFlow 架构设计（面向 `main` 最终形态）

### 1. 目标与范围

- **目标**：
  - 把 MemFlow 打造成一个本地运行的“记忆大脑”，统一管理屏幕活动、OCR 文本、终端输出等信息。
  - 提供给“人”和“AI”两种入口：
    - **桌面应用（Tauri + React）**：时间线回顾、搜索、智能回顾/自动化。
    - **MCP Server**：按照 MCP Tool Contract 提供标准工具给 LLM / IDE / Cursor。
- **范围**：
  - 统一在 `main` 分支落地，**完整集成 `dev` 分支已有和规划的功能**：
    - `memflow-core::ocr_enhance` OCR 增强模块。
    - MCP 工具集合：`search_memory` / `get_recent_activity` / `get_active_window_context` / `get_related_context` / `get_terminal_output` / `get_system_environment`。
    - 终端输出捕获与系统环境探测。
    - Agent 自动化、Proactive Context 等桌面端高级功能。

---

### 2. 顶层架构

#### 2.1 分层结构概览

- **核心引擎：`crates/memflow-core`**
  - 提供与 UI / 协议无关的核心能力：
    - 数据模型与数据库访问（SQLite）。
    - 向量库与 RAG 搜索。
    - OCR 增强与质量评估。
    - 终端输出处理（来自 `dev` 的 `terminal` 能力）。
    - 隐私脱敏与分析（`redact` / `focus_analytics`）。
    - Agent 自动化（`agent` 模块）。
    - 全局运行上下文 `RuntimeContext`。
- **桌面后端：`src-tauri`**
  - 负责 OS 交互与采集流水线：
    - 活动日志记录、截图采集、OCR 调用、终端输出捕获、系统环境探测。
    - 调用 `memflow-core` 完成入库、向量化、OCR 增强等。
    - 暴露 Tauri Commands 给前端。
- **桌面前端：`src`（React）**
  - 时间线视图、搜索界面、Agent 提案与执行 UI、Proactive Context 侧边栏等。
- **MCP Server：`crates/memflow-mcp`**
  - 实现 MCP 规范与 Tool Contract v1：
    - `search_memory`、`get_recent_activity`、`get_active_window_context`、`get_related_context`。
    - 规划中的 `get_terminal_output`、`get_system_environment`。
  - 协议解析与错误码管理，逻辑层调用 `memflow-core`。

---

### 3. 核心引擎 `memflow-core`

#### 3.1 模块划分

- **`context`**
  - 定义统一的 `RuntimeContext` 接口 / 实现，封装：
    - 数据库路径、截图目录、资源目录（模型、配置）。
    - 运行模式（本机 / 只读 / 测试环境）。
  - 所有调用方（Tauri / MCP / 将来的 HTTP）都通过 `RuntimeContext` 拿配置和资源路径。

- **`db`**
  - 使用 `sqlx` 封装 SQLite：
    - 表：`activity_logs`、`automation_proposals`、`agent_executions`、终端日志表等。
    - 提供面向业务的 API，而不是裸 SQL：
      - `get_activity_by_id`、`list_activities_by_time`、`insert_activity` 等。
      - 终端输出相关：如 `insert_terminal_output` / `get_recent_terminal_output`（整合 `dev` 的 `terminal.rs` 功能）。
  - 统一由 `get_pool()` 管理连接池与锁问题。

- **`vector_db`**
  - 封装向量存储与相似检索：
    - 写入 / 更新：`upsert_embedding(activity_id, embedding)`。
    - 搜索：`search_by_embedding(embedding, limit)`。
  - 对外只暴露接口，内部实现可随时替换（SQLite 表 / Faiss / 其他库）。

- **`ai`**
  - **`nlp` / `provider` / `prompt_engine` / `prompts` / `rag`**：
    - 统一抽象 LLM / Embedding Provider。
    - 提供 RAG 能力：`HybridSearch::search_with_embedding(query, embedding, limit)`。
    - 通过配置控制搜索模式：`hybrid` / `semantic` / `keyword`（对齐 MCP Tool Contract）。
  - **Embedding 模型管理（需补充）**：
    - 将当前 MCP 中的 `EMBEDDING_MODEL OnceLock` 抽到这里：
      - `get_global_embedding_model(ctx: &RuntimeContext) -> impl EmbeddingProvider`。
      - 负责模型下载缓存（`resource_dir/models`）与并发访问锁。

- **`ocr_enhance`（来自 `dev`，必须集成）**
  - 功能（参考 `docs/ocr_enhancement.md`）：
    - 图像预处理：灰度化、对比度增强、二值化。
    - 文本后处理：符号纠错、括号修复、空白规范。
    - 代码检测：`is_likely_code` + `detect_language` 支持多种语言。
    - 质量评估：`calculate_cer` / `calculate_wer` / `evaluate_ocr_quality`。
  - 目标：
    - 作为核心通用能力存放在 `memflow-core`，供 `src-tauri` 的 OCR 流水线与未来 HTTP/MCP 统一使用。
    - 在所有 OCR 场景中（尤其是终端 / 代码截图）统一走该模块提升精度。

- **`redact`**
  - 隐私脱敏：
    - 邮箱、手机号、Token、URL 等 PII 识别。
    - 对 OCR 文本与终端输出做统一过滤。
  - 提供：
    - `sanitize_ocr_text(raw: &str) -> String`。
    - `sanitize_terminal_text(raw: &str) -> String`。

- **`focus_analytics`**
  - 统计与分析：
    - 应用使用时间、窗口切换频率等。
    - 可供 UI 显示或将来 MCP 提供简要 summary。

- **`agent`**
  - 自动化提案与执行：
    - `propose_automation(params, ctx) -> Vec<AutomationProposalDto>`：基于最近活动 + OCR 文本，给出“自动生成笔记 / 一键恢复工作集”的提案。
    - `execute_automation(proposal_id, ctx) -> ExecutionResultDto`：安全执行低风险步骤（打开 URL、文件、应用、生成笔记等）。
    - `list_executions` / `cancel_execution`：审计与取消。
  - 内部特性：
    - 会话分段（`split_into_sessions`）、智能上下文选择。
    - `ToolRegistry` + Tool Trait（受 dify 风格启发的工具抽象）。
    - 严格的 allowlist + 风险等级（`risk_level = low` 才允许执行）。
  - 作用：
    - 被前端 Agent UI 调用。
    - 将来可以暴露为 MCP 工具（例如 `propose_automation`、`execute_automation`）——本设计先作为内部能力，MVP 不对外暴露写操作。

---

### 4. 桌面采集与服务：`src-tauri`

#### 4.1 采集流水线

- **负责模块（Rust）**：
  - `recorder.rs`：
    - 监听前台窗口变化，记录 `app_name` / `window_title` / 时间戳。
    - 调度截图任务。
  - `ocr_worker.rs`：
    - 加载截图 → `ocr_enhance::preprocess_terminal_image` → 调用 OCR 引擎 → 得到原始文本 → 对可能是代码的内容调用 `ocr_enhance::postprocess_terminal_text`。
    - 计算 OCR 质量指标（CER/WER），记录日志（便于回归）。
  - `proactive_context.rs`：
    - 主动生成“当前上下文”summary，供 UI / MCP 使用。
    - 重用 `memflow-core::ai::rag` / `focus_analytics`。
  - `scheduler.rs`：
    - 控制任务调度频率，防止 OCR / 向量化拖垮系统。
  - **终端捕获（来自 `dev`）**：
    - 引入终端捕获模块（原 `terminal.rs` 的逻辑）：
      - 在 Windows 上通过 UIA / 控件树读取终端窗口中的文本。
      - 按窗口 / 进程维度存储最近 N 行输出到数据库中的终端日志表。
      - 提供给 MCP 的 `get_terminal_output` 工具使用。

- **流水线步骤（活动记录）**：
  1. UI 事件 → 记录前台窗口元数据。
  2. 截图并交给 OCR worker。
  3. OCR worker：
     - 图像预处理（`ocr_enhance`）。
     - OCR 引擎识别。
     - 代码检测 + 文本后处理。
     - 隐私脱敏。
  4. `memflow-core::db` 写入 `activity_logs`。
  5. 调用 `memflow-core::ai` 做 embedding + `vector_db` 写入。

#### 4.2 Tauri Commands（API 面向前端）

- **只读命令**（安全暴露给前端）：
  - `list_activities(time_range, paging)` → 时间线。
  - `search_memory(query, mode, filters, limit)` → 使用 `HybridSearch`，支持 `hybrid` / `semantic` / `keyword` 模式（与 MCP `search_memory` 对齐）。
  - `get_activity_detail(id)` → 包含 OCR 文本（脱敏后）。
  - `get_recent_activity(minutes, limit)` → 复用 MCP 工具逻辑。
  - `get_active_window_context()` → 当前窗口 + 最近 OCR / 终端输出的简要文本。
- **Agent / Proactive 相关命令**：
  - `agent_propose(params)` → 调用 `memflow-core::agent::propose_automation`。
  - `agent_execute(proposal_id)` / `agent_cancel(execution_id)` / `agent_list_executions`。
- **配置与系统信息**：
  - `get_config` / `update_config`（包含 OCR 增强开关、隐私策略等）。
  - `get_system_environment()`（对 MCP 同名工具的本地封装，共用底层实现）。

---

### 5. 桌面前端：React（`src`）

#### 5.1 主要界面与组件

- **`Layout`**
  - 顶层布局，整合导航、侧边栏 `ContextSidebar`、主内容区。
- **时间线视图：`Timeline`**
  - 显示按时间排序的 `activity_logs`。
  - 支持按 app / 关键字过滤，与搜索结果联动高亮。
- **搜索视图：`QnA` / 搜索区块**
  - 中央搜索输入框：
    - 点击搜索 → 调用 Tauri `search_memory`（支持模式选择：`hybrid` / `semantic` / `keyword`）。
    - 结果列表展示 app / 时间 / OCR 摘要。
- **上下文与聊天：`ChatHistoryModal` / `ContextSidebar`**
  - 展示最近活动 / 当前上下文摘要。
  - 可选集成“问当前上下文”的入口（调用 MCP `get_related_context` 或本地 RAG）。
- **Agent 自动化：`AgentProposalModal` / `AgentHistoryModal`**
  - 展示 `agent::propose_automation` 生成的提案列表，支持用户确认执行。
  - 查看历史执行、状态和错误信息。
- **反馈与质量：`MessageRating` / `FeedbackModal`**
  - 针对搜索结果 / Agent 提案收集用户反馈，写入本地或日志，以便调整 prompt / 权重。

#### 5.2 前端与 MCP/本地引擎关系

- **搜索等读操作**：
  - 默认调用 Tauri Commands（离线优先，低延迟）。
  - MCP 主要给 IDE / LLM 使用，不直接暴露到 UI。
- **高级用法（将来）**：
  - 可在 UI 中提示用户“你也可以在 IDE 里通过 `search_memory` 工具访问这些记忆”，与 MCP 工具命名保持一致。

---

### 6. MCP Server：`crates/memflow-mcp`

#### 6.1 协议与结构

- 按 `doc/MCP_TOOL_CONTRACT_v1.md` 规范实现：
  - JSON-RPC 2.0 + MCP 2024-11-05。
  - 日志全部输出到 `stderr`，`stdout` 严格只输出 JSON-RPC 响应。
- **模块结构（建议回归 `dev` 结构）**：
  - `context.rs`：`McpContext`，从环境推断 app 目录、DB 路径、资源目录。
  - `protocol.rs`：JSON-RPC 解析 / 响应构建 / 错误码表。
  - `server.rs`：主循环与分发逻辑。
  - `tools.rs`：每个工具一个独立 handler，调用 `memflow-core`。
  - `tests/`：保留 `schema_validation_test`、`perf_benchmark`、`tauri_concurrency_test` 等 `dev` 中已有测试。

#### 6.2 工具集合（完整集成 `dev` Tool Contract）

- **`search_memory`（已实现，需增强）**
  - 功能：在本地记忆中做关键词 / 语义 / 混合搜索。
  - 输入：
    - `query`（必填）、`limit`、`mode`（`hybrid` / `semantic` / `keyword`）、`app_name`、`keywords`、`date_range`、`has_ocr`。
  - 实现要点：
    - 使用 `memflow-core::ai::rag::HybridSearch`，根据 `mode` 调整策略。
    - 支持 filters（app / 时间范围 / 是否有 OCR 文本）。
    - 输出 `content: [{type: "text", text: "..."}]` 的 summary 文本。

- **`get_recent_activity`**
  - 功能：返回最近 N 分钟的活动时间线。
  - 输入：`minutes`（默认 5，最大 30）、`limit`。
  - 实现：
    - 调用 `db::list_activities_by_time` + `redact`。
    - 输出一段可读文本，用于 IDE / LLM 回顾“刚刚发生了什么”。

- **`get_active_window_context`**
  - 功能：获取当前活跃窗口的 app 名、标题与最近相关 OCR / 终端输出。
  - 实现：
    - 从最近活动 / 终端日志缓存中查找。
    - 组合为一段紧凑的上下文文本。

- **`get_related_context`**
  - 功能：返回与 query 相关的精简上下文片段，适合直接拼进 LLM prompt。
  - 实现：
    - 调用 RAG。
    - 对每条截断到 `max_chars_per_item`，避免信息爆炸。

- **`get_terminal_output`（Phase 2 功能）**
  - 功能：捕获当前终端窗口最近 N 行输出。
  - 实现：
    - 在桌面端持续采集终端输出（`terminal` 模块），写入 DB。
    - MCP 通过 `db` 或专用 API 读取。
    - 输出纯文本，附带错误码：
      - `-32004`：终端未找到。
      - `-32005`：权限不足。

- **`get_system_environment`（Phase 2 功能）**
  - 功能：返回 OS 版本、硬件信息、关键开发工具版本、常见端口占用等。
  - 实现：
    - 在桌面端提供系统信息收集 API（`system_helpers.rs` 的能力）。
    - MCP 调用该 API 并转成文本返回。

- **错误码与兼容性**：
  - 完整实现 `MCP_TOOL_CONTRACT_v1.md` 中定义的标准与自定义错误码（`-32000` ~ `-32008`）。
  - 支持别名工具名（如 `search_visual_memory` / `get_recent_activities`），但标记为 deprecated，记录日志便于迁移。

---

### 7. 数据模型与存储

#### 7.1 SQLite 表

- **`activity_logs`**
  - 字段：`id`, `timestamp`, `app_name`, `window_title`, `ocr_text`, `screenshot_path` 等。
  - 索引：按时间和 app 建索引，配合 RAG / MCP。

- **`automation_proposals` / `agent_executions`**
  - 支持 Agent 提案与执行审计。

- **终端日志表**
  - 字段示例：`id`, `timestamp`, `terminal_session_id`, `app_name`, `window_title`, `text`。
  - 用于 `get_terminal_output`。

- **辅助表**
  - 向量索引（若存在）、统计表等。

#### 7.2 文件与目录

- 数据库文件：`{app_dir}/memflow.db`。
- 截图目录：`{app_dir}/screenshots/`。
- 模型资源：`{resource_dir}/models/`（embedding / OCR 等）。
- 日志：MCP / Tauri 日志均输出到各自 log 文件和 stderr。

---

### 8. 隐私与安全

- **本地优先**：
  - 所有数据（活动、OCR、终端输出）默认本地存储，不上传云端。

- **最小暴露**：
  - MCP 工具返回的内容默认是摘要文本，而不是完整 raw 日志，避免泄露敏感信息。
  - 可在配置中关闭某些来源（例如不记录密码管理器窗口）。

- **脱敏策略**：
  - 统一通过 `redact` 模块执行。
  - MCP 与前端均在返回前走一次脱敏。

- **执行安全（Agent）**：
  - 只有 `risk_level = "low"` 的提案可执行。
  - 所有执行动作（打开 URL / 文件 / 应用）都有 allowlist 校验与日志审计。
  - 执行在后台 task 中，支持取消。

---

### 9. `dev` 分支功能集成清单

为确保 `dev` 分支的成果在 `main` 最终形态中全部收口，这里列一份显式清单，对应本设计中的位置：

- **OCR 增强模块**
  - 文件：`crates/memflow-core/src/ocr_enhance.rs`、`docs/ocr_enhancement.md`。
  - 本设计：在 3.1 的 `ocr_enhance` 模块与 4.1 OCR 流水线中完整集成。

- **MCP Tool Contract v1**
  - 文件：`doc/MCP_TOOL_CONTRACT_v1.md`。
  - 本设计：在 6.2 中逐一列出所有工具（含 `get_terminal_output` / `get_system_environment`）、输入输出 Schema 和错误码，作为最终目标。

- **终端输出捕获**
  - 文件：`crates/memflow-core/src/terminal.rs`、`src-tauri` 相关采集逻辑。
  - 本设计：在 3.1 `db` / 4.1 流水线 / 6.2 `get_terminal_output` 中作为一等特性描述。

- **系统环境检测**
  - 文件：`src-tauri/src/system_helpers.rs` 等。
  - 本设计：在 4.2 配置与系统信息与 6.2 `get_system_environment` 中纳入。

- **MCP 服务器结构化实现与测试**
  - 文件：`crates/memflow-mcp/{protocol.rs, server.rs, prompts.rs}`、`tests/*`。
  - 本设计：在 6.1 中要求恢复模块化结构与测试套件。

- **OCR 集成测试与性能基准**
  - 文件：`src-tauri/tests/ocr_enhancement_integration.rs`、`crates/memflow-core/benches/cer_benchmark.rs` 等。
  - 本设计：在 OCR 模块设计中强调“测试/基准”为必需，保证质量回归。

- **Proactive Context 与 Agent 相关 UI / 后端**
  - 文件：`src-tauri/src/proactive_context.rs`、`src/components/AgentProposalModal.tsx` 等。
  - 本设计：在 3.1 `agent`、4.1 流水线、5.1 UI 中吸收。

---

### 10. 实施与迁移（简要）

- 阶段 1：在 `memflow-core` 中统一 `RuntimeContext` 与 Embedding 管理，将 MCP 的模型初始化迁入 `ai` 模块。
- 阶段 2：恢复并集成 `ocr_enhance` 模块到 Tauri OCR 流水线，同时补齐相关测试。
- 阶段 3：按 MCP Tool Contract 补全 MCP 工具逻辑，拆分 `main.rs` 为 `protocol` / `server` / `tools` 结构。
- 阶段 4：终端输出捕获与系统环境检测能力接回（桌面端 + MCP）。
- 阶段 5：前端 UI 与 Agent 功能打磨，统一走 `memflow-core` 能力，清理遗留 dead code。

