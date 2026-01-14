# MemFlow

智能桌面活动记录与分析工具

## 项目概述

MemFlow 是一个智能桌面活动记录与分析工具，专注于视觉记忆功能。通过自动截图、OCR文本提取、AI分析等技术，帮助用户记录和分析计算机活动，构建个人知识图谱。

**核心理念：隐私优先、极致性能、本地智能。**

## 技术栈

### 前端
- React 18 + TypeScript
- Vite
- Tailwind CSS
- Framer Motion
- Lucide React
- Recharts
- React Virtuoso
- React Force Graph 2D

### 后端
- Tauri 2.0 (Rust)
- SQLite + SQLx (WAL 模式)
- Tokio
- xcap (截图)
- image + image_hash (感知哈希)
- Windows OCR API + RapidOCR
- reqwest (AI 客户端)

## 快速开始

### 前置要求

- Node.js 18+ 和 pnpm
- Rust 1.70+
- Tauri CLI 2.0

### 安装依赖

```bash
# 安装前端依赖
pnpm install

# 安装 Rust 依赖（首次运行会自动安装）
cargo build --manifest-path src-tauri/Cargo.toml
```

### 开发模式

```bash
# 方式 1: 直接启动（推荐）
pnpm tauri:dev

# 方式 2: 使用开发脚本（Windows）
pnpm dev:all:ps1

# 方式 3: 先运行测试检查
pnpm test:setup
pnpm tauri:dev
```

### 构建

```bash
# 构建生产版本
pnpm tauri:build
```

## 项目结构

```
memflow/
├── src/                    # 前端源代码
│   ├── components/         # React 组件
│   │   ├── Layout.tsx     # 主布局
│   │   ├── Timeline.tsx   # 时间轴视图
│   │   ├── KnowledgeGraph.tsx  # 知识图谱
│   │   └── FlowState.tsx  # 统计面板
│   ├── contexts/          # React Context
│   │   └── AppContext.tsx # 全局状态管理
│   └── main.tsx           # 入口文件
├── src-tauri/             # 后端源代码 (Rust)
│   ├── src/
│   │   ├── main.rs        # 应用入口
│   │   ├── commands.rs    # Tauri 命令
│   │   ├── db.rs          # 数据库操作
│   │   ├── recorder.rs    # 录制模块
│   │   ├── ocr/           # OCR 子系统
│   │   ├── ai/            # AI 子系统
│   │   ├── agent/         # 智能代理
│   │   └── ...
│   └── migrations/        # 数据库迁移
└── PROJECT_ARCHITECTURE.md  # 详细架构文档
```

## 核心功能

- ✅ 活动录制（截图 + 应用监控）
- ✅ 智能去重（pHash 感知哈希）
- ✅ OCR 文本提取（Windows OCR + RapidOCR）
- ✅ PII 隐私脱敏
- ✅ 数据库存储（SQLite WAL 模式）
- 🚧 AI 分析（开发中）
- 🚧 知识图谱（开发中）
- 🚧 智能代理（开发中）

## 开发路线图

### Phase 1: 核心重构 ✅ 已完成
- [x] Tauri 2.0 项目初始化
- [x] 前端基础 UI
- [x] SQLite WAL 模式与并发读写
- [x] xcap 截图与 image_hash 去重
- [x] 图片加载优化

### Phase 2: 智能增强 ✅ 已完成
- [x] 图像预处理（灰度化、二值化）
- [x] PII 隐私脱敏逻辑
- [x] Vector DB 向量数据库
- [x] RAG 混合检索
- [x] 知识图谱构建

### Phase 3: 体验升级 🚧 进行中
- [x] 知识图谱可视化
- [x] 性能监控
- [ ] 集成实际 OCR 引擎
- [ ] 集成 LLM API
- [ ] Alpha 版本发布

详细进度请查看 [PROGRESS.md](./PROGRESS.md)

## 许可证

MIT

