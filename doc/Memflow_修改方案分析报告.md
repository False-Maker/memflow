# Memflow 项目修改方案分析报告

> 分析日期：2026年2月11日  
> 依据文档：Memflow_Project_Documentation.docx + Memflow_Developer_MCP_Design_Spec.md.docx

---

## 一、文档目标回顾

### 1.1 项目核心定位

Memflow 定位为**本地优先的视觉记忆系统**，通过自动截图、OCR、AI 分析帮助用户记录和分析桌面活动，构建个人知识图谱。核心理念强调隐私优先、极致性能、本地智能。

### 1.2 Developer MCP 设计目标

为 AI IDE（Cursor、Windsurf、Claude Desktop）提供 Memflow 数据的实时查询能力，使 AI 能够主动感知用户工作上下文，从"被动问答"转向"主动预判"。

---

## 二、当前项目状态评估

### 2.1 已完成能力

| 模块 | 状态 | 说明 |
|------|------|------|
| 活动录制（截图+OCR） | ✅ | xcap截图、Windows OCR + RapidOCR |
| 智能去重 | ✅ | pHash感知哈希 |
| PII隐私脱敏 | ✅ | redact.rs 实现 API Key/Password/JSON字段脱敏 |
| 数据库存储 | ✅ | SQLite WAL 模式、FTS5全文检索 |
| 向量检索 | ✅ | LanceDB/SQLite-VSS、EMBEDDING_DIM=384 |
| RAG混合检索 | ✅ | BM25 + Vector 加权 + 时间衰减 |
| MCP基础协议 | ✅ | JSON-RPC 2.0、tools/list、tools/call、prompts/list/get |
| Prompt模板 | ✅ | mcp_search_intent、mcp_answer_with_memory |
| 认证与只读控制 | ✅ | MEMFLOW_MCP_TOKEN、MCP_READ_ONLY 环境变量 |

### 2.2 已实现 MCP Tools（现状）

```
tools/list 输出：
├── search_memory        # 关键词/语义/混合搜索
├── get_recent_activity  # 最近N分钟活动时间线
├── get_related_context  # 相关上下文检索
└── search_memory (alias)

tools/call 额外支持：
├── search_visual_memory
├── get_recent_activities
├── get_active_window_context  # 已声明但实现依赖 db::get_activities
```

### 2.3 当前主要问题

1. **工具命名与协议不一致**：`tools/list` 与 `tools/call` 支持的工具名存在差异（如 `get_recent_activity` vs `get_recent_activities`）
2. **Phase 2 工具不完整**：`get_terminal_output` 缺失，`get_system_environment` 未实现
3. **Prompt 资源化不足**：仅有2个基础 Prompt，缺乏设计文档中的 `memflow://prompts/debug_context` 等资源
4. **OCR 未针对开发场景优化**：终端日志、代码片段等场景识别质量未知

---

## 三、差距分析（按优先级）

### P0 - 必须优先

| 序号 | 设计要求 | 现状 | 差距描述 | 建议方案 |
|------|----------|------|----------|----------|
| 1 | 统一 Tool 契约 | tools/list 与 tools/call 工具名不一致 | 命名混乱、外部接入方无法确定使用哪个工具名 | 确定正式工具名列表，保留别名兼容但标记 deprecated，输出 Tool Contract 文档 |
| 2 | get_terminal_output | 缺失 | 无法读取终端/控制台输出，开发场景核心能力缺失 | 实现终端文本提取功能，支持 iTerm/Windows Terminal/原生控制台 |
| 3 | get_system_environment | 缺失 | 无法感知 OS 版本、可用内存、Node/Rust 环境、端口占用等 | 输出系统环境信息最小集：OS、CPU、内存、活跃开发进程、端口 |
| 4 | Phase 2 三工具稳定化 | get_active_window_context/get_recent_activities 已有基础实现 | 边界参数、错误码、降级行为未明确定义 | 建立统一错误模型（-32602 参数错误、-32000 业务错误），明确各工具的 p50/p95 延迟目标 |

### P1 - 强烈建议

| 序号 | 设计要求 | 现状 | 差距描述 | 建议方案 |
|------|----------|------|----------|----------|
| 5 | Prompt 资源化 | 仅2个基础 Prompt | 无法支持设计文档中的调试、知识问答等场景化 Prompt | 新增 debug_context、visual_regression_fix、implicit_knowledge_retrieval 等 Prompt Resource |
| 6 | 开发场景 OCR 优化 | 通用 OCR | 终端日志、代码片段识别效果未验证 | 增加终端文本增强通道：字符集检测、行结构还原、符号纠错；建立 WER/CER 评估指标 |
| 7 | 安全审计能力 | 仅认证+只读开关 | 无调用审计、可疑行为检测、可配置脱敏规则 | 实现工具级权限矩阵、调用审计日志（用户/时间/工具/参数）、可配置脱敏规则（API Key/邮箱/文件路径等） |

### P2 - 下一阶段

| 序号 | 设计要求 | 现状 | 差距描述 | 建议方案 |
|------|----------|------|----------|----------|
| 8 | MCP Notification/Sampling | 缺失 | Phase 3 的主动通知能力无法实现 | 先实现被动通知 MVP（窗口突变检测、终端错误爆发），再扩展主动分析 |
| 9 | 可观测性体系 | 仅有 stderr 日志 | 无 SLA 指标、DB 锁竞争无观测 | 建立工具级延迟/错误率监控、结构化错误分类、DB 锁竞争检测 |
| 10 | 集成测试 | test_mcp.py 单机脚本 | 无协议层/工具层/回归测试 | 补充三层测试：JSON-RPC Schema 验证、参数边界与错误码测试、模拟数据库+OCR 样本回归 |

---

## 四、建议执行路线

### 迭代 1（1-2 周）：基础能力闭环

**目标**：统一 Tool 契约，补齐 Phase 2 核心工具可用性

- [ ] 统一 tools/list 与 tools/call 工具名，输出 `TOOLS_CONTRACT.md`
- [ ] 实现 `get_terminal_output` 工具
- [ ] 实现 `get_system_environment` 工具
- [ ] 为 Phase 2 三工具（get_active_window_context / get_recent_activities / get_terminal_output）建立统一错误模型
- [ ] 产出《MCP Tool Contract v1》

### 迭代 2（1-2 周）：开发场景增强

**目标**：提升开发场景体验，闭环安全审计

- [ ] 新增 Prompt Resource：debug_context、visual_regression_fix
- [ ] 实现终端 OCR 增强通道，添加质量评估指标
- [ ] 实现调用审计日志 + 可配置脱敏规则
- [ ] 建立工具级权限矩阵

### 迭代 3（2+ 周）：稳定性与可观测性

**目标**：可观测、可回滚、可测量

- [ ] Notification MVP（窗口突变、终端错误爆发）
- [ ] 监控面板：p50/p95 延迟、错误率、超时率
- [ ] DB 锁竞争检测与重试策略优化
- [ ] CI 集成测试门禁（Schema + 工具层 + 回归）

---

## 五、最小闭环方案（推荐）

如果只能做一轮修改，建议按以下**最小闭环**执行，以最快速度达到"可稳定接入 Cursor/Claude"的水平：

### 最小闭环范围

| 序号 | 内容 | 输出物 |
|------|------|--------|
| 1 | 统一 Tool 契约 | `TOOLS_CONTRACT.md`，明确 5-6 个正式工具名及别名策略 |
| 2 | 补齐 `get_terminal_output` | 实现终端文本提取功能 |
| 3 | 补齐 `get_system_environment` | 实现系统环境感知功能 |
| 4 | Phase 2 三工具稳定化 | 明确参数边界、错误码、降级行为 |
| 5 | MCP 自动化测试 | `test_mcp.py` 升级为可回归的协议+工具测试 |

### 验收标准

- [ ] Claude Desktop / Cursor 能通过 `@Memflow` 成功调用全部声明工具
- [ ] 工具调用成功率 > 95%，p95 延迟 < 2s
- [ ] 工具 schema 通过 `mcp-cli` 校验
- [ ] 有统一的 `TOOLS_CONTRACT.md` 文档

---

## 六、附录：当前代码结构参考

```
memflow/
├── crates/
│   ├── memflow-core/          # 核心逻辑库
│   │   ├── src/
│   │   │   ├── db.rs          # SQLite 操作、FTS5 检索
│   │   │   ├── vector_db.rs   # 向量存储与检索
│   │   │   ├── redact.rs      # PII 隐私脱敏
│   │   │   ├── context.rs     # RuntimeContext trait
│   │   │   └── ai/
│   │   │       ├── rag.rs     # HybridSearch 混合检索
│   │   │       ├── nlp.rs     # TF-IDF 关键词提取
│   │   │       └── prompt_engine.rs
│   │   └── migrations/         # 数据库迁移脚本
│   │
│   └── memflow-mcp/           # MCP Server 实现
│       └── src/
│           ├── main.rs        # 入口、协议处理、工具实现
│           ├── server.rs      # MCP Server 结构
│           ├── protocol.rs    # JSON-RPC 类型定义
│           ├── prompts.rs     # Prompt Resource 实现
│           └── context.rs      # McpContext
│
├── doc/
│   ├── Memflow_Project_Documentation.docx
│   └── Memflow_Developer_MCP_Design_Spec.md.docx
│
└── test_mcp.py               # 当前 MCP 集成测试脚本
```

---

> 本报告仅基于文档分析与代码现状给出修改方向，具体实现细节需进一步评审后执行。
