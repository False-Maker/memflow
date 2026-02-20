# 删除未实现功能：Q&A视图和智能代理

## TL;DR

> **目标**：删除前端未实现的 Q&A 视图和完整删除智能代理（Agent）功能的前后端代码
>
> **删除范围**：
> - 前端：Q&A 视图按钮、qa 类型定义
> - 后端：agent 模块、commands.rs 中的 agent 相关命令、数据库表相关
>
> **预计工作量**：中等
> **并行执行**：YES - 3 waves
> **关键路径**：前端清理 → 后端清理 → 数据库清理

---

## Context

### 原始请求
用户指出有几个功能前端页面存在但后端已经没有了，需要把前端也删掉。

### 调查发现
经过深入分析，实际发现了以下情况：

1. **Q&A 视图**：
   - 前端 `Layout.tsx` 第 174 行有 Q&A 视图按钮
   - `AppContext.tsx` 第 19 行 `currentView` 类型包含 `'qa'`
   - **但没有对应的组件实现**，点击后无内容显示

2. **智能代理（Agent）功能**：
   - 后端 `memflow-core` crate 有完整的 agent 模块实现
   - `commands.rs` 有 agent 相关命令（agent_propose_automation、agent_execute_automation、agent_list_executions、agent_cancel_execution）
   - **前端没有对应的 UI 组件**调用这些命令

### 用户确认
用户确认删除：
1. Q&A 视图（前端）
2. 智能代理功能（前后端）

---

## Work Objectives

### Core Objective
清理未使用或不完整的功能代码，保持代码库整洁。

### Concrete Deliverables
1. 从 `Layout.tsx` 删除 Q&A 视图按钮
2. 从 `AppContext.tsx` 删除 `qa` 类型定义
3. 从 `commands.rs` 删除 agent 相关命令
4. 删除 `memflow-core/src/agent` 目录
5. 从数据库迁移文件中删除 agent 相关表

### Definition of Done
- [ ] 所有 Q&A 视图相关代码已删除
- [ ] 所有 agent 相关代码已删除
- [ ] 代码编译通过
- [ ] Git 提交完成

### Must Have
- 删除所有 Q&A 视图前端代码
- 删除所有 agent 后端代码
- 删除 agent 相关数据库表

### Must NOT Have (Guardrails)
- 不要删除其他功能（如 ai、chat 相关，这些用于其他功能）
- 不要破坏现有的编译构建

---

## Verification Strategy

### Test Decision
- **Infrastructure exists**: YES
- **Automated tests**: NO (此任务仅删除代码)
- **Agent-Executed QA**: YES - 编译验证

### QA Policy
每个任务将包含具体的验证命令来确保删除后系统仍能正常工作。

---

## Execution Strategy

### Parallel Execution Waves

```
Wave 1 (前端清理 - 可以并行):
├── Task 1: 删除 Layout.tsx 中的 Q&A 视图按钮 [quick]
├── Task 2: 删除 AppContext.tsx 中的 qa 类型定义 [quick]
└── Task 3: 搜索并删除其他 Q&A 相关引用 [quick]

Wave 2 (后端清理 - 在 Wave 1 后):
├── Task 4: 删除 commands.rs 中的 agent 相关命令 [quick]
├── Task 5: 删除 memflow-core/src/agent 目录 [quick]
├── Task 6: 从 lib.rs 中删除 agent 模块导出 [quick]
└── Task 7: 搜索并删除其他 agent 相关引用 [quick]

Wave 3 (数据库清理 - 在 Wave 2 后):
├── Task 8: 删除 agent 相关数据库表 [quick]
└── Task 9: 编译验证 [quick]
```

### Dependency Matrix

- **1-3**: — — 4-7
- **4-7**: 1-3 — 8
- **8**: 4-7 — 9
- **9**: 1-8 — —

---

## TODOs

- [ ] 1. 删除 Layout.tsx 中的 Q&A 视图按钮

  **What to do**:
  - 在 `src/components/Layout.tsx` 中删除 Q&A 视图按钮
  - 第 174 行的 `{ id: 'qa', label: 'Q&A' }` 需要删除

  **Must NOT do**:
  - 不要删除其他视图按钮

  **Recommended Agent Profile**:
  - **Category**: `quick`
    - Reason: 简单的代码删除任务
  - **Skills**: N/A

  **Parallelization**:
  - **Can Run In Parallel**: YES
  - **Parallel Group**: Wave 1 (with Tasks 2, 3)
  - **Blocks**: Tasks 4-7
  - **Blocked By**: None

  **References**:
  - `src/components/Layout.tsx:174` - Q&A 视图按钮定义

  **Acceptance Criteria**:
  - [ ] Q&A 视图按钮已从视图切换数组中删除
  - [ ] 文件中不再包含 'qa' 字符串（除非是变量名等无关内容）

  **QA Scenarios**:
  ```
  Scenario: 验证 Q&A 按钮已删除
    Tool: Bash (grep)
    Steps:
      1. grep -n "qa" src/components/Layout.tsx
      2. grep -n "Q&A" src/components/Layout.tsx
    Expected Result: 不应找到 Q&A 相关的视图按钮定义
    Failure Indicators: 找到 `{ id: 'qa', label: 'Q&A' }` 行
    Evidence: .sisyphus/evidence/task-1-qa-removed.txt
  ```

  **Commit**: NO (与后续任务一起提交)
  - Message: `refactor(frontend): remove unimplemented Q&A view`
  - Files: `src/components/Layout.tsx`

---

- [ ] 2. 删除 AppContext.tsx 中的 qa 类型定义

  **What to do**:
  - 在 `src/contexts/AppContext.tsx` 中删除 `qa` 类型定义
  - 第 19 行 `currentView` 类型中的 `'qa'` 需要删除
  - 第 69 行 `SET_VIEW` action 类型中的 `'qa'` 需要删除

  **Must NOT do**:
  - 不要删除其他视图类型

  **Recommended Agent Profile**:
  - **Category**: `quick`
    - Reason: 简单的类型定义删除
  - **Skills**: N/A

  **Parallelization**:
  - **Can Run In Parallel**: YES
  - **Parallel Group**: Wave 1 (with Tasks 1, 3)
  - **Blocks**: Tasks 4-7
  - **Blocked By**: None

  **References**:
  - `src/contexts/AppContext.tsx:19` - AppState.currentView 类型
  - `src/contexts/AppContext.tsx:69` - SET_VIEW action 类型

  **Acceptance Criteria**:
  - [ ] currentView 类型中不再包含 `'qa'`
  - [ ] SET_VIEW action 类型中不再包含 `'qa'`

  **QA Scenarios**:
  ```
  Scenario: 验证 qa 类型已删除
    Tool: Bash (grep)
    Steps:
      1. grep -n "'qa'" src/contexts/AppContext.tsx
    Expected Result: 不应找到 'qa' 字符串在类型定义中
    Failure Indicators: 找到 `'qa'` 在 currentView 或 SET_VIEW 类型中
    Evidence: .sisyphus/evidence/task-2-qa-type-removed.txt
  ```

  **Commit**: NO (与后续任务一起提交)

---

- [ ] 3. 搜索并删除其他 Q&A 相关引用

  **What to do**:
  - 在整个前端代码库中搜索 `qa`、`Q&A` 相关引用
  - 删除任何相关的引用或处理逻辑

  **Must NOT do**:
  - 不要删除包含 `qa` 子字符串但无关的变量名（如 `quality`）

  **Recommended Agent Profile**:
  - **Category**: `quick`
    - Reason: 简单的搜索和确认任务
  - **Skills**: N/A

  **Parallelization**:
  - **Can Run In Parallel**: YES
  - **Parallel Group**: Wave 1 (with Tasks 1, 2)
  - **Blocks**: Tasks 4-7
  - **Blocked By**: None

  **References**:
  - N/A - 全局搜索

  **Acceptance Criteria**:
  - [ ] 确认没有其他 Q&A 相关的引用需要处理

  **QA Scenarios**:
  ```
  Scenario: 搜索 Q&A 相关引用
    Tool: Bash (grep)
    Steps:
      1. grep -r "Q&A" src/ --include="*.tsx" --include="*.ts"
      2. grep -r "\\bqa\\b" src/ --include="*.tsx" --include="*.ts"
    Expected Result: 找到的结果应该都是无关的（如注释、文档中的说明）
    Failure Indicators: 发现需要处理的功能代码引用
    Evidence: .sisyphus/evidence/task-3-qa-search.txt
  ```

  **Commit**: NO (与后续任务一起提交)

---

- [ ] 4. 删除 commands.rs 中的 agent 相关命令

  **What to do**:
  - 在 `src-tauri/src/commands.rs` 中删除 agent 相关命令：
    - `agent_propose_automation` (第 966-974 行)
    - `agent_execute_automation` (第 977-989 行)
    - `agent_list_executions` (第 992-999 行)
    - `agent_cancel_execution` (第 1011-1019 行)
  - 删除相关导入：`use memflow_core::agent;`

  **Must NOT do**:
  - 不要删除其他功能的命令

  **Recommended Agent Profile**:
  - **Category**: `quick`
    - Reason: 简单的命令函数删除
  - **Skills**: N/A

  **Parallelization**:
  - **Can Run In Parallel**: YES
  - **Parallel Group**: Wave 2 (with Tasks 5, 6, 7)
  - **Blocks**: Task 8
  - **Blocked By**: Tasks 1-3

  **References**:
  - `src-tauri/src/commands.rs:12` - agent 导入
  - `src-tauri/src/commands.rs:966-1019` - agent 相关命令

  **Acceptance Criteria**:
  - [ ] agent 导入已删除
  - [ ] 四个 agent 命令函数已删除

  **QA Scenarios**:
  ```
  Scenario: 验证 agent 命令已删除
    Tool: Bash (grep)
    Steps:
      1. grep -n "memflow_core::agent" src-tauri/src/commands.rs
      2. grep -n "agent_propose_automation\\|agent_execute_automation\\|agent_list_executions\\|agent_cancel_execution" src-tauri/src/commands.rs
    Expected Result: 不应找到 agent 导入和命令函数
    Failure Indicators: 找到 agent 导入或命令函数定义
    Evidence: .sisyphus/evidence/task-4-agent-commands-removed.txt
  ```

  **Commit**: NO (与后续任务一起提交)

---

- [ ] 5. 删除 memflow-core/src/agent 目录

  **What to do**:
  - 删除 `crates/memflow-core/src/agent` 整个目录
  - 包含 `mod.rs` 和 `tools.rs`

  **Must NOT do**:
  - 不要删除其他模块目录

  **Recommended Agent Profile**:
  - **Category**: `quick`
    - Reason: 简单的目录删除
  - **Skills**: N/A

  **Parallelization**:
  - **Can Run In Parallel**: YES
  - **Parallel Group**: Wave 2 (with Tasks 4, 6, 7)
  - **Blocks**: Task 8
  - **Blocked By**: Tasks 1-3

  **References**:
  - `crates/memflow-core/src/agent/` - agent 模块目录

  **Acceptance Criteria**:
  - [ ] agent 目录已删除

  **QA Scenarios**:
  ```
  Scenario: 验证 agent 目录已删除
    Tool: Bash (ls)
    Steps:
      1. ls crates/memflow-core/src/agent/ 2>&1 || echo "Directory not found"
    Expected Result: 目录不存在或返回错误
    Failure Indicators: 目录仍存在且包含文件
    Evidence: .sisyphus/evidence/task-5-agent-dir-removed.txt
  ```

  **Commit**: NO (与后续任务一起提交)

---

- [ ] 6. 从 lib.rs 中删除 agent 模块导出

  **What to do**:
  - 在 `crates/memflow-core/src/lib.rs` 中删除 agent 模块的导出
  - 查找类似 `pub mod agent;` 的行并删除

  **Must NOT do**:
  - 不要删除其他模块导出

  **Recommended Agent Profile**:
  - **Category**: `quick`
    - Reason: 简单的模块导出删除
  - **Skills**: N/A

  **Parallelization**:
  - **Can Run In Parallel**: YES
  - **Parallel Group**: Wave 2 (with Tasks 4, 5, 7)
  - **Blocks**: Task 8
  - **Blocked By**: Tasks 1-3

  **References**:
  - `crates/memflow-core/src/lib.rs` - 模块导出文件

  **Acceptance Criteria**:
  - [ ] agent 模块导出已删除

  **QA Scenarios**:
  ```
  Scenario: 验证 agent 模块导出已删除
    Tool: Bash (grep)
    Steps:
      1. grep -n "pub mod agent" crates/memflow-core/src/lib.rs
    Expected Result: 不应找到 agent 模块导出
    Failure Indicators: 找到 `pub mod agent;` 行
    Evidence: .sisyphus/evidence/task-6-agent-export-removed.txt
  ```

  **Commit**: NO (与后续任务一起提交)

---

- [ ] 7. 搜索并删除其他 agent 相关引用

  **What to do**:
  - 在整个代码库中搜索 agent 相关引用
  - 删除或处理遗留的引用

  **Must NOT do**:
  - 不要删除包含 `agent` 子字符串但无关的内容（如 `page_agent`、`navigation` 等）

  **Recommended Agent Profile**:
  - **Category**: `quick`
    - Reason: 简单的搜索任务
  - **Skills**: N/A

  **Parallelization**:
  - **Can Run In Parallel**: YES
  - **Parallel Group**: Wave 2 (with Tasks 4, 5, 6)
  - **Blocks**: Task 8
  - **Blocked By**: Tasks 1-3

  **References**:
  - N/A - 全局搜索

  **Acceptance Criteria**:
  - [ ] 确认没有其他 agent 功能相关的引用需要处理

  **QA Scenarios**:
  ```
  Scenario: 搜索 agent 相关引用
    Tool: Bash (grep)
    Steps:
      1. grep -r "agent_propose_automation\\|agent_execute_automation\\|agent_list_executions\\|agent_cancel_execution" src-tauri/ --include="*.rs"
      2. grep -r "AutomationProposal\\|ExecutionDto" src-tauri/ --include="*.rs"
    Expected Result: 不应找到需要处理的功能代码引用
    Failure Indicators: 发现需要处理的遗留引用
    Evidence: .sisyphus/evidence/task-7-agent-search.txt
  ```

  **Commit**: NO (与后续任务一起提交)

---

- [ ] 8. 删除 agent 相关数据库表

  **What to do**:
  - 查找并删除 agent 相关的数据库迁移文件
  - 可能的表名：`automation_proposals`、`agent_executions`
  - 位置：`src-tauri/migrations/` 或类似目录

  **Must NOT do**:
  - 不要删除其他功能相关的表

  **Recommended Agent Profile**:
  - **Category**: `quick`
    - Reason: 简单的文件查找和删除
  - **Skills**: N/A

  **Parallelization**:
  - **Can Run In Parallel**: NO
  - **Parallel Group**: Sequential (after Wave 2)
  - **Blocks**: Task 9
  - **Blocked By**: Tasks 4-7

  **References**:
  - 需要搜索数据库迁移文件目录

  **Acceptance Criteria**:
  - [ ] agent 相关数据库表定义已删除

  **QA Scenarios**:
  ```
  Scenario: 查找并删除 agent 数据库表
    Tool: Bash (find + grep)
    Steps:
      1. find src-tauri -name "*.sql" -type f
      2. grep -l "automation_proposals\\|agent_executions" $(find src-tauri -name "*.sql" -type f) 2>/dev/null || echo "No matches"
    Expected Result: 找到包含 agent 表的 SQL 文件并删除相关定义
    Failure Indicators: 表定义仍然存在
    Evidence: .sisyphus/evidence/task-8-agent-db-removed.txt
  ```

  **Commit**: NO (与编译验证一起提交)

---

- [ ] 9. 编译验证

  **What to do**:
  - 运行前端编译检查
  - 运行后端编译检查

  **Must NOT do**:
  - 不要忽略编译错误

  **Recommended Agent Profile**:
  - **Category**: `quick`
    - Reason: 编译验证是标准命令
  - **Skills**: N/A

  **Parallelization**:
  - **Can Run In Parallel**: NO
  - **Parallel Group**: Final (after all tasks)
  - **Blocks**: None
  - **Blocked By**: Tasks 1-8

  **References**:
  - N/A

  **Acceptance Criteria**:
  - [ ] 前端编译无错误
  - [ ] 后端编译无错误

  **QA Scenarios**:
  ```
  Scenario: 前端编译检查
    Tool: Bash (npm/pnpm)
    Steps:
      1. cd src-tauri && pnpm run check 2>&1 | tee .sisyphus/evidence/task-9-frontend-compile.log
    Expected Result: 无编译错误
    Failure Indicators: tsc 报错或其他编译错误
    Evidence: .sisyphus/evidence/task-9-frontend-compile.log

  Scenario: 后端编译检查
    Tool: Bash (cargo)
    Steps:
      1. cd src-tauri && cargo check 2>&1 | tee .sisyphus/evidence/task-9-backend-compile.log
    Expected Result: 无编译错误
    Failure Indicators: cargo check 报错
    Evidence: .sisyphus/evidence/task-9-backend-compile.log
  ```

  **Commit**: YES
  - Message: `refactor: remove unimplemented Q&A view and Agent feature`

---

## Final Verification Wave

- [ ] F1. **代码引用清理检查**
  全局搜索确认没有遗留的 Q&A 或 agent 相关引用。
  验证 `src/` 和 `src-tauri/` 目录中不再有相关代码引用。
  Output: `Q&A refs [CLEAN] | Agent refs [CLEAN] | VERDICT: APPROVE/REJECT`

- [ ] F2. **编译验证**
  运行完整的编译检查，确保前后端都能正常编译。
  Output: `Frontend [PASS/FAIL] | Backend [PASS/FAIL] | VERDICT`

- [ ] F3. **功能回归检查**
  验证其他功能（如 Timeline、Stats、Graph 等）仍能正常工作。
  Output: `Views [N/N working] | VERDICT`

- [ ] F4. **清理验证**
  确认删除的文件和目录不再存在。
  Output: `Files [CLEAN] | Directories [CLEAN] | VERDICT`

---

## Commit Strategy

- **Final**: `refactor: remove unimplemented Q&A view and Agent feature`
  - Frontend: Layout.tsx, AppContext.tsx
  - Backend: commands.rs, memflow-core agent module
  - Database: agent-related tables

---

## Success Criteria

### Verification Commands
```bash
# 检查 Q&A 引用
! grep -r "Q&A" src/ --include="*.tsx" --include="*.ts"
! grep -r "\\bqa\\b" src/ --include="*.tsx" --include="*.ts" | grep -v "quality"

# 检查 agent 引用
! grep -r "memflow_core::agent" src-tauri/
! ls crates/memflow-core/src/agent/ 2>&1 | grep -q "No such file"
```

### Final Checklist
- [ ] Q&A 视图按钮已删除
- [ ] qa 类型定义已删除
- [ ] agent 命令已删除
- [ ] agent 模块目录已删除
- [ ] agent 数据库表已删除
- [ ] 代码编译通过
