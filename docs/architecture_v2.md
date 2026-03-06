# MemFlow 产品架构 v2.0 - 分离式设计

> 版本：v2.0
> 更新日期：2026-02-28
> 目标：实现「一套核心 → 桌面产品 + Developer（MCP）」两条产品线

---

## 1. 核心理念

### 1.1 核心原则

```
┌─────────────────────────────────────────────────────────────────┐
│                                                                 │
│   "Core 是引擎，UI 是遥控器，MCP 是接口"                          │
│                                                                 │
│   - Core/Daemon：负责采集、存储、索引、脱敏（无 UI 也能稳定跑）    │
│   - Desktop UI：负责展示、管理、控制（纯展示/控制，不做采集）      │
│   - MCP Server：负责读数据（只做工具接口，不做实时监控）           │
│                                                                 │
└─────────────────────────────────────────────────────────────────┘
```

### 1.2 架构演进

| 阶段 | 形态 | 特点 |
|------|------|------|
| **v1.0（当前）** | 桌面端 = UI + 采集 + MCP 全绑定 | 关闭窗口 = 采集停止 |
| **v2.0（目标）** | Core 独立运行 | UI 关闭不影响采集，MCP 不依赖 UI |

---

## 2. 分层架构

### 2.1 整体架构图

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
│                    │                               │                       │
│                    ▼                               ▼                       │
│   ┌─────────────────────────────┐   ┌─────────────────────────────────┐   │
│   │      Desktop UI (客户端)     │   │       MCP Server (开发者接口)   │   │
│   │                             │   │                                 │   │
│   │   ┌─────────────────────┐   │   │   ┌─────────────────────────┐   │   │
│   │   │   时间轴/搜索/图谱   │   │   │   │  search_memory         │   │   │
│   │   └─────────────────────┘   │   │   │  get_recent_activity   │   │   │
│   │   ┌─────────────────────┐   │   │   │  get_active_window_ctx │   │   │
│   │   │   录制控制 (开始/暂停) │   │   │   │  get_related_context  │   │   │
│   │   └─────────────────────┘   │   │   │  get_terminal_output  │   │   │
│   │   ┌─────────────────────┐   │   │   │  get_system_env       │   │   │
│   │   │   设置/隐私/Agent   │   │   │   └─────────────────────────┘   │   │
│   │   └─────────────────────┘   │   │                                 │   │
│   └─────────────────────────────┘   └─────────────────────────────────┘   │
│                                                                             │
│   ┌─────────────────────────────────────────────────────────────────────┐   │
│   │                     用户视角                                        │   │
│   │                                                                       │   │
│   │   普通用户：下载 MemFlow-Setup.exe → 桌面端 + Core                  │   │
│   │   开发者：  下载 MemFlow-Setup.exe → 桌面端 + Core + MCP(可选)       │   │
│   │                                                                       │   │
│   └─────────────────────────────────────────────────────────────────────┘   │
│                                                                             │
└─────────────────────────────────────────────────────────────────────────────┘
```

### 2.2 组件职责

| 组件 | 职责 | 运行方式 |
|------|------|----------|
| **Core / Daemon** | 采集、存储、索引、脱敏、RAG | Windows 服务 / 开机自启后台进程 |
| **Desktop UI** | 展示时间轴、搜索、控制录制 | 用户手动打开（可选开机启动） |
| **MCP Server** | 读数据接口（给 AI / IDE 用） | 按需启动（IDE 打开时） |

---

## 3. Core / Daemon 设计

### 3.1 核心能力

```rust
// 核心 API 设计（伪代码）

mod core {
    
    /// Core 状态
    #[derive(Debug, Clone, Serialize)]
    pub enum CoreState {
        Running,      // 正在采集
        Paused,       // 已暂停（用户手动暂停）
        Standby,      // 待机（无活动）
    }

    /// Core 配置
    #[derive(Debug, Clone, Deserialize, Serialize)]
    pub struct CoreConfig {
        pub recording_enabled: bool,      // 是否允许录制
        pub capture_screenshots: bool,    // 是否截图
        pub capture_terminal: bool,       // 是否捕获终端
        pub app_blacklist: Vec<String>,   // 应用黑名单
        pub max_storage_gb: u32,         // 最大存储 GB
        pub retention_days: u32,          // 保留天数
    }

    /// IPC 命令
    pub trait IpcCommand {
        // 控制类（谁调用都行：Desktop UI / MCP / 脚本）
        fn start_recording(&self) -> Result<()>;
        fn stop_recording(&self) -> Result<()>;
        fn get_status(&self) -> Result<CoreStatus>;
        fn update_config(&self, config: CoreConfig) -> Result<()>;

        // 查询类（给 MCP / UI 用）
        fn search_memory(&self, query: SearchQuery) -> Result<Vec<SearchResult>>;
        fn get_recent_activity(&self, minutes: u32) -> Result<Vec<Activity>>;
        fn get_active_window_context(&self) -> Result<WindowContext>;
    }
}
```

### 3.2 IPC 通信机制

```
┌─────────────────────────────────────────────────────────────────┐
│                      IPC 通信架构                               │
├─────────────────────────────────────────────────────────────────┤
│                                                                 │
│   ┌──────────┐    Unix Socket    ┌──────────────────────────┐  │
│   │Desktop UI│  ──────────────►  │   Core Daemon           │  │
│   │          │    (本地socket)    │   IPC Server            │  │
│   └──────────┘                   │   - Commands            │  │
│                                  │   - Events              │  │
│   ┌──────────┐    Unix Socket   │                          │  │
│   │MCP Server│  ──────────────►  │   ┌──────────────────┐   │  │
│   │          │                   │   │ 采集引擎         │   │  │
│   └──────────┘                   │   │ - WindowCapture  │   │  │
│                                  │   │ - OCR Worker      │   │  │
│                                  │   │ - TerminalCapture│   │  │
│   ┌──────────┐                   │   └──────────────────┘   │  │
│   │ 外部脚本 │  ──────────────►   │                          │  │
│   └──────────┘                   │   ┌──────────────────┐   │  │
│                                  │   │ 数据引擎         │   │  │
│                                  │   │ - SQLite         │   │  │
│                                  │   │ - Vector DB      │   │  │
│                                  │   └──────────────────┘   │  │
│                                  └──────────────────────────┘  │
│                                                                 │
│   通信协议：JSON-RPC 2.0 over Unix Socket                       │
│   端口：默认 ~/.memflow/core.sock                               │
│                                                                 │
└─────────────────────────────────────────────────────────────────┘
```

### 3.3 启动与管理

| 场景 | Core 行为 |
|------|----------|
| **桌面端启动** | Desktop UI 启动时检查 Core 是否运行，未运行则启动 |
| **MCP 启动** | MCP 启动时检查 Core 是否运行，未运行则启动（或提示用户打开桌面端） |
| **开机自启** | 注册 Windows 服务，开机自动运行 Core |
| **手动控制** | 用户通过 Desktop UI 托盘 / 快捷键控制录制状态 |

---

## 4. Desktop UI 设计

### 4.1 职责定位

```
┌─────────────────────────────────────────────────────────────────┐
│                      Desktop UI 职责                            │
├─────────────────────────────────────────────────────────────────┤
│                                                                 │
│   ✅ 正确：                                                   │
│   - 展示时间轴、搜索、图谱                                     │
│   - 提供录制控制 UI（开始/暂停/设置）                          │
│   - 调用 Core API 控制采集                                     │
│   - 用户设置管理                                               │
│                                                                 │
│   ❌ 不做：                                                   │
│   - 不做数据采集（采集由 Core 负责）                          │
│   - 不做 OCR 处理（由 Core 负责）                              │
│   - 不依赖"窗口打开"才能采集（Core 独立运行）                  │
│                                                                 │
└─────────────────────────────────────────────────────────────────┘
```

### 4.2 UI 组件

| 组件 | 功能 |
|------|------|
| **Timeline** | 按时间展示活动记录 |
| **Search** | 调用 Core API 搜索 |
| **RecordingControl** | 录制状态切换（调用 Core start/stop） |
| **Settings** | 配置管理（调用 Core update_config） |
| **Tray** | 托盘图标，快速控制 |

### 4.3 与 Core 通信

```typescript
// Desktop UI 调用 Core API（通过 Tauri Commands）

// 1. 录制控制
await invoke('core_start_recording');
await invoke('core_stop_recording');
const status = await invoke<CoreStatus>('core_get_status');

// 2. 搜索（实际调用 Core，Core 内部用 RAG）
const results = await invoke('core_search_memory', {
  query: '用户输入',
  mode: 'hybrid',
  limit: 10
});

// 3. 配置
await invoke('core_update_config', {
  recording_enabled: true,
  app_blacklist: ['passwordmanager']
});
```

---

## 5. MCP Server 设计

### 5.1 职责定位

```
┌─────────────────────────────────────────────────────────────────┐
│                      MCP Server 职责                           │
├─────────────────────────────────────────────────────────────────┤
│                                                                 │
│   ✅ 正确：                                                   │
│   - 读数据接口（搜索、时间线、上下文）                          │
│   - 调用 Core API 获取数据                                     │
│   - 只做"工具层"，不采集数据                                   │
│                                                                 │
│   ❌ 不做：                                                   │
│   - 不做实时窗口监控                                           │
│   - 不做 OCR 处理                                              │
│   - 不依赖"桌面窗口打开"                                       │
│                                                                 │
│   核心价值：让 IDE / Cursor 中的 AI 能访问用户的记忆            │
│                                                                 │
└─────────────────────────────────────────────────────────────────┘
```

### 5.2 工具列表

| 工具 | 功能 | 数据来源 |
|------|------|----------|
| `search_memory` | 语义/关键词/混合搜索 | Core RAG |
| `get_recent_activity` | 最近 N 分钟活动 | Core DB |
| `get_active_window_context` | 当前窗口 + 上下文 | Core DB |
| `get_related_context` | 与 query 相关的上下文片段 | Core RAG |
| `get_terminal_output` | 终端最近输出 | Core DB |
| `get_system_environment` | 系统环境信息 | Core（调用系统 API） |

### 5.3 与 Core 通信

```rust
// MCP 调用 Core（通过 IPC）

fn search_memory_handler(params: SearchParams) -> Result<String> {
    // 1. 连接 Core IPC
    let client = IpcClient::connect()?;
    
    // 2. 调用 Core 搜索
    let results = client.search_memory(params)?;
    
    // 3. 返回格式化文本
    Ok(format_results(results))
}

fn get_recent_activity_handler(minutes: u32) -> Result<String> {
    let client = IpcClient::connect()?;
    let activities = client.get_recent_activity(minutes)?;
    Ok(format_timeline(activities))
}
```

---

## 6. 打包与分发

### 6.1 安装包设计

```
┌─────────────────────────────────────────────────────────────────┐
│                      安装包矩阵                                 │
├─────────────────────────────────────────────────────────────────┤
│                                                                 │
│   安装包                     包含内容                            │
│   ─────────────────────────────────────────────────────────    │
│                                                                 │
│   MemFlow-Setup.exe        │  Core (开机自启)                   │
│   (普通用户版)              │  Desktop UI                       │
│                            │  （不含 MCP）                      │
│                                                                 │
│   MemFlow-Setup.exe        │  Core (开机自启)                   │
│   (开发者模式勾选)          │  Desktop UI                       │
│                            │  MCP Server (可选)                 │
│                                                                 │
│   MemFlow-MCP-Standalone   │  MCP Server (独立包)              │
│   (可选)                   │  （需要 Core 运行）                │
│                                                                 │
└─────────────────────────────────────────────────────────────────┘
```

### 6.2 用户体验

| 用户群体 | 首次使用 | 日常使用 |
|----------|---------|----------|
| **普通用户** | 下载安装包 → 一路下一步 → 完成 | 开机自动采集，桌面端按需打开查看 |
| **开发者** | 下载安装包 → 勾选"安装 MCP" → 完成 | 开机自动采集 + MCP 随 IDE 启动 |

### 6.3 MCP 启动逻辑

```rust
// MCP Server 启动流程

fn main() {
    // 1. 检查 Core 是否运行
    if !check_core_running() {
        // 2. 尝试启动 Core（或者提示用户）
        if let Err(e) = start_core() {
            eprintln!("Core 未运行，请打开 MemFlow 桌面端");
            std::process::exit(1);
        }
    }

    // 3. 连接 Core IPC
    let client = IpcClient::connect_or_retry()?;

    // 4. 启动 MCP Server
    run_mcp_server(client);
}
```

---

## 7. 实施路线图

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

## 8. 附录

### 8.1 IPC 命令清单

```json
{
  "control": {
    "core_start_recording": {},
    "core_stop_recording": {},
    "core_get_status": {},
    "core_update_config": {
      "recording_enabled": "bool",
      "app_blacklist": ["string"],
      "retention_days": "u32"
    }
  },
  "query": {
    "core_search_memory": {
      "query": "string",
      "mode": "hybrid|semantic|keyword",
      "limit": "u32"
    },
    "core_get_recent_activity": {
      "minutes": "u32"
    },
    "core_get_active_window_context": {},
    "core_get_related_context": {
      "query": "string",
      "limit": "u32"
    }
  }
}
```

### 8.2 错误码

| 错误码 | 含义 |
|--------|------|
| `-32000` | 未找到 |
| `-32001` | 参数错误 |
| `-32002` | 服务不可用 |
| `-32009` | Core 未运行 |
| `-32010` | 降级模式（Core 不可用，MCP 可用但功能受限） |

---

> 文档版本：v2.0
> 维护者：MemFlow Team
> 更新：2026-02-28
