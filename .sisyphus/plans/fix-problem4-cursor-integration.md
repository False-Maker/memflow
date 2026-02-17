# Cursor/Claude Desktop Integration Verification (Problem 4)

## TL;DR

> **Quick Summary**: Configure memflow-mcp as MCP server in Cursor IDE and verify tool discovery and execution.
>
> **Deliverables**:
> - Cursor MCP configuration file updated with memflow server
> - Verification that tools are discovered in Cursor
> - Test results for key tools (search_memory, get_system_environment, etc.)
> - Updated E2E_VALIDATION_REPORT.md with integration test results
>
> **Estimated Effort**: Quick
> **Parallel Execution**: NO - single configuration and test task
> **Critical Path**: Configure Cursor → Restart → Test Tools → Document Results

---

## Context

### Original Request
From `doc/MCP_BLOCKING_ISSUES.md` Problem 4:
> E2E_VALIDATION_REPORT.md has passed stdin/stdout manual verification of JSON-RPC communication, but has not been actually configured and used in Cursor or Claude Desktop.

### Current State
- memflow-mcp.exe builds successfully (Problem 3 fixed)
- ONNX Runtime v1.24.1 deployed (Problem 1 fixed)
- JSON-RPC manual tests passed
- Cursor installed at D:\cursor on user's system

### User's Capabilities
- Local Cursor installation available
- Can restart Cursor to load new configuration

---

## Work Objectives

### Core Objective
Configure memflow-mcp as an MCP server in Cursor IDE and verify that AI assistants can discover and use Memflow tools.

### Concrete Deliverables
- Cursor MCP configuration file created/updated
- Tool discovery verified in Cursor Chat
- Test results documented in E2E_VALIDATION_REPORT.md

### Definition of Done
- [ ] Cursor settings.json includes memflow MCP server configuration
- [ ] Cursor restarted and server connected successfully
- [ ] Tools are visible/discoverable in Cursor Chat
- [ ] At least 2 tools tested successfully (get_system_environment, search_memory)
- [ ] E2E_VALIDATION_REPORT.md Section 7 updated with test results

### Must Have
- Configure memflow-mcp with read-only mode enabled
- Set correct absolute path to memflow-mcp.exe
- Test basic functionality
- Document results

### Must NOT Have (Guardrails)
- DO NOT modify Claude Desktop settings (unless requested)
- DO NOT change memflow-mcp source code
- DO NOT modify other Cursor settings unrelated to MCP

---

## Execution Strategy

### Task List

**Task 1: Create/Update Cursor MCP Configuration**

1. Locate Cursor settings.json at: `%APPDATA%\Cursor\User\settings.json`
2. Read current settings
3. Add/merge MCP server configuration:
```json
{
  "mcp.mcpServers": {
    "memflow": {
      "command": "D:\\Demo\\memflow\\target\\debug\\memflow-mcp.exe",
      "args": [],
      "env": {
        "MEMFLOW_MCP_READ_ONLY": "true"
      }
    }
  }
}
```
4. Preserve existing settings
5. Write updated configuration

**Task 2: Verify Configuration and Restart Cursor**

1. Inform user to restart Cursor
2. After restart, check for connection errors
3. Verify MCP server appears in Cursor's MCP status

**Task 3: Test Tool Discovery and Execution**

In Cursor Chat, test these scenarios:
1. "我的系统环境是什么？" → Should trigger get_system_environment
2. "搜索关于 Rust 的记忆" → Should trigger search_memory
3. Check that tools are listed in Chat interface

**Task 4: Document Results**

Update `doc/E2E_VALIDATION_REPORT.md` Section 7 with:
- Configuration used
- Connection status
- Tools tested and results
- Any issues encountered
- Screenshots or logs if applicable

---

## Success Criteria

### Verification
- [ ] settings.json contains valid memflow MCP server entry
- [ ] Cursor starts without errors
- [ ] Tools appear in Cursor Chat tool list
- [ ] At least one tool executes successfully
- [ ] E2E_VALIDATION_REPORT.md updated

### Final Checklist
- [ ] Cursor configured with memflow MCP server
- [ ] memflow-mcp.exe path is absolute and correct
- [ ] Read-only mode enabled
- [ ] Tools discoverable in Cursor Chat
- [ ] Integration test results documented
