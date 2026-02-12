# MCP Integration Guide

## Overview

This guide explains how to integrate Memflow MCP Server with AI IDEs like Cursor and Claude Desktop.

## Prerequisites

- Memflow MCP Server built and available
- Cursor or Claude Desktop installed
- Basic understanding of MCP (Model Context Protocol)

## Installation

### 1. Build the MCP Server

```bash
cd D:\Demo\memflow
cargo build --release -p memflow-mcp
```

The binary will be at: `target/release/memflow-mcp`

### 2. Configure Environment Variables

```bash
# Optional: Set authentication token
export MEMFLOW_MCP_TOKEN="your-secure-token"

# Optional: Set read-only mode (default: true)
export MEMFLOW_MCP_READ_ONLY="true"
```

## Cursor Integration

### Step 1: Open Cursor Settings

1. Open Cursor
2. Go to `Settings` → `Features` → `MCP`
3. Click `Add MCP Server`

### Step 2: Configure Memflow MCP

Add the following configuration:

```json
{
  "mcpServers": {
    "memflow": {
      "command": "D:\\Demo\\memflow\\target\\release\\memflow-mcp.exe",
      "env": {
        "MEMFLOW_MCP_READ_ONLY": "true"
      }
    }
  }
}
```

### Step 3: Test Integration

1. Open a chat in Cursor
2. Type: `@memflow What was I working on recently?`
3. You should see the tool being called and results appearing

## Claude Desktop Integration

### Step 1: Open Claude Configuration

1. Open Claude Desktop
2. Go to Settings → Developer → Edit Config
3. Edit the `claude_desktop_config.json` file

### Step 2: Add Memflow MCP

```json
{
  "mcpServers": {
    "memflow": {
      "command": "D:\\Demo\\memflow\\target\\release\\memflow-mcp.exe",
      "env": {
        "MEMFLOW_MCP_READ_ONLY": "true"
      }
    }
  }
}
```

### Step 3: Restart Claude

1. Close and reopen Claude Desktop
2. The MCP server should auto-connect

### Step 4: Test

1. Start a new conversation
2. Ask: "What did I work on today?"
3. Claude should use Memflow to retrieve your activity

## Available Tools

### 1. search_memory

Search your recorded memory with keyword/semantic/hybrid strategies.

**Parameters:**
- `query` (required): Search query
- `limit`: Max results (default: 5)
- `mode`: Search mode - "hybrid", "semantic", or "keyword"
- `app_name`: Filter by app name
- `date_range`: "today", "yesterday", "this_week", "last_week", "this_month"

**Example:**
```json
{
  "name": "search_memory",
  "arguments": {
    "query": "Docker setup",
    "limit": 10,
    "mode": "hybrid"
  }
}
```

### 2. get_recent_activity

Get your recent activity timeline.

**Parameters:**
- `minutes`: Minutes to look back (default: 5, max: 30)
- `limit`: Max activities to return (default: 20)

**Example:**
```json
{
  "name": "get_recent_activity",
  "arguments": {
    "minutes": 15,
    "limit": 10
  }
}
```

### 3. get_active_window_context

Get information about the currently active window.

**Parameters:** None

**Example:**
```json
{
  "name": "get_active_window_context",
  "arguments": {}
}
```

### 4. get_terminal_output

Capture recent terminal output.

**Parameters:**
- `lines`: Number of lines (default: 50, max: 500)

**Example:**
```json
{
  "name": "get_terminal_output",
  "arguments": {
    "lines": 100
  }
}
```

### 5. get_system_environment

Get system environment information.

**Parameters:**
- `include_dev_tools`: Include dev tool versions (default: true)
- `include_processes`: Include active processes (default: true)
- `include_ports`: Include port usage (default: false)

**Example:**
```json
{
  "name": "get_system_environment",
  "arguments": {
    "include_dev_tools": true,
    "include_processes": true
  }
}
```

### 6. get_related_context

Get compact context chunks related to a query.

**Parameters:**
- `query` (required): Query to find context
- `limit`: Max context items (default: 5)
- `max_chars_per_item`: Max chars per item (default: 1200)

**Example:**
```json
{
  "name": "get_related_context",
  "arguments": {
    "query": "authentication flow",
    "limit": 3
  }
}
```

## Prompt Resources

Memflow also provides prompt resources that can be used in conversations:

### 1. debug_context

Analyze recent error logs and terminal output.

**Parameters:**
- `time_range`: "5m", "15m", "30m", "1h"
- `error_pattern`: Optional pattern to focus on

### 2. visual_regression

Analyze UI changes across time periods.

**Parameters:**
- `app_name`: Application to analyze (required)
- `compare_range`: "today_vs_yesterday", "this_week_vs_last"

### 3. implicit_knowledge

Discover patterns from your work history.

**Parameters:**
- `topic`: Topic area to explore (required)
- `depth`: "surface", "deep", "comprehensive"

## Troubleshooting

### Server Not Starting

1. Check the binary path is correct
2. Ensure the binary has execute permissions
3. Check logs for errors

### Tools Not Available

1. Verify the server is running: `memflow-mcp --version`
2. Check Cursor/Claude logs for connection errors
3. Ensure database is initialized

### Database Errors

If you see "database locked" errors:
- Ensure Memflow desktop app is not actively recording
- Wait a moment and retry
- The MCP server will automatically retry

### No Data Found

If tools return no results:
1. Ensure Memflow desktop app has run at least once
2. Check that screenshots and database exist
3. Verify the database path is correct

## Security

### Authentication

Set `MEMFLOW_MCP_TOKEN` to require authentication:

```bash
export MEMFLOW_MCP_TOKEN="your-secure-token"
```

Then in Cursor/Claude config:
```json
{
  "mcpServers": {
    "memflow": {
      "command": "...",
      "env": {
        "MEMFLOW_MCP_TOKEN": "your-secure-token"
      }
    }
  }
}
```

### Read-Only Mode

By default, the MCP server runs in read-only mode. To disable:

```bash
export MEMFLOW_MCP_READ_ONLY="false"
```

⚠️ **Warning**: Disabling read-only mode allows write operations. Only do this if you understand the risks.

### Audit Logging

All tool calls are logged for security auditing. Logs are stored at:
- Windows: `%APPDATA%/memflow/audit.log`
- macOS: `~/Library/Application Support/memflow/audit.log`
- Linux: `~/.local/share/memflow/audit.log`

Sensitive data (API keys, passwords, emails) is automatically redacted.

## Advanced Configuration

### Custom Audit Config

Create `audit_config.json` in the memflow data directory:

```json
{
  "enabled": true,
  "max_file_size_mb": 100,
  "retention_days": 30,
  "redaction_rules": [
    {
      "name": "custom_secret",
      "pattern": "sk-[a-zA-Z0-9]{20,}",
      "replacement": "[SECRET]"
    }
  ]
}
```

### Performance Tuning

For better performance:
1. Ensure SQLite WAL mode is enabled (default)
2. Keep database size under 1GB
3. Regularly archive old screenshots
4. Use SSD storage for database

## Examples

### Example 1: Find Recent Work

```
@memflow What did I work on in the last hour?
```

Memflow will call `get_recent_activity` with minutes=60.

### Example 2: Search for Code

```
@memflow Find where I implemented the authentication middleware
```

Memflow will call `search_memory` with query="authentication middleware".

### Example 3: Debug Build Error

```
@memflow I got a compilation error. Check my terminal output.
```

Memflow will call `get_terminal_output` to retrieve recent terminal output.

### Example 4: Check Environment

```
@memflow What Node version am I using?
```

Memflow will call `get_system_environment` to check installed dev tools.

## Support

For issues or questions:
1. Check the troubleshooting section above
2. Review the audit logs
3. File an issue in the project repository

## Version History

| Version | Date | Changes |
|---------|------|---------|
| 1.0 | 2026-02-11 | Initial release with 6 tools, 3 prompts, audit logging |
