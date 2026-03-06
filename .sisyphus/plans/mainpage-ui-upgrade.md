# 主页面风格升级 + 目录中文化

## TL;DR

> **Quick Summary**: 将主页面（Timeline 等组件）的风格升级为科技感，与导航栏协调，同时将视图切换目录改为中文
>
> **Deliverables**:
> - 更新 `src/components/Layout.tsx` 视图切换目录为中文
> - 更新 `src/components/Timeline.tsx` 主页面样式
> - 添加科技感装饰元素
>
> **Estimated Effort**: Short
> **Parallel Execution**: NO - 单文件修改为主

---

## Context

### Original Request
用户觉得导航栏很炫酷，但主页面和导航栏不搭，需要调整主页面风格。同时把视图切换目录改为中文。

### Current State
- 导航栏：午夜蓝渐变 + 网格 + 扫描线
- 主页面：简洁风格，缺少科技感元素
- 视图切换：T G R K S Q（英文缩写）

---

## Work Objectives

### Core Objective
将主页面风格升级为科技感，与导航栏协调，同时将视图切换目录改为中文。

### Concrete Deliverables
- 视图切换改为中文：时间轴、图库、回放、知识图谱、统计、问答
- 主页面添加科技感装饰元素（网格、发光、渐变）
- 统一视觉语言

### Must Have
- ✅ 视图切换改为中文
- ✅ 添加科技感装饰
- ✅ 保持功能不变

### Must NOT Have
- ❌ 不改变切换逻辑
- ❌ 不破坏布局

---

## Execution Strategy

### 单步执行
1. 更新视图切换为中文
2. 更新 Timeline 主页面样式
3. 添加科技感装饰

---

## TODOs

- [ ] 1. 更新视图切换为中文

  **What to do**:
  将 `src/components/Layout.tsx` 中的视图切换从英文缩写改为中文：
  - T → 时间轴
  - G → 图库
  - R → 回放
  - K → 知识图谱
  - S → 统计
  - Q → 问答

  **Must NOT do**:
  - 不改变视图 ID
  - 不修改切换逻辑

  **Recommended Agent Profile**:
  > - **Category**: `quick`
    - Reason: 简单文本替换
  - **Skills**: N/A

  **Parallelization**:
  - **Can Run In Parallel**: NO
  - **Parallel Group**: Sequential
  - **Blocks**: 无
  - **Blocked By**: 无

  **References**:
  - `src/components/Layout.tsx:201-207` - 视图定义

  **Acceptance Criteria**:
  - [ ] 视图切换显示中文
  - [ ] 切换功能正常

  **QA Scenarios**:
  ```
  Scenario: 视图切换验证
    Tool: Bash (grep)
    Steps:
      1. grep "时间轴\|图库\|回放\|知识图谱\|统计\|问答" src/components/Layout.tsx
    Expected Result: 输出包含中文标签
    Failure Indicators: 找不到中文
    Evidence: .sisyphus/evidence/task-1-chinese-labels.txt
  ```

  **Commit**: NO

- [ ] 2. 更新 Timeline 主页面样式

  **What to do**:
  在 `src/components/Timeline.tsx` 主页面添加科技感装饰：
  
  **搜索栏区域**:
  - 添加网格背景图案
  - 增强发光效果
  - 添加装饰元素
  
  **活动卡片**:
  - 增强玻璃效果
  - 添加悬停发光动画
  - 添加边框发光

  **Must NOT do**:
  - 不修改搜索和筛选逻辑
  - 不改变数据流

  **Recommended Agent Profile**:
  > - **Category**: `visual-engineering`
    - Reason: UI 样式增强
  - **Skills**: N/A

  **Parallelization**:
  - **Can Run In Parallel**: NO
  - **Parallel Group**: Sequential
  - **Blocks**: 无
  - **Blocked By**: 任务 1

  **References**:
  - `src/components/Timeline.tsx:387-575` - 搜索栏和活动卡片

  **Acceptance Criteria**:
  - [ ] 主页面有科技感装饰
  - [ ] 与导航栏风格协调

  **QA Scenarios**:
  ```
  Scenario: 视觉效果验证
    Tool: Bash (pnpm)
    Steps:
      1. pnpm tauri:dev
    Expected Result: 主页面科技感装饰显示正常
    Failure Indicators: 样式错乱
    Evidence: .sisyphus/evidence/task-2-visual-effects.txt
  ```

  **Commit**: YES (与任务 1 一起提交)

---

## Success Criteria

- [ ] 视图切换显示中文
- [ ] 主页面有科技感装饰
- [ ] 与导航栏风格协调
- [ ] 类型检查通过
