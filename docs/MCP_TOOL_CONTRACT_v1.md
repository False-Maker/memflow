# MCP Tool Contract v1

本文档定义了 MemFlow MCP Server 的工具接口规范。

---

## 1. 概述

MCP Server 提供以下工具供 LLM / IDE / Cursor 调用：

| 工具名 | 功能 |
|--------|------|
| `search_memory` | 在本地记忆中做关键词 / 语义 / 混合搜索 |
| `get_recent_activity` | 返回最近 N 分钟的活动时间线 |
| `get_active_window_context` | 获取当前活跃窗口的上下文 |
| `get_related_context` | 返回与 query 相关的精简上下文片段 |
| `get_terminal_output` | 捕获当前终端窗口最近 N 行输出 |
| `get_system_environment` | 返回系统环境信息 |

---

## 2. 工具详细规范

### 2.1 search_memory

在本地记忆中搜索活动记录，支持关键词、语义和混合模式。

**输入参数：**

| 字段 | 类型 | 必填 | 默认值 | 说明 |
|------|------|------|--------|------|
| `query` | string | 是 | - | 搜索关键词 |
| `limit` | integer | 否 | 5 | 返回结果数量上限 (1-50) |
| `mode` | string | 否 | "hybrid" | 搜索模式：`hybrid` / `semantic` / `keyword` |
| `app_name` | string | 否 | - | 按应用名称过滤 |
| `keywords` | string[] | 否 | - | 额外关键词过滤 |
| `date_range` | string | 否 | - | 日期范围：`today` / `yesterday` / `this_week` / `last_week` / `this_month` |
| `has_ocr` | boolean | 否 | - | 是否必须包含 OCR 文本 |

**输出格式：**

```json
{
  "content": [
    {
      "type": "text",
      "text": "搜索结果摘要文本..."
    }
  ]
}
```

**结果文本格式：**

```
Keyword search results for 'query' (total N, showing up to M):

ID: xxx | Time: YYYY-MM-DD HH:mm:ss | App: xxx | Title: xxx
Content: xxx
---
```

---

### 2.2 get_recent_activity

返回最近 N 分钟内的活动记录。

**输入参数：**

| 字段 | 类型 | 必填 | 默认值 | 说明 |
|------|------|------|--------|------|
| `minutes` | integer | 否 | 5 | 最近 N 分钟 (1-30) |
| `limit` | integer | 否 | 50 | 返回结果数量上限 (1-200) |

**输出格式：**

```json
{
  "content": [
    {
      "type": "text",
      "text": "最近 N 分钟内共记录到 M 条活动：\n\n- 时间：...\n  应用：...\n  标题：...\n  OCR 摘要：...\n"
    }
  ]
}
```

---

### 2.3 get_active_window_context

获取当前活跃窗口的上下文信息。

**输入参数：** 无

**输出格式：**

```json
{
  "content": [
    {
      "type": "text",
      "text": "推断的当前活跃窗口上下文：\n\n- 时间：...\n- 应用：...\n- 标题：...\n\n相关 OCR 文本（已脱敏）：..."
    }
  ]
}
```

---

### 2.4 get_related_context

返回与给定 query 最相关的上下文片段。

**输入参数：**

| 字段 | 类型 | 必填 | 默认值 | 说明 |
|------|------|------|--------|------|
| `query` | string | 是 | - | 查询文本 |
| `limit` | integer | 否 | 5 | 返回片段数量 (1-20) |
| `max_chars_per_item` | integer | 否 | 400 | 每个片段的最大字符数 (100-2000) |

**输出格式：**

```json
{
  "content": [
    {
      "type": "text",
      "text": "与当前 query 最相关的上下文片段（已脱敏，按相关度排序）：\n\n片段 #1 (score = x.xx)\n时间：...\n应用：...\n标题：...\n内容：...\n\n---"
    }
  ]
}
```

---

### 2.5 get_terminal_output

获取终端窗口的最近输出。

**输入参数：**

| 字段 | 类型 | 必填 | 默认值 | 说明 |
|------|------|------|--------|------|
| `limit` | integer | 否 | 20 | 返回的终端日志条目数 |

**输出格式：**

```json
{
  "content": [
    {
      "type": "text",
      "text": "最近终端输出（已脱敏，按时间排序，最新在前）：\n\n[YYYY-MM-DD HH:mm:ss] session=xxx app=xxx title=xxx\n...\n\n---"
    }
  ]
}
```

**错误码：**

| 错误码 | 说明 |
|--------|------|
| `-32004` | 终端未找到 |
| `-32005` | 权限不足 |

---

### 2.6 get_system_environment

获取系统环境信息。

**输入参数：** 无

**输出格式：**

```json
{
  "content": [
    {
      "type": "text",
      "text": "系统环境概览（本地收集）：\n\n[基础信息]\n- OS：...\n- 架构：...\n...\n\n[开发工具版本]\n- Git: ...\n...\n\n[常见端口占用]\n..."
    }
  ]
}
```

---

## 3. 错误码定义

| 错误码 | 名称 | 说明 |
|--------|------|------|
| `-32000` | `MCP_PARSE_ERROR` | JSON-RPC 请求解析失败 |
| `-32001` | `MCP_INVALID_REQUEST` | 无效的请求结构 |
| `-32002` | `MCP_METHOD_NOT_FOUND` | 工具不存在 |
| `-32003` | `MCP_INVALID_PARAMS` | 参数无效或缺失 |
| `-32004` | `MCP_TERMINAL_NOT_FOUND` | 终端窗口未找到 |
| `-32005` | `MCP_PERMISSION_DENIED` | 权限不足，无法读取数据 |
| `-32006` | `MCP_INTERNAL` | 内部错误 |
| `-32007` | `MCP_CORE_UNAVAILABLE` | Core 服务不可用 |
| `-32008` | `MCP_DEGRADED_MODE` | 降级模式运行，部分功能不可用 |

---

## 4. 别名兼容

以下别名工具名已被废弃，但保持兼容：

| 别名 | 正式名称 |
|------|----------|
| `search_visual_memory` | `search_memory` |
| `get_recent_activities` | `get_recent_activity` |

使用别名时会在日志中记录 deprecation 警告。

---

## 5. 通用约定

1. **日志输出**：所有日志输出到 `stderr`，`stdout` 仅输出 JSON-RPC 响应。
2. **脱敏处理**：所有返回给外部的文本都会经过 `redact` 模块处理。
3. **超时控制**：向量搜索等耗时操作默认超时 30 秒。
4. **结果裁剪**：单个结果项的最大长度有限制，避免 prompt 爆炸。
