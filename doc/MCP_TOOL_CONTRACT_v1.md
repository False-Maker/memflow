# MCP Tool Contract v1

本文档定义了 MemFlow MCP Server 提供的工具集合、输入输出 Schema 以及错误码规范。

---

## 1. 工具列表

| 工具名 | 功能描述 | 状态 |
|--------|----------|------|
| `search_memory` | 在本地记忆中做关键词 / 语义 / 混合搜索 | ✅ 已实现 |
| `get_recent_activity` | 返回最近 N 分钟的活动时间线 | ✅ 已实现 |
| `get_active_window_context` | 获取当前活跃窗口的上下文 | ✅ 已实现 |
| `get_related_context` | 返回与 query 相关的精简上下文片段 | ✅ 已实现 |
| `get_terminal_output` | 捕获当前终端窗口最近 N 行输出 | ✅ 已实现 |
| `get_system_environment` | 返回系统环境信息 | ✅ 已实现 |

---

## 2. 工具 Schema

### 2.1 search_memory

在本地记忆中搜索活动记录，支持关键词、语义和混合模式。

**输入参数：**

| 字段 | 类型 | 必填 | 默认值 | 描述 |
|------|------|------|--------|------|
| `query` | string | ✅ | - | 搜索查询文本 |
| `limit` | number | ❌ | 5 | 返回结果数量上限 (1-50) |
| `mode` | string | ❌ | "hybrid" | 搜索模式：`hybrid` / `semantic` / `keyword` |
| `app_name` | string | ❌ | - | 按应用名称过滤 |
| `keywords` | string[] | ❌ | - | 额外关键词过滤 |
| `date_range` | string | ❌ | - | 时间范围：`today` / `yesterday` / `last_week` / `this_week` / `this_month` |
| `has_ocr` | boolean | ❌ | - | 是否必须有 OCR 文本 |

**输出结构：**

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

---

### 2.2 get_recent_activity

返回最近 N 分钟内的活动记录。

**输入参数：**

| 字段 | 类型 | 必填 | 默认值 | 描述 |
|------|------|------|--------|------|
| `minutes` | number | ❌ | 5 | 查询最近多少分钟 (1-30) |
| `limit` | number | ❌ | 50 | 返回结果数量上限 (1-200) |

**输出结构：**

```json
{
  "content": [
    {
      "type": "text",
      "text": "最近 X 分钟内的活动时间线文本..."
    }
  ]
}
```

---

### 2.3 get_active_window_context

获取当前活跃窗口的上下文信息，包括 OCR 文本。

**输入参数：** 无

**输出结构：**

```json
{
  "content": [
    {
      "type": "text",
      "text": "当前活跃窗口上下文文本..."
    }
  ]
}
```

---

### 2.4 get_related_context

返回与给定 query 最相关的上下文片段，适合拼入 LLM prompt。

**输入参数：**

| 字段 | 类型 | 必填 | 默认值 | 描述 |
|------|------|------|--------|------|
| `query` | string | ✅ | - | 查询文本 |
| `limit` | number | ❌ | 5 | 返回片段数量 (1-20) |
| `max_chars_per_item` | number | ❌ | 400 | 每个片段的最大字符数 (100-2000) |

**输出结构：**

```json
{
  "content": [
    {
      "type": "text",
      "text": "相关上下文片段文本..."
    }
  ]
}
```

---

### 2.5 get_terminal_output

获取当前终端窗口的最近输出。

**输入参数：**

| 字段 | 类型 | 必填 | 默认值 | 描述 |
|------|------|------|--------|------|
| `limit` | number | ❌ | 20 | 返回条目数 |

**输出结构：**

```json
{
  "content": [
    {
      "type": "text",
      "text": "终端输出文本..."
    }
  ]
}
```

**错误码：**
- `-32004`：终端未找到
- `-32005`：权限不足

---

### 2.6 get_system_environment

返回系统环境信息，包括 OS、硬件、开发工具版本、端口占用等。

**输入参数：** 无

**输出结构：**

```json
{
  "content": [
    {
      "type": "text",
      "text": "系统环境报告文本..."
    }
  ]
}
```

---

## 3. 错误码

| 错误码 | 名称 | 描述 |
|--------|------|------|
| `-32000` | `MCP_PARSE_ERROR` | JSON-RPC 请求解析失败 |
| `-32001` | `MCP_INVALID_REQUEST` | 请求格式无效 |
| `-32002` | `MCP_METHOD_NOT_FOUND` | 指定的工具不存在 |
| `-32003` | `MCP_INVALID_PARAMS` | 参数无效或缺失必填字段 |
| `-32004` | `MCP_TERMINAL_NOT_FOUND` | 终端窗口未找到 |
| `-32005` | `MCP_PERMISSION_DENIED` | 权限不足，无法访问资源 |
| `-32006` | `MCP_INTERNAL` | 内部错误 |
| `-32007` | `MCP_CORE_UNAVAILABLE` | Core 服务不可用 |
| `-32008` | `MCP_DEGRADED_MODE` | 运行在降级模式，部分功能不可用 |

---

## 4. 别名工具名（已废弃）

以下别名仍可使用，但会记录 deprecation 日志，建议迁移到新名称：

| 别名 | 建议使用 |
|------|----------|
| `search_visual_memory` | `search_memory` |
| `get_recent_activities` | `get_recent_activity` |

---

## 5. 协议要求

- 所有请求/响应使用 JSON-RPC 2.0 格式
- 日志输出到 `stderr`
- `stdout` 只输出 JSON-RPC 响应
- 使用 MCP 2024-11-05 规范

---

## 6. 实现参考

- MCP Server 实现：`crates/memflow-mcp/`
- 核心工具逻辑：`crates/memflow-mcp/src/tools.rs`
- 协议解析：`crates/memflow-mcp/src/protocol.rs`
