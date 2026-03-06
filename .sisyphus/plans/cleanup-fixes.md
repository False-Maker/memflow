# 清理与修复计划

## TL;DR

> **快速摘要**: 清理 MemFlow 项目中的冗余文件和目录，更新 README 同步实际完成状态
>
> **交付成果**:
> - 删除重复的 `doc/` 目录
> - 删除拼写错误的 `.sisphus/` 目录
> - 更新 README.md 中的路线图状态
> - 更新核心功能清单
>
> **预估工作量**: 简单
> **并行执行**: 否 - 简单顺序任务
> **关键路径**: 清理 → 更新 README

---

## Context

### Original Request
用户要求检查当前项目代码是否与文档设想相符合。经过分析发现以下需要修复的问题：
1. `doc/` 目录与 `docs/` 目录重复
2. `.sisphus/` 目录拼写错误（应为 `.sisyphus/`）
3. README.md 中的功能状态和路线图标记滞后于实际实现

### Interview Summary
**关键讨论**:
- 已完成代码与文档的全面对比分析
- 确认了三个需要修复的问题
- 用户同意开始操作

**Research Findings**:
- `doc/MCP_TOOL_CONTRACT_v1.md` 是 `docs/MCP_TOOL_CONTRACT_v1.md` 的重复
- `.sisphus/` 和 `.sisyphus/` 都存在，但前者是拼写错误
- README.md 显示 OCR 和 LLM API "开发中"，但代码已实现

---

## Work Objectives

### Core Objective
清理项目中的冗余文件和目录，更新 README 以准确反映当前实现状态。

### Concrete Deliverables
- 清理后的目录结构（无重复文件）
- 更新后的 README.md（准确的功能状态）

### Definition of Done
- [ ] `doc/` 目录已删除
- [ ] `.sisphus/` 目录已删除（evidence 已合并到 `.sisyphus/`）
- [ ] README.md 核心功能状态已更新
- [ ] README.md Phase 3 路线图已更新

### Must Have
- 保留 `docs/` 目录
- 保留 `.sisyphus/` 目录
- 更新后的状态必须与实际代码一致

### Must NOT Have (Guardrails)
- 不删除 `.sisyphus/evidence/` 中的任何文件
- 不改变项目其他配置或代码
- 不修改其他文档文件

---

## Verification Strategy

### Test Decision
- **Infrastructure exists**: NO
- **Automated tests**: None
- **Framework**: None

### QA Policy
每个任务包含 Agent-Executed QA 场景：

- **文件/目录操作**: 使用 Bash 验证删除/更新结果
- **README 更新**: 使用 Read 验证内容变更

---

## Execution Strategy

### Parallel Execution Waves

```
Wave 1 (Start Immediately — 清理冗余目录):
└── Task 1: 删除重复目录和文件 [quick]

Wave 2 (After Wave 1 — 更新文档):
└── Task 2: 更新 README.md 状态 [quick]

Wave 3 (After Wave 2 — 验证):
└── Task 3: 验证清理结果 [quick]
```

---

## TODOs

- [ ] 1. **清理冗余目录和文件**

  **What to do**:
  - 删除 `doc/` 目录（`docs/` 的重复）
  - 删除 `.sisphus/` 目录（拼写错误）
  - 在删除前将 `.sisphus/evidence/` 内容合并到 `.sisyphus/evidence/`

  **Must NOT do**:
  - 不删除 `docs/` 目录
  - 不删除 `.sisyphus/` 目录
  - 不删除 `.sisyphus/evidence/` 中的任何文件（应合并）

  **Recommended Agent Profile**:
  > Select category + skills based on task domain. Justify each choice.
  - **Category**: `quick`
    - Reason: 简单的文件系统操作，无需复杂逻辑
  - **Skills**: []
    - 无需特定技能

  **Parallelization**:
  - **Can Run In Parallel**: NO
  - **Parallel Group**: Sequential
  - **Blocks**: Task 2
  - **Blocked By**: None

  **References**:
  - 无需参考代码文件

  **Acceptance Criteria**:

  **QA Scenarios (MANDATORY)**:

  ```
  Scenario: 验证 doc 目录已删除
    Tool: Bash
    Preconditions: doc 目录存在
    Steps:
      1. 执行: ls -la D:/Demo/memflow/doc 2>&1
      2. 验证: 返回 "No such file or directory" 或类似错误
    Expected Result: doc 目录不存在
    Failure Indicators: 目录仍然存在
    Evidence: .sisyphus/evidence/task-1-doc-deleted.log

  Scenario: 验证 .sisphus 目录已删除，.sisyphus 保留且包含合并的 evidence
    Tool: Bash
    Preconditions: .sisphus 和 .sisyphus 目录都存在
    Steps:
      1. 执行: ls -la D:/Demo/memflow/.sisphus 2>&1
      2. 验证: 返回 "No such file or directory" 或类似错误
      3. 执行: ls -la D:/Demo/memflow/.sisyphus/evidence
      4. 验证: 目录存在且包含文件（至少有来自 .sisphus 的文件）
    Expected Result: .sisphus 不存在，.sisyphus/evidence 包含合并后的文件
    Failure Indicators: .sisyphus 仍存在 或 .sisyphus/evidence 文件丢失
    Evidence: .sisyphus/evidence/task-1-sisyphus-merged.log
  ```

  **Commit**: NO

- [ ] 2. **更新 README.md 功能状态**

  **What to do**:
  - 更新 "核心功能" 部分：
    - 将 "🚧 AI 分析（开发中）" 改为 "✅ AI 分析（RAG 混合检索）"
    - 将 "🚧 知识图谱（开发中）" 改为 "✅ 知识图谱可视化"
    - 将 "🚧 智能代理（开发中）" 改为 "✅ 智能代理（自动化提案与执行）"
  - 更新 "Phase 3: 体验升级" 部分：
    - 将 "[ ] 集成实际 OCR 引擎" 改为 "[x] 集成实际 OCR 引擎"
    - 将 "[ ] 集成 LLM API" 改为 "[x] 集成 LLM API"
    - 将 "🚧 进行中" 改为 "✅ 已完成"

  **Must NOT do**:
  - 不修改 README.md 其他部分
  - 不修改其他文件

  **Recommended Agent Profile**:
  - **Category**: `quick`
    - Reason: 简单的文本编辑操作
  - **Skills**: []
    - 无需特定技能

  **Parallelization**:
  - **Can Run In Parallel**: NO
  - **Parallel Group**: Sequential
  - **Blocks**: Task 3
  - **Blocked By**: Task 1

  **References**:

  **文件引用** (需要编辑的位置):
  - `D:\Demo\memflow\README.md:98-108` - 核心功能部分
  - `D:\Demo\memflow\README.md:125-130` - Phase 3 路线图

  **Acceptance Criteria**:

  **QA Scenarios (MANDATORY)**:

  ```
  Scenario: 验证 README.md 核心功能状态已更新
    Tool: Read
    Preconditions: README.md 已更新
    Steps:
      1. 读取: D:\Demo\memflow\README.md 第 98-108 行
      2. 验证: 包含 "✅ AI 分析（RAG 混合检索）"
      3. 验证: 包含 "✅ 知识图谱可视化"
      4. 验证: 包含 "✅ 智能代理（自动化提案与执行）"
      5. 验证: 不包含 "🚧" 标记的功能
    Expected Result: 所有功能都标记为 ✅ 已完成
    Failure Indicators: 仍有功能标记为 "🚧 开发中"
    Evidence: .sisyphus/evidence/task-2-readme-features.log

  Scenario: 验证 README.md Phase 3 路线图已更新
    Tool: Read
    Preconditions: README.md 已更新
    Steps:
      1. 读取: D:\Demo\memflow\README.md 第 125-130 行
      2. 验证: "[x] 集成实际 OCR 引擎"
      3. 验证: "[x] 集成 LLM API"
      4. 验证: 状态为 "✅ 已完成" 或 "✅ 进行中"（不是 "🚧 进行中"）
    Expected Result: OCR 和 LLM API 标记为已完成
    Failure Indicators: OCR 或 LLM API 仍标记为未完成
    Evidence: .sisyphus/evidence/task-2-readme-phase3.log
  ```

  **Commit**: NO

- [ ] 3. **验证清理和更新结果**

  **What to do**:
  - 验证所有目录清理操作正确完成
  - 验证 README.md 更新正确
  - 生成最终验证报告

  **Must NOT do**:
  - 不进行任何额外修改

  **Recommended Agent Profile**:
  - **Category**: `quick`
    - Reason: 验证操作
  - **Skills**: []
    - 无需特定技能

  **Parallelization**:
  - **Can Run In Parallel**: NO
  - **Parallel Group**: Sequential
  - **Blocks**: None
  - **Blocked By**: Task 1, Task 2

  **References**:
  - 无

  **Acceptance Criteria**:

  **QA Scenarios (MANDATORY)**:

  ```
  Scenario: 最终验证 - 目录结构正确
    Tool: Bash
    Preconditions: Task 1 和 Task 2 已完成
    Steps:
      1. 执行: ls -la D:/Demo/memflow/ | grep -E "^d.*(docs|\.sisyphus)"
      2. 验证: docs/ 目录存在
      3. 验证: .sisyphus/ 目录存在
      4. 执行: ls -la D:/Demo/memflow/ | grep -E "^d.*(doc|\.sisphus[^y])"
      5. 验证: doc/ 目录不存在
      6. 验证: .sisphus/ 目录不存在（不含 .sisyphus）
    Expected Result: 只有 docs/ 和 .sisyphus/ 存在
    Failure Indicators: 仍有冗余目录
    Evidence: .sisyphus/evidence/task-3-final-structure.log

  Scenario: 最终验证 - README 内容正确
    Tool: Bash (grep)
    Preconditions: README.md 已更新
    Steps:
      1. 执行: grep -c "✅ AI 分析" D:/Demo/memflow/README.md
      2. 验证: 返回值 >= 1
      3. 执行: grep -c "🚧.*开发中" D:/Demo/memflow/README.md
      4. 验证: 返回值 = 0（没有"开发中"标记）
      5. 执行: grep -c "\[x\] 集成实际 OCR" D:/Demo/memflow/README.md
      6. 验证: 返回值 = 1
    Expected Result: 所有状态标记正确
    Failure Indicators: 仍有未更新或错误的标记
    Evidence: .sisyphus/evidence/task-3-final-readme.log
  ```

  **Commit**: NO

---

## Final Verification Wave (MANDATORY — after ALL implementation tasks)

- [ ] F1. **Plan Compliance Audit** — `quick`
  验证所有 Must Have 完成，Must NOT Have 遵守。

- [ ] F2. **Clean Result Verification** — `quick`
  验证目录清理和 README 更新的最终结果。

---

## Commit Strategy

- **Final**: `chore: 清理冗余目录和更新文档状态`

---

## Success Criteria

### Verification Commands
```bash
# 验证目录结构
ls -la D:/Demo/memflow/ | grep -E "^d.*(docs|\.sisyphus|doc|\.sisphus)"

# 验证 README 状态
grep -E "(✅|🚧)" D:/Demo/memflow/README.md | head -10
grep -E "\[x?\] 集成" D:/Demo/memflow/README.md
```

### Final Checklist
- [ ] `doc/` 目录已删除
- [ ] `.sisphus/` 目录已删除
- [ ] `.sisyphus/evidence/` 包含合并的文件
- [ ] README.md 核心功能全部标记为 ✅
- [ ] README.md Phase 3 OCR 和 LLM 标记为 [x]
