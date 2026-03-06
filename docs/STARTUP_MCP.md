# MCP 产品启动手册

## 产品说明

**MCP 产品 = Core + MCP Server**

- Core：负责数据采集、存储、索引
- MCP Server：通过 JSON-RPC 与 IDE 通信，提供搜索工具
- 适用：**开发人员**

> **重要**：MCP Server 必须配合 Core 一起使用，单独运行 MCP Server 无法工作。

---

## 快速启动

### 步骤 1：启动 Core

```bash
# 编译并运行 Core
cargo run --package memflow-daemon
```

或使用编译好的 exe：
```bash
.\target\release\memflow-daemon.exe
```

确认输出显示：
```
IPC server listening on 0.0.0.0:9516
```

---

### 步骤 2：启动 MCP Server

```bash
# 编译并运行 MCP
cargo run --package memflow-mcp
```

或使用编译好的 exe：
```bash
.\target\release\memflow-mcp.exe
```

MCP 启动后会：
1. 连接 Core IPC（端口 9516）
2. 等待 stdin 输入（JSON-RPC）

---

## 托盘模式（开机自启）

MCP 用户也可以使用**托盘模式**来控制 Core：

### 启动托盘模式

```bash
.\target\release\memflow-daemon.exe --tray
```

或直接运行桌面端，选择「最小化到托盘」：

```
┌─────────────────────────┐
│  MemFlow               │
├─────────────────────────┤
│  ▶ 暂停录制             │
│  ✓ 开机自启             │
│  ─────────────          │
│  设置...                 │
│  退出                    │
└─────────────────────────┘
```

### 托盘功能

| 功能 | 说明 |
|------|------|
| **托盘图标** | 显示录制状态（绿=录制中，红=暂停） |
| **右键菜单** | 暂停/恢复、设置、开机自启、退出 |
| **全局快捷键** | `Ctrl+Shift+P` 切换录制状态 |
| **开机自启** | 勾选后 Core 开机自动运行 |

### 开机自启

在托盘菜单中勾选「开机自启」后：
- Core 会在 Windows 启动时自动运行
- 用户登录后自动进入托盘模式（无窗口）
- MCP 可以随时连接 Core 进行搜索

---

## 配置到 Cursor/VS Code

### 1. 编译 MCP 为可执行文件

```bash
cargo build --release --package memflow-daemon
cargo build --release --package memflow-mcp
```

需要两个 exe：
- `target\release\memflow-daemon.exe`
- `target\release\memflow-mcp.exe`

### 2. 配置 Cursor

打开 `C:\Users\<用户名>\AppData\Roaming\Cursor\User\settings.json`

添加：

```json
{
  "mcpServers": {
    "memflow": {
      "command": "D:\\Demo\\memflow\\target\\release\\memflow-mcp.exe",
      "env": {}
    }
  }
}
```

### 3. 启动方式

MCP 产品需要**两个进程**同时运行：

```
┌─────────────────────────────────────────────────────────┐
│                    MCP 产品启动方式                       │
├─────────────────────────────────────────────────────────┤
│                                                         │
│  方式一：手动启动                                        │
│  ┌────────────────┐    ┌────────────────┐                │
│  │ daemon.exe    │ +  │ mcp.exe       │                │
│  │ (Core)        │    │ (MCP Server)  │                │
│  └────────────────┘    └────────────────┘                │
│                                                         │
│  方式二：托盘模式（推荐）                                 │
│  ┌──────────────────────────────┐                       │
│  │ daemon.exe --tray           │                       │
│  │   - 托盘图标               │                       │
│  │   - 支持开机自启           │                       │
│  └──────────────────────────────┘                       │
│  ┌──────────────────────────────┐                       │
│  │ mcp.exe                     │                       │
│  └──────────────────────────────┘                       │
│                                                         │
└─────────────────────────────────────────────────────────┘
```

### 4. 重启 Cursor

重启后 MCP 连接成功，你可以在 Cursor 中使用以下工具：

| 工具 | 功能 |
|------|------|
| `search_memory` | 语义/关键词/混合搜索 |
| `get_recent_activity` | 最近 N 分钟活动 |
| `get_active_window_context` | 当前窗口 + 上下文 |
| `get_related_context` | 与 query 相关的上下文片段 |
| `get_terminal_output` | 终端最近输出 |
| `get_system_environment` | 系统环境信息 |
| `capture_screenshot` | 立即截图（30s 内复用已有截图） |

---

### capture_screenshot - 立即截图

AI 可以主动调用此工具截取当前屏幕。

**防重复机制**：调用时会检查最近 30 秒内是否有截图，如果有则返回已有的截图（不重复截），确保不与 Core 自动截图冲突。

**返回字段**：
| 字段 | 类型 | 说明 |
|------|------|------|
| `path` | string | 截图文件路径 |
| `is_new` | boolean | 是否为新截图（true=新截的，false=复用的） |
| `data` | string | base64 编码的图片数据（可选） |

**调用示例**：
```json
{
  "jsonrpc": "2.0",
  "method": "tools/call",
  "params": {
    "name": "capture_screenshot"
  }
}
```

**返回示例**：
```json
{
  "jsonrpc": "2.0",
  "id": 1,
  "result": {
    "content": [
      {
        "type": "image",
        "path": "C:\\Users\\wangx\\AppData\\Roaming\\MemFlow\\screenshots\\20260302_143052.png",
        "is_new": false
      }
    ]
  }
}
```

---

## 测试 MCP

### 手动测试

```bash
# 启动 MCP 后，输入 JSON-RPC 请求
{"jsonrpc":"2.0","method":"initialize","id":1}
```

应返回：
```json
{"jsonrpc":"2.0","id":1,"result":{"protocolVersion":"2024-11-05",...}}
```

### Cursor 中测试

在 Cursor 对话框中输入：
```
搜索一下我今天下午打开的代码文件
```

---

## 常见问题

### Q: MCP 启动报错 "Core unavailable"

**原因**：Core (daemon) 未运行

**解决**：先启动 `memflow-daemon.exe`

### Q: Cursor 提示 "MCP server disconnected"

**原因**：MCP 进程崩溃或 Core 断开

**解决**：
1. 检查 Core 是否还在运行
2. 重启 Cursor

### Q: 日志在哪里？

- Core 日志：`%APPDATA%\MemFlow\logs\`
- MCP 日志输出到 **stderr**（Cursor 控制台）

---

## 端口说明

| 端口 | 用途 |
|------|------|
| 9516 | Core IPC 服务端口 |

MCP 通过 TCP 连接 Core。

---

## 打包发布

### MCP 产品独立包

```bash
# 1. 编译两个组件
cargo build --release --package memflow-daemon
cargo build --release --package memflow-mcp

# 2. 创建发布目录
mkdir dist-mcp
cp target/release/memflow-daemon.exe dist-mcp/
cp target/release/memflow-mcp.exe dist-mcp/

# 3. 复制资源文件
cp -r src-tauri/resources dist-mcp/

# 4. 打包成 zip
powershell Compress-Archive -Path dist-mcp -DestinationPath MemFlow-MCP-v0.1.0.zip
```

### 输出内容

```
MemFlow-MCP-v0.1.0.zip
├── memflow-daemon.exe      ← Core（支持托盘/开机自启）
├── memflow-mcp.exe         ← MCP Server
└── resources/              ← ONNX 模型等
```

用户解压后：
1. 运行 `memflow-daemon.exe --tray`（托盘模式，开机自启）
2. 在 IDE 中配置 MCP 指向 `memflow-mcp.exe`

---

## 架构图

```
┌─────────────────────────────────────────────────────────────────┐
│                        MCP 产品工作流程                           │
├─────────────────────────────────────────────────────────────────┤
│                                                                 │
│   ┌──────────────┐                      ┌──────────────────┐   │
│   │   Cursor     │                      │  memflow-daemon  │   │
│   │   (IDE)      │                      │  (Core)          │   │
│   │              │   stdin/stdout        │                  │   │
│   │  ┌────────┐  │ ──────────────────►  │  ┌────────────┐  │   │
│   │  │ MCP    │  │   JSON-RPC           │  │ IPC Server │  │   │
│   │  │ Client │  │                      │  │  :9516     │  │   │
│   │  └────────┘  │ ◄──────────────────── │  └────────────┘  │   │
│   └──────────────┘                      └──────────────────┘   │
│          │                                      ▲              │
│          │                                      │              │
│          │         memflow-mcp.exe             │              │
│          └──────────────────────────────────────┘              │
│                                                                 │
└─────────────────────────────────────────────────────────────────┘
```
