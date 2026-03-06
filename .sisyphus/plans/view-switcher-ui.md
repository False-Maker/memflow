# 视图切换胶囊 UI 优化

## TL;DR

> **Quick Summary**: 将视图切换胶囊（T G R K S Q）升级为超炫科技风格
>
> **Deliverables**:
> - 更新 `src/components/Layout.tsx` 中的视图切换部分
>
> **Estimated Effort**: Short
> **Parallel Execution**: NO - 单文件修改

---

## Context

### Original Request
用户觉得当前的视图切换胶囊（T G R K S Q）太简单，需要优化。

### Current State
```tsx
<div className="flex items-center gap-1 bg-zinc-900/80 rounded-xl p-1.5 border border-white/5">
  {views.map(view => (
    <button className={currentView === view.id ? 'bg-neon-cyan text-black' : 'text-zinc-500'}>
      {view.label}
    </button>
  ))}
</div>
```

---

## Work Objectives

### Core Objective
将视图切换胶囊升级为超炫科技风格，增加视觉吸引力。

### Concrete Deliverables
- 更新的视图切换 JSX 代码
- 新增动画效果（扫描、shimmer、脉冲）
- 新增装饰元素（网格、角标、分隔线、装饰线）

### Must Have
- ✅ 保持现有功能不变
- ✅ 保持响应式布局
- ✅ 添加动画效果
- ✅ 增强视觉层次

### Must NOT Have
- ❌ 不改变切换逻辑
- ❌ 不破坏现有布局

---

## Execution Strategy

### 单步执行
1. 更新视图切换 JSX
2. 添加新动画定义（如需要）
3. 验证功能正常

---

## TODOs

- [ ] 1. 更新视图切换胶囊代码

  **What to do**:
  替换 `src/components/Layout.tsx` 中的视图切换部分（约第 181-220 行）

  **具体实现**:
  ```tsx
  {/* 视图切换 - 超炫科技胶囊 */}
  <div className="flex items-center">
    {/* 左装饰线 */}
    <div className="w-8 h-px bg-gradient-to-r from-transparent to-neon-cyan/50 mr-2" />
    
    <div className="flex items-center gap-0.5 bg-zinc-900/60 rounded-2xl p-1 border border-white/10 backdrop-blur-md relative overflow-hidden group">
      {/* 动态背景 - 扫描效果 */}
      <div className="absolute inset-0 opacity-20">
        <div className="absolute inset-0 bg-gradient-to-r from-transparent via-neon-cyan/30 to-transparent animate-scan" />
      </div>
      
      {/* 网格图案 */}
      <div className="absolute inset-0 opacity-10" style={{
        backgroundImage: `
          linear-gradient(rgba(0, 240, 255, 0.3) 1px, transparent 1px),
          linear-gradient(90deg, rgba(0, 240, 255, 0.3) 1px, transparent 1px)
        `,
        backgroundSize: '8px 8px'
      }} />

      {views.map((view, index) => (
        <button
          key={view.id}
          onClick={() => setCurrentView(view.id as any)}
          className={`relative px-4 py-2.5 text-xs font-mono font-bold tracking-wider rounded-xl transition-all duration-300 ${
            currentView === view.id
              ? 'bg-neon-cyan text-black shadow-[0_0_25px_rgba(0,240,255,0.6)] scale-105'
              : 'text-zinc-500 hover:text-neon-cyan hover:scale-105'
          }`}
          title={view.full}
          style={{
            textShadow: currentView === view.id ? '0 0 10px rgba(0,240,255,0.8)' : 'none'
          }}
        >
          {/* 激活状态下的内部动画 */}
          {currentView === view.id && (
            <>
              <div className="absolute inset-0 bg-neon-cyan/30 rounded-xl animate-pulse" />
              <div className="absolute inset-0 bg-gradient-to-br from-white/20 to-transparent rounded-xl" />
              {/* 扫描光效 */}
              <div className="absolute inset-0 overflow-hidden rounded-xl">
                <div className="absolute inset-0 bg-gradient-to-r from-transparent via-white/30 to-transparent -skew-x-12 animate-shimmer" />
              </div>
            </>
          )}
          
          {/* 悬停光效 */}
          <div className="absolute inset-0 rounded-xl opacity-0 hover:opacity-100 transition-opacity duration-300 bg-gradient-to-br from-neon-cyan/10 to-transparent pointer-events-none" />
          
          {/* 分隔线 */}
          {index < 5 && (
            <div className={`absolute right-0 top-1/2 -translate-y-1/2 h-4 w-px ${
              currentView === view.id || currentView === views[index + 1]?.id
                ? 'bg-transparent'
                : 'bg-white/10'
            }`} />
          )}
          
          {/* 角标装饰 */}
          <div className="absolute top-1 left-1.5 w-1 h-1 rounded-full bg-neon-cyan/50" />
          <div className="absolute bottom-1 right-1.5 w-1 h-1 rounded-full bg-neon-cyan/50" />
          
          <span className="relative z-10">{view.label}</span>
        </button>
      ))}
    </div>
    
    {/* 右装饰线 */}
    <div className="w-8 h-px bg-gradient-to-l from-transparent to-neon-cyan/50 ml-2" />
  </div>
  ```

  **Must NOT do**:
  - 不修改切换逻辑
  - 不改变视图数据结构

  **Recommended Agent Profile**:
  > - **Category**: `visual-engineering`
    - Reason: UI 样式增强
  - **Skills**: N/A

  **Parallelization**:
  - **Can Run In Parallel**: NO
  - **Parallel Group**: Sequential
  - **Blocks**: 无
  - **Blocked By**: 无

  **References**:
  - `src/components/Layout.tsx:181-220` - 当前视图切换代码

  **Acceptance Criteria**:
  - [ ] 视图切换功能正常
  - [ ] 动画效果流畅
  - [ ] 视觉效果增强

  **QA Scenarios**:
  ```
  Scenario: 视图切换验证
    Tool: Bash (pnpm)
    Steps:
      1. pnpm tauri:dev
    Expected Result: 应用启动，视图切换正常工作
    Failure Indicators: 切换失败、样式错乱
    Evidence: .sisyphus/evidence/view-switch-test.txt
  ```

  **Commit**: YES

- [ ] 2. 添加 shimmer 动画定义

  **What to do**:
  在 `tailwind.config.js` 中添加 shimmer 动画

  **Must NOT do**:
  - 不删除现有动画

  **Recommended Agent Profile**:
  > - **Category**: `quick`
    - Reason: 配置文件更新

  **References**:
  - `tailwind.config.js:35-48` - 现有动画定义

  **Acceptance Criteria**:
  - [ ] shimmer 动画已定义

  **Commit**: NO (与任务 1 一起提交)

---

## Success Criteria

- [ ] 视图切换功能正常
- [ ] 动画效果流畅
- [ ] 视觉效果增强（网格、扫描、角标、装饰线）
- [ ] 类型检查通过
