# 修复右上角按钮点击问题

## TL;DR

> **Quick Summary**: 修复右上角按钮点击无响应的问题
>
> **Deliverables**:
> - 更新 Layout.tsx 接口定义，移除 onOpenFeedback
> - 确保回调函数正确传递
>
> **Estimated Effort**: Trivial
> **Parallel Execution**: NO - 单文件修改

---

## Context

### Original Request
用户反馈右上角设置那些功能点击不了

### Problem Analysis
- App.tsx 没有传递 `onOpenFeedback` 给 Layout
- Layout.tsx 的接口定义中还包含 `onOpenFeedback`
- 这导致接口不匹配，可能影响其他回调的传递

### Root Cause
之前的 UI 优化中移除了 `onOpenFeedback`，但没有同步更新 Layout.tsx 的接口定义。

---

## Work Objectives

### Core Objective
修复右上角按钮点击问题，确保所有回调函数正确传递。

### Concrete Deliverables
- 更新 Layout.tsx 接口定义
- 验证按钮点击功能

### Must Have
- ✅ 修复按钮点击功能
- ✅ 保持现有功能不变

### Must NOT Have
- ❌ 不添加新功能

---

## Execution Strategy

### 单步执行
1. 修复 Layout.tsx 接口定义
2. 验证功能

---

## TODOs

- [ ] 1. 修复 Layout.tsx 接口定义

  **What to do**:
  从 `src/components/Layout.tsx` 的 `LayoutProps` 接口中移除 `onOpenFeedback`

  **当前代码**:
  ```tsx
  interface LayoutProps {
    onOpenSettings: () => void
    onOpenChatHistory: () => void
    onOpenPerformance: () => void
    // ...
  }
  ```

  **修复后**:
  ```tsx
  interface LayoutProps {
    onOpenSettings: () => void
    onOpenChatHistory: () => void
    onOpenPerformance: () => void
    // 其他保持不变...
  }
  ```

  **Must NOT do**:
  - 不修改其他接口定义
  - 不修改回调实现

  **Recommended Agent Profile**:
  > - **Category**: `quick`
    - Reason: 简单接口修复
  - **Skills**: N/A

  **Parallelization**:
  - **Can Run In Parallel**: NO
  - **Parallel Group**: Sequential
  - **Blocks**: 无
  - **Blocked By**: 无

  **References**:
  - `src/components/Layout.tsx:40-52` - 接口定义
  - `src/App.tsx:49-52` - 回调传递

  **Acceptance Criteria**:
  - [ ] 接口定义与 App.tsx 匹配
  - [ ] 所有按钮可以正常点击

  **QA Scenarios**:
  ```
  Scenario: 按钮点击测试
    Tool: Bash (pnpm)
    Steps:
      1. pnpm tauri:dev
      2. 点击右上角各个按钮
    Expected Result: 所有按钮可以正常打开对应弹窗
    Failure Indicators: 按钮无响应、控制台错误
    Evidence: .sisyphus/evidence/task-1-button-click.txt
  ```

  **Commit**: YES

---

## Success Criteria

- [ ] 所有按钮可以正常点击
- [ ] 无 TypeScript 错误
- [ ] 功能正常工作
