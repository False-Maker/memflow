# Memflow MCP 完善计划

## TL;DR

> **核心目标**：补齐 Phase 2 MCP 工具能力，统一工具契约，建立稳定可接入 Cursor/Claude 的 Developer MCP 服务。

**交付物清单：**
- 统一的 MCP Tool Contract 文档（v1.0）
- 5 个核心工具（search_memory / get_recent_activity / get_active_window_context / get_terminal_output / get_system_environment）
- 3 个 Prompt Resource（debug_context / visual_regression / implicit_knowledge）
- MCP 自动化测试套件（protocol + tool + regression 三层）
- 安全审计与可配置脱敏系统

**预估工作量**：3 个迭代周期（6-8 周）  
**关键路径**：Tool 契约统一 → 三工具实现 → 测试体系 → 集成验证  
**并行潜力**：中等（工具开发可并行，但共享 DB/schema 需要协调）

---

## Context

### 原始需求
基于 Memflow_Project_Documentation.docx 和 Memflow_Developer_MCP_Design_Spec.md.docx 的分析，项目需要：
1. Phase 2 实时感知能力（视觉上下文 + 终端输出）
2. 统一的 MCP Tool 契约对外暴露
3. 开发场景优化的 OCR 与 Prompt 资源
4. 安全审计与可观测性

### 现状基线
- ✅ MCP 基础协议已实现（JSON-RPC 2.0、tools/list、tools/call）
- ✅ search_memory / get_recent_activity / get_active_window_context 有基础实现
- ✅ SQLite + FTS5 + Vector DB 数据层完整
- ⚠️ Tool 命名与协议不一致（tools/list vs tools/call）
- ❌ get_terminal_output / get_system_environment 缺失
- ❌ Prompt Resource 体系不完整
- ❌ 缺乏自动化测试体系

### 技术栈约束
- **Backend**: Rust + Tokio + Tauri 2.0
- **Database**: SQLite (WAL mode) + SQLx
- **AI**: FastEmbed (BGESmallENV15) + ONNX Runtime
- **Protocol**: Model Context Protocol (MCP) 2024-11-05
- **Testing**: cargo test + Python integration tests

---

## Work Objectives

### Core Objective
构建稳定、可观测、符合 MCP 规范的 Developer 工具集，使 AI IDE 能够通过 Memflow 实时感知用户开发上下文。

### Concrete Deliverables
1. **Tool Contract v1.0** (`doc/MCP_TOOL_CONTRACT_v1.md`)
   - 正式工具名定义
   - JSON Schema 规范
   - 错误码体系
   - 降级行为说明

2. **5 个核心 MCP Tools**
   - `search_memory` - 混合检索（已有，需契约统一）
   - `get_recent_activity` - 最近活动时间线（已有，需优化）
   - `get_active_window_context` - 当前窗口上下文（已有，需优化）
   - `get_terminal_output` - 终端输出捕获（新建）
   - `get_system_environment` - 系统环境感知（新建）

3. **3 个 Prompt Resources**
   - `memflow://prompts/debug_context`
   - `memflow://prompts/visual_regression`
   - `memflow://prompts/implicit_knowledge`

4. **MCP 自动化测试套件**
   - `tests/mcp_protocol_test.rs` - 协议层测试
   - `tests/mcp_tool_test.rs` - 工具层测试
   - `tests/integration_test.py` - 集成回归测试

5. **安全与审计模块**
   - 调用审计日志 (`crates/memflow-core/src/audit.rs`)
   - 可配置脱敏规则
   - 工具级权限矩阵

### Definition of Done
- [x] 全部 5 个工具通过 Schema 验证（`mcp-cli validate`）
- [x] 工具调用成功率 > 95%，p95 延迟 < 2s
- [x] 100% 工具覆盖自动化测试
- [ ] Cursor/Claude Desktop 端到端验证通过

### Must Have
- 工具契约向后兼容（保留别名）
- 错误处理遵循 MCP 错误码规范
- Local-First 原则（数据不上云）
- Windows + macOS 双平台支持

### Must NOT Have (Guardrails)
- 不修改底层截图/录制架构
- 不引入新的外部 AI API 依赖（保持本地嵌入）
- 不做前端 UI 改动（专注 MCP Server）
- 不降低隐私脱敏级别

---

## Verification Strategy

> **UNIVERSAL RULE: ZERO HUMAN INTERVENTION**
> 所有验证必须通过自动化完成，禁止人工点击/确认。

### Test Strategy Decision
- **Infrastructure exists**: ✅ (cargo test + Python)
- **Automated tests**: Tests-after（现有代码无 TDD，新功能补充测试）
- **Framework**: cargo test + pytest

### Agent-Executed QA Scenarios (MANDATORY)

#### Scenario 1: Tool Contract Schema Validation
```
Tool: Bash
Preconditions: MCP Server 编译完成
Steps:
  1. cargo build -p memflow-mcp
  2. echo '{"jsonrpc":"2.0","method":"tools/list","id":1}' | cargo run -p memflow-mcp
  3. 解析输出，验证 tools 数组包含 5 个工具
  4. 验证每个工具的 name/description/inputSchema 不为空
Expected Result: 返回 5 个工具定义，Schema 结构正确
Evidence: .sisyphus/evidence/tools-list-output.json
```

#### Scenario 2: get_terminal_output 功能验证
```
Tool: Bash (curl + tmux)
Preconditions: 有活跃的终端会话
Steps:
  1. tmux new-session -d -s test_term 'echo "TEST_OUTPUT_12345" && sleep 10'
  2. 调用 get_terminal_output MCP 工具
  3. 验证返回文本包含 "TEST_OUTPUT_12345"
Expected Result: 成功捕获终端最近输出
Evidence: .sisyphus/evidence/terminal-output.json
```

#### Scenario 3: get_system_environment 验证
```
Tool: Bash
Preconditions: MCP Server 运行中
Steps:
  1. 调用 get_system_environment 工具
  2. 验证返回包含 os_version、memory_total、cpu_count
  3. 验证数值在合理范围（memory > 1GB, cpu >= 1）
Expected Result: 返回完整系统环境信息
Evidence: .sisyphus/evidence/system-env.json
```

#### Scenario 4: 错误处理验证
```
Tool: Bash
Preconditions: MCP Server 运行中
Steps:
  1. 调用不存在的工具 "nonexistent_tool"
  2. 验证返回 error.code = -32601 (Method not found)
  3. 调用工具时缺少必需参数
  4. 验证返回 error.code = -32602 (Invalid params)
Expected Result: 符合 MCP 错误码规范
Evidence: .sisyphus/evidence/error-codes.json
```

---

## Execution Strategy

### Parallel Execution Waves

```
Wave 1: 基础设施与契约（Week 1）
├── Task 1: Tool Contract 设计与文档化
├── Task 2: protocol.rs 重构（统一类型定义）
└── Task 3: 测试基础设施搭建

Wave 2: Phase 2 核心工具（Week 2-3）
├── Task 4: get_terminal_output 实现
├── Task 5: get_system_environment 实现
└── Task 6: 现有工具重构与契约对齐

Wave 3: 增强能力（Week 4-5）
├── Task 7: Prompt Resource 体系
├── Task 8: 安全审计模块
└── Task 9: OCR 开发场景优化

Wave 4: 测试与稳定化（Week 6-8）
├── Task 10: MCP 自动化测试套件
├── Task 11: 集成测试与性能调优
└── Task 12: Cursor/Claude 端到端验证
```

### Dependency Matrix

| Task | Depends On | Blocks | Can Parallelize With |
|------|------------|--------|---------------------|
| 1 (Contract) | None | 2, 4, 5, 6 | None |
| 2 (Protocol) | 1 | 4, 5, 6 | None |
| 3 (Test Infra) | None | 10, 11 | 1, 2 |
| 4 (Terminal) | 2 | 11 | 5, 6 |
| 5 (System Env) | 2 | 11 | 4, 6 |
| 6 (Refactor) | 1, 2 | 11 | 4, 5 |
| 7 (Prompts) | None | 11 | 4, 5, 6 |
| 8 (Audit) | None | 11 | 4, 5, 6, 7 |
| 9 (OCR) | None | 11 | 4, 5, 6, 7, 8 |
| 10 (Tests) | 3 | 11 | 4, 5, 6, 7, 8, 9 |
| 11 (Integration) | 4, 5, 6, 7, 8, 9, 10 | None | None |
| 12 (Validation) | 11 | None | None |

**Critical Path**: 1 → 2 → 4/5/6 → 11 → 12

---

## TODOs

### Task 1: Tool Contract 设计与文档化

**What to do:**
1. 创建 `doc/MCP_TOOL_CONTRACT_v1.md`
2. 定义正式工具名列表（去除复数形式、统一命名风格）
3. 定义每个工具的 JSON Schema（参数类型、约束、默认值）
4. 定义错误码体系（MCP 标准 + 业务错误）
5. 定义降级行为（DB 锁时、无数据时、OCR 失败时）
6. 保留向后兼容别名策略

**Must NOT do:**
- 不删除现有工具实现
- 不改前端 Tauri 命令接口

**Recommended Agent Profile:**
- **Category**: `writing`
- **Skills**: [`docx`]
  - `docx`: 生成结构化 Markdown 文档

**Parallelization:**
- **Can Run In Parallel**: NO
- **Parallel Group**: Sequential
- **Blocks**: Task 2, 4, 5, 6
- **Blocked By**: None

**References:**
- `doc/Memflow_Developer_MCP_Design_Spec.md.docx` - 原始设计要求
- `doc/Memflow_修改方案分析报告.md` - 差距分析
- `crates/memflow-mcp/src/protocol.rs` - 当前协议类型
- `crates/memflow-mcp/src/main.rs:236-318` - 当前 tools/list 实现
- MCP Specification 2024-11-05 - JSON Schema 规范

**Acceptance Criteria:**
- [x] `doc/MCP_TOOL_CONTRACT_v1.md` 文件存在且包含全部 5 个工具定义
- [x] 每个工具有完整的 inputSchema、outputSchema、errorCodes
- [x] 包含向后兼容策略说明
- [x] 通过内部评审（2+ 人 review）

**Agent-Executed QA Scenarios:**
```
Scenario: Tool Contract 文档完整性
  Tool: Bash
  Preconditions: 无
  Steps:
    1. cat doc/MCP_TOOL_CONTRACT_v1.md
    2. grep -c "### Tool:"  # 应返回 5
    3. grep "search_memory\|get_recent_activity\|get_active_window_context\|get_terminal_output\|get_system_environment"
    4. grep "errorCodes\|inputSchema\|outputSchema"
  Expected Result: 文档结构完整，包含全部必需字段
  Evidence: .sisyphus/evidence/tool-contract-check.txt
```

**Commit**: YES
- Message: `docs(mcp): add Tool Contract v1.0`
- Files: `doc/MCP_TOOL_CONTRACT_v1.md`

---

### Task 2: Protocol 层重构

**What to do:**
1. 在 `protocol.rs` 添加统一的 Tool 类型定义
2. 添加工具名常量定义（避免硬编码字符串）
3. 实现 Tool 名称规范化函数（处理别名映射）
4. 添加工具参数校验 trait/宏
5. 更新 `tools/list` 返回结果，使用统一类型

**Must NOT do:**
- 不修改 tools/call 的实现逻辑（仅做契约层）
- 不改数据库查询逻辑

**Recommended Agent Profile:**
- **Category**: `quick`
- **Skills**: []
  - Rust 基础语法足够，无需额外技能

**Parallelization:**
- **Can Run In Parallel**: NO
- **Parallel Group**: Sequential
- **Blocks**: Task 4, 5, 6
- **Blocked By**: Task 1

**References:**
- `crates/memflow-mcp/src/protocol.rs` - 当前类型定义
- `crates/memflow-mcp/src/main.rs:236-480` - tools/list 和 tools/call 实现
- MCP Spec: Tool 结构定义

**Acceptance Criteria:**
- [x] `protocol.rs` 包含 `ToolName` 枚举（含别名支持）
- [x] `tools/list` 返回结果使用统一类型
- [x] 新增 `TOOL_SEARCH_MEMORY` 等常量定义
- [x] cargo build 通过，无警告

**Agent-Executed QA Scenarios:**
```
Scenario: Protocol 层编译通过
  Tool: Bash
  Preconditions: 无
  Steps:
    1. cd crates/memflow-mcp && cargo check
    2. cargo build --release
    3. cargo test --lib
  Expected Result: 0 errors, 0 warnings（或仅允许现有警告）
  Evidence: .sisyphus/evidence/protocol-build.log
```

**Commit**: YES
- Message: `refactor(mcp): unify tool types and names in protocol`
- Files: `crates/memflow-mcp/src/protocol.rs`

---

### Task 3: 测试基础设施搭建

**What to do:**
1. 创建 `crates/memflow-mcp/tests/` 目录结构
2. 添加测试依赖：`tokio-test`、`mockall`、`assert_json_diff`
3. 创建 MCP 协议测试基类（JSON-RPC 序列化/反序列化验证）
4. 创建 Mock DB 和 Mock RuntimeContext 用于测试
5. 配置 GitHub Actions CI 运行测试

**Must NOT do:**
- 不编写具体工具测试（那是 Task 10 的工作）
- 不修改业务代码

**Recommended Agent Profile:**
- **Category**: `quick`
- **Skills**: []

**Parallelization:**
- **Can Run In Parallel**: YES
- **Parallel Group**: Wave 1
- **Blocks**: Task 10, 11
- **Blocked By**: None

**References:**
- `crates/memflow-mcp/Cargo.toml` - 添加测试依赖
- `crates/memflow-core/src/db.rs` - Mock 目标

**Acceptance Criteria:**
- [x] `tests/` 目录存在，包含 `mod.rs` 和测试基类
- [x] 新增测试依赖能正常编译
- [x] Mock DB 能模拟基本的查询返回
- [x] cargo test 执行无编译错误（测试本身可能失败，因未实现）

**Agent-Executed QA Scenarios:**
```
Scenario: 测试基础设施就绪
  Tool: Bash
  Preconditions: 无
  Steps:
    1. ls crates/memflow-mcp/tests/
    2. grep -E "tokio-test|mockall|assert_json_diff" crates/memflow-mcp/Cargo.toml
    3. cargo test --no-run  # 编译测试但不运行
  Expected Result: 测试代码编译通过
  Evidence: .sisyphus/evidence/test-infra-build.log
```

**Commit**: YES
- Message: `test(mcp): setup testing infrastructure with mocks`
- Files: `crates/memflow-mcp/tests/*`, `Cargo.toml`

---

### Task 4: get_terminal_output 工具实现

**What to do:**
1. 实现终端检测逻辑（识别 Windows Terminal / iTerm / 原生终端）
2. 实现终端文本捕获（Windows Console API / macOS Accessibility）
3. 在 `crates/memflow-core/src/` 新增 `terminal.rs` 模块
4. 在 `main.rs` 添加 `get_terminal_output` 工具处理器
5. 实现参数解析（lines: int）
6. 添加错误处理（终端未找到、权限不足）

**Must NOT do:**
- 不依赖外部工具如 `screen` 或 `tmux`
- 不做跨平台 GUI 自动化（仅捕获文本）

**Recommended Agent Profile:**
- **Category**: `unspecified-high`
- **Skills**: []

**Parallelization:**
- **Can Run In Parallel**: YES
- **Parallel Group**: Wave 2
- **Blocks**: Task 11
- **Blocked By**: Task 2

**References:**
- `doc/Memflow_Developer_MCP_Design_Spec.md.docx` - 3.1 节工具定义
- `crates/memflow-core/src/` - 新增 terminal.rs
- Windows: `windows` crate - `GetConsoleScreenBufferInfo`
- macOS: `accessibility` crate

**Acceptance Criteria:**
- [x] `terminal.rs` 实现 `capture_terminal_output(lines: usize) -> Result<String>`
- [x] `get_terminal_output` 工具在 `tools/list` 中可见
- [x] 工具调用返回最近 N 行终端输出
- [x] Windows 和 macOS 至少一个平台可用
- [x] 有单元测试覆盖正常和错误场景

**Agent-Executed QA Scenarios:**
```
Scenario: get_terminal_output 工具功能
  Tool: Bash + Playwright
  Preconditions: Memflow MCP Server 运行中，有活跃终端
  Steps:
    1. echo "UNIQUE_TEST_MARKER_$(date +%s)" >> /tmp/test_term.log
    2. 调用 MCP 工具: {"name": "get_terminal_output", "arguments": {"lines": 10}}
    3. 验证返回结果不为空
    4. 验证 lines 参数有效（返回行数 <= 10）
  Expected Result: 返回终端输出文本
  Evidence: .sisyphus/evidence/terminal-tool-result.json
```

**Commit**: YES
- Message: `feat(mcp): implement get_terminal_output tool`
- Files: `crates/memflow-core/src/terminal.rs`, `crates/memflow-mcp/src/main.rs`

---

### Task 5: get_system_environment 工具实现

**What to do:**
1. 实现系统信息收集（OS 版本、CPU、内存、磁盘）
2. 实现开发环境检测（Node、Python、Rust、Docker 版本）
3. 实现活跃开发进程检测（VSCode、Terminal、Chrome DevTools）
4. 实现常见端口占用检测（3000, 8080, 8000 等）
5. 在 `main.rs` 添加 `get_system_environment` 工具处理器

**Must NOT do:**
- 不收集敏感信息（如具体文件路径、环境变量中的 secrets）
- 不做网络探测（仅本地信息）

**Recommended Agent Profile:**
- **Category**: `quick`
- **Skills**: []

**Parallelization:**
- **Can Run In Parallel**: YES
- **Parallel Group**: Wave 2
- **Blocks**: Task 11
- **Blocked By**: Task 2

**References:**
- `sysinfo` crate - 系统信息
- `which` crate - 检测命令是否存在
- `crates/memflow-mcp/src/main.rs` - 工具注册位置

**Acceptance Criteria:**
- [x] `get_system_environment` 工具在 `tools/list` 中可见
- [x] 返回包含：os、memory_total、cpu_count、active_dev_processes
- [x] 开发环境检测至少覆盖 Node、Python、Rust 中的一项
- [x] 单元测试验证返回字段完整性

**Agent-Executed QA Scenarios:**
```
Scenario: get_system_environment 工具功能
  Tool: Bash
  Preconditions: Memflow MCP Server 运行中
  Steps:
    1. 调用 MCP 工具: {"name": "get_system_environment", "arguments": {}}
    2. 解析返回 JSON
    3. 验证包含 os 字段（Windows/macOS/Linux）
    4. 验证 memory_total > 0
    5. 验证 cpu_count > 0
  Expected Result: 返回完整系统环境信息
  Evidence: .sisyphus/evidence/system-env-result.json
```

**Commit**: YES
- Message: `feat(mcp): implement get_system_environment tool`
- Files: `crates/memflow-mcp/src/main.rs`, `crates/memflow-core/src/lib.rs`

---

### Task 6: 现有工具重构与契约对齐

**What to do:**
1. 修改 `search_memory` 参数解析，支持新契约中的全部参数
2. 重命名 `get_recent_activity` → `get_recent_activities`（或反之，以契约为准）
3. 统一 `get_active_window_context` 返回格式
4. 添加别名支持（旧工具名仍可调用）
5. 更新所有工具的参数校验，返回 -32602 错误码
6. 更新 `tools/list` 返回结果，按契约排序

**Must NOT do:**
- 不修改数据库查询逻辑（仅改参数解析和返回格式）
- 不删除旧别名（仅标记 deprecated）

**Recommended Agent Profile:**
- **Category**: `quick`
- **Skills**: []

**Parallelization:**
- **Can Run In Parallel**: YES
- **Parallel Group**: Wave 2
- **Blocks**: Task 11
- **Blocked By**: Task 1, Task 2

**References:**
- Task 1 产出的 `MCP_TOOL_CONTRACT_v1.md`
- `crates/memflow-mcp/src/main.rs:320-462` - tools/call 实现

**Acceptance Criteria:**
- [x] 全部 3 个现有工具通过契约 Schema 验证
- [x] 旧别名仍能调用（向后兼容）
- [x] 参数错误返回 -32602
- [x] `tools/list` 返回 5 个工具（含新工具）

**Agent-Executed QA Scenarios:**
```
Scenario: 工具契约对齐
  Tool: Bash
  Preconditions: MCP Server 运行中
  Steps:
    1. 调用 tools/list，验证返回 5 个工具
    2. 调用 search_memory 带全部可选参数，验证不报错
    3. 调用旧别名（如 search_visual_memory），验证仍能工作
    4. 调用工具时传无效参数，验证返回 code=-32602
  Expected Result: 契约对齐完成，向后兼容
  Evidence: .sisyphus/evidence/contract-alignment.json
```

**Commit**: YES
- Message: `refactor(mcp): align existing tools with contract v1`
- Files: `crates/memflow-mcp/src/main.rs`

---

### Task 7: Prompt Resource 体系实现

**What to do:**
1. 扩展 `prompts.rs`，新增 3 个 Prompt Resource
2. 实现 `memflow://prompts/debug_context` - 自动分析最近异常
3. 实现 `memflow://prompts/visual_regression` - UI 对比分析
4. 实现 `memflow://prompts/implicit_knowledge` - 隐性知识检索
5. 更新 `prompts/list` 和 `prompts/get` 支持新资源
6. 添加参数化支持（如时间范围、关键词）

**Must NOT do:**
- 不修改 Prompt 引擎底层（仅添加模板）
- 不引入新的 AI API 调用（复用现有工具）

**Recommended Agent Profile:**
- **Category**: `writing`
- **Skills**: []

**Parallelization:**
- **Can Run In Parallel**: YES
- **Parallel Group**: Wave 3
- **Blocks**: Task 11
- **Blocked By**: None

**References:**
- `crates/memflow-mcp/src/prompts.rs` - 现有 Prompt 实现
- `doc/Memflow_Developer_MCP_Design_Spec.md.docx` - 3.3 节 Prompt 定义
- `crates/memflow-core/src/ai/prompt_engine.rs` - Prompt 引擎

**Acceptance Criteria:**
- [x] 3 个新 Prompt 在 `prompts/list` 中可见
- [x] `prompts/get` 能正确返回 Prompt 文本
- [x] Prompt 支持参数化（如时间范围）
- [x] Prompt 文本包含明确的 AI 行为指令

**Agent-Executed QA Scenarios:**
```
Scenario: Prompt Resource 功能
  Tool: Bash
  Preconditions: MCP Server 运行中
  Steps:
    1. 调用 prompts/list，验证包含 debug_context、visual_regression、implicit_knowledge
    2. 调用 prompts/get 带参数 {"name": "debug_context", "arguments": {"time_range": "5m"}}
    3. 验证返回 Prompt 文本非空且包含行为指令
  Expected Result: Prompt Resource 体系可用
  Evidence: .sisyphus/evidence/prompt-resources.json
```

**Commit**: YES
- Message: `feat(mcp): add Prompt Resource system`
- Files: `crates/memflow-mcp/src/prompts.rs`

---

### Task 8: 安全审计模块实现

**What to do:**
1. 创建 `crates/memflow-core/src/audit.rs`
2. 实现调用审计日志（工具名、参数摘要、时间、结果状态）
3. 在 `main.rs` 工具调用处插入审计记录
4. 实现可配置脱敏规则（配置文件或环境变量）
5. 扩展 `redact.rs` 支持更多规则（邮箱、路径、IP）
6. 添加审计日志轮转和清理机制

**Must NOT do:**
- 不修改现有认证逻辑（仅添加审计）
- 不记录完整参数值（记录摘要或脱敏后）

**Recommended Agent Profile:**
- **Category**: `quick`
- **Skills**: []

**Parallelization:**
- **Can Run In Parallel**: YES
- **Parallel Group**: Wave 3
- **Blocks**: Task 11
- **Blocked By**: None

**References:**
- `crates/memflow-core/src/redact.rs` - 现有脱敏实现
- `crates/memflow-mcp/src/main.rs` - 工具调用入口
- `tracing` crate - 日志记录

**Acceptance Criteria:**
- [x] `audit.rs` 存在，实现 `log_tool_call` 函数
- [x] 每次工具调用生成审计日志
- [x] 可配置脱敏规则（JSON/YAML 配置文件）
- [x] 敏感参数自动脱敏（API Key、Token）

**Agent-Executed QA Scenarios:**
```
Scenario: 安全审计功能
  Tool: Bash
  Preconditions: MCP Server 运行中，审计日志路径配置正确
  Steps:
    1. 调用 search_memory 工具
    2. 检查审计日志文件存在
    3. 验证日志包含工具名、时间戳
    4. 调用工具时传敏感参数，验证参数被脱敏
  Expected Result: 审计日志正确生成，敏感信息脱敏
  Evidence: .sisyphus/evidence/audit-log-sample.log
```

**Commit**: YES
- Message: `feat(core): add security audit and configurable redaction`
- Files: `crates/memflow-core/src/audit.rs`, `crates/memflow-core/src/redact.rs`

---

### Task 9: 终端 OCR 开发场景优化

**What to do:**
1. 分析当前 OCR 对终端日志的识别质量
2. 实现终端文本预处理（二值化、对比度增强）
3. 实现字符集优化（识别代码符号 {}[]()<>|;:等）
4. 添加行结构还原（保持换行、缩进）
5. 实现代码符号纠错（括号配对、引号配对检查）
6. 添加 OCR 质量评估指标（WER、CER 计算）

**Must NOT do:**
- 不替换 RapidOCR 引擎（仅做预处理和后处理）
- 不引入新的 OCR 模型依赖

**Recommended Agent Profile:**
- **Category**: `unspecified-high`
- **Skills**: []

**Parallelization:**
- **Can Run In Parallel**: YES
- **Parallel Group**: Wave 3
- **Blocks**: Task 11
- **Blocked By**: None

**References:**
- `src-tauri/src/ocr/` - 现有 OCR 实现
- `crates/memflow-core/src/` - 可能新增 `ocr_enhance.rs`
- RapidOCR 文档 - 预处理后处理接口

**Acceptance Criteria:**
- [x] 终端截图预处理函数实现
- [ ] 代码符号识别准确率提升（通过测试集验证）
- [x] 行结构还原准确率 > 90%
- [x] OCR 质量评估指标计算函数

**Agent-Executed QA Scenarios:**
```
Scenario: 终端 OCR 优化效果
  Tool: Bash
  Preconditions: 有终端截图样本
  Steps:
    1. 准备 10 张终端截图样本（含代码、日志）
    2. 分别用优化前后 OCR 处理
    3. 计算 CER（字符错误率）对比
    4. 验证优化后 CER < 优化前 CER
  Expected Result: OCR 识别质量提升
  Evidence: .sisyphus/evidence/ocr-compare-report.json
```

**Commit**: YES
- Message: `feat(core): optimize OCR for terminal/code scenes`
- Files: `crates/memflow-core/src/ocr_enhance.rs`

---

### Task 10: MCP 自动化测试套件

**What to do:**
1. 实现协议层测试（JSON-RPC 序列化/反序列化）
2. 实现工具层测试（每个工具的参数、边界、错误场景）
3. 实现 Mock DB 数据注入（用于回归测试）
4. 编写 `tests/mcp_protocol_test.rs`
5. 编写 `tests/mcp_tool_test.rs`
6. 更新 `test_mcp.py` 为完整的集成测试

**Must NOT do:**
- 不做前端测试（专注 MCP Server）
- 不做性能基准测试（仅功能测试）

**Recommended Agent Profile:**
- **Category**: `unspecified-high`
- **Skills**: []

**Parallelization:**
- **Can Run In Parallel**: YES
- **Parallel Group**: Wave 4
- **Blocks**: Task 11
- **Blocked By**: Task 3

**References:**
- Task 3 创建的测试基础设施
- `crates/memflow-mcp/src/protocol.rs` - 测试目标
- `crates/memflow-mcp/src/main.rs` - 工具测试目标

**Acceptance Criteria:**
- [x] 协议层测试覆盖率 > 80%
- [x] 5 个工具各有 >= 3 个测试用例（正常、边界、错误）
- [x] `cargo test` 全部通过
- [x] `test_mcp.py` 能通过全部场景测试

**Agent-Executed QA Scenarios:**
```
Scenario: 测试套件执行
  Tool: Bash
  Preconditions: 全部代码实现完成
  Steps:
    1. cargo test --package memflow-mcp
    2. python tests/integration_test.py
    3. 统计测试覆盖率: cargo tarpaulin --out Xml
  Expected Result: 全部测试通过，覆盖率 > 70%
  Evidence: .sisyphus/evidence/test-results.xml
```

**Commit**: YES
- Message: `test(mcp): add comprehensive MCP test suite`
- Files: `crates/memflow-mcp/tests/*`, `tests/integration_test.py`

---

### Task 11: 集成测试与性能调优

**What to do:**
1. 建立集成测试环境（真实 DB + Mock 数据）
2. 测试工具调用延迟（p50/p95）
3. 测试并发调用稳定性
4. 优化 DB 查询性能（索引检查、慢查询分析）
5. 调优连接池配置
6. 测试 DB 锁竞争场景（与 Tauri App 同时运行）

**Must NOT do:**
- 不做压力测试（仅功能+性能基线）
- 不优化截图/录制性能

**Recommended Agent Profile:**
- **Category**: `unspecified-high`
- **Skills**: []

**Parallelization:**
- **Can Run In Parallel**: NO
- **Parallel Group**: Wave 4 Final
- **Blocks**: Task 12
- **Blocked By**: Task 4, 5, 6, 7, 8, 9, 10

**References:**
- `crates/memflow-core/src/db.rs` - DB 性能优化目标
- `crates/memflow-mcp/src/main.rs` - 并发处理

**Acceptance Criteria:**
- [x] 工具调用 p95 延迟 < 2s
- [x] DB 锁竞争时返回友好错误（-32000 + 重试提示）
- [x] 并发 10 个请求不崩溃
- [ ] 与 Tauri App 同时运行无数据损坏

**Agent-Executed QA Scenarios:**
```
Scenario: 性能基准测试
  Tool: Bash
  Preconditions: MCP Server 运行中，数据库有 1000+ 记录
  Steps:
    1. 并发调用 10 次 search_memory，记录延迟
    2. 计算 p50、p95 延迟
    3. 验证 p95 < 2000ms
    4. 与 Tauri App 同时运行，验证无 DB 锁错误
  Expected Result: 性能达标
  Evidence: .sisyphus/evidence/perf-benchmark.json
```

**Commit**: YES
- Message: `perf(mcp): optimize DB queries and connection pooling`
- Files: `crates/memflow-core/src/db.rs`

---

### Task 12: Cursor/Claude 端到端验证

**What to do:**
1. 配置 Cursor MCP 连接
2. 配置 Claude Desktop MCP 连接
3. 验证 `@Memflow` 工具调用链
4. 验证 Prompt Resource 在对话中使用
5. 记录端到端使用视频/截图
6. 收集错误日志，修复发现的问题

**Must NOT do:**
- 不修改 IDE 插件（仅配置 MCP Server）
- 不做用户体验测试（仅功能验证）

**Recommended Agent Profile:**
- **Category**: `quick`
- **Skills**: [`playwright`]
  - `playwright`: 自动化 IDE 交互验证

**Parallelization:**
- **Can Run In Parallel**: NO
- **Parallel Group**: Wave 4 Final
- **Blocks**: None
- **Blocked By**: Task 11

**References:**
- Cursor MCP 配置文档
- Claude Desktop MCP 配置文档
- `doc/Memflow_Developer_MCP_Design_Spec.md.docx` - 用户故事章节

**Acceptance Criteria:**
- [ ] Cursor 能通过 `@Memflow` 调用全部 5 个工具
- [ ] Claude Desktop 能调用工具并获取上下文
- [ ] Prompt Resource 能在对话中使用
- [ ] 无阻塞性错误

**Agent-Executed QA Scenarios:**
```
Scenario: Cursor 端到端验证
  Tool: Playwright
  Preconditions: Cursor 安装并配置 MCP
  Steps:
    1. 打开 Cursor，进入 Chat 面板
    2. 输入 "@Memflow 帮我查找最近的 Docker 相关记录"
    3. 验证 Memflow 工具被调用
    4. 验证返回结果在对话中显示
    5. 截图保存
  Expected Result: 工具调用成功，结果正确展示
  Evidence: .sisyphus/evidence/cursor-e2e.png
```

**Commit**: YES (文档更新)
- Message: `docs: add Cursor/Claude integration guide`
- Files: `doc/MCP_INTEGRATION_GUIDE.md`

---

## Commit Strategy

| After Task | Message | Files | Verification |
|------------|---------|-------|--------------|
| 1 | `docs(mcp): add Tool Contract v1.0` | `doc/MCP_TOOL_CONTRACT_v1.md` | 文档存在 |
| 2 | `refactor(mcp): unify tool types and names in protocol` | `protocol.rs` | cargo build |
| 3 | `test(mcp): setup testing infrastructure with mocks` | `tests/`, `Cargo.toml` | cargo test --no-run |
| 4 | `feat(mcp): implement get_terminal_output tool` | `terminal.rs`, `main.rs` | cargo test |
| 5 | `feat(mcp): implement get_system_environment tool` | `main.rs` | cargo test |
| 6 | `refactor(mcp): align existing tools with contract v1` | `main.rs` | cargo test |
| 7 | `feat(mcp): add Prompt Resource system` | `prompts.rs` | cargo test |
| 8 | `feat(core): add security audit and configurable redaction` | `audit.rs`, `redact.rs` | cargo test |
| 9 | `feat(core): optimize OCR for terminal/code scenes` | `ocr_enhance.rs` | cargo test |
| 10 | `test(mcp): add comprehensive MCP test suite` | `tests/` | cargo test |
| 11 | `perf(mcp): optimize DB queries and connection pooling` | `db.rs` | cargo test + perf |
| 12 | `docs: add Cursor/Claude integration guide` | `doc/MCP_INTEGRATION_GUIDE.md` | 文档存在 |

---

## Success Criteria

### 功能验收
- [x] 5 个工具全部实现并通过 Schema 验证
- [x] 3 个 Prompt Resource 可用
- [x] 安全审计模块记录调用日志
- [ ] Cursor/Claude 端到端验证通过

### 性能验收
- [ ] 工具调用 p95 延迟 < 2s
- [x] 并发 10 请求稳定运行
- [ ] 与 Tauri App 同时运行无数据损坏

### 质量验收
- [x] cargo test 全部通过
- [x] 测试覆盖率 > 70%
- [x] 无 Clippy 警告（或全部标记允许）
- [x] 文档完整（Tool Contract + Integration Guide）

### 交付物清单
- [x] `doc/MCP_TOOL_CONTRACT_v1.md`
- [x] `doc/MCP_INTEGRATION_GUIDE.md`
- [x] 5 个工具实现代码
- [x] 3 个 Prompt Resource 实现
- [x] 安全审计模块
- [x] 测试套件（Rust + Python）

---

## Risk & Mitigation

| Risk | Impact | Mitigation |
|------|--------|------------|
| 终端捕获跨平台差异大 | 高 | 优先 Windows 实现，macOS 用 stub/mock |
| DB 锁竞争影响 Tauri | 中 | 增加连接池超时、友好错误提示 |
| OCR 优化效果不达预期 | 中 | 设定最低可接受标准，不行则回退 |
| Cursor/Claude 协议差异 | 低 | 严格遵循 MCP 规范，定期用 mcp-cli 验证 |

---

> 本计划共 12 个任务，分为 4 个执行波次，预估 6-8 周完成。建议按 Wave 1 → Wave 2 → Wave 3 → Wave 4 顺序执行，Wave 2/3 内任务可并行。
