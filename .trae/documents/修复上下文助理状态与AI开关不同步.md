## 现象复盘（结论）
- “上下文助理已启用”需要同时满足：录制中 + AI 启用 + 主动助理启用 + 未开启隐私模式；判定逻辑在 [ContextSidebar.tsx](file:///d:/Demo/memflow/src/components/ContextSidebar.tsx#L122-L133)。
- 目前 `aiEnabled` 默认是 `false`（前端初始值与后端默认配置都是 false），但设置页没有提供 `aiEnabled` 的开关，因此即使录制已启动，右侧也会长期显示“AI 未启用”。对应：
  - 前端默认值：[AppContext.tsx](file:///d:/Demo/memflow/src/contexts/AppContext.tsx#L68-L85)
  - 后端默认值写入 config.json：[app_config.rs](file:///d:/Demo/memflow/src-tauri/src/app_config.rs#L30-L51)
  - 设置页仅有“主动式 AI 助理”开关，没有 AI 总开关：[SettingsModal.tsx](file:///d:/Demo/memflow/src/components/SettingsModal.tsx#L1275-L1304)

## 要改什么（目标）
- 让用户在 UI 里能明确开启/关闭 AI（`aiEnabled`），并在配置了 API Key 后不再出现“AI 未启用”卡死状态。
- 同时把“主动式 AI 助理”的依赖条件提示清楚：未录制/隐私模式/AI 总开关/主动助理开关。

## 实施步骤（最小可用改动）
1) 设置页增加“AI 能力总开关”
- 在 [SettingsModal.tsx](file:///d:/Demo/memflow/src/components/SettingsModal.tsx) 的 general tab 增加一个 toggle，绑定 `draftConfig.aiEnabled`。
- 复用现有保存逻辑（`update_config` + `dispatch(SET_CONFIG)`），保证开关落盘并立刻影响右侧状态。

2) 配置 API Key 后自动解除“AI 未启用”
- 在 `handleSaveApiKey` 成功保存后，如果 `state.config.aiEnabled === false`，自动调用一次 `update_config` 把 `aiEnabled` 置为 `true`（并同步 `draftConfig`/全局 config）。
- 这样用户只要“保存 API Key”，就不会再看到“AI 未启用”挡住上下文助理。

3)（可选但推荐）AI 总开关与实际 AI 调用对齐
- 目前部分 AI 功能（如智能搜索的 `parse_query_intent`）不受 `aiEnabled` 控制，会在配置了 Key 时直接调用模型。[ai/mod.rs](file:///d:/Demo/memflow/src-tauri/src/ai/mod.rs#L357-L485)
- 将其改为：当 `ai_enabled == false` 时直接走 fallback 解析，避免 UI 显示“AI 未启用”但后台仍发起 LLM 请求的割裂。

4) 验证点
- 启动录制后：右侧状态应从“AI 未启用/主动助理未启用”正确变化到“上下文助理已启用”。
- 保存 API Key 后无需额外点击“保存设置”，右侧不再显示“AI 未启用”。
- 关闭 AI 总开关后：上下文助理与建议操作均不再触发（符合隐私/成本预期）。

## 涉及文件
- 前端： [ContextSidebar.tsx](file:///d:/Demo/memflow/src/components/ContextSidebar.tsx), [SettingsModal.tsx](file:///d:/Demo/memflow/src/components/SettingsModal.tsx), [AppContext.tsx](file:///d:/Demo/memflow/src/contexts/AppContext.tsx)
- 后端（可选对齐）：[ai/mod.rs](file:///d:/Demo/memflow/src-tauri/src/ai/mod.rs)
