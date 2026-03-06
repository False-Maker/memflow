# 打包两种产品并更新 README

## TL;DR

> **快速摘要**: 打包 MemFlow 桌面端产品和 MCP 产品，更新 README.md
>
> **交付成果**:
> - 桌面端安装包 (MemFlow-Setup.exe)
> - MCP 独立发布包 (MemFlow-MCP.zip)
> - 更新的 README.md（双产品说明）
>
> **预估工作量**: 中等
> **并行执行**: 是（两个产品可并行构建）
> **关键路径**: 构建 → 打包 → 验证 → 更新 README

---

## Context

### Original Request
用户要求：
1. 打包桌面端产品（给普通用户使用）
2. 打包 MCP 产品（给开发人员使用）
3. 更新 README.md

### Interview Summary
**关键讨论**:
- 桌面端 = Core + Desktop UI (Tauri 窗口)
- MCP 产品 = Core + MCP Server (IDE 集成)
- 两种产品共享 Core Daemon
- MCP 产品使用托盘模式控制录制

**Research Findings**:
- Tauri 配置已完整 (`src-tauri/tauri.conf.json`)
- 资源文件存在 (`src-tauri/resources/` 包含 onnxruntime.dll, rapidocr.exe)
- MCP 和 Daemon 已配置 Release 优化

---

## Work Objectives

### Core Objective
构建并打包两种独立产品，更新 README 以清晰说明两种产品的用途和使用方式。

### Concrete Deliverables
- **桌面端**: `dist-desktop/MemFlow-Setup.exe` (NSIS 安装包)
- **MCP 产品**: `dist-mcp/MemFlow-MCP-v0.1.0.zip` (独立发布包)
- **README.md**: 更新为双产品说明

### Definition of Done
- [ ] 桌面端安装包构建成功
- [ ] MCP 产品发布包构建成功
- [ ] 两个包可以独立运行
- [ ] README.md 更新完成

### Must Have
- 桌面端包含 Core + Desktop UI
- MCP 产品包含 Core Daemon + MCP Server
- 包含必要的资源文件 (onnxruntime.dll, rapidocr.exe, icons)
- README 清晰区分两种产品

### Must NOT Have (Guardrails)
- 不修改源代码（仅构建和打包）
- 不破坏现有构建配置
- 不混用两个产品的文件

---

## Verification Strategy

### Test Decision
- **Infrastructure exists**: NO
- **Automated tests**: None
- **Framework**: None

### QA Policy
每个任务包含 Agent-Executed QA 场景：
- **构建验证**: 检查编译成功、输出文件存在
- **安装包验证**: 检查安装包结构、文件完整性
- **README 验证**: 检查内容正确、链接有效

---

## Execution Strategy

### Parallel Execution Waves

```
Wave 1 (Start Immediately — 准备构建环境):
├── Task 1: 创建打包输出目录 [quick]
└── Task 2: 备份现有 README.md [quick]

Wave 2 (After Wave 1 — 构建两个产品，MAX PARALLEL):
├── Task 3: 构建桌面端产品 [unspecified-high]
└── Task 4: 构建 MCP 产品 [unspecified-high]

Wave 3 (After Wave 2 — 打包发布):
├── Task 5: 打包桌面端安装包 [quick]
└── Task 6: 打包 MCP 发布包 [quick]

Wave 4 (After Wave 3 — 更新文档):
└── Task 7: 更新 README.md [writing]

Wave 5 (After Wave 4 — 验证):
├── Task 8: 验证桌面端安装包 [quick]
├── Task 9: 验证 MCP 发布包 [quick]
└── Task 10: 生成构建报告 [quick]
```

---

## TODOs

- [ ] 1. **创建打包输出目录**

  **What to do**:
  - 创建 `dist-desktop/` 目录（桌面端打包输出）
  - 创建 `dist-mcp/` 目录（MCP 产品打包输出）
  - 创建 `build-logs/` 目录（构建日志）

  **Must NOT do**:
  - 不删除现有文件

  **Recommended Agent Profile**:
  - **Category**: `quick`
    - Reason: 简单的目录创建操作
  - **Skills**: `[]`

  **Parallelization**:
  - **Can Run In Parallel**: NO
  - **Parallel Group**: Sequential (Wave 1)
  - **Blocks**: Task 2, 3, 4
  - **Blocked By**: None

  **Acceptance Criteria**:

  **QA Scenarios (MANDATORY)**:

  ```
  Scenario: 验证打包目录已创建
    Tool: Bash
    Preconditions: 项目根目录存在
    Steps:
      1. 执行: ls -la D:/Demo/memflow/ | grep -E "dist-|build-logs"
      2. 验证: dist-desktop 目录存在
      3. 验证: dist-mcp 目录存在
      4. 验证: build-logs 目录存在
    Expected Result: 所有打包目录已创建
    Failure Indicators: 任何目录不存在
    Evidence: .sisyphus/evidence/task-1-dirs-created.log
  ```

  **Commit**: NO

- [ ] 2. **备份现有 README.md**

  **What to do**:
  - 复制 `README.md` 到 `README.md.backup`
  - 记录备份时间戳

  **Must NOT do**:
  - 不修改原 README.md

  **Recommended Agent Profile**:
  - **Category**: `quick`
    - Reason: 简单的文件复制操作
  - **Skills**: `[]`

  **Parallelization**:
  - **Can Run In Parallel**: YES
  - **Parallel Group**: Wave 1 (with Task 1)
  - **Blocks**: Task 7
  - **Blocked By**: None

  **Acceptance Criteria**:

  **QA Scenarios (MANDATORY)**:

  ```
  Scenario: 验证 README 备份已创建
    Tool: Bash
    Preconditions: README.md 存在
    Steps:
      1. 执行: ls -la D:/Demo/memflow/README.md.backup
      2. 验证: 备份文件存在
      3. 执行: diff D:/Demo/memflow/README.md D:/Demo/memflow/README.md.backup
      4. 验证: 文件内容相同
    Expected Result: README.md.backup 与 README.md 相同
    Failure Indicators: 备份文件不存在或内容不同
    Evidence: .sisyphus/evidence/task-2-readme-backup.log
  ```

  **Commit**: NO

- [ ] 3. **构建桌面端产品**

  **What to do**:
  - 前端构建: `pnpm install && pnpm build`
  - Tauri 构建: `cargo tauri build --release`
  - 收集输出文件到 `dist-desktop/`

  **Must NOT do**:
  - 不修改源代码

  **Recommended Agent Profile**:
  - **Category**: `unspecified-high`
    - Reason: 构建操作耗时较长，需要完整编译
  - **Skills**: `[]`

  **Parallelization**:
  - **Can Run In Parallel**: YES
  - **Parallel Group**: Wave 2 (with Task 4)
  - **Blocks**: Task 5, 8
  - **Blocked By**: Task 1

  **References**:
  - `src-tauri/tauri.conf.json` - 打包配置
  - `package.json` - 构建命令
  - `src-tauri/resources/` - 资源文件

  **Acceptance Criteria**:

  **QA Scenarios (MANDATORY)**:

  ```
  Scenario: 验证桌面端构建成功
    Tool: Bash
    Preconditions: 依赖已安装
    Steps:
      1. 执行: pnpm install (如果需要)
      2. 执行: pnpm build
      3. 验证: dist/ 目录存在且包含前端文件
      4. 执行: cargo tauri build --release
      5. 验证: src-tauri/target/release/bundle/ 目录存在
      6. 验证: MemFlow-Setup.exe 或 MemFlow.exe 存在
    Expected Result: 桌面端安装包构建成功
    Failure Indicators: 构建失败或输出文件缺失
    Evidence: .sisyphus/evidence/task-3-desktop-build.log

  Scenario: 验证桌面端资源文件包含
    Tool: Bash
    Preconditions: 构建完成
    Steps:
      1. 检查安装包或输出目录
      2. 验证: resources/ 目录包含 onnxruntime.dll
      3. 验证: resources/ 目录包含 rapidocr.exe
      4. 验证: icons/ 目录包含图标文件
    Expected Result: 所有资源文件已包含
    Failure Indicators: 关键资源文件缺失
    Evidence: .sisyphus/evidence/task-3-desktop-resources.log
  ```

  **Commit**: NO

- [ ] 4. **构建 MCP 产品**

  **What to do**:
  - 构建 Core Daemon: `cargo build --release -p memflow-daemon`
  - 构建 MCP Server: `cargo build --release -p memflow-mcp`
  - 收集可执行文件到 `dist-mcp/`

  **Must NOT do**:
  - 不修改源代码

  **Recommended Agent Profile**:
  - **Category**: `unspecified-high`
    - Reason: 构建操作耗时较长，需要完整编译
  - **Skills**: `[]`

  **Parallelization**:
  - **Can Run In Parallel**: YES
  - **Parallel Group**: Wave 2 (with Task 3)
  - **Blocks**: Task 6, 9
  - **Blocked By**: Task 1

  **References**:
  - `crates/memflow-daemon/Cargo.toml` - Daemon 配置
  - `crates/memflow-mcp/Cargo.toml` - MCP 配置
  - `src-tauri/resources/` - 共享资源文件

  **Acceptance Criteria**:

  **QA Scenarios (MANDATORY)**:

  ```
  Scenario: 验证 MCP 产品构建成功
    Tool: Bash
    Preconditions: Rust 工具链已安装
    Steps:
      1. 执行: cargo build --release -p memflow-daemon
      2. 验证: target/release/memflow-daemon.exe 存在
      3. 执行: cargo build --release -p memflow-mcp
      4. 验证: target/release/memflow-mcp.exe 存在
      5. 执行: file target/release/memflow-daemon.exe (或 ls -la)
      6. 验证: 文件大小合理（strip + LTO 优化后应该较小）
    Expected Result: MCP 产品两个 exe 都构建成功
    Failure Indicators: 任何 exe 缺失或构建失败
    Evidence: .sisyphus/evidence/task-4-mcp-build.log

  Scenario: 验证 MCP 产品依赖
    Tool: Bash
    Preconditions: 构建完成
    Steps:
      1. 检查 memflow-daemon.exe 和 memflow-mcp.exe
      2. 执行: ldd target/release/memflow-daemon.exe (Windows) 或 objdump -p
      3. 验证: 没有缺失的动态链接库
    Expected Result: 可执行文件依赖完整
    Failure Indicators: 缺少必要的 DLL
    Evidence: .sisyphus/evidence/task-4-mcp-deps.log
  ```

  **Commit**: NO

- [ ] 5. **打包桌面端安装包**

  **What to do**:
  - 从 `src-tauri/target/release/bundle/` 复制安装包
  - 复制便携版 `MemFlow.exe`（如果有）
  - 创建版本信息和说明文件
  - 生成 SHA256 校验和

  **Must NOT do**:
  - 不重新构建

  **Recommended Agent Profile**:
  - **Category**: `quick`
    - Reason: 文件复制和打包操作
  - **Skills**: `[]`

  **Parallelization**:
  - **Can Run In Parallel**: NO
  - **Parallel Group**: Sequential (Wave 3)
  - **Blocks**: Task 8
  - **Blocked By**: Task 3

  **References**:
  - `src-tauri/target/release/bundle/` - Tauri 打包输出

  **Acceptance Criteria**:

  **QA Scenarios (MANDATORY)**:

  ```
  Scenario: 验证桌面端安装包
    Tool: Bash
    Preconditions: 构建已完成
    Steps:
      1. 列出: ls -la D:/Demo/memflow/dist-desktop/
      2. 验证: MemFlow-Setup.exe 存在（或类似名称）
      3. 验证: 文件大小合理（> 50MB）
      4. 验证: README.txt 或类似说明文件存在
      5. 验证: SHA256.txt 校验和文件存在
    Expected Result: 完整的桌面端安装包
    Failure Indicators: 安装包缺失或不完整
    Evidence: .sisyphus/evidence/task-5-desktop-package.log
  ```

  **Commit**: NO

- [ ] 6. **打包 MCP 发布包**

  **What to do**:
  - 创建 `dist-mcp/MemFlow-MCP-v0.1.0/` 目录
  - 复制 `memflow-daemon.exe` 和 `memflow-mcp.exe`
  - 复制资源文件: `resources/` 目录
  - 创建 `README.txt` 使用说明
  - 创建 `SHA256.txt` 校验和
  - 打包为 ZIP 文件

  **Must NOT do**:
  - 不重新构建

  **Recommended Agent Profile**:
  - **Category**: `quick`
    - Reason: 文件复制和打包操作
  - **Skills**: `[]`

  **Parallelization**:
  - **Can Run In Parallel**: NO
  - **Parallel Group**: Sequential (Wave 3)
  - **Blocks**: Task 9
  - **Blocked By**: Task 4

  **References**:
  - `docs/STARTUP_MCP.md` - 使用说明
  - `src-tauri/resources/` - 资源文件

  **Acceptance Criteria**:

  **QA Scenarios (MANDATORY)**:

  ```
  Scenario: 验证 MCP 发布包
    Tool: Bash
    Preconditions: 构建已完成
    Steps:
      1. 列出: ls -la D:/Demo/memflow/dist-mcp/
      2. 验证: MemFlow-MCP-v0.1.0.zip 存在
      3. 验证: ZIP 文件大小合理
      4. 解压验证: unzip -l 或类似命令
      5. 验证: ZIP 包含两个 exe 和 resources/
    Expected Result: 完整的 MCP 发布包
    Failure Indicators: 发布包缺失或不完整
    Evidence: .sisyphus/evidence/task-6-mcp-package.log

  Scenario: 验证 MCP 包内容完整性
    Tool: Bash
    Preconditions: ZIP 已创建
    Steps:
      1. 解压到临时目录: unzip -q MemFlow-MCP-v0.1.0.zip -d /tmp/test-mcp
      2. 验证: memflow-daemon.exe 存在
      3. 验证: memflow-mcp.exe 存在
      4. 验证: resources/onnxruntime.dll 存在
      5. 验证: README.txt 存在
      6. 验证: SHA256.txt 存在
    Expected Result: MCP 包包含所有必要文件
    Failure Indicators: 关键文件缺失
    Evidence: .sisyphus/evidence/task-6-mcp-contents.log
  ```

  **Commit**: NO

- [ ] 7. **更新 README.md**

  **What to do**:
  - 重写 README.md 为双产品说明
  - 包含以下部分：
    - 项目简介（两种产品）
    - 产品对比表格
    - 桌面端产品说明（安装、使用、功能）
    - MCP 产品说明（安装、配置、使用）
    - 架构图
    - 开发者指南（简要）

  **Must NOT do**:
  - 不删除现有内容中的有用信息

  **Recommended Agent Profile**:
  - **Category**: `writing`
    - Reason: 文档编写，需要清晰的结构和表达
  - **Skills**: `[]`

  **Parallelization**:
  - **Can Run In Parallel**: NO
  - **Parallel Group**: Sequential (Wave 4)
  - **Blocks**: Task 10
  - **Blocked By**: Task 2, 5, 6

  **References**:
  - `README.md.backup` - 原始内容参考
  - `docs/STARTUP_DESKTOP.md` - 桌面端说明
  - `docs/STARTUP_MCP.md` - MCP 说明
  - `docs/architecture_v2.md` - 架构文档

  **Acceptance Criteria**:

  **QA Scenarios (MANDATORY)**:

  ```
  Scenario: 验证 README 内容完整
    Tool: Read
    Preconditions: README.md 已更新
    Steps:
      1. 读取: D:/Demo/memflow/README.md
      2. 验证: 包含"桌面端产品"章节
      3. 验证: 包含"MCP 产品"章节
      4. 验证: 包含产品对比表格
      5. 验证: 包含安装说明
      6. 验证: 包含架构图或说明
    Expected Result: README 包含所有必要章节
    Failure Indicators: 关键章节缺失
    Evidence: .sisyphus/evidence/task-7-readme-content.log

  Scenario: 验证 README 格式正确
    Tool: Bash
    Preconditions: README.md 已更新
    Steps:
      1. 执行: head -50 D:/Demo/memflow/README.md
      2. 验证: Markdown 格式正确（标题、列表、表格）
      3. 验证: 链接格式正确
      4. 验证: 代码块格式正确
    Expected Result: README 格式正确无语法错误
    Failure Indicators: 格式错误或链接失效
    Evidence: .sisyphus/evidence/task-7-readme-format.log
  ```

  **Commit**: NO

- [ ] 8. **验证桌面端安装包**

  **What to do**:
  - 验证安装包完整性
  - 检查安装包结构
  - 生成验证报告

  **Must NOT do**:
  - 不修改已打包的文件

  **Recommended Agent Profile**:
  - **Category**: `quick`
    - Reason: 验证操作
  - **Skills**: `[]`

  **Parallelization**:
  - **Can Run In Parallel**: YES
  - **Parallel Group**: Wave 5 (with Task 9, 10)
  - **Blocks**: None
  - **Blocked By**: Task 5

  **Acceptance Criteria**:

  **QA Scenarios (MANDATORY)**:

  ```
  Scenario: 最终验证桌面端安装包
    Tool: Bash
    Preconditions: 安装包已打包
    Steps:
      1. 执行: ls -la D:/Demo/memflow/dist-desktop/
      2. 验证: MemFlow-Setup.exe 存在
      3. 验证: 文件大小 > 50MB
      4. 验证: SHA256.txt 存在且内容有效
      5. 执行: sha256sum -c SHA256.txt（如果支持）
    Expected Result: 桌面端安装包完整且校验通过
    Failure Indicators: 文件缺失或校验失败
    Evidence: .sisyphus/evidence/task-8-desktop-final.log
  ```

  **Commit**: NO

- [ ] 9. **验证 MCP 发布包**

  **What to do**:
  - 验证发布包完整性
  - 解压测试
  - 生成验证报告

  **Must NOT do**:
  - 不修改已打包的文件

  **Recommended Agent Profile**:
  - **Category**: `quick`
    - Reason: 验证操作
  - **Skills**: `[]`

  **Parallelization**:
  - **Can Run In Parallel**: YES
  - **Parallel Group**: Wave 5 (with Task 8, 10)
  - **Blocks**: None
  - **Blocked By**: Task 6

  **Acceptance Criteria**:

  **QA Scenarios (MANDATORY)**:

  ```
  Scenario: 最终验证 MCP 发布包
    Tool: Bash
    Preconditions: 发布包已打包
    Steps:
      1. 执行: ls -la D:/Demo/memflow/dist-mcp/
      2. 验证: MemFlow-MCP-v0.1.0.zip 存在
      3. 解压测试: unzip -t MemFlow-MCP-v0.1.0.zip
      4. 验证: 无错误输出
      5. 验证: SHA256.txt 存在
    Expected Result: MCP 发布包完整且有效
    Failure Indicators: 文件损坏或校验失败
    Evidence: .sisyphus/evidence/task-9-mcp-final.log
  ```

  **Commit**: NO

- [ ] 10. **生成构建报告**

  **What to do**:
  - 汇总所有构建和验证结果
  - 生成 BUILD_REPORT.md
  - 包含文件清单、校验和、使用说明

  **Must NOT do**:
  - 不修改已打包的文件

  **Recommended Agent Profile**:
  - **Category**: `writing`
    - Reason: 报告编写
  - **Skills**: `[]`

  **Parallelization**:
  - **Can Run In Parallel**: YES
  - **Parallel Group**: Wave 5 (with Task 8, 9)
  - **Blocks**: None
  - **Blocked By**: Task 8, 9

  **Acceptance Criteria**:

  **QA Scenarios (MANDATORY)**:

  ```
  Scenario: 验证构建报告
    Tool: Read
    Preconditions: 构建已完成
    Steps:
      1. 读取: D:/Demo/memflow/BUILD_REPORT.md
      2. 验证: 包含桌面端产品信息
      3. 验证: 包含 MCP 产品信息
      4. 验证: 包含文件清单
      5. 验证: 包含 SHA256 校验和
      6. 验证: 包含使用说明
    Expected Result: 构建报告完整清晰
    Failure Indicators: 关键信息缺失
    Evidence: .sisyphus/evidence/task-10-report.log
  ```

  **Commit**: NO

---

## Final Verification Wave (MANDATORY — after ALL implementation tasks)

- [ ] F1. **Build Verification** — `quick`
  验证两个产品都能正常构建和运行。

- [ ] F2. **Documentation Verification** — `quick`
  验证 README.md 准确描述两种产品。

---

## Commit Strategy

- **Final**: `chore: 打包桌面端和MCP产品，更新README`

---

## Success Criteria

### Verification Commands
```bash
# 桌面端
ls -la dist-desktop/MemFlow-Setup.exe
sha256sum dist-desktop/MemFlow-Setup.exe

# MCP 产品
ls -la dist-mcp/MemFlow-MCP-v0.1.0.zip
unzip -t dist-mcp/MemFlow-MCP-v0.1.0.zip

# README
grep -E "桌面端|MCP" README.md
```

### Final Checklist
- [ ] 桌面端安装包可安装运行
- [ ] MCP 发布包可解压使用
- [ ] README.md 清晰说明两种产品
- [ ] 构建报告完整
