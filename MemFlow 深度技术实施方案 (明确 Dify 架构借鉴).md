# MemFlow 深度技术实施方案 (明确 Dify 架构借鉴)

针对您的反馈，本计划明确标注了**参考 Dify 架构**的具体实现点，将 Dify 的先进理念（模板化、变量注入、工具抽象）下沉到 MemFlow 的 Rust 后端实现中。

## 1. AI 架构层：类 Dify 智能引擎 (Dify-Inspired Architecture)

### A. 结构化 Prompt 模板引擎 (Ref: Dify Prompt Orchestration)
- **Dify 特性**: 将 Prompt 拆解为 System/User/Context 模块，支持 `{{variable}}` 动态注入。
- **MemFlow 实现**:
    - **新增** `src-tauri/src/ai/prompt_engine.rs`。
    - **设计**: 实现 `PromptTemplate` 结构体，支持类似 Dify 的变量替换机制。
      ```rust
      // 借鉴 Dify 的变量注入思想
      let template = "基于以下上下文回答：\n{{context}}\n用户问题：{{query}}";
      let prompt = template.replace("{{context}}", logs).replace("{{query}}", user_input);
      ```
    - **价值**: 彻底解耦“提示词逻辑”与“代码逻辑”，未来可支持用户在 UI 上像 Dify 一样直接编辑 Prompt。

### B. 工具抽象层 (Ref: Dify Tools/Plugin System)
- **Dify 特性**: Agent 不是死板的脚本，而是动态选择工具 (Google Search, Python Sandbox 等)。
- **MemFlow 实现**:
    - **重构** `agent/mod.rs`: 将硬编码的 `AutomationStep` 升级为通用的 `Tool` Trait。
    - **定义**:
      ```rust
      trait Tool {
          fn name(&self) -> &str;
          fn description(&self) -> &str; // 供 LLM 决策使用
          fn execute(&self, args: Value) -> Result<Value>;
      }
      ```
    - **价值**: 让 Agent 具备扩展性，未来可轻松添加“发送邮件”、“运行脚本”等新工具，即插即用。

### C. 变量与上下文管理 (Ref: Dify Context Management)
- **Dify 特性**: 自动将 RAG 检索结果注入到 `Context` 变量。
- **MemFlow 实现**:
    - **规范化**: 在 `ai/mod.rs` 中建立标准的 Context 构建管线，将 `ocr_text`, `window_title`, `timestamp` 统一格式化为结构化数据，而非散乱的字符串拼接。

---

## 2. 数据采集层：UI Automation 与事件驱动 (Core Optimization)

### D. 引入 Windows UI Automation (UIA)
- **目标**: 实现毫秒级、无损文本提取，大幅减少 OCR 依赖。
- **实现**:
    - `Cargo.toml`: 增加 `windows` crate 的 UIA features。
    - **新增** `src-tauri/src/uia.rs`: 实现 `get_foreground_text()`，优先直接获取窗口文本结构。

### E. 实现事件驱动采样 (Event Loop)
- **目标**: 仅在屏幕内容变动时采样，消除冗余。
- **实现**:
    - **新增** `src-tauri/src/win_event.rs`: 使用 `SetWinEventHook` 监听窗口切换事件，驱动录制循环。

---

## 3. 知识图谱层：中文分词集成
- **实现**: `graph.rs` 集成 `jieba-rs`，解决中文分词问题。

**执行确认**:
此方案将 Dify 的**Prompt 编排**和**工具抽象**思想融入了 Rust 后端，配合底层的 **UIA 采集优化**，实现了从数据源头到智能处理的全链路升级。是否开始执行？
