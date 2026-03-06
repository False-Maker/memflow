# 修复截图压缩和数据目录显示问题

## TL;DR

> **快速摘要**: 修复两个问题：1) 数据目录一直显示"加载中" 2) 截图占用空间过大（14张85M）
>
> **交付成果**:
> - 数据目录路径正确显示（显示实际路径或默认路径说明）
> - 截图压缩质量降低（默认80改为60）减少存储占用
>
> **预估工作量**: 简单
> **并行执行**: 否 - 两个独立修复
> **关键路径**: 修复后端配置 → 测试验证

---

## Context

### Original Request
用户报告两个问题：
1. 数据目录那里一直显示"加载中"
2. 14个截图占了85M（平均每张约6MB，压缩率太低）

### Interview Summary
**关键讨论**:
- 数据目录加载逻辑需要处理 `Option<String>` 返回值
- 当前压缩质量 80 导致文件过大
- 用户希望减少存储占用

**Research Findings**:
- `loadDataSavePath()` 调用 `invoke<string>('get_data_save_path')` 返回 `Option<String>`
- 返回 None 时，显示空字符串导致"加载中"状态
- 默认 `compression_quality: 80`（80 质量）压缩不够
- 实际测试发现 60-70 质量效果更好

---

## Work Objectives

### Core Objective
修复两个问题：1) 数据目录路径正确显示 2) 降低截图压缩质量以减少存储占用

### Concrete Deliverables
- **数据目录显示修复**: 显示实际路径或默认路径说明
- **截图压缩优化**: 降低默认压缩质量从 80 → 60

### Definition of Done
- [ ] 数据目录显示实际路径或默认路径说明
- [ ] 不再显示"加载中"状态
- [ ] 默认压缩质量改为 60
- [ ] 配置正确保存和加载

### Must Have
- 处理 `Option<String>` 返回值（None 时显示默认说明）
- 保持向后兼容（不破坏现有配置）
- 压缩质量可调（用户仍可在设置中修改）

### Must NOT Have (Guardrails)
- 不破坏现有压缩配置（用户已设置的保持不变）
- 不修改其他配置项

---

## Verification Strategy

### Test Decision
- **Infrastructure exists**: NO
- **Automated tests**: None
- **Framework**: None

### QA Policy
每个任务包含 Agent-Executed QA 场景：
- **数据目录显示**: 验证路径正确显示
- **压缩配置**: 验证默认值已更新

---

## Execution Strategy

### Parallel Execution Waves

```
Wave 1 (Start Immediately — 修复数据目录显示):
└── Task 1: 修复数据目录加载逻辑 [quick]

Wave 2 (After Wave 1 — 修复压缩配置):
└── Task 2: 更新默认压缩质量 [quick]
```

---

## TODOs

- [ ] 1. **修复数据目录加载逻辑**

  **What to do**:
  - 修改 `src/components/SettingsModal.tsx` 中的 `loadDataSavePath()` 函数
  - 处理 `invoke<string>('get_data_save_path')` 返回的 `null` 值
  - 当路径为 null 时，显示默认路径说明：
    - Windows: "C:\Users\xxx\AppData\Roaming\MemFlow（默认）"
    - 或使用 `appDataDir()` API 获取实际默认路径
  - 添加错误处理

  **Must NOT do**:
  - 不修改其他加载逻辑
  - 不破坏现有功能

  **Recommended Agent Profile**:
  - **Category**: `quick`
    - Reason: 简单的逻辑修复
  - **Skills**: `[]`

  - **Parallelization**:
  - **Can Run In Parallel**: NO
  - **Parallel Group**: Sequential (Wave 1)
  - **Blocks**: Task 2
  - **Blocked By**: None

  **References**:
  - `D:/Demo/memflow/src/components/SettingsModal.tsx:500-507` - loadDataSavePath 函数
  - `@tauri-apps/api/path` 模块：用于获取默认 app 数据目录

  **Acceptance Criteria**:

  **QA Scenarios (MANDATORY)**:

  ```
  Scenario: 验证数据目录正确显示
    Tool: Bash (Playwright if UI testing needed)
    Preconditions: 设置界面已打开
    Steps:
      1. 打开应用并进入设置
      2. 切换到"存储管理"标签页
      3. 检查: "数据目录"部分显示路径（不是"加载中"）
      4. 验证: 显示实际路径或默认说明
    Expected Result: 数据目录路径正确显示
    Failure Indicators: 仍显示"加载中"或显示错误
    Evidence: .sisyphus/evidence/task-1-data-display.log
  ```

  **Commit**: NO

- [ ] 2. **更新默认压缩质量**

  **What to do**:
  - 修改 `src-tauri/src/commands.rs` 中的 `default_compression_quality()` 函数
  - 将返回值从 `80` 改为 `60`
  - 保持向后兼容（配置文件中的值不会被覆盖）

  **Must NOT do**:
  - 不修改用户已保存的配置值
  - 不修改配置序列化逻辑

  **Recommended Agent Profile**:
  - **Category**: `quick`
    - Reason: 简单的数值修改
  - **Skills**: `[]`

  - **Parallelization**:
  - **Can Run In Parallel**: NO
  - **Parallel Group**: Sequential (Wave 2)
  - **Blocks**: None
  - **Blocked By**: Task 1

  **References**:
  - `D:/Demo/memflow/src-tauri/src/commands.rs:142-145` - default_compression_quality 函数

  **Acceptance Criteria**:

  **QA Scenarios (MANDATORY)**:

  ```
  Scenario: 验证默认压缩质量已更新
    Tool: Bash
    Preconditions: 代码已更新
    Steps:
      1. 读取: D:/Demo/memflow/src-tauri/src/commands.rs
      2. 验证: default_compression_quality() 返回 60
      3. 验证: 原来的 80 值已移除
    Expected Result: 默认压缩质量已改为 60
    Failure Indicators: 仍返回 80
    Evidence: .sisyphus/evidence/task-2-compression.log
  ```

  **Commit**: NO

---

## Final Verification Wave (MANDATORY — after ALL implementation tasks)

- [ ] F1. **Data Display Verification** — `quick`
  验证数据目录路径正确显示。

- [ ] F2. **Compression Quality Verification** — `quick`
  验证默认压缩质量已更新。

---

## Commit Strategy

- **Final**: `fix: 修复数据目录显示和截图压缩问题`

---

## Success Criteria

### Verification Commands
```bash
# 检查压缩质量默认值
grep -A2 "fn default_compression_quality" D:/Demo/memflow/src-tauri/src/commands.rs

# 检查数据目录加载逻辑
grep -A5 "const loadDataSavePath" D:/Demo/memflow/src/components/SettingsModal.tsx
```

### Final Checklist
- [ ] 数据目录不再显示"加载中"
- [ ] 显示实际路径或默认说明
- [ ] 默认压缩质量改为 60
- [ ] 现有配置兼容性保持
