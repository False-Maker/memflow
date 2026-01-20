## 问题结论（基于现有实现）
### 1) AI 智能搜索“点击没反应”的主要原因
- 前端 `handleSmartSearch` 只有 `console.error`，没有任何 UI 级提示；即使后端调用失败/超时，用户也看不到反馈：[Timeline.tsx:L160-L267](file:///d:/Demo/memflow/src/components/Timeline.tsx#L160-L267)
- 当后端解析结果 `keywords` 为空时，前端会把输入框直接清空 `newQuery = ''`，并用 `query: undefined` 去搜；如果此时也没有其它过滤条件，界面看起来就像“什么也没发生”： [Timeline.tsx:L222-L260](file:///d:/Demo/memflow/src/components/Timeline.tsx#L222-L260)
- 后端 `parse_query_intent` 调用大模型没有外层短超时（取决于 provider 的网络超时），可能卡 10~30s；前端只有按钮轻微 `animate-pulse`，也容易被认为无响应。

### 2) “上下文助理模型写死吗？”
- 模型并没写死：
  - 智能搜索使用的是设置里的 `chatModel`（后端 `config.chat_model`）：[ai/mod.rs:L344-L352](file:///d:/Demo/memflow/src-tauri/src/ai/mod.rs#L344-L352)
  - 上下文助理（context-suggestion）也使用同一份 `chatModel`： [proactive_context.rs:L286-L294](file:///d:/Demo/memflow/src-tauri/src/proactive_context.rs#L286-L294)
- 但 prompt 是硬编码的（不是从设置读取）。
- 上下文助理是否触发完全受配置与录制状态约束：必须 `enableProactiveAssistant && aiEnabled && !privacyModeEnabled`，否则后端直接 return 不发事件：[proactive_context.rs:L169-L178](file:///d:/Demo/memflow/src-tauri/src/proactive_context.rs#L169-L178)

## 实施计划（我将按顺序修改并验证）
### 1) 修复智能搜索“无响应”体验与逻辑缺陷
- 在 Timeline 增加可见的状态反馈：解析中提示、解析失败提示（不再只写 console）。
- 修复清空 query 的问题：当 `keywords` 为空时保持原始 `query.trim()`，至少确保搜索动作可感知。
- 将按钮禁用条件从 `!query` 改为 `!query.trim()`，避免纯空格误触。
- 智能搜索触发的搜索默认加上合适的排序（例如有关键词时用 `orderBy: 'rank'`）。

涉及文件：
- [Timeline.tsx](file:///d:/Demo/memflow/src/components/Timeline.tsx)

### 2) 给后端意图解析加短超时与更可靠回退
- 在 `ai::parse_query_intent` 外层增加 `tokio::time::timeout`（例如 6~8s）；超时或调用失败时走 `fallback_filter_params`，避免前端长时间等待。
- （可选）返回结构增加 `source: 'ai' | 'fallback'` 便于前端显示“已使用回退解析”。

涉及文件：
- [ai/mod.rs](file:///d:/Demo/memflow/src-tauri/src/ai/mod.rs)
- （如需要调整 command 返回类型）[commands.rs](file:///d:/Demo/memflow/src-tauri/src/commands.rs)

### 3) 上下文助理：明确展示“当前模型来自设置”并提示未触发原因
- 在 ContextSidebar 增加一条轻量状态提示：
  - 当前模型名（来自 `state.config.chatModel`）
  - 若未开启录制 / AI / 主动助理 / 开了隐私模式，则显示“为什么不会触发”的提示（减少误判为坏了/写死）。

涉及文件：
- [ContextSidebar.tsx](file:///d:/Demo/memflow/src/components/ContextSidebar.tsx)
- [AppContext.tsx](file:///d:/Demo/memflow/src/contexts/AppContext.tsx)（仅用于读取现有 `state.config`，一般不需要改）

### 4) 验证
- 运行 `pnpm type-check`、`pnpm lint`。
- 启动 `pnpm tauri:dev`：
  - 输入自然语言（如“查找上周看过的 PDF”）点智能搜索，确认 UI 有反馈且不会无故清空输入。
  - 在设置切换模型后再次触发智能搜索/上下文助理，确认实际读取最新设置。
