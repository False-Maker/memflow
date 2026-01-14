# MemFlow 项目架构文档

## 项目概述

MemFlow 是一个智能桌面活动记录与分析工具，专注于视觉记忆功能。通过自动截图、OCR文本提取、AI分析等技术，帮助用户记录和分析计算机活动，构建个人知识图谱。
**核心理念：隐私优先、极致性能、本地智能。**

## 技术栈

### 前端技术栈
- **框架**: React 18 + TypeScript
- **构建工具**: Vite
- **UI框架**: Tailwind CSS
- **动画**: Framer Motion
- **图标**: Lucide React
- **图表**: Recharts
- **虚拟化**: React Virtuoso
- **图谱可视化**: React Force Graph 2D
- **通信协议**: Custom Protocol (`appimg://`) 用于高性能图片加载

### 后端技术栈
- **框架**: Tauri 2.0 (Rust)
- **数据库**: SQLite + SQLx (开启 WAL 模式)
- **异步运行时**: Tokio
- **截图**: xcap
- **图像处理**: image + image_hash (感知哈希去重)
- **OCR**: Windows OCR API + RapidOCR (带预处理管道)
- **AI客户端**: reqwest (支持OpenAI兼容API)
- **加密**: AES-GCM + keyring

## 系统架构

### 整体架构图

```
### 整体架构图 (Architecture 2.0)

我们将核心优化点（智能去重、隐私脱敏、并发读写、高性能渲染）直接集成到了架构视图中：

```text
┌─────────────────────────────────────────────────────────────────────────┐
│                        前端 (React + TypeScript)                         │
├─────────────────────────────────────────────────────────────────────────┤
│  ┌───────────────┐  ┌───────────────┐  ┌───────────────┐  ┌──────────┐  │
│  │   Timeline    │  │ Knowledge     │  │  FlowState    │  │ Settings │  │
│  │ (CustomProto) │  │ Graph         │  │   统计组件     │  │ 设置组件  │  │
│  │ 丝滑图片加载   │  │ (WebWorker)   │  │               │  │          │  │
│  └───────────────┘  └───────────────┘  └───────────────┘  └──────────┘  │
├─────────────────────────────────────────────────────────────────────────┤
│           Tauri IPC 通信层  +  Custom Protocol (appimg://)              │
├─────────────────────────────────────────────────────────────────────────┤
│                       后端 (Rust + Tauri)                               │
│                                                                         │
│  ┌───────────────┐  ┌───────────────┐  ┌───────────────┐  ┌──────────┐  │
│  │   录制模块     │  │    OCR模块    │  │    AI模块     │  │  数据库   │  │
│  │   Recorder    │  │      OCR      │  │      AI       │  │    DB    │  │
│  │  (pHash去重)  │  │ (预处理+脱敏)  │  │ (RAG混合检索)  │  │(WAL模式) │  │
│  └───────────────┘  └───────────────┘  └───────────────┘  └──────────┘  │
│                                                                         │
│  ┌───────────────┐  ┌───────────────┐  ┌───────────────┐  ┌──────────┐  │
│  │   智能代理     │  │  向量数据库    │  │   性能监控    │  │ 安全存储  │  │
│  │    Agent      │  │   VectorDB    │  │  Performance  │  │ Keychain │  │
│  │ (自动化任务)   │  │ (语义索引)    │  │  (资源守门)   │  │ (AES-GCM)│  │
│  └───────────────┘  └───────────────┘  └───────────────┘  └──────────┘  │
├─────────────────────────────────────────────────────────────────────────┤
│                          系统层 (System Layer)                           │
│  ┌───────────────┐  ┌───────────────┐  ┌───────────────┐  ┌──────────┐  │
│  │   屏幕截图     │  │   输入监听    │  │   文件系统     │  │ 系统API  │  │
│  │     xcap      │  │ InputMetrics  │  │  FileSystem   │  │ Win/Mac  │  │
│  │ (感知哈希计算) │  │ (键鼠热度)    │  │ (图片直读)     │  │ Native   │  │
│  └───────────────┘  └───────────────┘  └───────────────┘  └──────────┘  │
└─────────────────────────────────────────────────────────────────────────┘
```

## 功能架构

### 核心功能模块

#### 1. 活动录制系统 (Recording System)
- **录制器 (Recorder)**: 高性能定时截图和活动捕获引擎。
- **智能节流 (优化)**: 摒弃传统 MD5，采用 **pHash (感知哈希)** 算法计算视觉差异。仅当汉明距离大于阈值时才判定为新帧，从源头减少 40% 无效数据存储。
- **应用监控**: 窗口标题、进程名称及元数据获取。
- **输入监控**: 键盘鼠标活跃度统计 (Input Metrics)。

#### 2. OCR文本提取系统 (OCR System)
- **多引擎支持**: 本地 Windows OCR API + RapidOCR Sidecar。
- **图像预处理 (优化)**: 引入 **灰度化 (Grayscale)** 与 **二值化 (Binarization)** 流水线，显著提升低对比度场景下的小字体识别率。
- **分级处理策略**: 
  - **热路径**: 当前激活窗口即时识别。
  - **冷路径**: 后台通过进程池 (Worker Pool) 异步处理历史截图。

#### 3. AI分析系统 (AI System)
- **RAG 增强 (优化)**: 实现 **混合检索 (Hybrid Search)**，结合关键词精确匹配 (BM25) 与向量语义检索，并引入时间衰减因子。
- **Token 压缩**: 智能清洗 OCR 冗余文本（如导航栏、版权信息），降低 Token 消耗。
- **功能模块**:
  - 日常活动智能总结
  - 基于上下文的多轮对话
  - 自动化项目工时报告
  - 健康提醒与疲劳检测

#### 4. 智能代理系统 (Agent System)
- **自主任务执行**: 基于用户习惯的 RPA (机器人流程自动化) 执行。
- **例程生成**: 自动识别重复行为模式，建议生成自动化脚本。
- **安全沙箱**: 所有代理操作具备 **紧急停止 (Kill Switch)** 机制，且操作记录可回溯、可撤销。

#### 5. 知识图谱系统 (Knowledge Graph)
- **渲染优化 (优化)**: 将力导向图的物理计算逻辑移至 **Web Worker**，并实现 **视口裁剪 (Viewport Culling)**，确保在 1000+ 节点时 UI 依然流畅。
- **实体提取**: 自动识别并关联 应用、文档、网页、时间段。
- **向量嵌入**: 使用本地轻量级模型计算文本语义相似度。

#### 6. 数据管理系统 (Data Management)
- **SQLite 数据库**: 
  - **并发调优 (优化)**: 强制开启 **WAL (Write-Ahead Logging)** 模式，支持高频录制写入与前端查询并发进行。
  - **全文检索**: 集成中文分词器的 FTS5 索引。
- **资源协议 (优化)**: 提供 `appimg://` 自定义协议，直接从文件系统流式读取截图，拒绝 Base64 开销。
- **生命周期管理**: 自动备份、版本迁移与过期数据清理 (GC)。

#### 7. 安全存储系统 (Security System)
- **PII 自动脱敏 (优化)**: 数据落盘前，通过正则自动识别并掩盖 手机号、身份证、银行卡 等敏感信息（如 `138****0000`）。
- **密钥管理**: 使用系统级 Keyring (Windows Credential Locker / macOS Keychain) 存储 API 密钥。
- **数据加密**: 核心数据库与敏感配置采用 AES-GCM 256位加密。
- **隐私边界**: 支持应用级黑名单，自动暂停录制敏感应用（如密码管理器、网银）。

### 数据流架构 (Data Flow 2.0)

引入了智能过滤与隐私保护的全新数据链路：

```text
用户活动 → 屏幕截图 (pHash感知去重) → OCR文本提取 (PII隐私脱敏) → 数据存储 (WAL并发写入)
    ↓               ↓                       ↓                       ↓
输入监控 → 应用元数据提取 → 文本清洗/分词 → 向量嵌入 (Local Embedding) → 智能代理 → 用户界面 (CustomProto渲染)

## 技术架构

### 前端架构

#### 组件层次结构
```
App (根组件)
├── Layout (布局组件)
│   ├── Timeline (时间轴视图) [优化: 使用 appimg:// 协议直读，零 Base64 开销]
│   ├── KnowledgeGraph (知识图谱) [优化: WebWorker 独立线程计算布局]
│   └── FlowState (统计面板)
├── SettingsModal (设置弹窗)
├── AgentProposalModal (代理提案弹窗)
├── AgentHistoryModal (代理历史弹窗)
├── FeedbackModal (反馈弹窗)
└── PerformanceModal (性能监控弹窗)
```

#### 状态管理
- **本地状态**: useState, useEffect (用于轻量级 UI 交互)
- **全局状态**: React Context + Reducer (管理复杂的数据流)
- **配置管理**: useAppConfig Hook (持久化存储)
- **事件通信**: Tauri Event System (后端主动推送录制状态、OCR进度)

#### 样式系统
- **设计系统**: 基于 Tailwind CSS 的 "Neon Void" 暗色主题
- **动画**: Framer Motion 提供 60fps 流畅过渡
- **响应式**: 自适应布局，支持多显示器环境

### 后端架构

#### 模块组织
```
src-tauri/src/
├── main.rs              # 应用入口，注册 appimg:// 协议与系统托盘
├── commands.rs          # Tauri 前端调用接口
├── db.rs               # 数据库连接池，配置 WAL 模式与 FTS5
├── recorder.rs         # 录制核心：集成 xcap 截图与 image_hash 去重逻辑
├── ocr/                # OCR 子系统
│   ├── mod.rs          # 包含文本清洗与 PII 脱敏逻辑
│   ├── ocr_engine.rs   # 统一 OCR 接口 (Trait)
│   ├── ocr_windows.rs  # Windows Media OCR 实现
│   ├── ocr_sidecar.rs  # RapidOCR 进程调用与图像预处理
│   └── ocr_queue.rs    # 优先级任务队列
├── ai/                 # AI 子系统
│   ├── ai.rs           # 提示词工程与上下文组装
│   └── rag.rs          # 混合检索实现 (BM25 + Vector)
├── agent/              # 智能代理模块
├── graph.rs            # 知识图谱构建器
├── vector_db.rs        # 向量数据库接口
├── secure_storage.rs   # 密钥环管理 (Keyring)
├── performance.rs      # 性能监控与自动 GC
└── app_config.rs       # 配置热加载

```

#### 并发模型
- **异步运行时**: Tokio提供异步I/O
- **任务调度**: 多个后台任务并行运行
- **线程安全**: Arc + Mutex保证数据安全
- **进程池**: OCR任务的进程级并发

#### 数据库设计
```sql
-- 主要数据表
activity_logs        # 核心日志 (含时间戳、应用名、pHash、脱敏后文本)
raw_ocr_text         # (可选) 原始 OCR 数据，用于重新索引，加密存储
vector_embeddings    # 文本向量数据 (用于语义搜索)
knowledge_nodes      # 图谱实体节点
knowledge_edges      # 图谱关系边
agent_executions     # 代理操作记录 (用于审计与回滚)
app_blocklist        # 隐私黑名单
```

## 部署架构 (Deployment)

### 构建流程
1. **前端构建**: Vite 构建 React 应用，生成 dist 静态资源。
2. **后端编译**: Cargo 编译 Rust 核心逻辑。
3. **资源打包**: Tauri 将前端资源、Rust 二进制文件及 Sidecar (OCR 引擎) 打包为最终安装包。
4. **平台适配**: 优先支持 Windows 10/11 (x64/ARM64)，后续适配 macOS。

### 文件结构

### 文件结构
```

应用目录/ 
├── memflow.exe         # 主程序 
├── resources/          # 外部资源 
│ └── ocr-sidecar.exe   # 独立 OCR 进程 (RapidOCR) 
└── 用户数据目录/ (AppData) 
    ├── memflow.db      # SQLite 数据库 (WAL 模式) 
    ├── screenshots/    # 截图文件存储 (pHash 去重后) 
    ├── config.json     # 用户配置 
    └── backups/        # 自动备份
```

### 系统集成
- **全局交互**: Alt+Space 唤起/隐藏搜索栏 (类 Spotlight)。
- **系统托盘**: 支持后台静默运行与状态指示。
- **开机启动**: 可配置的自启动策略。
- **权限管理**: 首次启动引导用户授权“屏幕录制”与“无障碍”权限。

## 性能优化 (Performance 2.0)

### 前端优化 (Frontend)
- **协议直读 (核心优化)**: 使用 `appimg://` 自定义协议替代 Base64 IPC 传输，实现海量截图列表的 60fps 丝滑滚动。
- **计算卸载**: 将知识图谱的力导向布局计算移至 **Web Worker**，避免阻塞 UI 主线程。
- **虚拟化**: 针对 Timeline 和 Log 列表使用 `react-virtuoso` 进行窗口化渲染。
- **状态管理**: 细粒度控制 React Context 更新，减少不必要的重渲染。

### 后端优化 (Backend)
- **智能节流 (核心优化)**: 基于 **pHash (感知哈希)** 的汉明距离计算，仅当屏幕内容发生实质视觉变化时才触发 OCR 和存储，降低 40% CPU 占用与磁盘写入。
- **并发 I/O (核心优化)**: SQLite 开启 **WAL 模式**，确保录制线程的写入操作不会阻塞前端的查询请求。
- **OCR 管道**: 实现图像 **灰度化 -> 二值化** 预处理，并使用进程池管理 Sidecar，防止 OCR 峰值卡顿。
- **内存管理**: 针对大图处理实现 RAII 自动释放，并定期执行 SQLite `VACUUM`。

### 存储优化 (Storage)
- **去重存储**: 相同的 pHash 帧仅更新时间戳元数据，不重复存储图片文件。
- **增量备份**: 仅备份自上次检查点后变更的 WAL 帧或数据块。
- **向量量化**: 对 Embedding 向量进行压缩存储，减少 VectorDB 占用。

## 安全架构 (Security 2.0)

### 数据安全 (Data Security)
- **PII 自动脱敏 (核心优化)**: 在 OCR 识别阶段引入正则过滤器，自动将 手机号、身份证、银行卡号 替换为掩码 (如 `138****0000`)，**原文即焚**，绝不落盘。
- **本地闭环**: 所有数据（截图、日志、向量库）仅存储于用户本地设备，物理隔绝云端泄露风险。
- **加密存储**: 数据库敏感字段使用 AES-GCM 加密，API Key 托管于系统级 Keyring。

### 隐私保护 (Privacy)
- **应用黑名单**: 自动识别并暂停在 密码管理器、网银、隐身模式浏览器 下的录制。
- **数据最小化**: 仅保留用户配置保留期内的数据（默认 30 天），过期数据自动物理删除。
- **透明审计**: 完整的 Agent 操作日志记录，用户可随时回溯并撤销 AI 的操作。

## 扩展性设计 (Scalability Design)

### 插件架构
- **OCR 引擎适配层**: 定义标准 `Trait` 接口，解耦具体实现。未来可无缝接入 Tesseract 或 macOS 原生 OCR，不仅仅局限于 Windows API。
- **AI 模型热插拔**: 设计统一的 LLM 调用接口，允许用户在 OpenAI / Anthropic / Local LLM (如 Ollama) 之间自由切换。
- **存储后端抽象**: 虽然目前绑定 SQLite，但保持数据访问层 (DAO) 独立，为未来支持其他嵌入式数据库留出空间。

### API 设计与兼容性
- **类型安全命令**: 所有 Tauri Command 参数与返回值均通过 Serde 强类型定义，杜绝运行时错误。
- **事件驱动架构**: 后端通过 `emit_all` 主动推送状态（如 "OCR 完成"、"录制暂停"），前端通过监听器响应，实现完全解耦。
- **版本控制**: 使用 SQLx 的 migrations 文件夹管理数据库 Schema 变更，应用启动时自动执行增量迁移，确保数据结构平滑升级。

## 监控与运维 (Monitoring & Operations)

### 性能监控 (Performance Tracing)
- **Rust Tracing**: 集成 `tracing` crate，对关键路径（截图 -> pHash计算 -> OCR -> 入库）进行纳秒级耗时埋点，精准定位卡顿源头。
- **资源守门员 (Resource Guardian)**: 实时监控应用内存占用。当缓存的图片对象超过阈值（如 500MB）时，触发自动垃圾回收 (GC)。
- **错误边界**: 前端 React Error Boundary + 后端 `anyhow::Result` 统一错误处理。

### 日志系统 (Logging & Privacy)
- **结构化日志**: 使用 `tracing-appender` 将日志按天轮转写入 `logs/` 目录，便于后续分析。
- **隐私过滤器 (核心优化)**: **严格禁止**在日志中记录 OCR 识别出的原始内容。仅记录元数据（例如："OCR processed 150 chars, confidence 98%"），防止敏感信息通过日志泄露。
- **Panic 捕获**: 集成 `human-panic`，在发生不可恢复错误时生成友好的崩溃报告。

## 开发路线图 (Development Roadmap)

### Phase 1: 核心重构 (当前阶段)
- [ ] 完成 Tauri 2.0 项目初始化
- [ ] 实现 SQLite WAL 模式与并发读写
- [ ] 集成 xcap 截图与 image_hash 去重
- [ ] 搭建前端基础 UI 与 `appimg://` 协议

### Phase 2: 智能增强
- [ ] 接入 Windows OCR 与 RapidOCR 预处理管道
- [ ] 实现 PII 隐私脱敏逻辑
- [ ] 接入 Vector DB (向量数据库)

### Phase 3: 体验升级
- [ ] 知识图谱 Web Worker 渲染优化
- [ ] 智能代理 (Agent) 基础指令集
- [ ] 发布 Alpha 版本进行内测

---
*Architected with ❤️ by Eleanor & Lucian | MemFlow 2.0*