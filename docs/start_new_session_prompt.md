# MemFlow 架构 v2.0 实施 - 新会话 Prompt

---

## 项目背景

**项目名称**：MemFlow - 本地"记忆大脑"

**核心理念**：「Core 是引擎，UI 是遥控器，MCP 是接口」

- Core/Daemon：负责采集、存储、索引、脱敏（无 UI 也能稳定跑）
- Desktop UI：负责展示、管理、控制（纯展示/控制，不做采集）
- MCP Server：负责读数据（只做工具接口，不做实时监控）

---

## 当前状态

### 现有代码结构

```
memflow/
├── crates/
│   ├── memflow-core/          # 核心引擎（含 OCR 增强、DB、RAG）
│   │   ├── src/
│   │   │   ├── db.rs
│   │   │   ├── ai/ (embedding, rag, nlp)
│   │   │   ├── ocr_enhance.rs
│   │   │   ├── redact.rs
│   │   │   └── context.rs (RuntimeContext)
│   │   └── tests/
│   └── memflow-mcp/           # MCP Server
│       ├── src/
│       │   ├── protocol.rs (JSON-RPC)
│       │   ├── server.rs
│       │   ├── tools.rs
│       │   └── main.rs
│       └── tests/
├── src-tauri/                 # 桌面端后端（当前耦合采集逻辑）
│   └── src/
│       ├── recorder.rs
│       ├── ocr_worker.rs
│       ├── commands.rs (Tauri Commands)
│       └── ...
└── src/                      # 桌面端前端（React）
    └── components/
```

### 关键问题

1. **Core 和桌面端耦合太紧** - `src-tauri` 里直接调用很多 core 功能，采集逻辑混在一起
2. **MCP 还没完全独立** - 还在依赖"桌面端必须开着"
3. **daemon 模式没有** - 没有实现"无 UI 也能跑"

---

## 目标架构

```
┌─────────────────────────────────────────────────────────────────────────────┐
│                           MemFlow 产品矩阵                                  │
├─────────────────────────────────────────────────────────────────────────────┤
│                                                                             │
│   ┌─────────────────────────────────────────────────────────────────────┐   │
│   │                     Core / Daemon（核心引擎）                       │   │
│   │                                                                       │   │
│   │   ┌─────────────┐  ┌─────────────┐  ┌─────────────┐  ┌───────────┐  │   │
│   │   │  数据采集   │  │  存储/索引  │  │  OCR 增强   │  │  隐私脱敏 │  │   │
│   │   │ (窗口/终端) │  │ (SQL/向量)  │  │ (预处理/后处理)│  │(邮箱/Token)│  │   │
│   │   └─────────────┘  └─────────────┘  └─────────────┘  └───────────┘  │   │
│   │                                                                       │   │
│   │   ┌───────────────────────────────────────────────────────────────┐  │   │
│   │   │                     IPC API Server                           │  │   │
│   │   │    - start_recording / stop_recording                       │  │   │
│   │   │    - get_status / get_config                                 │  │   │
│   │   │    - search_memory / get_recent_activity                     │  │   │
│   │   └───────────────────────────────────────────────────────────────┘  │   │
│   │                                                                       │   │
│   └─────────────────────────────────────────────────────────────────────┘   │
│                                    │                                         │
│                    ┌───────────────┴───────────────┐                       │
│                    ▼                               ▼                       │
│   ┌─────────────────────────────┐   ┌─────────────────────────────────┐   │
│   │      Desktop UI (客户端)     │   │       MCP Server (开发者接口)   │   │
│   │                             │   │                                 │   │
│   │   时间轴/搜索/图谱/控制      │   │   search_memory/get_recent_... │   │
│   │   (纯展示/调用Core API)     │   │   (纯读接口/不依赖UI)          │   │
│   └─────────────────────────────┘   └─────────────────────────────────┘   │
│                                                                             │
└─────────────────────────────────────────────────────────────────────────────┘
```

---

## 核心设计决策

### 1. IPC 通信

- **协议**：JSON-RPC 2.0 over Unix Socket
- **端口**：默认 `~/.memflow/core.sock`
- **实现**：Core 启动 IPC Server，Desktop UI/MCP 通过 socket 连接

### 2. 控制权不丢失

- **误解**："Core 独立 = 不能控制"
- **真相**：Core 是"执行引擎"，UI 是"遥控器"
- 用户点"开始录制" → Desktop UI 调用 Core API → Core 执行

### 3. 打包策略

| 用户 | 下载 | 包含 |
|------|------|------|
| 普通用户 | 标准版 | Core + Desktop UI |
| 开发者 | 标准版 + MCP | Core + Desktop UI + MCP（可选） |

---

## IPC 命令清单

### 控制类

```rust
// 命令
core_start_recording()      // 开始录制
core_stop_recording()      // 停止录制
core_get_status() -> CoreStatus  // 获取状态
core_update_config(config) // 更新配置

// 状态枚举
enum CoreState {
    Running,   // 正在采集
    Paused,    // 已暂停
    Standby,   // 待机
}
```

### 查询类

```rust
core_search_memory(query, mode, limit)
core_get_recent_activity(minutes)
core_get_active_window_context()
core_get_related_context(query, limit)
```

---

## 实施路线图

### Phase 1：Core 分离（独立进程）

- [ ] 将采集逻辑从 Tauri 拆分到独立 Core 进程
- [ ] 实现 IPC Server（Unix Socket）
- [ ] 实现 Core 状态管理（Running/Paused）
- [ ] 实现开机自启动（Windows 服务）

### Phase 2：Desktop UI 改造

- [ ] Desktop UI 改为"纯客户端"，调用 Core API
- [ ] 保留录制控制 UI，但实际调用 Core
- [ ] 托盘功能与 Core 状态同步

### Phase 3：MCP 独立

- [ ] MCP 改为通过 IPC 调用 Core
- [ ] MCP 不依赖桌面窗口
- [ ] 实现 MCP 降级策略（Core 不可用时）

### Phase 4：打包配置

- [ ] 实现"开发者模式"安装选项
- [ ] MCP 独立安装包（可选）
- [ ] 验证两种用户体验

---

## 重要文件位置

| 用途 | 文件路径 |
|------|----------|
| 架构文档 | `docs/architecture_v2.md` |
| 实施清单 | `docs/implementation_todo.md` |
| MCP 工具契约 | `docs/MCP_TOOL_CONTRACT_v1.md` |
| 核心 DB | `crates/memflow-core/src/db.rs` |
| Core 上下文 | `crates/memflow-core/src/context.rs` |
| Tauri 命令 | `src-tauri/src/commands.rs` |
| 采集逻辑 | `src-tauri/src/recorder.rs`, `ocr_worker.rs` |

---

## 当前分支状态

- **当前分支**：`main`
- **状态**：有大量未提交改动
- **主要改动**：OCR 增强、Embedding 管理、MCP 工具增强

---

## 开始实施

建议从 **Phase 1** 开始：

1. **先拆分 Core 进程** - 将 `src-tauri` 中的采集逻辑抽到 `crates/memflow-core`
2. **实现 IPC 层** - Core 暴露 Unix Socket 接口
3. **改造 Desktop UI** - 变成纯客户端
4. **改造 MCP** - 通过 IPC 读数据

**关键原则**：Core 必须是"无 UI 也能跑"的长期进程，不依赖桌面窗口。

参考> 文档：`docs/architecture_v2.md`


---

> 生成时间：2026-02-28
> 文档：`docs/architecture_v2.md`
