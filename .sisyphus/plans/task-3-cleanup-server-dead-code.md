# 任务 3：清理 server.rs 死代码

## TL;DR

> **快速摘要**：标记 `crates/memflow-mcp/src/server.rs` 为 deprecated，因为这是一个未使用的旧版 MCP server 实现。当前实现使用 `main.rs` 中的 `process_line()` 函数和 `ToolName` enum 路由。
>
> **交付物**：
> - 在 `server.rs` 顶部添加 deprecated 文档注释
> - 在 `lib.rs` 中给 server 模块添加 `#[deprecated]` 属性
> - 确认编译通过（可能需要 `#[allow(deprecated)]`）
>
> **预计工作量**：小
> **并行执行**：否 - 单一标记任务
> **关键路径**：添加注释 → 验证编译

---

## 上下文

### 原始需求
标记 `crates/memflow-mcp/src/server.rs` 为 deprecated，因为这是一个未使用的旧版实现。

### 面试摘要
- **确认问题**：`server.rs` 包含旧的 MCP server 实现，但从未被使用
- **确认范围**：仅添加 deprecated 标记，不删除代码
- **确认输出**：编译通过，有清晰的弃用说明

### 研究发现

#### server.rs 的旧实现特征（lines 1-200）
```rust
use crate::context::McpContext;
use crate::prompts;
use crate::protocol::{JsonRpcError, JsonRpcRequest, JsonRpcResponse, ListToolsResult, Tool};
// ...

pub struct McpServer {
    context: Arc<McpContext>,
}

impl McpServer {
    pub fn new(context: Arc<McpContext>) -> Self { ... }
    pub async fn handle_request(&self, request: JsonRpcRequest) -> JsonRpcResponse { ... }
    // ...
}
```

**旧工具命名**（lines 55, 68）：
- `memflow_search_activities` - 搜索活动
- `memflow_get_activity` - 获取单个活动

**当前实现**（main.rs）：
- `search_memory` - 搜索内存/活动
- `get_recent_activities` - 获取最近活动
- `get_terminal_output` - 获取终端输出
- `get_system_environment` - 获取系统环境
- `get_cursor_info` - 获取光标信息
- `ping` - 心跳检测

**问题**：
1. 工具命名不匹配（`memflow_search_activities` vs `search_memory`）
2. 只定义了 2 个工具，而 main.rs 有 6 个
3. `lib.rs` 导出 `pub mod server`，但 `main.rs` 的 `process_line()` 是实际入口

#### lib.rs 的导出（lines 1-5）
```rust
pub mod context;
pub mod prompts;
pub mod protocol;
pub mod server;  // ← 导出旧实现，但未被使用
```

#### main.rs 的实际实现
- `process_line()` 函数处理所有 JSON-RPC 请求
- 使用 `ToolName` enum 进行路由（lines 535-537, 637-943）
- 完全独立于 `server.rs` 的实现

---

## 工作目标

### 核心目标
将 `server.rs` 标记为 deprecated，为未来的代码清理做准备。

### 具体交付物
- `server.rs` 顶部的 deprecated 文档注释
- `lib.rs` 中的 `#[deprecated]` 属性
- 验证编译通过

### 完成定义
- [ ] `server.rs` 有清晰的 deprecated 说明
- [ ] `lib.rs` 的 server 模块有 `#[deprecated]` 属性
- [ ] 代码能成功编译
- [ ] 如有必要，在使用处添加 `#[allow(deprecated)]`

### 必须包含
- 清晰的说明为什么弃用（旧实现 vs 新实现）
- 指向当前实现的引用
- 保留代码以便参考

### 必须不包含（护栏）
- 不删除 `server.rs` 文件
- 不修改 `server.rs` 的实现代码
- 不移除 `lib.rs` 中的 `pub mod server` 导出

---

## 验证策略

> **通用规则：零人工干预**
>
> 本计划中的所有任务必须能够在无需人工操作的情况下进行验证。

### 测试决策
- **基础设施存在**：否（此任务不需要测试）
- **自动化测试**：否（仅添加注释）
- **Agent 执行 QA 场景**：是

### Agent 执行 QA 场景（必填）

#### 场景 1：编译检查
```bash
Scenario: 添加 deprecated 标记后代码能成功编译
  Tool: Bash (cargo)
  Preconditions: Rust 工具链已安装
  Steps:
    1. cd D:\Demo\memflow\crates\memflow-mcp
    2. cargo build --release
    3. Assert: exit code is 0
    4. Assert: output contains "Finished"
  Expected Result: 编译成功，无错误
  Evidence: 编译输出捕获
```

#### 场景 2：验证 server.rs 注释
```bash
Scenario: server.rs 顶部包含 deprecated 文档注释
  Tool: Read
  Preconditions: 代码已修改
  Steps:
    1. 读取 crates/memflow-mcp/src/server.rs 前 10 行
    2. Assert: 包含 "DEPRECATED"
    3. Assert: 包含 "legacy MCP server implementation"
    4. Assert: 包含 "main.rs process_line()"
  Expected Result: 注释存在且包含关键信息
  Evidence: 文件内容捕获
```

#### 场景 3：验证 lib.rs deprecated 属性
```bash
Scenario: lib.rs 中 server 模块有 #[deprecated] 属性
  Tool: Read
  Preconditions: 代码已修改
  Steps:
    1. 读取 crates/memflow-mcp/src/lib.rs
    2. Assert: 包含 "#[deprecated"
    3. Assert: 包含 "note = \"Use main.rs process_line() instead"
  Expected Result: deprecated 属性存在
  Evidence: 文件内容捕获
```

#### 场景 4：验证 clippy 警告（预期）
```bash
Scenario: clippy 应该检测到 deprecated 模块（如果被使用）
  Tool: Bash (cargo clippy)
  Preconditions: 代码已修改
  Steps:
    1. cd D:\Demo\memflow\crates\memflow-mcp
    2. cargo clippy --release
    3. 检查是否有关于 deprecated 模块的警告（可选）
  Expected Result: clippy 可能显示 deprecated 警告（如果模块被引用）
  Evidence: clippy 输出捕获
```

---

## 执行策略

### 并行执行波次
单一任务，无需并行。

---

## TODOs

- [ ] 1. 在 server.rs 添加 deprecated 注释

  **做什么**：
  - 在 `crates/memflow-mcp/src/server.rs` 文件顶部添加 deprecated 文档注释
  - 注释应包含：DEPRECATED 标记、原因说明、当前实现位置

  **具体实现步骤**：

  **1.1 添加文件顶部注释**：
  ```rust
  //! ⚠️ DEPRECATED: This module contains the legacy MCP server implementation.
  //! The active implementation is in main.rs using ToolName enum routing.
  //! This module is kept for reference only and should not be used.
  //! See: main.rs process_line() for the current implementation.

  use crate::context::McpContext;
  use crate::prompts;
  use crate::protocol::{JsonRpcError, JsonRpcRequest, JsonRpcResponse, ListToolsResult, Tool};
  // ... rest of the file
  ```

  **为什么这个注释重要**：
  - 清晰标记为 DEPRECATED，警告开发者不要使用
  - 解释为什么弃用（旧实现 vs 新实现）
  - 指向当前实现的正确位置
  - 说明保留代码的目的（参考用途）

  **不能做的事**：
  - 不修改文件中的其他代码
  - 不删除任何函数或结构
  - 不改变任何实现逻辑

  **推荐的代理配置**：
  - **类别**: `quick`
    - 理由: 单一文件注释添加，任务范围非常明确
  - **技能**: 无特定技能需求
    - 标准的 Rust 文档注释任务

  **并行化**：
  - **可并行运行**: 否
  - **并行组**: 顺序执行
  - **阻塞**: 无
  - **被阻塞**: 无（可立即开始）

  **参考**（关键 - 请详尽）：

  **模式参考**（需要遵循的风格）：
  - 参照 Rust 标准库的 deprecated 模式
  - 参照项目中的其他文档注释风格（如 `protocol.rs`, `main.rs`）

  **API/类型参考**：
  - Rust 文档注释语法：`//!` - 文件级注释
  - `#[deprecated]` 属性语法

  **测试参考**：
  - 无现有测试模式

  **文档参考**：
  - Rust 文档注释指南: https://doc.rust-lang.org/rustdoc/how-to-write-documentation.html
  - Deprecated 属性: https://doc.rust-lang.org/reference/attributes/diagnostics.html

  **外部参考**：
  - Rust API Guidelines (C-DEPRECATED): https://rust-lang.github.io/api-guidelines/documentation.html#c-deprecated

  **为什么每个参考重要**：
  - 文档注释指南确保注释格式符合 Rust 约定
  - Deprecated 属性文档确保正确使用属性
  - Rust API Guidelines 提供弃用 API 的最佳实践

  **验收标准**：

  > **可由代理执行的验证**

  - [ ] `server.rs` 文件顶部包含 `//! ⚠️ DEPRECATED:` 注释
  - [ ] 注释包含 "legacy MCP server implementation"
  - [ ] 注释包含 "main.rs process_line()" 引用
  - [ ] 注释说明模块保留用于参考

  **证据捕获**：
  - [ ] 文件头部内容保存到 `.sisyphus/evidence/task-3-server-rs-header.txt`

---

- [ ] 2. 在 lib.rs 添加 deprecated 属性

  **做什么**：
  - 在 `crates/memflow-mcp/src/lib.rs` 中给 `server` 模块添加 `#[deprecated]` 属性
  - 属性应包含说明消息

  **具体实现步骤**：

  **2.1 修改 pub mod server 声明**：
  ```rust
  pub mod context;
  pub mod prompts;
  pub mod protocol;
  #[deprecated(note = "Use main.rs process_line() instead. This module contains legacy tool definitions.")]
  pub mod server;
  ```

  **为什么这个属性重要**：
  - 编译时警告任何使用此模块的代码
  - 清晰说明应该使用的替代方案
  - 提供上下文信息说明为什么弃用

  **不能做的事**：
  - 不删除 `pub mod server` 声明
  - 不修改其他模块的导出

  **推荐的代理配置**：
  - **类别**: `quick`
    - 理由: 单一属性添加，任务范围明确
  - **技能**: 无特定技能需求
    - 标准的 Rust 属性添加任务

  **并行化**：
  - **可并行运行**: 否（依赖任务 1）
  - **并行组**: 顺序执行
  - **阻塞**: 无
  - **被阻塞**: 任务 1（虽然技术上可以并行，但顺序执行更清晰）

  **参考**（关键 - 请详尽）：

  **模式参考**：
  - Rust 标准库中大量使用 `#[deprecated]` 属性
  - 参考 Rust 源码中的示例

  **API/类型参考**：
  - `#[deprecated]` 属性语法
  - `#[deprecated(note = "...")]` - 自定义消息

  **测试参考**：
  - 无现有测试模式

  **文档参考**：
  - Deprecated 属性: https://doc.rust-lang.org/reference/attributes/diagnostics.html

  **外部参考**：
  - Rust API Guidelines (C-DEPRECATED): https://rust-lang.github.io/api-guidelines/documentation.html#c-deprecated

  **为什么每个参考重要**：
  - Deprecated 属性文档确保正确使用属性语法
  - Rust API Guidelines 提供编写弃用说明的最佳实践

  **验收标准**：

  > **可由代理执行的验证**

  - [ ] `lib.rs` 中 `pub mod server` 前有 `#[deprecated]` 属性
  - [ ] 属性包含 `note = "Use main.rs process_line() instead. This module contains legacy tool definitions."`
  - [ ] `pub mod server` 仍然是公开的（未被删除）

  **证据捕获**：
  - [ ] `lib.rs` 内容保存到 `.sisyphus/evidence/task-3-lib-rs.txt`

---

- [ ] 3. 验证编译通过

  **做什么**：
  - 运行 `cargo build` 确保修改后的代码能成功编译
  - 如有编译错误，添加必要的 `#[allow(deprecated)]`

  **具体实现步骤**：

  **3.1 运行编译**：
  ```bash
  cd D:\Demo\memflow\crates\memflow-mcp
  cargo build --release
  ```

  **3.2 如果编译失败**：
  - 检查错误消息
  - 如果是 deprecated 警告导致的编译错误，在相关位置添加 `#[allow(deprecated)]`
  - 示例：
    ```rust
    #[allow(deprecated)]
    use crate::server;  // 如果某处引用了 server 模块
    ```

  **为什么编译验证重要**：
  - 确保标记 deprecated 不会破坏构建
  - 验证语法正确
  - 检查是否有意外的依赖关系

  **不能做的事**：
  - 不删除 deprecated 标记来解决编译错误
  - 不修改除添加 `#[allow(deprecated)]` 以外的代码

  **推荐的代理配置**：
  - **类别**: `quick`
    - 理由: 标准的编译验证任务
  - **技能**: 无特定技能需求
    - 基本的编译验证任务

  **并行化**：
  - **可并行运行**: 否（依赖任务 1 和 2）
  - **并行组**: 顺序执行
  - **阻塞**: 无
  - **被阻塞**: 任务 1 和 2

  **参考**（关键 - 请详尽）：

  **模式参考**：
  - 项目中现有的编译验证流程（如 task 1, task 2）

  **API/类型参考**：
  - `#[allow(deprecated)]` - 允许 deprecated 警告

  **测试参考**：
  - 参考其他任务的编译验证场景

  **文档参考**：
  - Rust 属性文档: https://doc.rust-lang.org/reference/attributes/diagnostics.html

  **外部参考**：
  - Cargo Build 文档: https://doc.rust-lang.org/cargo/commands/cargo-build.html

  **为什么每个参考重要**：
  - 属性文档确保正确使用 `#[allow(deprecated)]`
  - Cargo Build 文档确保使用正确的编译命令

  **验收标准**：

  > **可由代理执行的验证**

  - [ ] `cargo build --release` 退出码为 0
  - [ ] 编译输出包含 "Finished" 或 "Compiling"
  - [ ] 无编译错误（error 级别）
  - [ ] 警告（warning）可以接受（如 deprecated 警告）

  **Agent 执行 QA 场景（必填 —— 每场景超详细）**：

  ```
  场景: 编译验证
    工具: Bash
    前提条件: Rust 工具链已安装
    步骤:
      1. cd "D:\Demo\memflow\crates\memflow-mcp"
      2. cargo build --release
      3. 验证: 退出码为 0
      4. 验证: 输出包含 "Finished" 或 "Compiling"
      5. 验证: 输出不包含 "error[" （编译错误）
    预期结果: 编译成功
    失败指示: 退出码非 0 或输出包含 "error["
    证据: 编译输出保存到 .sisyphus/evidence/task-3-compile.txt

  场景: 验证 server.rs 注释
    工具: Read
    前提条件: 代码已修改
    步骤:
      1. 读取 crates/memflow-mcp/src/server.rs 的前 10 行
      2. 验证: 包含 "//! ⚠️ DEPRECATED:"
      3. 验证: 包含 "legacy MCP server implementation"
      4. 验证: 包含 "main.rs process_line()"
    预期结果: 注释存在且包含关键信息
    失败指示: 任何关键信息缺失
    证据: 文件头部保存到 .sisyphus/evidence/task-3-server-header.txt

  场景: 验证 lib.rs deprecated 属性
    工具: Read
    前提条件: 代码已修改
    步骤:
      1. 读取 crates/memflow-mcp/src/lib.rs
      2. 验证: 包含 "#[deprecated"
      3. 验证: 包含 "note = \"Use main.rs process_line() instead"
      4. 验证: 包含 "pub mod server"
    预期结果: deprecated 属性存在且格式正确
    失败指示: 属性缺失或格式错误
    证据: 文件内容保存到 .sisyphus/evidence/task-3-lib-rs.txt

  场景: Clippy 检查（可选）
    工具: Bash
    前提条件: 代码已编译
    步骤:
      1. cd "D:\Demo\memflow\crates\memflow-mcp"
      2. cargo clippy --release
      3. 记录警告数量和类型
    预期结果: clippy 可能显示 deprecated 警告（如果模块被使用）
    失败指示: 有新的 error 级别警告
    证据: clippy 输出保存到 .sisyphus/evidence/task-3-clippy.txt
  ```

  **证据捕获**：
  - [ ] 编译输出保存到 `.sisyphus/evidence/task-3-compile.txt`
  - [ ] server.rs 头部保存到 `.sisyphus/evidence/task-3-server-header.txt`
  - [ ] lib.rs 内容保存到 `.sisyphus/evidence/task-3-lib-rs.txt`
  - [ ] clippy 输出（可选）保存到 `.sisyphus/evidence/task-3-clippy.txt`

  **提交**: 是
  - 消息: `chore(mcp): mark server.rs as deprecated`
  - 文件: `crates/memflow-mcp/src/server.rs`, `crates/memflow-mcp/src/lib.rs`
  - 提交前验证: `cargo build --release`

---

## 提交策略

| 任务后 | 消息 | 文件 | 验证 |
|--------|------|------|------|
| 1, 2, 3 | `chore(mcp): mark server.rs as deprecated` | crates/memflow-mcp/src/server.rs, crates/memflow-mcp/src/lib.rs | cargo build --release |

**说明**：所有三个任务完成后创建单个提交，因为它们是一个逻辑单元。

---

## 成功标准

### 验证命令
```bash
cd D:\Demo\memflow\crates\memflow-mcp
cargo build --release
```

### 最终检查清单
- [ ] `server.rs` 顶部有 deprecated 文档注释
- [ ] 注释包含 DEPRECATED 标记
- [ ] 注释解释为什么弃用
- [ ] 注释指向当前实现（main.rs process_line()）
- [ ] `lib.rs` 中 server 模块有 `#[deprecated]` 属性
- [ ] `lib.rs` 中属性包含说明消息
- [ ] 代码能成功编译（cargo build）
- [ ] `server.rs` 文件未被删除
- [ ] `server.rs` 的实现代码未被修改

### 额外验证
- [ ] （可选）clippy 没有新的 error 级别警告
- [ ] （可选）如果有编译错误，已用 `#[allow(deprecated)]` 处理

---

## 附录：完整修改示例

### server.rs 修改后头部
```rust
//! ⚠️ DEPRECATED: This module contains the legacy MCP server implementation.
//! The active implementation is in main.rs using ToolName enum routing.
//! This module is kept for reference only and should not be used.
//! See: main.rs process_line() for the current implementation.

use crate::context::McpContext;
use crate::prompts;
use crate::protocol::{JsonRpcError, JsonRpcRequest, JsonRpcResponse, ListToolsResult, Tool};
use memflow_core::context::RuntimeContext;
use memflow_core::db;
use serde_json::{json, Value};
use std::sync::Arc;
use tracing::{error, info};

pub struct McpServer {
    // ... rest unchanged
}
```

### lib.rs 修改后内容
```rust
pub mod context;
pub mod prompts;
pub mod protocol;
#[deprecated(note = "Use main.rs process_line() instead. This module contains legacy tool definitions.")]
pub mod server;
```

---

## 为什么这个任务重要

1. **代码清理准备**：标记死代码为 deprecated 是清理的第一步
2. **开发者体验**：清晰的弃用标记警告新开发者不要使用旧代码
3. **技术债务管理**：为未来的代码删除做好准备
4. **文档改进**：记录为什么存在两套实现以及应该使用哪个

---

## 风险和缓解

| 风险 | 可能性 | 影响 | 缓解措施 |
|------|--------|------|----------|
| 编译错误 | 低 | 中 | 添加 `#[allow(deprecated)]` |
| 其他模块依赖 server | 低 | 高 | 检查编译错误，如果存在则分析依赖关系 |
| 过度激进删除代码 | N/A | 高 | 明确不删除，仅标记 deprecated |

---

## 依赖关系

- **前置依赖**：无
- **后置任务**：
  - Task 4: Handler 级集成测试（不依赖此任务）
  - Task 5: Cursor / Claude Desktop 端到端验证（不依赖此任务）

**执行顺序建议**：Task 1 → Task 2 → Task 3（此任务）→ Task 4 → Task 5
