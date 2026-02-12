# MCP Tool Contract v1.0

## Overview

**Version:** 1.0  
**Protocol:** Model Context Protocol (MCP) 2024-11-05  
**Date:** 2026-02-11  
**Status:** Draft  

This document defines the official Tool Contract for Memflow MCP Server, establishing standardized interfaces for AI IDE integration.

---

## Tool Definitions

### Tool: search_memory

**Description:** Search user's recorded memory with keyword/semantic/hybrid strategies. Returns OCR text, app names, and timestamps from past activities.

**Formal Name:** `search_memory`  
**Backward Compatibility Aliases:** `search_visual_memory`  

#### Input Schema

```json
{
  "type": "object",
  "properties": {
    "query": {
      "type": "string",
      "description": "The search query to match against memory."
    },
    "limit": {
      "type": "integer",
      "description": "Maximum number of results to return (default: 5).",
      "default": 5,
      "minimum": 1,
      "maximum": 50
    },
    "mode": {
      "type": "string",
      "description": "Search mode: hybrid, semantic, or keyword (default: hybrid).",
      "enum": ["hybrid", "semantic", "keyword"],
      "default": "hybrid"
    },
    "app_name": {
      "type": "string",
      "description": "Optional app name filter."
    },
    "keywords": {
      "type": "array",
      "items": { "type": "string" },
      "description": "Optional keyword list to override query parsing."
    },
    "date_range": {
      "type": "string",
      "description": "Optional date range filter.",
      "enum": ["today", "yesterday", "this_week", "last_week", "this_month"]
    },
    "has_ocr": {
      "type": "boolean",
      "description": "Filter records that contain OCR text."
    }
  },
  "required": ["query"]
}
```

#### Output Schema

```json
{
  "type": "object",
  "properties": {
    "content": {
      "type": "array",
      "items": {
        "type": "object",
        "properties": {
          "type": { "type": "string", "enum": ["text"] },
          "text": { "type": "string" }
        }
      }
    }
  }
}
```

#### Error Codes

| Code | Meaning | Description |
|------|---------|-------------|
| -32602 | Invalid params | Missing required 'query' parameter or invalid parameter type |
| -32000 | Internal error | Database query failed or search service unavailable |

---

### Tool: get_recent_activity

**Description:** Get the user's recent activity timeline. Use this to understand what happened in the last few minutes.

**Formal Name:** `get_recent_activity`  
**Backward Compatibility Aliases:** `get_recent_activities`  

#### Input Schema

```json
{
  "type": "object",
  "properties": {
    "minutes": {
      "type": "integer",
      "description": "Number of minutes to look back (default: 5, max: 30).",
      "default": 5,
      "minimum": 1,
      "maximum": 30
    },
    "limit": {
      "type": "integer",
      "description": "Max number of activities to return (default: 20).",
      "default": 20,
      "minimum": 1,
      "maximum": 100
    }
  }
}
```

#### Output Schema

```json
{
  "type": "object",
  "properties": {
    "content": {
      "type": "array",
      "items": {
        "type": "object",
        "properties": {
          "type": { "type": "string", "enum": ["text"] },
          "text": { "type": "string" }
        }
      }
    }
  }
}
```

#### Error Codes

| Code | Meaning | Description |
|------|---------|-------------|
| -32602 | Invalid params | Invalid parameter values (negative minutes, etc.) |
| -32000 | Internal error | Database query failed |

---

### Tool: get_active_window_context

**Description:** Get information about the currently active window, including app name, window title, and recent OCR text from that window.

**Formal Name:** `get_active_window_context`  
**Backward Compatibility Aliases:** None

#### Input Schema

```json
{
  "type": "object",
  "properties": {}
}
```

#### Output Schema

```json
{
  "type": "object",
  "properties": {
    "content": {
      "type": "array",
      "items": {
        "type": "object",
        "properties": {
          "type": { "type": "string", "enum": ["text"] },
          "text": { "type": "string" }
        }
      }
    }
  }
}
```

#### Error Codes

| Code | Meaning | Description |
|------|---------|-------------|
| -32000 | Internal error | Failed to retrieve active window information |

---

### Tool: get_terminal_output

**Description:** Capture the recent output from the active terminal window. Useful for debugging build errors, test failures, and command outputs.

**Formal Name:** `get_terminal_output`  
**Backward Compatibility Aliases:** None  
**Status:** Planned - To be implemented in Phase 2

#### Input Schema

```json
{
  "type": "object",
  "properties": {
    "lines": {
      "type": "integer",
      "description": "Number of lines to capture from terminal output (default: 50).",
      "default": 50,
      "minimum": 1,
      "maximum": 500
    },
    "terminal_type": {
      "type": "string",
      "description": "Type of terminal to capture from (default: auto-detect).",
      "enum": ["auto", "windows_terminal", "iterm", "terminal_app", "vscode"],
      "default": "auto"
    }
  }
}
```

#### Output Schema

```json
{
  "type": "object",
  "properties": {
    "content": {
      "type": "array",
      "items": {
        "type": "object",
        "properties": {
          "type": { "type": "string", "enum": ["text"] },
          "text": { "type": "string" }
        }
      }
    }
  }
}
```

#### Error Codes

| Code | Meaning | Description |
|------|---------|-------------|
| -32602 | Invalid params | Invalid lines parameter |
| -32000 | Internal error | Failed to capture terminal output |
| -32004 | Terminal not found | No active terminal window detected |
| -32005 | Permission denied | Insufficient permissions to access terminal |

---

### Tool: get_system_environment

**Description:** Retrieve system environment information including OS version, hardware specs, development tools, and running processes.

**Formal Name:** `get_system_environment`  
**Backward Compatibility Aliases:** None  
**Status:** Planned - To be implemented in Phase 2

#### Input Schema

```json
{
  "type": "object",
  "properties": {
    "include_dev_tools": {
      "type": "boolean",
      "description": "Include development tool versions (Node, Python, Rust, Docker).",
      "default": true
    },
    "include_processes": {
      "type": "boolean",
      "description": "Include active development processes.",
      "default": true
    },
    "include_ports": {
      "type": "boolean",
      "description": "Include common port usage (3000, 8080, 8000, etc.).",
      "default": false
    }
  }
}
```

#### Output Schema

```json
{
  "type": "object",
  "properties": {
    "content": {
      "type": "array",
      "items": {
        "type": "object",
        "properties": {
          "type": { "type": "string", "enum": ["text"] },
          "text": { "type": "string" }
        }
      }
    }
  }
}
```

#### Error Codes

| Code | Meaning | Description |
|------|---------|-------------|
| -32000 | Internal error | Failed to retrieve system information |

---

### Tool: get_related_context

**Description:** Return compact context chunks related to the query for downstream LLM reasoning.

**Formal Name:** `get_related_context`  
**Backward Compatibility Aliases:** None

#### Input Schema

```json
{
  "type": "object",
  "properties": {
    "query": {
      "type": "string",
      "description": "User query to find related context."
    },
    "limit": {
      "type": "integer",
      "description": "Max number of context items (default: 5).",
      "default": 5,
      "minimum": 1,
      "maximum": 20
    },
    "max_chars_per_item": {
      "type": "integer",
      "description": "Max chars of OCR per item (default: 1200).",
      "default": 1200,
      "minimum": 100,
      "maximum": 5000
    }
  },
  "required": ["query"]
}
```

#### Output Schema

```json
{
  "type": "object",
  "properties": {
    "content": {
      "type": "array",
      "items": {
        "type": "object",
        "properties": {
          "type": { "type": "string", "enum": ["text"] },
          "text": { "type": "string" }
        }
      }
    }
  }
}
```

#### Error Codes

| Code | Meaning | Description |
|------|---------|-------------|
| -32602 | Invalid params | Missing required 'query' parameter |
| -32000 | Internal error | Context retrieval failed |

---

## Error Code Reference

### MCP Standard Error Codes

| Code | Name | Description |
|------|------|-------------|
| -32700 | Parse error | Invalid JSON was received |
| -32600 | Invalid Request | The JSON sent is not a valid Request object |
| -32601 | Method not found | The method does not exist |
| -32602 | Invalid params | Invalid method parameter(s) |
| -32603 | Internal error | Internal JSON-RPC error |

### Memflow-Specific Error Codes

| Code | Name | Description |
|------|------|-------------|
| -32000 | Server error | General internal error |
| -32001 | Unauthorized | Authorization check failed |
| -32003 | Read-only mode | Write operation attempted in read-only mode |
| -32004 | Terminal not found | No active terminal detected (get_terminal_output) |
| -32005 | Permission denied | Insufficient permissions |
| -32006 | Database locked | SQLite database is locked by another process |
| -32007 | No data available | Query returned no results |
| -32008 | OCR failed | Text recognition failed |

---

## Backward Compatibility

### Alias Mapping

| Formal Name | Alias(es) | Status |
|-------------|-----------|--------|
| search_memory | search_visual_memory | Supported, deprecated |
| get_recent_activity | get_recent_activities | Supported, deprecated |
| get_active_window_context | None | Primary |
| get_terminal_output | None | Primary |
| get_system_environment | None | Primary |
| get_related_context | None | Primary |

### Compatibility Strategy

1. **Primary Names:** All new integrations MUST use formal tool names.
2. **Alias Support:** Aliases are supported for backward compatibility but marked as deprecated.
3. **Deprecation Timeline:** Aliases will be removed in v2.0 (6 months after v1.0 release).
4. **Logging:** Usage of deprecated aliases is logged for migration tracking.

---

## Fallback Behaviors

### Database Locked

When the SQLite database is locked by the Tauri application:

- **Error Code:** -32006
- **Message:** "Database temporarily unavailable, please retry"
- **Retry Strategy:** Exponential backoff (100ms, 200ms, 400ms, max 3 retries)
- **User Message:** "Memflow is currently syncing data. Please try again in a moment."

### No Data Available

When a query returns no results:

- **Error Code:** -32007 (or success with empty results depending on tool)
- **Behavior:** Return empty result set, not an error
- **User Message:** "No matching records found in your memory."

### OCR Failed

When text recognition fails on a screenshot:

- **Error Code:** -32008
- **Fallback:** Return metadata without OCR text
- **User Message:** "Unable to extract text from this image."

### Terminal Not Found

When get_terminal_output cannot find an active terminal:

- **Error Code:** -32004
- **User Message:** "No active terminal window detected. Please open a terminal and try again."

### Read-Only Mode

When a write tool is called in read-only mode:

- **Error Code:** -32003
- **User Message:** "This operation is not allowed in read-only mode."

---

## Tool Summary

| Tool Name | Implementation Status | Priority |
|-----------|----------------------|----------|
| search_memory | ✅ Implemented | High |
| get_recent_activity | ✅ Implemented | High |
| get_active_window_context | ✅ Implemented | High |
| get_related_context | ✅ Implemented | Medium |
| get_terminal_output | 🚧 Planned | High |
| get_system_environment | 🚧 Planned | High |

---

## Version History

| Version | Date | Changes |
|---------|------|---------|
| 1.0 | 2026-02-11 | Initial contract definition |

---

*This document is part of the Memflow MCP Completion Plan. For implementation details, see the plan document at `.sisyphus/plans/memflow-mcp-completion-plan.md`.*
