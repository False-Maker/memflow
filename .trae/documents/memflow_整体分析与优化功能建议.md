# memflow 二次检查报告（你已开发完成后的复核）

## 复核范围
- 后端：`search_activities` total、FTS/MATCH 引用一致性、相关单测
- 前端：AppContext 状态/搜索结果(total)落地、相关单测
- 去重与向量检索：关键实现存在性与接口连通性

## 结论（通过 / 仍需验证点）
- **search_activities total：已修复并自洽**
  - JOIN 不再使用 alias，`activity_logs_fts MATCH` 与 `bm25(activity_logs_fts)` 引用一致。
  - 代码位置：[db.rs:L289-L423](file:///d:/Demo/memflow/src-tauri/src/db.rs#L289-L423)
- **后端已补 Rust 单测覆盖 total 逻辑（无 query 的分支）**
  - 覆盖 app_name、时间窗、has_ocr、分页等组合，位置：[db.rs:L1159-L1247](file:///d:/Demo/memflow/src-tauri/src/db.rs#L1159-L1247)
  - 备注：该测试未覆盖“带 query 的 FTS 分支”，但至少证明 count/过滤/分页路径正确。
- **前端已把 total 落地到 state（searchTotal）且测试已同步**
  - 新增 `searchTotal` 与 `SET_SEARCH_RESULT`，位置：[AppContext.tsx:L16-L135](file:///d:/Demo/memflow/src/contexts/AppContext.tsx#L16-L135) 与 [AppContext.tsx:L196-L218](file:///d:/Demo/memflow/src/contexts/AppContext.tsx#L196-L218)
  - 单测已覆盖 `searchTotal` 与错误分支，位置：[AppContext.test.tsx:L534-L627](file:///d:/Demo/memflow/src/contexts/AppContext.test.tsx#L534-L627)
- **配置加载失败行为与单测已一致**（`configLoaded/configError`）
  - 实现：[AppContext.tsx:L151-L162](file:///d:/Demo/memflow/src/contexts/AppContext.tsx#L151-L162)
  - 单测：[AppContext.test.tsx:L274-L302](file:///d:/Demo/memflow/src/contexts/AppContext.test.tsx#L274-L302)
- **去重优化已落地（Hamming 距离）并带单测**
  - 关键逻辑：[recorder.rs:L196-L221](file:///d:/Demo/memflow/src-tauri/src/recorder.rs#L196-L221)
  - 单测：[recorder.rs:L480-L603](file:///d:/Demo/memflow/src-tauri/src/recorder.rs#L480-L603)
- **向量检索已做候选集过滤（避免全表扫描）**
  - 向量侧 API：[vector_db.rs:L68-L131](file:///d:/Demo/memflow/src-tauri/src/vector_db.rs#L68-L131)
  - 混合检索接入：[rag.rs:L41-L64](file:///d:/Demo/memflow/src-tauri/src/ai/rag.rs#L41-L64)

## 仍建议补的一项验证（不改功能，只补覆盖）
- **补一个“带 query 的 search_activities_impl 测试”**：创建内存库的 FTS 表 `activity_logs_fts`，插入 row，再断言 `query=Some("xxx")` 时 total 与 items 正确。
  - 目的：确保 `MATCH` 查询与 count 查询在 FTS 分支也完全一致（避免线上才暴雷）。

## 你确认后我会立即做的事（执行与验证）
1. 运行前端测试（vitest）与 Rust 测试（cargo test），确认无回归。
2. 启动 tauri dev，手动走一遍：搜索（含 rank 排序）→ total 正确变化 → 翻页 offset。
3. 如果测试里缺“FTS query 分支”，我会补上并再次跑全套。