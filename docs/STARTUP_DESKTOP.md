# 桌面端产品启动手册

## 产品说明

**桌面端产品 = Core + Desktop UI**

- Core：负责数据采集、存储、索引（后台服务）
- Desktop UI：展示时间轴、搜索、控制录制（图形界面）
- 适用：**普通用户**

---

## 快速启动

### 方式一：开发模式（推荐）

```bash
# 进入项目目录
cd D:\Demo\memflow

# 启动 Tauri 开发服务器
cargo tauri dev
```

这会同时启动：
1. Core Daemon（后台进程）
2. Desktop UI（窗口）

---

### 方式二：分别启动

#### 1. 先启动 Core

```bash
# 编译并运行 Core
cargo run --package memflow-daemon
```

或使用编译好的 exe：
```bash
.\target\release\memflow-daemon.exe
```

#### 2. 再启动 Desktop UI

```bash
cargo tauri dev
```

> **注意**：Desktop UI 启动时会自动检测 Core 是否运行，未运行则自动启动。

---

## 托盘模式

桌面端支持**托盘模式**：隐藏主窗口，仅保留托盘图标运行。

### 功能列表

| 功能 | 说明 |
|------|------|
| **托盘图标** | 显示录制状态（绿=录制中，红=暂停） |
| **右键菜单** | 暂停/恢复、设置、开机自启、退出 |
| **全局快捷键** | `Ctrl+Shift+P` 切换录制状态 |
| **开机自启** | 托盘菜单中一键开启/关闭 |

### 开机在自启

托盘图标**右键菜单**中勾选「开机自启」：

```
┌─────────────────────────┐
│  MemFlow               │
├─────────────────────────┤
│  ▶ 暂停录制             │
│  ✓ 开机自启             │  ← 勾选即生效
│  ─────────────          │
│  设置...                 │
│  退出                    │
└─────────────────────────┘
```

开启后：
- Core 会在 Windows 启动时自动运行
- 用户登录后自动进入托盘模式（无窗口）
- 录制持续进行

---

## 构建发布版

```bash
# 构建桌面端安装包
cargo tauri build
```

输出位置：
```
src-tauri/target/release/bundle/
├── nsis/MemFlow-Setup.exe    ← Windows 安装包
└── app/MemFlow.exe           ← 便携版
```

---

## 常见问题

### Q: 日志在哪里？

```
%APPDATA%\MemFlow\logs\
```

### Q: 数据存在哪里？

```
%APPDATA%\MemFlow\
```

### Q: 托盘模式下怎么退出？

托盘图标 → 右键 → 「退出」

---

## 端口说明

| 端口 | 用途 |
|------|------|
| 9516 | Core IPC 服务端口 |

Desktop UI 通过 IPC 调用 Core。
