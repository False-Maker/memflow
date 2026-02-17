# Memflow MCP 剩余任务 Prompt 文档

> 基于 2026-02-15 代码分析，共 5 个任务。  
> 每个任务独立，可按顺序执行，也可按优先级挑选。  
> 直接复制 Prompt 区块内容即可使用。

---

## 任务 1：接通 get_system_environment 的开发工具检测（P0）

**背景**：`main.rs` 中已有 6 个完整的 `detect_*_version()` 函数（lines 1101-1199），但 `call_get_system_environment()` 函数（lines 1202-1227）完全没有调用它们，三个参数 `include_dev_tools` / `include_processes` / `include_ports` 被忽略。路由层已正确传递参数（lines 535-537）。

**涉及文件**：`crates/memflow-mcp/src/main.rs`

```text
PROMPT:

修改 crates/memflow-mcp/src/main.rs 中的 call_get_system_environment 函数（约第 1202-1227 行）。

当前问题：函数接收 include_dev_tools、include_processes、include_ports 三个 bool 参数，但函数体只返回基础 OS 信息（OS/Kernel/Hostname/CPU/Memory），完全忽略了这三个参数。

同文件中已有 6 个现成的检测函数（约第 1101-1199 行）：
- detect_node_version()
- detect_python_version()
- detect_rust_version()
- detect_docker_version()
- detect_go_version()
- detect_java_version()

请做以下修改：

1. 当 include_dev_tools 为 true 时，调用上述 6 个 detect 函数（它们都是 async，返回 Option<String>），将结果追加到输出中，格式如：
   [Development Tools]
   Node.js: v20.10.0
   Python: Python 3.12.0
   Rust: rustc 1.75.0
   Docker: Docker version 24.0.7
   Go: go1.21.5
   Java: openjdk version "21.0.1"
   （检测不到的工具显示 "Not found"）

2. 当 include_processes 为 true 时，使用已有的 sysinfo::System 实例，遍历 sys.processes()，筛选出常见开发进程（node, python, python3, cargo, rustc, java, docker, code, cursor 等），输出格式如：
   [Active Dev Processes]
   node (PID 1234)
   code (PID 5678)

3. 当 include_ports 为 true 时，使用 tokio::process::Command 运行 netstat -ano（Windows）检查常用端口（3000, 3001, 4200, 5000, 5173, 8000, 8080, 8443）是否被占用，输出格式如：
   [Port Usage]
   :3000 - LISTENING (PID 1234)
   :8080 - LISTENING (PID 5678)
   如果 netstat 执行失败，跳过此段并用 tracing::warn! 记录。

注意：所有外部命令调用都要加 3 秒 tokio::time::timeout 保护，参照文件中已有的 detect_*_version 函数风格。
```

---

## 任务 2：运行 cargo test 并修复编译/测试问题（P0）

**背景**：项目声称 37 个测试全部通过，但实际有 ~56 个测试，且未经独立验证。需要确认当前代码能编译并通过测试。

**涉及文件**：整个 workspace

```text
PROMPT:

请在 memflow 项目根目录执行以下步骤：

1. 运行 cargo build --workspace 确认编译通过。如果有编译错误，请修复。

2. 运行 cargo test --workspace -- --nocapture 执行所有测试。记录测试结果（通过/失败/忽略的数量）。

3. 如果有测试失败：
   - 分析失败原因
   - 如果是代码 bug，修复它
   - 如果是测试环境问题（如需要数据库文件、需要运行中的终端窗口），在测试上添加 #[ignore] 并写明原因注释

4. 运行 cargo clippy --workspace 检查代码质量警告。修复所有 warning 级别的问题。

5. 最后输出测试结果汇总：
   - 编译状态
   - 测试通过数 / 失败数 / 忽略数
   - clippy 警告数
```

---

## 任务 3：清理 server.rs 死代码（P1）

**背景**：`crates/memflow-mcp/src/server.rs` 包含一个旧的 MCP server 实现，使用不同的工具命名（`memflow_search_activities` / `memflow_get_activity`），只定义了 2 个工具。它通过 `lib.rs` 导出但从未被 `main.rs` 使用。当前所有请求都由 `main.rs` 中的 `process_line()` 处理。

**涉及文件**：
- `crates/memflow-mcp/src/server.rs`
- `crates/memflow-mcp/src/lib.rs`

```text
PROMPT:

crates/memflow-mcp/src/server.rs 是一个未使用的旧版 MCP server 实现，存在以下问题：
- 使用旧的工具命名（memflow_search_activities / memflow_get_activity），与当前 protocol.rs 定义的标准名不一致
- 只声明了 2 个工具，而 main.rs 已实现 6 个
- 通过 lib.rs 的 pub mod server 导出，但 main.rs 的 process_line 函数是实际入口，从未调用 server.rs 的逻辑

请做以下修改：

1. 在 server.rs 文件顶部添加 deprecated 注释：
   //! ⚠️ DEPRECATED: This module contains the legacy MCP server implementation.
   //! The active implementation is in main.rs using ToolName enum routing.
   //! This module is kept for reference only and should not be used.
   //! See: main.rs process_line() for the current implementation.

2. 在 lib.rs 中，给 server 模块添加 #[deprecated] 属性和说明：
   #[deprecated(note = "Use main.rs process_line() instead. This module contains legacy tool definitions.")]
   pub mod server;

3. 确认修改后 cargo build 编译通过（可能需要在使用处加 #[allow(deprecated)]）。
```

---

## 任务 4：补充 Handler 级集成测试（P2）

**背景**：当前 `mcp_tool_test.rs`（25 个测试）全部只验证 JSON Schema 结构，不执行真实的 handler 逻辑。需要补充至少覆盖核心工具实际执行路径的测试。

**涉及文件**：
- `crates/memflow-mcp/tests/mcp_tool_test.rs`（或新建测试文件）
- `crates/memflow-mcp/tests/mocks/mock_db.rs`

```text
PROMPT:

在 crates/memflow-mcp/tests/ 目录下新建文件 handler_integration_test.rs，编写针对 MCP 工具实际执行逻辑的集成测试。

当前情况：
- mcp_tool_test.rs 和 schema_validation_test.rs 只验证 JSON 结构，不调用真实 handler
- tests/mocks/ 下已有 mock_db.rs 和 mock_context.rs 可复用
- main.rs 中的 handler 函数都是独立的 async fn，如 call_search_memory、call_get_recent_activities、call_get_terminal_output、call_get_system_environment 等

请实现以下测试：

1. test_get_system_environment_returns_os_info
   - 调用 call_get_system_environment(false, false, false)
   - 断言返回字符串包含 "OS:" 和 "CPU Count:" 和 "Total Memory:"

2. test_get_system_environment_with_dev_tools
   - 调用 call_get_system_environment(true, false, false)
   - 断言返回字符串包含 "[Development Tools]" 段落
   - 断言至少检测到一个工具（在开发机上 Rust 应该总是存在的）

3. test_get_terminal_output_handles_no_terminal
   - 调用 call_get_terminal_output(50)
   - 在 CI/测试环境中可能没有终端窗口，断言不 panic 即可
   - 返回 Ok 或 TerminalError::NotFound / CaptureFailed 都算通过

4. test_search_memory_empty_query
   - 构造 SearchMemoryArgs { query: Some("".to_string()), .. }
   - 断言返回错误或空结果（不 panic）

5. test_get_recent_activities_default_params
   - 调用 call_get_recent_activities(5, 20)
   - 如果数据库未初始化，断言返回友好的错误提示（包含 "not initialized" 或类似文字）

注意：
- 这些测试需要能在没有完整数据库的环境中运行，所以需要处理数据库未初始化的情况
- 使用 #[tokio::test] 作为测试宏
- 在 mod.rs 中注册该模块
```

---

## 任务 5：Cursor / Claude Desktop 端到端验证（P2）

**背景**：MCP_INTEGRATION_GUIDE.md 已写好配置文档，但从未进行过实际端到端验证。需要确认 MCP server 能被 AI 客户端正常调用。

**涉及文件**：
- `doc/MCP_INTEGRATION_GUIDE.md`
- Cursor 或 Claude Desktop 配置文件

```text
PROMPT:

请帮我验证 memflow-mcp server 能被 Cursor IDE 正常调用。步骤如下：

1. 首先确认 memflow-mcp 能编译成功：
   cd crates/memflow-mcp && cargo build --release

2. 找到编译产物路径，通常在 target/release/memflow-mcp.exe (Windows)

3. 手动测试 stdin/stdout JSON-RPC 通信：
   - 启动 memflow-mcp.exe
   - 发送 initialize 请求：
     {"jsonrpc":"2.0","method":"initialize","params":{"protocolVersion":"2024-11-05","capabilities":{},"clientInfo":{"name":"test","version":"0.1"}},"id":1}
   - 验证返回包含 capabilities.tools 和 capabilities.prompts
   - 发送 tools/list 请求：
     {"jsonrpc":"2.0","method":"tools/list","params":{},"id":2}
   - 验证返回 6 个工具定义
   - 发送一个工具调用测试：
     {"jsonrpc":"2.0","method":"tools/call","params":{"name":"get_system_environment","arguments":{"include_dev_tools":true}},"id":3}
   - 验证返回系统信息

4. 生成 Cursor MCP 配置，参照 doc/MCP_INTEGRATION_GUIDE.md 的格式：
   {
     "mcpServers": {
       "memflow": {
         "command": "<绝对路径>/memflow-mcp.exe",
         "args": [],
         "env": {}
       }
     }
   }

5. 将测试结果记录到 doc/E2E_VALIDATION_REPORT.md，包括：
   - 编译状态
   - initialize 响应是否正确
   - tools/list 返回的工具数量
   - 每个工具的调用测试结果
   - Cursor 配置是否生成成功
```

---

## 执行顺序建议

```
任务1 (P0, 30min) → 任务2 (P0, 15min) → 任务3 (P1, 15min) → 任务4 (P2, 2hr) → 任务5 (P2, 2hr)
```

任务 1 和 2 完成后，项目核心功能即完整可用。任务 3-5 为质量提升项。
