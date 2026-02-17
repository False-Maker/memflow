# Memflow MCP 阻塞问题修复 Prompt 文档

> 基于 2026-02-16 验证结果，共 4 个待解决问题。  
> 每个问题独立，可按优先级挑选执行。

---

## 问题 1：ONNX Runtime 版本冲突（影响语义搜索）

**现象**：启动 memflow-mcp 时报错：
```
Failed to load ONNX Runtime dylib: ort 2.0.0-rc.11 is not compatible with
the ONNX Runtime binary found at `onnxruntime.dll`;
expected version >= '1.23.x', but got '1.17.1'
```

**影响**：Embedding 模型初始化失败，语义搜索退化为 placeholder embeddings，搜索质量下降。关键词搜索不受影响。

**涉及文件**：`Cargo.toml`（workspace 和 `memflow-core`）、ONNX Runtime DLL

```text
PROMPT:

Memflow 项目中 ONNX Runtime 版本冲突导致 embedding 模型加载失败。

当前情况：
- Cargo.toml 中 ort 依赖版本为 2.0.0-rc.11，要求 ONNX Runtime >= 1.23.x
- 系统中实际存在的 onnxruntime.dll 版本为 1.17.1
- 启动时报错 "ort 2.0.0-rc.11 is not compatible"

请按以下方案之一修复：

方案 A（推荐）：更新 ONNX Runtime DLL
1. 查找项目中或系统 PATH 中的 onnxruntime.dll 位置
2. 从 https://github.com/microsoft/onnxruntime/releases 下载 1.23.x 或更高版本的 Windows x64 预编译包
3. 替换旧的 onnxruntime.dll
4. 验证：运行 memflow-mcp.exe，确认 "Failed to load ONNX Runtime" 不再出现

方案 B（降级 ort crate）：
1. 将 Cargo.toml 中 ort 依赖降级到与 1.17.1 兼容的版本（ort 1.x 系列）
2. 同时检查 fastembed 依赖是否需要对应调整
3. 运行 cargo build 确认编译通过
4. 运行 cargo test 确认测试通过

无论哪种方案，最终验证：
- cargo build -p memflow-mcp 编译通过
- 启动 memflow-mcp.exe，发送 search_memory 请求，确认语义搜索正常工作
```

---

## 问题 2：macOS 终端捕获未实现

**现象**：`get_terminal_output` 在 macOS 上返回 `PlatformNotSupported`。

**影响**：macOS 用户无法使用终端输出捕获功能。

**涉及文件**：`crates/memflow-core/src/terminal.rs`

```text
PROMPT:

crates/memflow-core/src/terminal.rs 中 get_terminal_output 在 macOS 上仅返回 PlatformNotSupported。
Windows 实现已完成（1744 行，Console API + UI Automation），需要补充 macOS 实现。

macOS 实现要求：

1. 使用 AppleScript 获取 Terminal.app 内容：
   osascript -e 'tell application "Terminal" to get contents of front window'

2. 使用 AppleScript 获取 iTerm2 内容（如果存在）：
   osascript -e 'tell application "iTerm2" to tell current session of current window to get contents'

3. 实现逻辑：
   - 先检测 iTerm2 是否在运行，如果在运行则优先从 iTerm2 获取
   - 否则检测 Terminal.app 是否在运行，从 Terminal.app 获取
   - 都没有运行则返回 TerminalError::NotFound
   - osascript 命令加 3 秒 timeout 保护

4. 参照 Windows 实现的代码风格：
   - 使用 tokio::process::Command 执行 osascript
   - 使用 tokio::time::timeout 保护
   - 返回最后 N 行（参数 lines）
   - 错误映射到 TerminalError 枚举

5. 在 #[cfg(target_os = "macos")] 块中实现 capture_terminal_output_impl()

6. 添加对应的单元测试（可以用 #[ignore] 标记需要 macOS 环境的测试）

注意：不需要实现 Linux 支持，Linux 仍可返回 PlatformNotSupported。
```

---

## 问题 3：Workspace 构建失败（Tauri App Crate）

**现象**：`cargo build --workspace` 失败（exit code 1），但 `cargo check -p memflow-mcp` 和 `cargo check -p memflow-core` 均通过。错误来自 Tauri 应用 crate。

**影响**：无法一次性构建整个 workspace，影响 CI/CD 流程。

**涉及文件**：Tauri app crate（`src-tauri/` 或相关目录）

```text
PROMPT:

memflow 项目 cargo build --workspace 失败，但 memflow-mcp 和 memflow-core 两个 crate 编译正常。
错误来自 Tauri 应用 crate。

请按以下步骤排查和修复：

1. 运行 cargo build --workspace 2>&1 并记录完整错误信息

2. 根据错误类型处理：
   - 如果是 unused import 警告升级为错误（如 image::GrayImage）：
     修复对应的 use 语句，删除未使用的导入
   - 如果是类型不匹配或 API 变更：
     检查 Cargo.toml 中相关依赖版本，修复代码适配
   - 如果是缺少系统依赖（如 WebView2、GTK）：
     记录所需依赖但不在此处修复

3. 修复后运行 cargo build --workspace 确认通过

4. 运行 cargo clippy --workspace 检查并修复所有 warning

5. 输出修复摘要：
   - 错误数量和类型
   - 修复了哪些文件
   - 是否有残留的 warning
```

---

## 问题 4：Cursor/Claude Desktop 实际接入验证

**现象**：`E2E_VALIDATION_REPORT.md` 已通过 stdin/stdout 手动验证 JSON-RPC 通信，但未在 Cursor 或 Claude Desktop 中实际配置和使用。

**影响**：无法确认 AI 客户端能正常发现和调用 Memflow 工具。

**涉及文件**：
- `doc/MCP_INTEGRATION_GUIDE.md`
- `doc/E2E_VALIDATION_REPORT.md`

```text
PROMPT:

请帮我在 Cursor IDE 中实际配置和测试 memflow-mcp server。

前置条件：
- memflow-mcp.exe 已编译成功（位于 target/debug/ 或 target/release/）
- E2E_VALIDATION_REPORT.md 中已有 JSON-RPC 手动测试通过的记录

步骤：

1. 在 Cursor 配置文件中添加 MCP server：
   打开 Cursor Settings → Features → MCP Servers → Add
   或编辑 .cursor/mcp.json：
   {
     "mcpServers": {
       "memflow": {
         "command": "D:\\Demo\\memflow\\target\\debug\\memflow-mcp.exe",
         "args": [],
         "env": {
           "MEMFLOW_MCP_READ_ONLY": "true"
         }
       }
     }
   }

2. 重启 Cursor，确认 MCP server 连接成功（状态栏应显示已连接）

3. 在 Cursor Chat 中测试以下场景：
   a. "我的系统环境是什么？" → 应触发 get_system_environment
   b. "最近 5 分钟我在做什么？" → 应触发 get_recent_activity
   c. "帮我搜索关于 Rust 的记忆" → 应触发 search_memory
   d. "当前终端输出是什么？" → 应触发 get_terminal_output

4. 记录每个测试的结果：
   - Cursor 是否正确识别了工具
   - 工具调用是否成功
   - 返回的内容是否合理
   - 响应时间是否可接受（< 5秒）

5. 将测试结果更新到 doc/E2E_VALIDATION_REPORT.md 的 Section 7 Integration Testing Checklist 中

6. 如果 Claude Desktop 也已安装，重复步骤 1-5 使用 Claude Desktop 配置
```

---

## 优先级排序

```
问题 3 (Workspace 构建) → 问题 1 (ONNX 版本) → 问题 4 (实际接入) → 问题 2 (macOS)
```

- **问题 3** 影响 CI/CD，应最先修复
- **问题 1** 影响核心搜索质量，优先级次之
- **问题 4** 是最终验收环节
- **问题 2** 仅影响 macOS 用户，如果当前不需要 macOS 支持可推迟
