## 不使用 Skill 的原因
- 当前仅有 skill-creator（用于创建新技能），本任务是实现功能与测试，不需要创建技能。

## 最高优先级约束：不影响原功能
- 默认关闭：`enableFocusAnalytics` 默认 `false`，未开启时不创建任务、不访问 device_query、不写库。
- 主录制链路不改语义：`capture_and_save()` 的截图→写入活动→emit→OCR 流程保持不变；不在该链路中插入任何可能阻塞的逻辑。
- 三层防御落地：
  - Feature Flag：每次运行前检查开关（关闭即短路）。
  - 异步隔离：专注度采样独立 `tokio::spawn` 任务，不占用截图周期。
  - 异常隔离：tick 内 `catch_unwind`，任何 panic 都只记录日志，不传播到 recorder。
- DB 写入降噪：按“分钟”写 1 条数据（不是每秒写），避免 DB 压力影响录制写入。

## 目标与范围
- 实现“专注度分析（Focus Analytics）”端到端：输入采样（每秒）、窗口切换统计、按分钟落库到 `focus_metrics`、提供查询 API、前端趋势图展示、设置页开关。
- 完成后执行编译与测试（Rust 单测 + 前端 type-check/lint）。

## 后端：配置开关（Feature Flag）
- 在后端 `AppConfig` 增加 `enable_focus_analytics: bool`（前端字段 `enableFocusAnalytics`）。
- 补齐 serde default/alias（保证老 `config.json` 兼容），更新 `commands.rs` 现有 defaults/alias 测试。
- 在 `app_config.rs` 默认配置写入里补上该字段默认值（`false`）。

## 后端：专注度采样任务（完全与录制循环解耦）
- 新增 `focus_analytics` 模块，提供 `spawn_if_enabled()`：
  - `recorder::start()` 里仅新增一行调用（读取一次 config，若未开启直接 return）。
  - 采样任务内部 while 循环以 `RECORDING` 原子标记为退出条件。
- 采样逻辑（每 1 秒 tick，内存里计算 delta，不存具体键值）：
  - 使用 `device_query`（计划 `device_query = "0.2.4"`）获取鼠标位置与 pressed keys（只用于计算数量差分，立即丢弃）[2]。
  - `key_press_count`：本秒 pressed keys 集合与上一秒差分，累计“新增按下数量”。
  - `mouse_move_distance`：本秒鼠标坐标与上一秒的距离累加。
  - `window_switch_count`：每秒读取一次 `window_info::get_foreground_window_info()`，当 `process_path` 变化时计 1（只统计“应用切换”，不因 title 抖动导致虚高）。
- 分钟聚合：每 60 秒生成 1 条：
  - `apm`：由 key press count + mouse distance 按固定比例折算成 actions/min（常量可调）。
  - `focus_score`：`calculate_focus_score(apm, switch_count)`，权重常量 + clamp 到 0..100。
  - 写入 DB（单条 insert），失败只记录 warn/error，不影响采样循环继续。
  - 可选 emit `focus-metrics-updated`（失败忽略）。
- 异常隔离：tick 内 `catch_unwind(AssertUnwindSafe(...))`；任何 panic 只写日志并继续下一次 tick。

## 数据库：focus_metrics 表与读写 API
- 增加 migration（例如 `0009_add_focus_metrics.sql`）：
  - `focus_metrics(timestamp INTEGER PRIMARY KEY, apm INTEGER NOT NULL, window_switch_count INTEGER NOT NULL, focus_score REAL NOT NULL)`。
- 在 `db.rs` 增加：
  - `FocusMetric`（对前端序列化 camelCase）。
  - `insert_focus_metric(...)`。
  - `get_focus_metrics(from_ts, to_ts, limit)`（按时间升序返回，用于趋势图）。
- 在 `commands.rs` 增加 tauri command `get_focus_metrics(...)`，并在 `lib.rs` 注册。

## 前端：设置开关与趋势图（无侵入现有 UI）
- `AppContext.tsx`：
  - `AppConfig` interface 增加 `enableFocusAnalytics: boolean`；`initialState.config` 增加默认 `false`。
- `SettingsModal.tsx`：
  - 新增开关项“专注度分析”，保存时复用现有 `draftConfig -> update_config`；不改变其他设置逻辑。
- `FlowState.tsx`：
  - 仅新增一个区域展示“专注度趋势”：调用 `get_focus_metrics` 拉取最近 24h（默认）。
  - 用现有 recharts 渲染 focusScore 折线图；无数据时展示占位提示。

## 测试与验证（确保零影响）
- Rust 单测：
  - `calculate_focus_score` 边界测试（0/高 APM、极高切换数、clamp）。
  - focus_metrics 插入/查询测试：`:memory:` 建最小表结构，验证范围过滤与排序。
  - 更新 `AppConfig` serde defaults/alias 测试覆盖新字段。
- 编译/质量检查：
  - `npm run test`（cargo test）。
  - `npm run type-check`、`npm run lint`。

## 交付物（确认后会改动的文件范围）
- 后端：`src-tauri/Cargo.toml`、`src-tauri/src/{recorder.rs,db.rs,commands.rs,lib.rs,app_config.rs}`、`src-tauri/src/focus_analytics.rs`、`src-tauri/migrations/0009_add_focus_metrics.sql`
- 前端：`src/contexts/AppContext.tsx`、`src/components/SettingsModal.tsx`、`src/components/FlowState.tsx`

引用
- [2] https://crates.io/crates/device_query/0.2.4