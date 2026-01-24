# memflow 优化检查报告与修复计划


## 发现的问题（需要修）

### 1) search_activities 的 FTS JOIN alias 与 MATCH/bm25 引用不一致（阻断级）
- 现状：`JOIN activity_logs_fts f ...`，但条件与排序用的是 `activity_logs_fts MATCH` 与 `bm25(activity_logs_fts)`。
- 位置：[db.rs:L289-L424](file:///d:/Demo/memflow/src-tauri/src/db.rs#L289-L424)
- 风险：SQLite 下如果表被 alias 后，原表名通常不再可直接引用；这会导致 `no such column/table` 或 `unable to use function MATCH in the requested context` 等错误，search 接口可能直接不可用。

- A（最简单稳妥）：不使用 alias。
  - 把两处 JOIN 改成 `JOIN activity_logs_fts ON a.id = activity_logs_fts.rowid`，保持 `activity_logs_fts MATCH` 与 `bm25(activity_logs_fts)` 不变。


**需要补的验证**
- Rust 侧为 `search_activities_impl` 增加最小测试：
  - 插入 activity + 对应 fts row；断言 query=Some("foo") 时不报错且 total 正确。

---

### 2) AppContext 前端单测与新状态字段不一致（编译/测试失败级）
- 现状：`AppState` 新增 `configLoaded/configError`，[AppContext.tsx](file:///d:/Demo/memflow/src/contexts/AppContext.tsx#L16-L24)；但 `AppContext.test.tsx` 内手写的 `initialState: AppState` 缺少这些字段。
- 位置：[AppContext.test.tsx:L33-L51](file:///d:/Demo/memflow/src/contexts/AppContext.test.tsx#L33-L51)
- 风险：TypeScript 编译直接报错；或测试运行失败。

**修复方案**
- 更新测试的 `initialState` 结构，补齐 `configLoaded/configError` 字段。
- 同步测试预期：
  - 目前实现里配置加载失败走 `console.error + SET_CONFIG_ERROR`，[AppContext.tsx:L142-L152](file:///d:/Demo/memflow/src/contexts/AppContext.tsx#L142-L152)
  - 但测试仍断言“失败时 console.warn 且使用默认值”，[AppContext.test.tsx:L272-L291](file:///d:/Demo/memflow/src/contexts/AppContext.test.tsx#L272-L291)
  - 需要把断言改为：`configError` 被写入、`configLoaded=true`，并且 `config` 保持初始占位值。

---

### 3) 前端 searchActivities 没有落地 total 到状态（非阻断，但与修复目标不完全匹配）
- 现状：invoke 返回 `{items,total}`，但只 dispatch 了 `SET_ACTIVITIES` 与 `SET_SEARCH_PARAMS`，[AppContext.tsx:L187-L207](file:///d:/Demo/memflow/src/contexts/AppContext.tsx#L187-L207)。
- 风险：后端 total 修复虽完成，但前端目前不会保存/展示 total，后续分页/统计无法利用。

**修复方案**
- 给 `AppState` 增加 `searchTotal?: number`（或 `searchMeta`），新增 action `SET_SEARCH_RESULT`（一次性设置 items+total+params）。

---
