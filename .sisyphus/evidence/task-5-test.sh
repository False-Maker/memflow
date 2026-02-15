#!/bin/bash
MCP_EXE="D:\Demo\memflow\target\debug\memflow-mcp.exe"

echo "=== Task 5: E2E Validation Tests ==="
echo ""
echo "Using executable: $MCP_EXE"
echo ""

# Test 1: initialize
echo "Test 1: initialize"
echo '{"jsonrpc":"2.0","method":"initialize","params":{"protocolVersion":"2024-11-05","capabilities":{},"clientInfo":{"name":"test","version":"0.1"}},"id":1}' | "$MCP_EXE"
echo ""
echo "---"
echo ""

# Test 2: tools/list
echo "Test 2: tools/list"
echo '{"jsonrpc":"2.0","method":"tools/list","params":{},"id":2}' | "$MCP_EXE"
echo ""
echo "---"
echo ""

# Test 3: Call get_system_environment
echo "Test 3: tools/call - get_system_environment"
echo '{"jsonrpc":"2.0","method":"tools/call","params":{"name":"get_system_environment","arguments":{"include_dev_tools":false,"include_processes":false,"include_ports":false}},"id":3}' | "$MCP_EXE"
echo ""
echo "---"
echo ""

# Test 4: Call search_memory
echo "Test 4: tools/call - search_memory"
echo '{"jsonrpc":"2.0","method":"tools/call","params":{"name":"search_memory","arguments":{"query":"test"}},"id":4}' | "$MCP_EXE"
echo ""
echo "---"
echo ""

# Test 5: Call get_recent_activities
echo "Test 5: tools/call - get_recent_activities"
echo '{"jsonrpc":"2.0","method":"tools/call","params":{"name":"get_recent_activities","arguments":{"minutes":5,"limit":10}},"id":5}' | "$MCP_EXE"
echo ""
echo "---"
echo ""

echo "=== All tests completed ==="
