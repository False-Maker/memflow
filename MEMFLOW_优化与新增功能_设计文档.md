# MemFlow 优化与新增功能设计文档

版本：v0.1  
日期：2026-01-11  
范围：桌面活动采集、OCR、检索/问答、图谱、智能代理、对话历史与反馈、性能与隐私治理

## 1. 背景与现状

MemFlow 是一个桌面活动记录与分析工具，当前已具备以下能力：

- 活动录制：周期性截屏，记录应用名/窗口标题/截图路径/可选 OCR 文本与 pHash
- OCR：本地 OCR 服务（RapidOCR API）异步识别
- 检索与问答：对活动内容进行检索，结合 LLM 进行问答；支持多供应商与自定义兼容接口
- 知识图谱：基于活动内容构建并展示图谱
- 对话历史与反馈：会话/消息持久化、评价、反馈提交与检索
- 智能代理 MVP：自动化提案、执行、审计与取消
- 性能监控：展示 CPU/内存/磁盘占用等指标

当前项目主要痛点集中在：

- 隐私与可控性尚未闭环（如黑名单表已存在但需要贯通 UI→配置→采集链路）
- 搜索体验仍偏“分散”（聊天历史有搜索；活动时间线缺少统一的检索/过滤入口）
- 数据生命周期缺少“可验证闭环”（保留天数存在，但清理 DB/截图/embedding/图谱等需一致）
- 后台任务（OCR/embedding/图谱）存在进一步队列化与增量化空间
- 维护能力缺少面向用户的“自检/修复/重建索引”操作入口

## 2. 设计目标

### 2.1 目标（Goals）

- 提升隐私与可控性：让用户可以明确控制“采集什么、不采集什么、保留多久、是否脱敏”
- 提升检索与复盘效率：在时间线提供一站式搜索与筛选，并能把结果无缝用于问答上下文
- 提升稳定性与可维护性：提供数据库/索引自检与可恢复能力，降低 FTS/迁移相关问题的用户成本
- 提升性能与成本：减少无效采集与 OCR 负载；让 embedding/图谱更可控（队列化、增量化）
- 强化可解释性：问答答案可回溯至活动证据（contextIds），并能从 UI 一键跳转

### 2.2 非目标（Non-goals）

- 不在本设计中引入复杂的云端账户体系、多人协作与跨设备同步（可作为后续扩展）
- 不在本设计中实现完整的任务自动化编排平台（Agent 保持低风险 MVP 方向）

## 3. 总体方案（按优先级）

### 3.1 P0：隐私与可控性闭环

#### 3.1.1 应用黑名单/白名单贯通

现状：数据库存在 `app_blocklist` 表，但需要贯通到采集链路与设置 UI。

方案：
- 新增配置项（AppConfig）
  - blocklistEnabled：是否启用黑名单过滤
  - blocklistMode：blocklist（默认）/ allowlist（可选）
  - blocklistItems：应用名数组（或使用现有表为主，配置只记录开关）
- 新增后端命令（Tauri invoke）
  - get_blocklist / add_blocklist_item / remove_blocklist_item / clear_blocklist
  - get_privacy_status：返回当前录制/过滤状态摘要（用于 UI 显示）
- 录制链路接入
  - 在采集前根据 processName 判断是否跳过，并统计被跳过次数（用于调参与反馈）
- 前端设置页
  - “隐私”分区：开关 + 列表管理 + 一键从最近活动里加入黑名单

验收：
- 将某应用加入黑名单后，该应用活动不再产生新截图/活动记录
- UI 可见当前启用状态与条目数

#### 3.1.2 临时隐私模式（快速暂停/遮蔽）

方案：
- 新增 UI 快捷开关：“隐私模式（暂停采集）”
- 支持定时自动恢复：15/30/60 分钟（防止忘记开启/关闭）
- 隐私模式期间：
  - 不截屏、不 OCR、不写 DB
  - UI 显示状态与剩余时间

验收：
- 开启隐私模式后录制状态与后台任务立即停止
- 到期自动恢复，并在 UI 显示提示

#### 3.1.3 OCR 文本脱敏（可选）

方案：
- 引入规则脱敏（邮箱、手机号、token/密钥样式、URL 参数等）
- 脱敏策略：仅对 OCR 文本存储脱敏，原截图仍可保留（或可选同时模糊截图）
- 配置项：
  - ocrRedactionEnabled
  - ocrRedactionLevel（basic/strict）

验收：
- 开启后 OCR 入库文本符合脱敏规则

### 3.2 P0：时间线一站式搜索与过滤（FTS + 条件过滤）

#### 3.2.1 功能范围

在时间线视图提供统一入口：
- 关键词搜索：基于 `activity_logs_fts`（OCR 文本）
- 条件筛选：应用名、时间范围、有无 OCR、窗口标题包含、是否重复（pHash）
- 排序：时间倒序（默认）、相关度（FTS rank）
- 结果操作：
  - 选中若干条作为“问答上下文”
  - 一键加入黑名单
  - 跳转到截图预览、复制 OCR 文本

#### 3.2.2 后端接口

新增命令：
- search_activities(params)
  - query: string（可为空）
  - appName?: string
  - fromTs?: number
  - toTs?: number
  - hasOcr?: boolean
  - limit?: number
  - offset?: number
  - orderBy?: "time" | "rank"
返回：
- items: ActivityLog[]
- total: number（可选）

说明：
- query 为空时退化为普通过滤查询
- orderBy=rank 时使用 FTS 的 bm25 或 rank 函数（SQLite FTS5）

#### 3.2.3 前端交互

- Timeline 顶部搜索栏 + 高级筛选抽屉
- 筛选条件变更自动查询（带防抖）
- 选中结果集后可“发送到问答”
  - 在 QnA 输入框上方显示已选上下文条目数与可移除列表

验收：
- 关键词 + 时间范围组合查询可用，性能可接受
- 结果可一键进入问答并保持可回溯

### 3.3 P1：问答可解释性与溯源体验增强

现状：chat_messages 已有 `context_ids` 字段（JSON 数组）与前端类型支持。

方案：
- 在回答气泡下展示“引用活动（N）”
- 点击打开侧边栏：活动列表（截图缩略图/应用/时间/窗口标题/ocr 片段）
- 可从引用列表跳转回时间线定位该活动

验收：
- 任意一条包含 contextIds 的 assistant 消息都可跳转到活动证据

### 3.4 P1：数据生命周期闭环（保留策略 + 立即清理）

#### 3.4.1 清理对象

- activity_logs 过期记录
- 对应截图文件
- vector_embeddings（与 activity_id 关联）
- 图谱派生数据（如以活动为源生成的节点/边，若存在关联字段需同步删除）
- chat/feedback 是否清理：默认不清理（避免误删），可提供独立开关

#### 3.4.2 清理策略

- 定期清理：应用启动后/每天一次（由后台定时任务触发）
- 主动清理：设置页提供“立即清理”按钮与结果统计（删除条数/释放空间估算）

#### 3.4.3 后端接口

新增命令：
- run_retention_cleanup(dryRun?: boolean)
返回：
- deletedActivities
- deletedEmbeddings
- deletedScreenshots
- freedBytesEstimate

验收：
- dryRun 返回估算，真实执行后 DB/文件一致
- 删除后时间线与搜索不出现“缺图/孤儿 embedding”

### 3.5 P1：后台任务队列化（OCR/Embedding/图谱）

#### 3.5.1 OCR 任务状态可见

现状：OCR 失败可能静默；并发=2。

方案：
- 维护任务状态表（或内存状态 + 持久化最后错误）
  - pending/running/success/failed + error
- UI 显示 OCR 服务状态、最近错误、重启按钮

#### 3.5.2 Embedding 生成队列

方案：
- OCR 成功后将 activity_id 入队
- 限流并可暂停（当 AI 关闭或 API key 缺失时不入队）
- 失败重试与指数退避

#### 3.5.3 图谱增量更新

方案：
- build_graph_range(fromTs, toTs) 或 update_graph_since(lastTs)
- rebuild_graph 保留为维护操作

验收：
- 长时间运行不会累积失控任务；性能面板可看到队列长度与处理速率

### 3.6 P2：复盘与输出能力

#### 3.6.1 每日/每周摘要

方案：
- 聚合统计：应用使用时长、窗口标题主题聚类、关键词趋势
- 可选 AI 总结：以“脱敏后的 OCR + 应用/标题”作为上下文生成摘要
- 输出：复制/导出为 Markdown/纯文本

#### 3.6.2 导出/备份/迁移

方案：
- 导出活动（CSV/JSON），可选包含 OCR/窗口标题/路径（默认不导出截图）
- 备份数据库：复制 sqlite 文件（需要停止写入或使用 sqlite backup API）

## 4. 数据模型变更（建议）

### 4.1 AppConfig 扩展

建议在现有 AppConfig 基础上扩展：
- privacyModeEnabled: bool
- privacyModeUntilTs?: number
- blocklistEnabled: bool
- blocklistMode: "blocklist" | "allowlist"
- ocrRedactionEnabled: bool
- ocrRedactionLevel: "basic" | "strict"

### 4.2 数据库表（可选新增）

为增强可观测性，可新增（可选）：
- task_queue / task_status（ocr/embedding）
  - id, type, payload_json, status, retries, last_error, created_at, updated_at

若不新增表，也可先用内存队列 + 简单持久化最后错误信息以降低复杂度。

## 5. 接口设计（建议新增 Tauri commands）

### 5.1 搜索与过滤
- search_activities(params)

### 5.2 隐私与黑名单
- get_blocklist
- add_blocklist_item
- remove_blocklist_item
- clear_blocklist
- set_privacy_mode(enabled, untilTs?)

### 5.3 数据保留与维护
- run_retention_cleanup(dryRun?)
- rebuild_fts_indexes（如需要）
- db_health_check（integrity_check + fts 写入 smoke test + 空间占用）

### 5.4 任务状态（可选）
- get_task_status
- restart_ocr_service
- pause_background_jobs / resume_background_jobs

## 6. UI/交互设计（落地建议）

- Timeline
  - 顶部搜索与筛选区（关键词/时间/应用/有无 OCR）
  - 结果多选：加入问答上下文、加入黑名单、复制 OCR、打开预览
- QnA
  - 上下文条目展示：可移除、可跳转
  - 回答溯源：引用活动列表
- Settings
  - 隐私分区：黑名单管理、隐私模式、脱敏开关
  - 数据管理：保留天数、立即清理、空间统计
  - 维护工具：DB 健康检查、重建索引、重建图谱
- 性能面板
  - 新增队列指标：OCR/embedding pending、失败次数、最近错误

## 7. 安全与权限（Tauri 能力）

- opener 等高风险能力必须通过 capabilities 权限控制
- 对外网络请求（LLM/embedding）必须避免日志中输出密钥
- 建议默认：
  - AI 关闭（需要用户显式开启）
  - 脱敏默认开启（至少 basic）
  - 搜索与问答仅使用本地数据库与用户配置的模型端点

## 8. 性能与容量规划

- 数据增长：活动日志随时间线性增长，截图占用通常是主要瓶颈
- 优先策略：
  - 去重（pHash + 稳定性）
  - 保留策略自动清理
  - OCR/embedding 队列化与限流
  - 图谱增量化

建议在性能面板中展示：
- 截图数量与占用（MB）
- DB 文件大小
- OCR/embedding 队列长度、处理速率

## 9. 迁移与兼容策略

- AppConfig 新字段默认值必须具备向后兼容（serde default）
- 数据库新增表使用 migration，保持可回滚策略
- 对 FTS 的维护操作应提供“重建索引”以应对历史遗留差异

## 10. 测试与验收

### 10.1 单元测试（后端）
- search_activities：关键词/空 query/时间范围边界
- run_retention_cleanup：dryRun 与真实执行一致性（模拟数据）
- blocklist：过滤逻辑与边界值（空列表、大小写）

### 10.2 集成测试（端到端）
- 录制→产生活动→OCR→可被搜索命中→加入问答上下文→回答带溯源
- 隐私模式开关：开启后不再写入新活动
- 保留策略：触发清理后截图与 DB 一致

## 11. 实施路线图（建议）

- 里程碑 M1（P0）
  - 时间线搜索/过滤 + 发送到问答上下文
  - 黑名单贯通 + 隐私模式
- 里程碑 M2（P1）
  - 溯源 UI（引用活动跳转）
  - 保留策略闭环 + 维护操作入口
- 里程碑 M3（P1/P2）
  - embedding 队列与图谱增量
  - 每日/每周摘要 + 导出/备份

