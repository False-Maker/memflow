# 任务 1：接通 get_system_environment 的开发工具检测

## TL;DR

> **快速摘要**：修改 `call_get_system_environment` 函数，使其实际使用三个参数（`include_dev_tools`/`include_processes`/`include_ports`），调用现有的 6 个 `detect_*_version()` 函数，并添加进程和端口检测功能。
>
> **交付物**：
> - 修改后的 `call_get_system_environment` 函数（约 1202-1227 行）
> - 新增开发工具检测功能（调用 6 个现有 detect 函数）
> - 新增开发进程检测功能（遍历 sys.processes()）
> - 新增端口占用检测功能（netstat -ano）
>
> **预计工作量**：中等
> **并行执行**：否 - 单一函数修改
> **关键路径**：理解现有模式 → 修改函数 → 测试验证

---

## 上下文

### 原始需求
修改 `crates/memflow-mcp/src/main.rs` 中的 `call_get_system_environment` 函数，使其真正使用 `include_dev_tools`、`include_processes`、`include_ports` 三个参数。

### 面试摘要
- **确认问题**：函数接收三个参数但完全忽略它们
- **确认范围**：仅修改 `call_get_system_environment` 函数，不改动检测函数本身
- **确认输出格式**：需要符合现有输出风格

### 研究发现

#### 现有检测函数模式（lines 1101-1199）
```rust
async fn detect_node_version() -> Option<String> {
    let timeout = Duration::from_secs(3);
    let mut cmd = Command::new("node");
    cmd.args(["--version"]);

    match tokio::time::timeout(timeout, cmd.output()).await {
        Ok(Ok(output)) if output.status.success() => {
            Some(String::from_utf8_lossy(&output.stdout).trim().to_string())
        }
        _ => None,
    }
}
```

**6 个检测函数**：
- `detect_node_version()` - `node --version`
- `detect_python_version()` - `python --version` (fallback to `python3`)
- `detect_rust_version()` - `rustc --version`
- `detect_docker_version()` - `docker --version`
- `detect_go_version()` - `go version`
- `detect_java_version()` - `java -version` (从 stderr 读取)

#### 当前函数状态（lines 1202-1227）
```rust
async fn call_get_system_environment(
    include_dev_tools: bool,
    include_processes: bool,
    include_ports: bool,
) -> Result<String> {
    use sysinfo::System;

    info!("Getting system environment");

    let mut sys = System::new_all();
    sys.refresh_all();

    let mut output = String::new();

    // Basic system info
    output.push_str("[System Environment]\n\n");
    output.push_str(&format!("OS: {}\n", System::name().unwrap_or_default()));
    output.push_str(&format!("OS Version: {}\n", System::os_version().unwrap_or_default()));
    output.push_str(&format!("Kernel: {}\n", System::kernel_version().unwrap_or_default()));
    output.push_str(&format!("Hostname: {}\n", System::host_name().unwrap_or_default()));
    output.push_str(&format!("CPU Count: {}\n", sys.cpus().len()));
    output.push_str(&format!("Total Memory: {} GB\n", sys.total_memory() / 1024 / 1024 / 1024));
    output.push_str(&format!("Used Memory: {} GB\n", sys.used_memory() / 1024 / 1024 / 1024));

    Ok(output)
}
```

**问题**：三个参数完全被忽略。

#### 路由层参数传递（lines 535-537）
```rust
let include_dev = args["include_dev_tools"].as_bool().unwrap_or(true);
let include_procs = args["include_processes"].as_bool().unwrap_or(true);
let include_ports = args["include_ports"].as_bool().unwrap_or(false);
```

参数正确传递到函数调用。

---

## 工作目标

### 核心目标
使 `call_get_system_environment` 函数实际使用三个参数，返回完整的系统环境信息。

### 具体交付物
- 修改后的 `call_get_system_environment` 函数，包含：
  1. 开发工具版本检测（当 `include_dev_tools = true`）
  2. 开发进程检测（当 `include_processes = true`）
  3. 端口占用检测（当 `include_ports = true`）

### 完成定义
- [ ] 函数能正确响应三个参数
- [ ] 输出格式符合要求的风格
- [ ] 所有异步调用都有 3 秒超时保护

### 必须包含
- 调用 6 个现有的 `detect_*_version` 函数
- 使用 `sys.processes()` 遍历进程
- 使用 `netstat -ano` 检查端口（Windows）
- 所有外部命令调用带超时保护

### 必须不包含（护栏）
- 不修改现有的 6 个 `detect_*_version` 函数
- 不添加新的检测函数
- 不改变函数签名

---

## 验证策略

> **通用规则：零人工干预**
>
> 本计划中的所有任务必须能够在无需人工操作的情况下进行验证。

### 测试决策
- **基础设施存在**：否（未发现测试框架配置）
- **自动化测试**：否（本任务为单一函数修改）
- **Agent 执行 QA 场景**：是

### Agent 执行 QA 场景（必填）

#### 场景 1：编译检查
```bash
Scenario: 修改后的代码能成功编译
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

#### 场景 2：基础输出验证
```bash
Scenario: 函数返回基础系统信息（参数全为 false）
  Tool: Bash (cargo run)
  Preconditions: 编译成功
  Steps:
    1. 启动 MCP 服务器
    2. 发送 JSON-RPC 请求调用 get_system_environment，参数全设为 false
    3. Assert: 输出包含 "[System Environment]"
    4. Assert: 输出包含 "OS:"
    5. Assert: 输出不包含 "[Development Tools]"
    6. Assert: 输出不包含 "[Active Dev Processes]"
    7. Assert: 输出不包含 "[Port Usage]"
  Expected Result: 仅返回基础系统信息
  Evidence: 响应内容捕获
```

#### 场景 3：开发工具检测
```bash
Scenario: include_dev_tools=true 时检测开发工具
  Tool: Bash (cargo run)
  Preconditions: Node.js 已安装
  Steps:
    1. 启动 MCP 服务器
    2. 发送 JSON-RPC 请求，include_dev_tools=true, 其他=false
    3. Assert: 输出包含 "[Development Tools]"
    4. Assert: 输出包含 "Node.js:" 或 "Not found"
    5. Assert: 输出包含 "Python:" 或 "Not found"
    6. Assert: 输出包含 "Rust:" 或 "Not found"
    7. Assert: 输出包含 "Docker:" 或 "Not found"
    8. Assert: 输出包含 "Go:" 或 "Not found"
    9. Assert: 输出包含 "Java:" 或 "Not found"
  Expected Result: 显示开发工具版本或 "Not found"
  Evidence: 响应内容捕获
```

#### 场景 4：进程检测
```bash
Scenario: include_processes=true 时检测开发进程
  Tool: Bash (cargo run)
  Preconditions: 系统运行中
  Steps:
    1. 启动 MCP 服务器
    2. 发送 JSON-RPC 请求，include_processes=true, 其他=false
    3. Assert: 输出包含 "[Active Dev Processes]"
    4. Assert: 如果存在开发进程，输出包含 "(PID"
  Expected Result: 显示开发进程列表
  Evidence: 响应内容捕获
```

#### 场景 5：端口检测
```bash
Scenario: include_ports=true 时检测端口占用
  Tool: Bash (cargo run)
  Preconditions: Windows 系统
  Steps:
    1. 启动 MCP 服务器
    2. 发送 JSON-RPC 请求，include_ports=true, 其他=false
    3. Assert: 输出包含 "[Port Usage]"
    4. Assert: 输出包含端口信息格式 ":3000"
  Expected Result: 显示端口占用信息
  Evidence: 响应内容捕获
```

#### 场景 6：全部参数启用
```bash
Scenario: 所有参数为 true 时返回完整信息
  Tool: Bash (cargo run)
  Preconditions: 编译成功
  Steps:
    1. 启动 MCP 服务器
    2. 发送 JSON-RPC 请求，所有参数=true
    3. Assert: 输出包含 "[System Environment]"
    4. Assert: 输出包含 "[Development Tools]"
    5. Assert: 输出包含 "[Active Dev Processes]"
    6. Assert: 输出包含 "[Port Usage]"
    7. Assert: 输出包含基础系统信息
  Expected Result: 返回完整的系统环境信息
  Evidence: 响应内容捕获
```

---

## 执行策略

### 并行执行波次
单一任务，无需并行。

---

## TODOs

- [ ] 1. 修改 call_get_system_environment 函数

  **做什么**：
  - 修改 `crates/memflow-mcp/src/main.rs` 的 `call_get_system_environment` 函数（约 1202-1227 行）
  - 添加 `include_dev_tools` 参数处理逻辑
  - 添加 `include_processes` 参数处理逻辑
  - 添加 `include_ports` 参数处理逻辑

  **具体实现步骤**：

  **1.1 开发工具检测（当 `include_dev_tools = true`）**：
  ```rust
  // 在基础系统信息后添加
  if include_dev_tools {
      output.push_str("\n[Development Tools]\n\n");

      let (node, python, rust, docker, go, java) = tokio::join!(
          detect_node_version(),
          detect_python_version(),
          detect_rust_version(),
          detect_docker_version(),
          detect_go_version(),
          detect_java_version(),
      );

      output.push_str(&format!("Node.js: {}\n", node.unwrap_or_else(|| "Not found".to_string())));
      output.push_str(&format!("Python: {}\n", python.unwrap_or_else(|| "Not found".to_string())));
      output.push_str(&format!("Rust: {}\n", rust.unwrap_or_else(|| "Not found".to_string())));
      output.push_str(&format!("Docker: {}\n", docker.unwrap_or_else(|| "Not found".to_string())));
      output.push_str(&format!("Go: {}\n", go.unwrap_or_else(|| "Not found".to_string())));
      output.push_str(&format!("Java: {}\n", java.unwrap_or_else(|| "Not found".to_string())));
  }
  ```

  **1.2 开发进程检测（当 `include_processes = true`）**：
  ```rust
  if include_processes {
      output.push_str("\n[Active Dev Processes]\n\n");

      let dev_process_names = [
          "node", "python", "python3", "cargo", "rustc", "java",
          "docker", "code", "cursor", "npm", "yarn", "pnpm",
          "git", "cargo", "go", "gradle", "mvn"
      ];

      let mut found_processes = false;
      for (pid, process) in sys.processes() {
          let name = process.name().to_string_lowercase();
          if dev_process_names.iter().any(|&n| name.contains(n)) {
              output.push_str(&format!("{} (PID {})\n", process.name(), pid));
              found_processes = true;
          }
      }

      if !found_processes {
          output.push_str("No development processes found\n");
      }
  }
  ```

  **1.3 端口占用检测（当 `include_ports = true`）**：
  ```rust
  if include_ports {
      output.push_str("\n[Port Usage]\n\n");

      let ports_to_check = [3000, 3001, 4200, 5000, 5173, 8000, 8080, 8443];
      let timeout = Duration::from_secs(3);

      match tokio::time::timeout(timeout, Command::new("netstat").args(["-ano"]).output()).await {
          Ok(Ok(netstat_output)) if netstat_output.status.success() => {
              let output_str = String::from_utf8_lossy(&netstat_output.stdout);

              for port in ports_to_check {
                  let port_pattern = format!(":{}", port);
                  let line = output_str.lines().find(|line| {
                      line.contains(&port_pattern) && line.contains("LISTENING")
                  });

                  if let Some(l) = line {
                      let parts: Vec<&str> = l.split_whitespace().collect();
                      let pid = parts.get(4).unwrap_or(&"");
                      output.push_str(&format!(":{} - LISTENING (PID {})\n", port, pid));
                  } else {
                      output.push_str(&format!(":{} - Available\n", port));
                  }
              }
          }
          Ok(Ok(_)) | Ok(Err(_)) | Err(_) => {
              output.push_str("Port check failed or timed out\n");
          }
      }
  }
  ```

  **不能做的事**：
  - 不修改现有的 6 个 `detect_*_version` 函数
  - 不改变函数签名
  - 不添加新的检测函数

  **推荐的代理配置**：
  - **类别**: `quick`
    - 理由: 单一文件修改，模式明确，任务范围清晰
  - **技能**: 无特定技能需求
    - 标准的 Rust 代码编辑任务

  **并行化**：
  - **可并行运行**: 否
  - **并行组**: 顺序执行
  - **阻塞**: 无
  - **被阻塞**: 无（可立即开始）

  **参考**（关键 - 请详尽）：

  **模式参考**（现有代码需遵循）：
  - `crates/memflow-mcp/src/main.rs:1102-1113` - `detect_node_version` 函数的超时和错误处理模式
  - `crates/memflow-mcp/src/main.rs:1211-1224` - 当前 `call_get_system_environment` 的输出格式风格
  - `crates/memflow-mcp/src/main.rs:535-537` - 路由层参数传递方式

  **API/类型参考**：
  - `sysinfo::System` - 进程枚举 API：`sys.processes()` 返回 `HashMap<Pid, Process>`
  - `tokio::process::Command` - 异步命令执行
  - `tokio::time::timeout` - 超时保护

  **测试参考**：
  - 无现有测试模式

  **文档参考**：
  - sysinfo 文档: https://docs.rs/sysinfo/latest/sysinfo/ - `System::processes()` 方法

  **外部参考**：
  - tokio::time::timeout 文档: https://docs.rs/tokio/latest/tokio/time/fn.timeout.html

  **为什么每个参考重要**：
  - `detect_node_version` 模式展示了如何在所有 6 个检测函数中使用一致的错误处理
  - 当前函数的输出格式确保新增部分风格一致
  - sysinfo::System::processes() 文档展示了如何正确遍历进程

  **验收标准**：

  > **可由代理执行的验证**

  - [ ] 代码编译成功：`cargo build --release` 在 `crates/memflow-mcp` 目录下执行 → 退出码 0
  - [ ] 当 `include_dev_tools=false` 时，输出不包含 "[Development Tools]"
  - [ ] 当 `include_dev_tools=true` 时，输出包含 "[Development Tools]" 和所有 6 个工具行
  - [ ] 当 `include_processes=false` 时，输出不包含 "[Active Dev Processes]"
  - [ ] 当 `include_processes=true` 时，输出包含 "[Active Dev Processes]"
  - [ ] 当 `include_ports=false` 时，输出不包含 "[Port Usage]"
  - [ ] 当 `include_ports=true` 时，输出包含 "[Port Usage]"

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
    预期结果: 编译成功，无错误
    失败指示: 退出码非 0 或输出包含 "error"
    证据: 编译输出保存到 .sisyphus/evidence/task-1-compile.txt

  场景: 基础功能测试（参数全为 false）
    工具: Bash
    前提条件: 代码已编译
    步骤:
      1. cd "D:\Demo\memflow\crates\memflow-mcp"
      2. cargo run --release -- --stdio
      3. 发送 JSON-RPC 请求（stdin）:
         {"jsonrpc":"2.0","method":"tools/call","params":{"name":"get_system_environment","arguments":{"include_dev_tools":false,"include_processes":false,"include_ports":false}},"id":1}
      4. 验证: 响应包含 "[System Environment]"
      5. 验证: 响应不包含 "[Development Tools]"
      6. 验证: 响应不包含 "[Active Dev Processes]"
      7. 验证: 响应不包含 "[Port Usage]"
    预期结果: 仅返回基础系统信息
    失败指示: 输出包含任何额外的部分
    证据: 响应保存到 .sisyphus/evidence/task-1-basic-output.txt

  场景: 开发工具检测
    工具: Bash
    前提条件: 代码已编译
    步骤:
      1. cd "D:\Demo\memflow\crates\memflow-mcp"
      2. cargo run --release -- --stdio
      3. 发送 JSON-RPC 请求:
         {"jsonrpc":"2.0","method":"tools/call","params":{"name":"get_system_environment","arguments":{"include_dev_tools":true,"include_processes":false,"include_ports":false}},"id":1}
      4. 验证: 响应包含 "[Development Tools]"
      5. 验证: 响应包含 "Node.js:"
      6. 验证: 响应包含 "Python:"
      7. 验证: 响应包含 "Rust:"
      8. 验证: 响应包含 "Docker:"
      9. 验证: 响应包含 "Go:"
      10. 验证: 响应包含 "Java:"
    预期结果: 显示所有开发工具版本或 "Not found"
    失败指示: 任何工具行缺失
    证据: 响应保存到 .sisyphus/evidence/task-1-dev-tools.txt

  场景: 进程检测
    工具: Bash
    前提条件: 代码已编译
    步骤:
      1. cd "D:\Demo\memflow\crates\memflow-mcp"
      2. cargo run --release -- --stdio
      3. 发送 JSON-RPC 请求:
         {"jsonrpc":"2.0","method":"tools/call","params":{"name":"get_system_environment","arguments":{"include_dev_tools":false,"include_processes":true,"include_ports":false}},"id":1}
      4. 验证: 响应包含 "[Active Dev Processes]"
    预期结果: 显示开发进程列表
    失败指示: 输出不包含 "[Active Dev Processes]"
    证据: 响应保存到 .sisyphus/evidence/task-1-processes.txt

  场景: 端口检测
    工具: Bash
    前提条件: Windows 系统，代码已编译
    步骤:
      1. cd "D:\Demo\memflow\crates\memflow-mcp"
      2. cargo run --release -- --stdio
      3. 发送 JSON-RPC 请求:
         {"jsonrpc":"2.0","method":"tools/call","params":{"name":"get_system_environment","arguments":{"include_dev_tools":false,"include_processes":false,"include_ports":true}},"id":1}
      4. 验证: 响应包含 "[Port Usage]"
      5. 验证: 响应包含 ":3000" 或 ":8080" 或其他检查的端口
    预期结果: 显示端口占用信息
    失败指示: 输出不包含 "[Port Usage]"
    证据: 响应保存到 .sisyphus/evidence/task-1-ports.txt
  ```

  **证据捕获**：
  - [ ] 编译输出保存到 `.sisyphus/evidence/task-1-compile.txt`
  - [ ] 每个测试场景的响应保存到对应的 `.sisyphus/evidence/task-1-*.txt` 文件

  **提交**: 是
  - 消息: `fix(mcp): connect get_system_environment parameters to detection functions`
  - 文件: `crates/memflow-mcp/src/main.rs`
  - 提交前验证: `cargo build --release`

---

## 提交策略

| 任务后 | 消息 | 文件 | 验证 |
|--------|------|------|------|
| 1 | `fix(mcp): connect get_system_environment parameters to detection functions` | crates/memflow-mcp/src/main.rs | cargo build --release |

---

## 成功标准

### 验证命令
```bash
cd D:\Demo\memflow\crates\memflow-mcp
cargo build --release
```

### 最终检查清单
- [ ] 当 `include_dev_tools=true` 时显示开发工具版本
- [ ] 当 `include_processes=true` 时显示开发进程列表
- [ ] 当 `include_ports=true` 时显示端口占用信息
- [ ] 所有参数为 false 时仅显示基础系统信息
- [ ] 代码编译成功
- [ ] 现有的 6 个 `detect_*_version` 函数未被修改
