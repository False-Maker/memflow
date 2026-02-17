# Learnings - Fix Problem 4 Cursor Integration

## Session 1: 2026-02-17

### Conventions Found
- Cursor settings stored in %APPDATA%\Cursor\User\settings.json
- MCP servers configured via mcp.mcpServers key
- Absolute paths required for command

### Gotchas
- Must preserve existing settings when merging
- Need to escape backslashes in JSON paths
- Cursor must be restarted to load new MCP configuration

### Decisions Made
- Use read-only mode for safety (MEMFLOW_MCP_READ_ONLY=true)
- Configure only memflow-mcp, don't touch other servers

## Session 2: 2026-02-17

### MCP Server Configuration Completed
- Successfully added memflow MCP server to Cursor settings
- Configuration includes:
  - Command: D:\\Demo\\memflow\\target\\debug\\memflow-mcp.exe
  - Empty args array
  - Read-only mode enabled via MEMFLOW_MCP_READ_ONLY=true
- All existing settings preserved (database-client.autoSync, etc.)
- JSON validated and confirmed valid
- Cursor restart required to load new MCP configuration
