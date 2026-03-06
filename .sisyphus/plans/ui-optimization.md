# UI 优化计划 - MemFlow Digital Horizon 风格迁移

## TL;DR

> **Quick Summary**: 将 MemFlow 项目的 UI 风格升级至与官网 Digital Horizon 一致，全量迁移所有组件的颜色、边框、毛玻璃效果和微交互动画。
>
> **Deliverables**:
> - 更新的设计 token 系统 (tailwind.config.js + index.css)
> - 全量 UI 组件样式迁移（18+ 组件）
> - 支持减少动效模式 (prefers-reduced-motion)
>
> **Estimated Effort**: Large
> **Parallel Execution**: YES - 4 waves
> **Critical Path**: Token 定义 → 基础样式 → 核心组件 → Modal 组件 → 收尾扫描

---

## Context

### Original Request
优化 MemFlow 项目的 UI，参考官网 (D:\Demo\digital-horizon) 的配色和设计风格，布局自主设计。

### Interview Summary
**Key Discussions**:
- **主强调色**: Neon Cyan (#00f0ff) 替换 Amber (#F59E0B)，全部替换所有 signal/neon-blue 场景
- **次要强调色**: Neon Red (#ff003c) 用于错误、删除、危险操作
- **警告色策略**: 保留 amber 作为独立 warning 色，不与 red 混淆
- **边框风格**: 透明边框 glass 风格 (border-white/10) 替换实线边框
- **视觉效果**: 增强毛玻璃效果 + 微交互动画（不添加粒子背景）
- **组件范围**: 全量包含所有 src/components/** 组件
- **可访问性**: 支持 prefers-reduced-motion

**Research Findings**:
- 当前 tailwind.config.js 中 signal = #F59E0B，neon-blue 是其别名
- index.css 中 .glass 类被禁用 blur，需要恢复
- SettingsModal 是超大组件（1250+ 行），需要分区块处理
- ActivityHeatmap.tsx:113 有硬编码 #00f3ff，需统一为 #00f0ff
- 存在 AgentHistoryModal 等未在原始清单中的组件

### Metis Review
**Identified Gaps** (addressed):
- **组件清单**: 已确认全量包含所有组件，包括遗漏的 AgentHistoryModal
- **硬编码颜色**: 已识别 #00f3ff 硬编码点，将在任务中处理
- **语义色策略**: 已明确 amber/warning 保留，red 用于 danger
- **可访问性**: 已确认支持 prefers-reduced-motion
- **SettingsModal 风险**: 将分区块迁移，避免一次性大改

---

## Work Objectives

### Core Objective
将 MemFlow 项目的 UI 视觉风格全面升级至 Digital Horizon 设计语言，保持所有业务行为和功能不变。

### Concrete Deliverables
- 更新的 `tailwind.config.js` - 新设计 token 定义
- 更新的 `src/index.css` - 恢复并增强 glass 效果，添加动画类
- 全量组件样式迁移 - 所有 .tsx 组件
- 新增 `src/styles/animations.css` - 微交互动画和 reduced-motion 支持

### Definition of Done
- [ ] `pnpm type-check` → PASS (exit code 0)
- [ ] `pnpm lint` → PASS (exit code 0)
- [ ] `pnpm test:unit` → PASS (all tests)
- [ ] 无残留 signal/neon-blue 旧 token（除白名单）
- [ ] 核心组件使用新的 neon-cyan/neon-red token
- [ ] 所有组件应用 glass 风格边框和 backdrop-blur
- [ ] 支持 prefers-reduced-motion 媒体查询

### Must Have
- ✅ Neon Cyan (#00f0ff) 主强调色替换
- ✅ Neon Red (#ff003c) 次要强调色（danger）
- ✅ 保留 Amber (#F59E0B) 作为 warning 语义色
- ✅ Glass 风格透明边框 (border-white/10)
- ✅ 增强毛玻璃效果 (backdrop-blur-md)
- ✅ 微交互动画（transform、transition、group effects）
- ✅ 支持减少动效模式

### Must NOT Have (Guardrails)
- ❌ 粒子背景/Three.js 效果（性能考虑）
- ❌ 业务逻辑/状态流/数据结构变更
- ❌ 文案和交互流程修改
- ❌ 新功能引入
- ❌ 过度动画导致性能下降
- ❌ 语义色混淆（warning/danger/success/info）

---

## Verification Strategy

> **ZERO HUMAN INTERVENTION** — ALL verification is agent-executed.

### Test Decision
- **Infrastructure exists**: NO
- **Automated tests**: None (视觉任务，无新单元测试)
- **Framework**: N/A
- **TDD**: Not applicable

### QA Policy
每个任务包含 Agent-Executed QA Scenarios：
- **Type Checking**: `pnpm type-check` 验证 TypeScript
- **Linting**: `pnpm lint` 验证代码风格
- **Visual Smoke**: 启动 dev server 验证页面加载
- **Token Audit**: grep 扫描验证旧 token 清理

---

## Execution Strategy

### Parallel Execution Waves

```
Wave 1 (Start Immediately — 设计系统基础):
├── Task 1: 更新 tailwind.config.js 设计 token [quick]
├── Task 2: 更新 index.css 恢复 glass 效果 [quick]
├── Task 3: 创建 animations.css 动画系统 [quick]
└── Task 4: 创建类型定义文件 [quick]

Wave 2 (After Wave 1 — 核心布局组件):
├── Task 5: 更新 Layout.tsx 主布局 [visual-engineering]
├── Task 6: 更新 Timeline.tsx 时间轴 [visual-engineering]
├── Task 7: 更新 GalleryView.tsx 图库 [visual-engineering]
└── Task 8: 更新 ContextSidebar.tsx 侧边栏 [visual-engineering]

Wave 3 (After Wave 2 — 视图组件):
├── Task 9: 更新 KnowledgeGraph.tsx 知识图谱 [visual-engineering]
├── Task 10: 更新 FlowState.tsx 统计面板 [visual-engineering]
├── Task 11: 更新 QnA.tsx 对话界面 [visual-engineering]
├── Task 12: 更新 ImmersiveReplay.tsx 沉浸回放 [visual-engineering]
└── Task 13: 更新 ActivityHeatmap.tsx 热力图 [visual-engineering]

Wave 4 (After Wave 3 — Modal 组件，分批并行):
├── Task 14: SettingsModal.tsx Header & Nav 区块 [visual-engineering]
├── Task 15: SettingsModal.tsx 表单控件区块 [visual-engineering]
├── Task 16: SettingsModal.tsx Privacy 区块 [visual-engineering]
├── Task 17: SettingsModal.tsx Storage 区块 [visual-engineering]
├── Task 18: 其他 Modal 组件批量更新 [visual-engineering]
└── Task 19: 小型组件批量更新 [visual-engineering]

Wave FINAL (After ALL tasks — 验收与清理):
├── Task F1: Token 残留扫描 [quick]
├── Task F2: 类型检查和 Lint 验证 [quick]
├── Task F3: Dev Server Smoke 测试 [quick]
└── Task F4: 构建验证 [quick]

Critical Path: T1 → T2 → T3 → T5 → T9 → T14-17 → F1-F4
Parallel Speedup: ~75% faster than sequential
Max Concurrent: 4 (Waves 2-4)
```

---

## TODOs

- [x] 1. 更新 tailwind.config.js 设计 token

  **What to do**:
  - 在 `colors` 对象中添加新颜色定义：
    - `neon-cyan: '#00f0ff'` (主强调色)
    - `neon-red: '#ff003c'` (次要强调色/danger)
  - 更新 `signal` 别名指向 `neon-cyan`（保持向后兼容）
  - 更新 `neon-blue` 别名指向 `neon-cyan`
  - 保持 `amber: '#F59E0B'` 用于 warning 语义
  - 保持 `neon-red: '#EF4444'` 的现有映射，可考虑重命名

  **Must NOT do**:
  - 不删除现有的 token 定义（保持向后兼容）
  - 不修改非颜色相关的配置

  **Recommended Agent Profile**:
  > - **Category**: `quick`
    - Reason: 简单的配置文件更新，单文件修改
  - **Skills**: N/A

  **Parallelization**:
  - **Can Run In Parallel**: NO
  - **Parallel Group**: Wave 1 (Sequential, must complete first)
  - **Blocks**: Tasks 2, 3, 4, 所有组件迁移任务
  - **Blocked By**: None

  **References**:
  - `tailwind.config.js:9-23` - 当前颜色定义位置
  - `D:\Demo\digital-horizon\src\style.css:3-9` - 官网颜色定义参考

  **Acceptance Criteria**:
  - [ ] 文件包含 `neon-cyan: '#00f0ff'` 定义
  - [ ] 文件包含 `neon-red: '#ff003c'` 定义
  - [ ] `signal` 别名指向 `neon-cyan`
  - [ ] `amber: '#F59E0B'` 保持不变

  **QA Scenarios**:
  ```
  Scenario: Token 定义验证
    Tool: Bash (grep)
    Preconditions: tailwind.config.js 已更新
    Steps:
      1. grep -E "neon-cyan|neon-red|signal|amber" tailwind.config.js
    Expected Result: 输出包含所有新 token 定义
    Failure Indicators: 找不到 neon-cyan 或 neon-red 定义
    Evidence: .sisyphus/evidence/task-1-tokens.txt

  Scenario: 语法验证
    Tool: Bash (pnpm)
    Steps:
      1. cd D:\Demo\memflow && pnpm type-check
    Expected Result: Exit code 0, 无 TypeScript 错误
    Failure Indicators: Exit code 非 0
    Evidence: .sisyphus/evidence/task-1-typecheck.txt
  ```

  **Commit**: NO (与其他 Wave 1 任务一起提交)

- [x] 2. 更新 src/index.css 恢复 glass 效果

  **What to do**:
  - 恢复 `.glass` 类的 `backdrop-blur-md` 效果
  - 更新 `.glass` 类为 `bg-white/5 backdrop-blur-md border border-white/10`
  - 恢复 `.neon-glow` 类的阴影效果（可选，添加 `shadow-[0_0_15px_rgba(0,240,255,0.3)]`）
  - 添加 `@layer utilities` 下的新 glass 变体类：
    - `.glass-strong` - 更强模糊效果
    - `.glass-subtle` - 更弱模糊效果

  **Must NOT do**:
  - 不删除现有的 `.elucid-panel` 和 `.elucid-btn` 类（保持兼容）
  - 不修改 scrollbar 样式（除非必要）

  **Recommended Agent Profile**:
  > - **Category**: `quick`
    - Reason: CSS 文件更新，简单样式修改
  - **Skills**: N/A

  **Parallelization**:
  - **Can Run In Parallel**: YES
  - **Parallel Group**: Wave 1 (with Tasks 3, 4)
  - **Blocks**: 所有组件迁移任务
  - **Blocked By**: Task 1 (token 定义依赖)

  **References**:
  - `src/index.css:40-67` - 当前 utilities 定义位置
  - `D:\Demo\digital-horizon\src\style.css:17-29` - 官网 glass 效果参考

  **Acceptance Criteria**:
  - [ ] `.glass` 类包含 `backdrop-blur-md`
  - [ ] `.glass` 类使用 `border-white/10`
  - [ ] 新增 `.glass-strong` 和 `.glass-subtle` 类

  **QA Scenarios**:
  ```
  Scenario: CSS 类验证
    Tool: Bash (grep)
    Steps:
      1. grep -E "\.glass|backdrop-blur|border-white" src/index.css
    Expected Result: 输出包含更新的 glass 类定义
    Failure Indicators: 找不到 backdrop-blur-md 或 border-white/10
    Evidence: .sisyphus/evidence/task-2-glass.txt
  ```

  **Commit**: NO

- [ ] 3. 创建 src/styles/animations.css 动画系统

  **What to do**:
  - 创建新文件 `src/styles/animations.css`
  - 添加基础动画类：
    - `.animate-in` - 入场动画
    - `.animate-out` - 退场动画
    - `.hover-lift` - 悬停提升效果
    - `.hover-glow` - 悬停发光效果
  - 添加 `@media (prefers-reduced-motion: reduce)` 禁用动画
  - 在 `src/main.tsx` 中导入此文件

  **Must NOT do**:
  - 不添加粒子背景或复杂 WebGL 效果
  - 不创建性能敏感型动画（如大面积 blur 变化）

  **Recommended Agent Profile**:
  > - **Category**: `quick`
    - Reason: 新建 CSS 文件，添加动画类
  - **Skills**: N/A

  **Parallelization**:
  - **Can Run In Parallel**: YES
  - **Parallel Group**: Wave 1 (with Tasks 2, 4)
  - **Blocks**: 所有需要动画的组件任务
  - **Blocked By**: Task 1 (token 定义依赖)

  **References**:
  - `src/main.tsx` - 需要导入新 CSS 文件的位置
  - `D:\Demo\digital-horizon\src\components\Navbar.vue:142-151` - 官网动画参考

  **Acceptance Criteria**:
  - [ ] `src/styles/animations.css` 文件存在
  - [ ] 包含 `prefers-reduced-motion` 媒体查询
  - [ ] `src/main.tsx` 导入此文件

  **QA Scenarios**:
  ```
  Scenario: 文件存在验证
    Tool: Bash
    Steps:
      1. test -f src/styles/animations.css && echo "EXISTS"
    Expected Result: 输出 EXISTS
    Failure Indicators: 文件不存在
    Evidence: .sisyphus/evidence/task-3-file-exists.txt

  Scenario: 导入验证
    Tool: Bash (grep)
    Steps:
      1. grep "animations.css" src/main.tsx
    Expected Result: 找到导入语句
    Failure Indicators: 找不到导入
    Evidence: .sisyphus/evidence/task-3-import.txt
  ```

  **Commit**: NO

- [ ] 4. 更新 src/types/design.ts 类型定义

  **What to do**:
  - 创建或更新 `src/types/design.ts` 文件
  - 导出设计 token 类型：
    - `DesignToken` 类型包含所有颜色 token
    - `AnimationDuration` 类型
    - `SpacingScale` 类型
  - 添加 Tailwind 主题类型扩展（如果需要）

  **Must NOT do**:
  - 不修改业务逻辑类型
  - 不添加运行时设计系统代码

  **Recommended Agent Profile**:
  > - **Category**: `quick`
    - Reason: 类型定义文件，简单 TypeScript
  - **Skills**: N/A

  **Parallelization**:
  - **Can Run In Parallel**: YES
  - **Parallel Group**: Wave 1 (with Tasks 2, 3)
  - **Blocks**: 后续组件类型检查
  - **Blocked By**: Task 1 (token 定义依赖)

  **References**:
  - `src/types/` - 类型定义目录
  - `tailwind.config.js` - Token 定义参考

  **Acceptance Criteria**:
  - [ ] `src/types/design.ts` 文件存在
  - [ ] 导出 `DesignToken` 类型

  **QA Scenarios**:
  ```
  Scenario: 类型检查
    Tool: Bash (pnpm)
    Steps:
      1. pnpm type-check
    Expected Result: Exit code 0
    Failure Indicators: 类型错误
    Evidence: .sisyphus/evidence/task-4-typecheck.txt
  ```

  **Commit**: YES (with message "design: 更新设计 token 系统和动画基础")

- [ ] 2. 更新 src/index.css 恢复 glass 效果

  **What to do**:
  - 恢复 `.glass` 类的 `backdrop-blur-md` 效果
  - 更新 `.glass` 类为 `bg-white/5 backdrop-blur-md border border-white/10`
  - 恢复 `.neon-glow` 类的阴影效果（可选，添加 `shadow-[0_0_15px_rgba(0,240,255,0.3)]`）
  - 添加 `@layer utilities` 下的新 glass 变体类：
    - `.glass-strong` - 更强模糊效果
    - `.glass-subtle` - 更弱模糊效果

  **Must NOT do**:
  - 不删除现有的 `.elucid-panel` 和 `.elucid-btn` 类（保持兼容）
  - 不修改 scrollbar 样式（除非必要）

  **Recommended Agent Profile**:
  > - **Category**: `quick`
    - Reason: CSS 文件更新，简单样式修改
  - **Skills**: N/A

  **Parallelization**:
  - **Can Run In Parallel**: YES
  - **Parallel Group**: Wave 1 (with Tasks 3, 4)
  - **Blocks**: 所有组件迁移任务
  - **Blocked By**: Task 1 (token 定义依赖)

  **References**:
  - `src/index.css:40-67` - 当前 utilities 定义位置
  - `D:\Demo\digital-horizon\src\style.css:17-29` - 官网 glass 效果参考

  **Acceptance Criteria**:
  - [ ] `.glass` 类包含 `backdrop-blur-md`
  - [ ] `.glass` 类使用 `border-white/10`
  - [ ] 新增 `.glass-strong` 和 `.glass-subtle` 类

  **QA Scenarios**:
  ```
  Scenario: CSS 类验证
    Tool: Bash (grep)
    Steps:
      1. grep -E "\.glass|backdrop-blur|border-white" src/index.css
    Expected Result: 输出包含更新的 glass 类定义
    Failure Indicators: 找不到 backdrop-blur-md 或 border-white/10
    Evidence: .sisyphus/evidence/task-2-glass.txt
  ```

  **Commit**: NO

- [ ] 3. 创建 src/styles/animations.css 动画系统

  **What to do**:
  - 创建新文件 `src/styles/animations.css`
  - 添加基础动画类：
    - `.animate-in` - 入场动画
    - `.animate-out` - 退场动画
    - `.hover-lift` - 悬停提升效果
    - `.hover-glow` - 悬停发光效果
  - 添加 `@media (prefers-reduced-motion: reduce)` 禁用动画
  - 在 `src/main.tsx` 中导入此文件

  **Must NOT do**:
  - 不添加粒子背景或复杂 WebGL 效果
  - 不创建性能敏感型动画（如大面积 blur 变化）

  **Recommended Agent Profile**:
  > - **Category**: `quick`
    - Reason: 新建 CSS 文件，添加动画类
  - **Skills**: N/A

  **Parallelization**:
  - **Can Run In Parallel**: YES
  - **Parallel Group**: Wave 1 (with Tasks 2, 4)
  - **Blocks**: 所有需要动画的组件任务
  - **Blocked By**: Task 1 (token 定义依赖)

  **References**:
  - `src/main.tsx` - 需要导入新 CSS 文件的位置
  - `D:\Demo\digital-horizon\src\components\Navbar.vue:142-151` - 官网动画参考

  **Acceptance Criteria**:
  - [ ] `src/styles/animations.css` 文件存在
  - [ ] 包含 `prefers-reduced-motion` 媒体查询
  - [ ] `src/main.tsx` 导入此文件

  **QA Scenarios**:
  ```
  Scenario: 文件存在验证
    Tool: Bash
    Steps:
      1. test -f src/styles/animations.css && echo "EXISTS"
    Expected Result: 输出 EXISTS
    Failure Indicators: 文件不存在
    Evidence: .sisyphus/evidence/task-3-file-exists.txt

  Scenario: 导入验证
    Tool: Bash (grep)
    Steps:
      1. grep "animations.css" src/main.tsx
    Expected Result: 找到导入语句
    Failure Indicators: 找不到导入
    Evidence: .sisyphus/evidence/task-3-import.txt
  ```

  **Commit**: NO

- [ ] 4. 更新 src/types/design.ts 类型定义

  **What to do**:
  - 创建或更新 `src/types/design.ts` 文件
  - 导出设计 token 类型：
    - `DesignToken` 类型包含所有颜色 token
    - `AnimationDuration` 类型
    - `SpacingScale` 类型
  - 添加 Tailwind 主题类型扩展（如果需要）

  **Must NOT do**:
  - 不修改业务逻辑类型
  - 不添加运行时设计系统代码

  **Recommended Agent Profile**:
  > - **Category**: `quick`
    - Reason: 类型定义文件，简单 TypeScript
  - **Skills**: N/A

  **Parallelization**:
  - **Can Run In Parallel**: YES
  - **Parallel Group**: Wave 1 (with Tasks 2, 3)
  - **Blocks**: 后续组件类型检查
  - **Blocked By**: Task 1 (token 定义依赖)

  **References**:
  - `src/types/` - 类型定义目录
  - `tailwind.config.js` - Token 定义参考

  **Acceptance Criteria**:
  - [ ] `src/types/design.ts` 文件存在
  - [ ] 导出 `DesignToken` 类型

  **QA Scenarios**:
  ```
  Scenario: 类型检查
    Tool: Bash (pnpm)
    Steps:
      1. pnpm type-check
    Expected Result: Exit code 0
    Failure Indicators: 类型错误
    Evidence: .sisyphus/evidence/task-4-typecheck.txt
  ```

  **Commit**: YES (with message "design: 更新设计 token 系统和动画基础")

- [ ] 5. 更新 Layout.tsx 主布局组件

  **What to do**:
  - 替换 `text-signal` → `text-neon-cyan`
  - 替换 `bg-signal/*` → `bg-neon-cyan/*`
  - 替换 `border-signal/*` → `border-neon-cyan/*`
  - 替换 `border-zinc-800` → `border-white/10`
  - 更新录制按钮样式：添加 hover 发光效果 `shadow-[0_0_12px_rgba(0,240,255,0.5)]`
  - 更新视图切换胶囊样式，使用 neon-cyan
  - 更新图标按钮的 hover 效果
  - 添加微交互动画（group hover, transition）

  **Must NOT do**:
  - 不修改布局结构和组件层级
  - 不更改事件处理函数
  - 不修改状态管理逻辑

  **Recommended Agent Profile**:
  > - **Category**: `visual-engineering`
    - Reason: UI 样式更新，需要细致的视觉调整
  - **Skills**: N/A

  **Parallelization**:
  - **Can Run In Parallel**: YES
  - **Parallel Group**: Wave 2 (with Tasks 6, 7, 8)
  - **Blocks**: 无
  - **Blocked By**: Wave 1 (Tasks 1-4)

  **References**:
  - `src/components/Layout.tsx:88-160` - 头部工具栏和视图切换
  - `D:\Demo\digital-horizon\src\components\Navbar.vue:2-88` - 官网 Navbar 参考

  **Acceptance Criteria**:
  - [ ] 无 `text-signal` 或 `bg-signal` 残留
  - [ ] 使用 `border-white/10` 替代实线边框
  - [ ] 录制按钮有 hover 发光效果
  - [ ] 视图切换胶囊使用 neon-cyan 激活色

  **QA Scenarios**:
  ```
  Scenario: Token 替换验证
    Tool: Bash (grep)
    Steps:
      1. grep -E "text-signal|bg-signal|border-signal" src/components/Layout.tsx | wc -l
    Expected Result: 输出 0
    Failure Indicators: 输出 > 0
    Evidence: .sisyphus/evidence/task-5-tokens.txt

  Scenario: 类型检查
    Tool: Bash (pnpm)
    Steps:
      1. pnpm type-check
    Expected Result: Exit code 0
    Failure Indicators: TypeScript 错误
    Evidence: .sisyphus/evidence/task-5-typecheck.txt
  ```

  **Commit**: NO (与其他 Wave 2 任务一起提交)

- [ ] 6. 更新 Timeline.tsx 时间轴组件

  **What to do**:
  - 替换所有 `text-signal` → `text-neon-cyan`
  - 替换 `bg-signal/*` → `bg-neon-cyan/*`
  - 替换活动卡片的 `border-zinc-800` → `border-white/10`
  - 更新搜索栏样式：使用 glass 效果 + neon-cyan focus ring
  - 更新筛选按钮样式
  - 更新活动卡片 hover 效果：添加边框发光 `hover:shadow-[0_0_15px_rgba(0,240,255,0.3)]`
  - 更新智能搜索按钮样式
  - 为加载动画添加 neon-cyan 颜色

  **Must NOT do**:
  - 不修改虚拟列表逻辑
  - 不更改搜索和筛选功能
  - 不修改日期格式化逻辑

  **Recommended Agent Profile**:
  > - **Category**: `visual-engineering`
    - Reason: 复杂列表组件 UI 更新
  - **Skills**: N/A

  **Parallelization**:
  - **Can Run In Parallel**: YES
  - **Parallel Group**: Wave 2 (with Tasks 5, 7, 8)
  - **Blocks**: 无
  - **Blocked By**: Wave 1 (Tasks 1-4)

  **References**:
  - `src/components/Timeline.tsx:387-575` - 搜索栏和活动卡片区域
  - `src/components/Timeline.tsx:619-659` - ScreenshotImage 组件

  **Acceptance Criteria**:
  - [ ] 搜索栏使用 glass 效果
  - [ ] 活动卡片使用 `border-white/10`
  - [ ] hover 时有 neon-cyan 发光效果
  - [ ] 无 signal 残留

  **QA Scenarios**:
  ```
  Scenario: Token 替换验证
    Tool: Bash (grep)
    Steps:
      1. grep -c "text-signal\|bg-signal\|border-signal" src/components/Timeline.tsx || echo "0"
    Expected Result: 输出 0
    Failure Indicators: 输出 > 0
    Evidence: .sisyphus/evidence/task-6-tokens.txt
  ```

  **Commit**: NO

- [ ] 7. 更新 GalleryView.tsx 图库组件

  **What to do**:
  - 替换 `text-neon-blue` → `text-neon-cyan`
  - 替换 `bg-neon-blue/*` → `bg-neon-cyan/*`
  - 替换 `border-neon-blue/*` → `border-neon-cyan/*`
  - 替换 `border-glass-border` → `border-white/10`
  - 更新网格项 hover 效果：保持现有 `hover:shadow-[0_0_15px_rgba(0,243,255,0.3)]` 并更新为 `rgba(0,240,255,0.3)`
  - 更新侧边栏选中项样式
  - 为 OCR 标签保持 neon-green（保留成功语义色）
  - 添加微交互动画（卡片 hover 缩放）

  **Must NOT do**:
  - 不修改网格布局逻辑
  - 不更改应用筛选功能
  - 不修改图片加载逻辑

  **Recommended Agent Profile**:
  > - **Category**: `visual-engineering`
    - Reason: 网格布局 UI 更新
  - **Skills**: N/A

  **Parallelization**:
  - **Can Run In Parallel**: YES
  - **Parallel Group**: Wave 2 (with Tasks 5, 6, 8)
  - **Blocks**: 无
  - **Blocked By**: Wave 1 (Tasks 1-4)

  **References**:
  - `src/components/GalleryView.tsx:33-87` - 侧边栏和主布局
  - `src/components/GalleryView.tsx:164-243` - GalleryItem 组件

  **Acceptance Criteria**:
  - [ ] 网格项 hover 发光颜色更新为 neon-cyan
  - [ ] 侧边栏选中项使用 neon-cyan
  - [ ] OCR 标签保持 neon-green

  **QA Scenarios**:
  ```
  Scenario: 颜色验证
    Tool: Bash (grep)
    Steps:
      1. grep -c "neon-blue" src/components/GalleryView.tsx || echo "0"
    Expected Result: 输出 0
    Failure Indicators: 输出 > 0
    Evidence: .sisyphus/evidence/task-7-neon-blue.txt

  Scenario: 新 Token 验证
    Tool: Bash (grep)
    Steps:
      1. grep -c "neon-cyan" src/components/GalleryView.tsx || echo "0"
    Expected Result: 输出 > 5
    Failure Indicators: 输出 <= 5
    Evidence: .sisyphus/evidence/task-7-neon-cyan.txt
  ```

  **Commit**: NO

- [ ] 8. 更新 ContextSidebar.tsx 上下文侧边栏

  **What to do**:
  - 替换 `text-signal` → `text-neon-cyan`
  - 替换 `bg-signal/*` → `bg-neon-cyan/*`
  - 替换 `border-signal/*` → `border-neon-cyan/*`
  - 更新状态指示灯：使用 neon-cyan 发光 `shadow-[0_0_8px_rgba(0,240,255,0.5)]`
  - 更新展开动画和 hover 效果
  - 更新建议操作按钮样式
  - 更新相关记忆卡片样式

  **Must NOT do**:
  - 不修改展开/收起逻辑
  - 不更改事件监听器
  - 不修改 Tauri invoke 调用

  **Recommended Agent Profile**:
  > - **Category**: `visual-engineering`
    - Reason: 侧边栏交互 UI 更新
  - **Skills**: N/A

  **Parallelization**:
  - **Can Run In Parallel**: YES
  - **Parallel Group**: Wave 2 (with Tasks 5, 6, 7)
  - **Blocks**: 无
  - **Blocked By**: Wave 1 (Tasks 1-4)

  **References**:
  - `src/components/ContextSidebar.tsx:138-271` - 完整组件

  **Acceptance Criteria**:
  - [ ] 状态指示灯使用 neon-cyan
  - [ ] 展开/收起动画流畅
  - [ ] 无 signal 残留

  **QA Scenarios**:
  ```
  Scenario: Signal 清理验证
    Tool: Bash (grep)
    Steps:
      1. grep -c "text-signal\|bg-signal\|border-signal" src/components/ContextSidebar.tsx || echo "0"
    Expected Result: 输出 0
    Failure Indicators: 输出 > 0
    Evidence: .sisyphus/evidence/task-8-signal.txt
  ```

  **Commit**: YES (with message "style(core): 更新核心布局组件 UI 风格")

- [ ] 9. 更新 KnowledgeGraph.tsx 知识图谱组件

  **What to do**:
  - 替换 `text-neon-purple` → `text-neon-cyan`（主视觉色统一）
  - 替换 `bg-neon-purple/*` → `bg-neon-cyan/*`
  - 替换 `border-neon-purple/*` → `border-neon-cyan/*`
  - 保持图谱节点颜色映射（zinc 灰度系用于节点层级）
  - 更新工具栏按钮样式
  - 更新加载动画颜色
  - 更新重建按钮样式

  **Must NOT do**:
  - 不修改力导向图布局逻辑
  - 不更改图谱数据结构
  - 不修改 D3 力配置

  **Recommended Agent Profile**:
  > - **Category**: `visual-engineering`
    - Reason: 图谱可视化组件 UI 更新
  - **Skills**: N/A

  **Parallelization**:
  - **Can Run In Parallel**: YES
  - **Parallel Group**: Wave 3 (with Tasks 10, 11, 12, 13)
  - **Blocks**: 无
  - **Blocked By**: Wave 1 (Tasks 1-4)

  **References**:
  - `src/components/KnowledgeGraph.tsx:225-263` - 工具栏和控制按钮

  **Acceptance Criteria**:
  - [ ] 主视觉色使用 neon-cyan
  - [ ] 图谱节点保持 zinc 灰度系
  - [ ] 无 neon-purple 残留

  **QA Scenarios**:
  ```
  Scenario: Neon-purple 清理
    Tool: Bash (grep)
    Steps:
      1. grep -c "neon-purple" src/components/KnowledgeGraph.tsx || echo "0"
    Expected Result: 输出 0
    Failure Indicators: 输出 > 0
    Evidence: .sisyphus/evidence/task-9-neon-purple.txt
  ```

  **Commit**: NO

- [ ] 10. 更新 FlowState.tsx 统计面板组件

  **What to do**:
  - 替换 `text-neon-blue` → `text-neon-cyan`
  - 替换 `bg-neon-blue/*` → `bg-neon-cyan/*`
  - 替换 `text-neon-purple` → 保持（用于区分数据系列）
  - 替换 `text-neon-green` → 保持（成功语义）
  - 替换 `border-glass-border` → `border-white/10`
  - 更新统计卡片样式
  - 更新图表颜色（Recharts 配置）
  - 保持 COLORS 数组中的多样性（用于数据系列区分）

  **Must NOT do**:
  - 不修改数据获取逻辑
  - 不更改图表类型和配置
  - 不修改统计计算

  **Recommended Agent Profile**:
  > - **Category**: `visual-engineering`
    - Reason: 图表密集组件 UI 更新
  - **Skills**: N/A

  **Parallelization**:
  - **Can Run In Parallel**: YES
  - **Parallel Group**: Wave 3 (with Tasks 9, 11, 12, 13)
  - **Blocks**: 无
  - **Blocked By**: Wave 1 (Tasks 1-4)

  **References**:
  - `src/components/FlowState.tsx:110` - COLORS 数组定义
  - `src/components/FlowState.tsx:125-173` - 统计卡片

  **Acceptance Criteria**:
  - [ ] 主色调使用 neon-cyan
  - [ ] 图表数据系列保持多样性颜色
  - [ ] neon-blue 已清理

  **QA Scenarios**:
  ```
  Scenario: Neon-blue 清理
    Tool: Bash (grep)
    Steps:
      1. grep -c "text-neon-blue\|bg-neon-blue" src/components/FlowState.tsx || echo "0"
    Expected Result: 输出 0
    Failure Indicators: 输出 > 0
    Evidence: .sisyphus/evidence/task-10-neon-blue.txt
  ```

  **Commit**: NO

- [ ] 11. 更新 QnA.tsx 对话界面组件

  **What to do**:
  - 替换 `text-neon-blue` → `text-neon-cyan`
  - 替换 `bg-neon-blue/*` → `bg-neon-cyan/*`
  - 替换 `border-neon-blue/*` → `border-neon-cyan/*`
  - 替换 `ring-neon-blue/*` → `ring-neon-cyan/*`
  - 更新用户消息气泡：使用 neon-cyan 边框和背景
  - 更新助手消息气泡：保持对比样式
  - 更新发送按钮样式
  - 更新输入框 focus ring
  - 更新加载动画颜色

  **Must NOT do**:
  - 不修改消息流逻辑
  - 不更改流式监听器
  - 不修改 Tauri 事件处理

  **Recommended Agent Profile**:
  > - **Category**: `visual-engineering`
    - Reason: 对话 UI 组件更新
  - **Skills**: N/A

  **Parallelization**:
  - **Can Run In Parallel**: YES
  - **Parallel Group**: Wave 3 (with Tasks 9, 10, 12, 13)
  - **Blocks**: 无
  - **Blocked By**: Wave 1 (Tasks 1-4)

  **References**:
  - `src/components/QnA.tsx:258-358` - 消息列表和输入区域

  **Acceptance Criteria**:
  - [ ] 用户气泡使用 neon-cyan
  - [ ] 发送按钮使用 neon-cyan
  - [ ] focus ring 使用 neon-cyan
  - [ ] 无 neon-blue 残留

  **QA Scenarios**:
  ```
  Scenario: Neon-blue 清理
    Tool: Bash (grep)
    Steps:
      1. grep -c "neon-blue" src/components/QnA.tsx || echo "0"
    Expected Result: 输出 0
    Failure Indicators: 输出 > 0
    Evidence: .sisyphus/evidence/task-11-neon-blue.txt
  ```

  **Commit**: NO

- [ ] 12. 更新 ImmersiveReplay.tsx 沉浸回放组件

  **What to do**:
  - 替换 `text-neon-blue` → `text-neon-cyan`
  - 替换 `bg-neon-blue/*` → `bg-neon-cyan/*`
  - 替换 `border-neon-blue/*` → `border-neon-cyan/*`
  - 替换 `ring-neon-blue/*` → `ring-neon-cyan/*`
  - 更新控制栏样式
  - 更新播放控制按钮
  - 更新时间轴指示器
  - 更新加载动画颜色
  - 增强 glass 效果

  **Must NOT do**:
  - 不修改回放逻辑
  - 不更改时间轴控制
  - 不修改图片加载

  **Recommended Agent Profile**:
  > - **Category**: `visual-engineering`
    - Reason: 媒体播放器 UI 更新
  - **Skills**: N/A

  **Parallelization**:
  - **Can Run In Parallel**: YES
  - **Parallel Group**: Wave 3 (with Tasks 9, 10, 11, 13)
  - **Blocks**: 无
  - **Blocked By**: Wave 1 (Tasks 1-4)

  **References**:
  - `src/components/ImmersiveReplay.tsx:234-391` - 控制栏和时间轴

  **Acceptance Criteria**:
  - [ ] 控制按钮使用 neon-cyan
  - [ ] 时间轴指示器使用 neon-cyan
  - [ ] 无 neon-blue 残留

  **QA Scenarios**:
  ```
  Scenario: Neon-blue 清理
    Tool: Bash (grep)
    Steps:
      1. grep -c "neon-blue" src/components/ImmersiveReplay.tsx || echo "0"
    Expected Result: 输出 0
    Failure Indicators: 输出 > 0
    Evidence: .sisyphus/evidence/task-12-neon-blue.txt
  ```

  **Commit**: NO

- [ ] 13. 更新 ActivityHeatmap.tsx 活动热力图组件

  **What to do**:
  - 替换 `text-neon-blue` → `text-neon-cyan`
  - 替换 `#00f3ff` 硬编码 → `#00f0ff` 或 neon-cyan token
  - 替换 `border-glass-border` → `border-white/10`
  - 更新热力图颜色配置
  - 更新容器样式

  **Must NOT do**:
  - 不修改热力图生成逻辑
  - 不更改日历组件

  **Recommended Agent Profile**:
  > - **Category**: `visual-engineering`
    - Reason: 热力图组件 UI 更新
  - **Skills**: N/A

  **Parallelization**:
  - **Can Run In Parallel**: YES
  - **Parallel Group**: Wave 3 (with Tasks 9, 10, 11, 12)
  - **Blocks**: 无
  - **Blocked By**: Wave 1 (Tasks 1-4)

  **References**:
  - `src/components/ActivityHeatmap.tsx:53-113` - 标题和热力图配置

  **Acceptance Criteria**:
  - [ ] 无 #00f3ff 硬编码
  - [ ] 使用 neon-cyan token
  - [ ] 无 neon-blue 残留

  **QA Scenarios**:
  ```
  Scenario: 硬编码颜色清理
    Tool: Bash (grep)
    Steps:
      1. grep "#00f3ff" src/components/ActivityHeatmap.tsx | wc -l
    Expected Result: 输出 0
    Failure Indicators: 输出 > 0
    Evidence: .sisyphus/evidence/task-13-hardcode.txt
  ```

  **Commit**: YES (with message "style(views): 更新视图组件 UI 风格")

- [ ] 14. 更新 SettingsModal.tsx - Header & Nav 区块

  **What to do**:
  - 更新 Header 样式：
    - 替换 `text-signal` → `text-neon-cyan`
    - 替换 `bg-signal/*` → `bg-neon-cyan/*`
    - 更新装饰渐变 `from-signal/10` → `from-neon-cyan/10`
  - 更新 Sidebar Tab 按钮：
    - 激活状态使用 `bg-neon-cyan/20 text-neon-cyan`
    - Hover 效果增强
  - 更新关闭按钮 hover 样式
  - 添加微交互动画

  **Must NOT do**:
  - 不修改 Tab 切换逻辑
  - 不更改模态框显示/隐藏状态
  - 不修改第 1-100 行之外的内容

  **Recommended Agent Profile**:
  > - **Category**: `visual-engineering`
    - Reason: SettingsModal 第一批区块更新
  - **Skills**: N/A

  **Parallelization**:
  - **Can Run In Parallel**: YES
  - **Parallel Group**: Wave 4 (with Tasks 15, 16, 17, 18, 19)
  - **Blocks**: 无
  - **Blocked By**: Wave 1 (Tasks 1-4)

  **References**:
  - `src/components/SettingsModal.tsx:870-934` - Header 和 Sidebar

  **Acceptance Criteria**:
  - [ ] Header 装饰使用 neon-cyan
  - [ ] Tab 激活状态使用 neon-cyan
  - [ ] 第 1-934 行无 signal 残留

  **QA Scenarios**:
  ```
  Scenario: Header 区域 Signal 清理
    Tool: Bash (grep)
    Steps:
      1. sed -n '1,934p' src/components/SettingsModal.tsx | grep -c "text-signal\|bg-signal" || echo "0"
    Expected Result: 输出 0
    Failure Indicators: 输出 > 0
    Evidence: .sisyphus/evidence/task-14-header-signal.txt
  ```

  **Commit**: NO

- [ ] 15. 更新 SettingsModal.tsx - General/AI 区块

  **What to do**:
  - 更新 AI 能力开关：
    - 激活状态 `bg-neon-cyan`（而非 bg-neon-blue）
    - 图标容器 `bg-neon-cyan/20 text-neon-cyan`
  - 更新上下文助理开关
  - 更新自启动开关
  - 更新录制设置区块：
    - 滑块 accent 使用 neon-cyan
    - 数值高亮使用 neon-cyan
  - 保持 amber warning 色（如果存在）

  **Must NOT do**:
  - 不修改开关状态逻辑
  - 不更改表单验证

  **Recommended Agent Profile**:
  > - **Category**: `visual-engineering`
    - Reason: SettingsModal 表单控件区块
  - **Skills**: N/A

  **Parallelization**:
  - **Can Run In Parallel**: YES
  - **Parallel Group**: Wave 4 (with Tasks 14, 16, 17, 18, 19)
  - **Blocks**: 无
  - **Blocked By**: Wave 1 (Tasks 1-4)

  **References**:
  - `src/components/SettingsModal.tsx:938-1103` - General 设置区块

  **Acceptance Criteria**:
  - [ ] 所有开关激活态使用 neon-cyan
  - [ ] 滑块 accent 使用 neon-cyan
  - [ ] 第 938-1103 行无 neon-blue 残留

  **QA Scenarios**:
  ```
  Scenario: General 区块颜色验证
    Tool: Bash (grep)
    Steps:
      1. sed -n '938,1103p' src/components/SettingsModal.tsx | grep -c "neon-blue" || echo "0"
    Expected Result: 输出 0
    Failure Indicators: 输出 > 0
    Evidence: .sisyphus/evidence/task-15-neon-blue.txt
  ```

  **Commit**: NO

- [ ] 16. 更新 SettingsModal.tsx - Chat Model 区块

  **What to do**:
  - 更新模型选择下拉框：
    - Focus ring 使用 neon-cyan
    - Hover border 使用 neon-cyan
  - 更新 API Key 输入框：
    - Focus ring 使用 neon-cyan
    - Hover border 使用 neon-cyan
  - 更新保存按钮：
    - 成功状态保持绿色
    - 删除按钮使用 neon-red
  - 更新连接测试按钮

  **Must NOT do**:
  - 不修改表单提交逻辑
  - 不更改 API Key 验证

  **Recommended Agent Profile**:
  > - **Category**: `visual-engineering`
    - Reason: SettingsModal 模型配置区块
  - **Skills**: N/A

  **Parallelization**:
  - **Can Run In Parallel**: YES
  - **Parallel Group**: Wave 4 (with Tasks 14, 15, 17, 18, 19)
  - **Blocks**: 无
  - **Blocked By**: Wave 1 (Tasks 1-4)

  **References**:
  - `src/components/SettingsModal.tsx:1105-1287` - Chat Model 区块

  **Acceptance Criteria**:
  - [ ] 表单控件 focus ring 使用 neon-cyan
  - [ ] 删除按钮使用 neon-red
  - [ ] 第 1105-1287 行无 neon-blue 残留

  **QA Scenarios**:
  ```
  Scenario: Chat Model 区块颜色验证
    Tool: Bash (grep)
    Steps:
      1. sed -n '1105,1287p' src/components/SettingsModal.tsx | grep -c "neon-blue" || echo "0"
    Expected Result: 输出 0
    Failure Indicators: 输出 > 0
    Evidence: .sisyphus/evidence/task-16-neon-blue.txt
  ```

  **Commit**: NO

- [ ] 17. 更新 SettingsModal.tsx - Privacy & Storage 区块

  **What to do**:
  - 更新 Privacy 区块样式
  - 更新 Storage 统计区块：
    - 进度条使用 neon-cyan
    - 数值高亮使用 neon-cyan
  - 更新导出按钮样式
  - 更新清理按钮（danger 操作，使用 neon-red）
  - 保持所有 glass-border 为 border-white/10

  **Must NOT do**:
  - 不修改数据管理逻辑
  - 不更改存储计算

  **Recommended Agent Profile**:
  > - **Category**: `visual-engineering`
    - Reason: SettingsModal 隐私和存储区块
  - **Skills**: N/A

  **Parallelization**:
  - **Can Run In Parallel**: YES
  - **Parallel Group**: Wave 4 (with Tasks 14, 15, 16, 18, 19)
  - **Blocks**: 无
  - **Blocked By**: Wave 1 (Tasks 1-4)

  **References**:
  - `src/components/SettingsModal.tsx:1289-1952` - Privacy 和 Storage 区块

  **Acceptance Criteria**:
  - [ ] 进度条使用 neon-cyan
  - [ ] 清理按钮使用 neon-red
  - [ ] 第 1289-1952 行无 neon-blue 残留

  **QA Scenarios**:
  ```
  Scenario: Privacy/Storage 区块颜色验证
    Tool: Bash (grep)
    Steps:
      1. sed -n '1289,1952p' src/components/SettingsModal.tsx | grep -c "neon-blue" || echo "0"
    Expected Result: 输出 0
    Failure Indicators: 输出 > 0
    Evidence: .sisyphus/evidence/task-17-neon-blue.txt
  ```

  **Commit**: NO

- [ ] 18. 更新其他 Modal 组件批量

  **What to do**:
  批量更新以下 Modal 组件：
  - `ChatHistoryModal.tsx`
  - `FeedbackModal.tsx`
  - `PerformanceModal.tsx`
  - `AgentModal.tsx`
  - `AgentProposalModal.tsx`
  - `AgentHistoryModal.tsx`（新增到范围）
  - `ImagePreviewModal.tsx`

  每个组件的更新内容：
  - 替换 `text-signal` → `text-neon-cyan`
  - 替换 `bg-signal/*` → `bg-neon-cyan/*`
  - 替换 `border-signal/*` → `border-neon-cyan/*`
  - 替换 `text-neon-blue` → `text-neon-cyan`
  - 替换 `bg-neon-blue/*` → `bg-neon-cyan/*`
  - 替换 `border-glass-border` → `border-white/10`
  - 增强 glass 效果
  - 删除操作按钮使用 neon-red

  **Must NOT do**:
  - 不修改任何 Modal 的显示/隐藏逻辑
  - 不更改事件处理函数

  **Recommended Agent Profile**:
  > - **Category**: `visual-engineering`
    - Reason: 多个 Modal 组件批量更新
  - **Skills**: N/A

  **Parallelization**:
  - **Can Run In Parallel**: YES
  - **Parallel Group**: Wave 4 (with Tasks 14, 15, 16, 17, 19)
  - **Blocks**: 无
  - **Blocked By**: Wave 1 (Tasks 1-4)

  **References**:
  - `src/components/ChatHistoryModal.tsx`
  - `src/components/FeedbackModal.tsx`
  - `src/components/PerformanceModal.tsx`
  - `src/components/AgentModal.tsx`
  - `src/components/AgentProposalModal.tsx`
  - `src/components/AgentHistoryModal.tsx`
  - `src/components/ImagePreviewModal.tsx`

  **Acceptance Criteria**:
  - [ ] 所有 Modal 无 signal 残留
  - [ ] 所有 Modal 无 neon-blue 残留
  - [ ] 删除按钮使用 neon-red

  **QA Scenarios**:
  ```
  Scenario: Modal 组件 Token 清理
    Tool: Bash (grep)
    Steps:
      1. grep -l "text-signal\|bg-signal\|neon-blue" src/components/*.tsx 2>/dev/null | grep -v "SettingsModal" | wc -l
    Expected Result: 输出 0
    Failure Indicators: 输出 > 0
    Evidence: .sisyphus/evidence/task-18-modals.txt
  ```

  **Commit**: NO

- [ ] 19. 更新小型组件批量

  **What to do**:
  更新 `MessageRating.tsx` 组件：
  - 替换所有颜色 token
  - 更新按钮样式
  - 添加 hover 效果

  **Must NOT do**:
  - 不修改评分逻辑
  - 不更改 API 调用

  **Recommended Agent Profile**:
  > - **Category**: `quick`
    - Reason: 小型组件，简单更新
  - **Skills**: N/A

  **Parallelization**:
  - **Can Run In Parallel**: YES
  - **Parallel Group**: Wave 4 (with Tasks 14, 15, 16, 17, 18)
  - **Blocks**: 无
  - **Blocked By**: Wave 1 (Tasks 1-4)

  **References**:
  - `src/components/MessageRating.tsx`

  **Acceptance Criteria**:
  - [ ] 无 signal 或 neon-blue 残留

  **QA Scenarios**:
  ```
  Scenario: MessageRating 颜色验证
    Tool: Bash (grep)
    Steps:
      1. grep -c "text-signal\|bg-signal\|neon-blue" src/components/MessageRating.tsx || echo "0"
    Expected Result: 输出 0
    Failure Indicators: 输出 > 0
    Evidence: .sisyphus/evidence/task-19-rating.txt
  ```

  **Commit**: YES (with message "style(modals): 更新 Modal 组件 UI 风格")

---

## Final Verification Wave

> 4 个验收任务串行执行。

- [ ] F1. **Token 残留扫描** — `quick`
  使用 grep 扫描残留的旧 token：
  - 搜索 `signal|neon-blue|amber|#F59E0B|#f59e0b`（排除白名单）
  - 搜索硬编码颜色如 `#00f3ff`
  - 验证扫描结果符合预期
  Output: `残留数量: [N] | 白名单: [files] | VERDICT`

- [ ] F2. **类型检查和 Lint 验证** — `quick`
  运行：
  - `pnpm type-check`
  - `pnpm lint`
  Output: `TypeScript [PASS/FAIL] | Lint [PASS/FAIL] | VERDICT`

- [ ] F3. **Dev Server Smoke 测试** — `quick`
  启动开发服务器：
  - `pnpm tauri:dev`
  - 验证应用启动无报错
  - 截图关键页面
  Output: `启动 [SUCCESS/FAIL] | 截图: [paths] | VERDICT`

- [ ] F4. **构建验证** — `quick`
  运行：
  - `pnpm build`
  - 验证构建产物生成
  Output: `构建 [PASS/FAIL] | 产物大小: [size] | VERDICT`

---

## Commit Strategy

- **Wave 1**: `design: 更新设计 token 系统和动画基础` (tailwind.config.js, index.css, animations.css, types/design.ts)
- **Wave 2**: `style(core): 更新核心布局组件 UI 风格` (Layout, Timeline, GalleryView, ContextSidebar)
- **Wave 3**: `style(views): 更新视图组件 UI 风格` (KnowledgeGraph, FlowState, QnA, ImmersiveReplay, ActivityHeatmap)
- **Wave 4**: `style(modals): 更新 Modal 组件 UI 风格` (SettingsModal 分批, 其他 Modal)
- **FINAL**: `chore: UI 优化收尾和验收` (残留扫描, 类型修复)

---

## Success Criteria

### Verification Commands
```bash
# Token 残留扫描
pnpm exec grep -r "signal\|neon-blue\|#F59E0B\|#f59e0b" src/ --include="*.tsx" --include="*.css" |
  grep -v "node_modules" | grep -v ".sisyphus" |
  wc -l  # 应为 0 或仅白名单文件

# 硬编码颜色扫描
pnpm exec grep -r "#00f3ff" src/ --include="*.tsx" --include="*.css" |
  wc -l  # 应为 0

# 新 token 验证
pnpm exec grep -r "neon-cyan\|#00f0ff\|neon-red\|#ff003c\|border-white/10\|backdrop-blur" src/ --include="*.tsx" --include="*.css" |
  wc -l  # 应 > 50

# 类型检查
pnpm type-check  # Exit 0

# Lint
pnpm lint  # Exit 0

# 构建
pnpm build  # Exit 0
```

### Final Checklist
- [ ] Neon Cyan (#00f0ff) 替换所有 signal/neon-blue
- [ ] Neon Red (#ff003c) 用于所有 danger 场景
- [ ] Amber (#F59E0B) 保留用于 warning 语义
- [ ] 所有组件使用 border-white/10 透明边框
- [ ] 所有组件应用 backdrop-blur 毛玻璃效果
- [ ] 微交互动画正常工作
- [ ] 支持 prefers-reduced-motion
- [ ] 无残留旧 token
- [ ] 类型检查通过
- [ ] Lint 通过
- [ ] 构建成功

