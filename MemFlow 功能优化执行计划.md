# MemFlow 功能优化执行计划

本计划旨在解决代码分析中发现的硬编码、上下文缺失及算法单一问题，将 MemFlow 从 MVP 提升至 Alpha 可用状态。

## Phase 1: 智能代理 (Agent) 与 AI 核心增强
**目标**: 提升 AI 提案的准确性与灵活性，解决硬编码路径与 Context 限制问题。

### 1.1 外部化 Prompt 与配置 (Refactor)
- **任务**: 将 `src-tauri/src/ai/mod.rs` 和 `agent/mod.rs` 中的 System Prompts 移至资源文件。
- **执行步骤**:
    1. 在 `src-tauri/resources/` 下创建 `prompts.json`。
    2. 修改 `AppConfig`，增加 `prompts_path` 配置项。
    3. 重构 AI 调用逻辑，优先从配置/文件读取 Prompt，失败则使用代码内置默认值。

### 1.2 增强 Agent 上下文构建 (Feature)
- **任务**: 修复 `agent/mod.rs` 中 `take(40)` 的暴力截断问题。
- **执行步骤**:
    1. 实现 `get_activities_by_session`：基于时间间隔（如 >5分钟无操作）自动切分“会话”。
    2. 修改 `propose_automation`，使其根据用户请求的时间窗口（如“今天”）获取完整会话数据，而非固定数量。
    3. 优化 Token 使用：对 OCR 文本过长的记录进行摘要或截断，保留更多上下文条目。

### 1.3 自动化输出可配置 (Feature)
- **任务**: 允许用户自定义 Agent 生成笔记的位置。
- **执行步骤**:
    1. 在 `AppConfig` 中添加 `agent_note_path` (默认为文档目录)。
    2. 修改 `agent/mod.rs` 的 `AutomationStep::CreateNote` 实现，使用配置的路径。

## Phase 2: 知识图谱与 NLP 引擎升级
**目标**: 让知识图谱真正反映内容关联，而非简单的字符串匹配。

### 2.1 集成 NLP 分词引擎 (Enhancement)
- **任务**: 引入 `jieba-rs` 替换 `graph.rs` 中的简单 split。
- **执行步骤**:
    1. 在 `Cargo.toml` 添加 `jieba-rs` 依赖。
    2. 创建 `src-tauri/src/ai/nlp.rs` 模块，封装关键词提取逻辑（支持停用词过滤）。

### 2.2 重构图谱构建逻辑 (Refactor)
- **任务**: 利用 NLP 引擎重建图谱。
- **执行步骤**:
    1. 修改 `graph.rs` 调用 `nlp::extract_keywords`。
    2. 优化 `build_graph`：增加缓存机制或增量更新，避免每次全量扫描 DB。

## Phase 3: 专注度分析算法优化
**目标**: 提供更符合直觉的“心流”评分。

### 3.1 上下文感知评分 (Algorithm)
- **任务**: 优化 `focus_analytics.rs` 算法。
- **执行步骤**:
    1. 引入“白名单应用”概念（如 IDE, 文档阅读器），在这些应用间切换减少惩罚。
    2. 调整公式：`score = apm_score * 0.6 + stability_score * 0.4`。

---

## 立即执行建议
建议优先执行 **Phase 1**，因为它是用户直接感知的核心智能功能。

**是否确认执行 Phase 1 (Agent & AI 增强)?**
