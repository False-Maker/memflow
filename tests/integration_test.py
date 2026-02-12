#!/usr/bin/env python3
"""
MCP Integration Tests

Tests the Memflow MCP server through its JSON-RPC interface.
Run with: python tests/integration_test.py

Requirements:
- Memflow MCP server running
- Python 3.8+
- requests library (optional, for HTTP mode)
"""

import subprocess
import json
import sys
import time
import os
from typing import Dict, Any, Optional


class MCPClient:
    """Client for communicating with MCP server via stdin/stdout"""

    def __init__(self, server_path: str = "./target/debug/memflow-mcp"):
        self.server_path = server_path
        self.process: Optional[subprocess.Popen] = None
        self.request_id = 0

    def start(self):
        """Start the MCP server process"""
        self.process = subprocess.Popen(
            [self.server_path],
            stdin=subprocess.PIPE,
            stdout=subprocess.PIPE,
            stderr=subprocess.PIPE,
            text=True,
            bufsize=1,
        )
        # Give server time to initialize
        time.sleep(1)

    def stop(self):
        """Stop the MCP server process"""
        if self.process:
            self.process.terminate()
            self.process.wait(timeout=5)
            self.process = None

    def send_request(
        self, method: str, params: Optional[Dict] = None
    ) -> Dict[str, Any]:
        """Send a JSON-RPC request and return the response"""
        self.request_id += 1
        request = {"jsonrpc": "2.0", "method": method, "id": self.request_id}
        if params:
            request["params"] = params

        # Send request
        request_line = json.dumps(request) + "\n"
        self.process.stdin.write(request_line)
        self.process.stdin.flush()

        # Read response
        response_line = self.process.stdout.readline()
        return json.loads(response_line)

    def initialize(self) -> Dict[str, Any]:
        """Send initialize request"""
        return self.send_request(
            "initialize",
            {
                "protocolVersion": "2024-11-05",
                "capabilities": {},
                "clientInfo": {"name": "test-client", "version": "1.0.0"},
            },
        )

    def list_tools(self) -> Dict[str, Any]:
        """List available tools"""
        return self.send_request("tools/list")

    def call_tool(self, name: str, arguments: Dict[str, Any]) -> Dict[str, Any]:
        """Call a tool"""
        return self.send_request("tools/call", {"name": name, "arguments": arguments})

    def list_prompts(self) -> Dict[str, Any]:
        """List available prompts"""
        return self.send_request("prompts/list")

    def get_prompt(self, name: str, arguments: Optional[Dict] = None) -> Dict[str, Any]:
        """Get a prompt"""
        params = {"name": name}
        if arguments:
            params["arguments"] = arguments
        return self.send_request("prompts/get", params)


class TestSuite:
    """Test suite for MCP functionality"""

    def __init__(self, client: MCPClient):
        self.client = client
        self.passed = 0
        self.failed = 0

    def assert_true(self, condition: bool, message: str):
        """Assert a condition is true"""
        if condition:
            print(f"  ✓ {message}")
            self.passed += 1
        else:
            print(f"  ✗ {message}")
            self.failed += 1

    def assert_eq(self, actual, expected, message: str):
        """Assert two values are equal"""
        if actual == expected:
            print(f"  ✓ {message}")
            self.passed += 1
        else:
            print(f"  ✗ {message}")
            print(f"    Expected: {expected}")
            print(f"    Actual: {actual}")
            self.failed += 1

    def run_all_tests(self):
        """Run all tests"""
        print("\n" + "=" * 60)
        print("Memflow MCP Integration Tests")
        print("=" * 60)

        self.test_initialize()
        self.test_tools_list()
        self.test_tool_search_memory()
        self.test_tool_get_recent_activity()
        self.test_tool_get_active_window_context()
        self.test_tool_get_terminal_output()
        self.test_tool_get_system_environment()
        self.test_prompts_list()
        self.test_error_handling()

        print("\n" + "=" * 60)
        print(f"Results: {self.passed} passed, {self.failed} failed")
        print("=" * 60)

        return self.failed == 0

    def test_initialize(self):
        """Test server initialization"""
        print("\n[Test] Initialize")
        response = self.client.initialize()

        self.assert_true("result" in response, "Response contains result")
        if "result" in response:
            result = response["result"]
            self.assert_true(
                "protocolVersion" in result, "Result contains protocolVersion"
            )
            self.assert_true("serverInfo" in result, "Result contains serverInfo")

    def test_tools_list(self):
        """Test tools/list method"""
        print("\n[Test] Tools List")
        response = self.client.list_tools()

        self.assert_true("result" in response, "Response contains result")
        if "result" in response:
            tools = response["result"].get("tools", [])
            self.assert_true(
                len(tools) >= 5, f"Has at least 5 tools (found {len(tools)})"
            )

            tool_names = [t["name"] for t in tools]
            expected_tools = [
                "search_memory",
                "get_recent_activity",
                "get_active_window_context",
                "get_terminal_output",
                "get_system_environment",
            ]

            for tool in expected_tools:
                self.assert_true(tool in tool_names, f"Tool '{tool}' is listed")

    def test_tool_search_memory(self):
        """Test search_memory tool"""
        print("\n[Test] Tool: search_memory")
        response = self.client.call_tool("search_memory", {"query": "test"})

        self.assert_true(
            "result" in response or "error" in response,
            "Response contains result or error",
        )

        if "result" in response:
            result = response["result"]
            self.assert_true("content" in result, "Result contains content")

    def test_tool_get_recent_activity(self):
        """Test get_recent_activity tool"""
        print("\n[Test] Tool: get_recent_activity")
        response = self.client.call_tool("get_recent_activity", {"minutes": 5})

        self.assert_true(
            "result" in response or "error" in response,
            "Response contains result or error",
        )

        if "result" in response:
            self.assert_true("content" in response["result"], "Result contains content")

    def test_tool_get_active_window_context(self):
        """Test get_active_window_context tool"""
        print("\n[Test] Tool: get_active_window_context")
        response = self.client.call_tool("get_active_window_context", {})

        self.assert_true(
            "result" in response or "error" in response,
            "Response contains result or error",
        )

    def test_tool_get_terminal_output(self):
        """Test get_terminal_output tool"""
        print("\n[Test] Tool: get_terminal_output")
        response = self.client.call_tool("get_terminal_output", {"lines": 10})

        self.assert_true(
            "result" in response or "error" in response,
            "Response contains result or error",
        )

    def test_tool_get_system_environment(self):
        """Test get_system_environment tool"""
        print("\n[Test] Tool: get_system_environment")
        response = self.client.call_tool("get_system_environment", {})

        self.assert_true(
            "result" in response or "error" in response,
            "Response contains result or error",
        )

        if "result" in response:
            content = response["result"].get("content", [])
            if content:
                text = content[0].get("text", "")
                self.assert_true(
                    "OS" in text
                    or "System" in text
                    or "CPU" in text
                    or "Memory" in text,
                    "Response contains system information",
                )

    def test_prompts_list(self):
        """Test prompts/list method"""
        print("\n[Test] Prompts List")
        response = self.client.list_prompts()

        self.assert_true("result" in response, "Response contains result")
        if "result" in response:
            prompts = response["result"].get("prompts", [])
            self.assert_true(
                len(prompts) >= 3, f"Has at least 3 prompts (found {len(prompts)})"
            )

            prompt_names = [p["name"] for p in prompts]
            expected_prompts = [
                "debug_context",
                "visual_regression",
                "implicit_knowledge",
            ]

            for prompt in expected_prompts:
                self.assert_true(prompt in prompt_names, f"Prompt '{prompt}' is listed")

    def test_error_handling(self):
        """Test error handling"""
        print("\n[Test] Error Handling")

        # Test unknown tool
        response = self.client.call_tool("unknown_tool", {})
        self.assert_true("error" in response, "Unknown tool returns error")

        if "error" in response:
            error = response["error"]
            self.assert_eq(
                error.get("code"), -32601, "Error code is -32601 (Method not found)"
            )


def main():
    """Main entry point"""
    # Check if server binary exists
    server_path = "./target/debug/memflow-mcp"
    if not os.path.exists(server_path):
        print(f"Server binary not found at {server_path}")
        print("Please build with: cargo build")
        sys.exit(1)

    client = MCPClient(server_path)

    try:
        print("Starting MCP server...")
        client.start()

        # Run tests
        suite = TestSuite(client)
        success = suite.run_all_tests()

        sys.exit(0 if success else 1)

    except Exception as e:
        print(f"\nError: {e}")
        import traceback

        traceback.print_exc()
        sys.exit(1)
    finally:
        print("\nStopping MCP server...")
        client.stop()


if __name__ == "__main__":
    main()
