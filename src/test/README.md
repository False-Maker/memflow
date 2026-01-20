# 单元测试说明

本项目使用 [Vitest](https://vitest.dev/) 作为测试框架，配合 [@testing-library/react](https://testing-library.com/react) 进行 React 组件测试。

## 运行测试

### 运行所有测试
```bash
pnpm test:unit
```

### 运行测试（单次运行，不监听）
```bash
pnpm test:unit --run
```

### 监听模式（自动重新运行）
```bash
pnpm test:unit:watch
```

### 使用 UI 界面
```bash
pnpm test:unit:ui
```

### 生成覆盖率报告
```bash
pnpm test:unit:coverage
```

## 测试文件结构

测试文件应该与被测试的文件放在同一目录下，使用 `.test.ts` 或 `.test.tsx` 后缀：

```
src/
  utils/
    imageLoader.ts
    imageLoader.test.ts
  types/
    chat.ts
    chat.test.ts
```

## 编写测试

### 工具函数测试示例

```typescript
import { describe, it, expect, vi } from 'vitest'
import { myFunction } from './myModule'

describe('myModule', () => {
  it('应该正确执行功能', () => {
    const result = myFunction('input')
    expect(result).toBe('expected')
  })
})
```

### React 组件测试示例

```typescript
import { describe, it, expect } from 'vitest'
import { render, screen } from '@testing-library/react'
import { MyComponent } from './MyComponent'

describe('MyComponent', () => {
  it('应该渲染组件', () => {
    render(<MyComponent />)
    expect(screen.getByText('Hello')).toBeInTheDocument()
  })
})
```

## Mock Tauri API

由于测试环境无法访问 Tauri API，所有 Tauri API 调用都需要被 mock。在 `setupTests.ts` 中已经配置了基本的 mock，你可以在测试文件中使用：

```typescript
import { invoke } from '@tauri-apps/api/core'

vi.mocked(invoke).mockResolvedValue('mock-value')
```

## 测试覆盖率

当前覆盖率（运行 `pnpm test:unit:coverage` 查看最新）：
- 总体：78.67% 语句，89.58% 分支
- 工具函数：91.3% ✅
- 组件：94.73% ✅ (MessageRating)
- 类型定义：100% ✅
- Context：65.21% ⚠️ (需要改进)

目标覆盖率：
- 工具函数：> 90% ✅
- 组件：> 80% ✅
- 类型定义：> 95% ✅
- Context/状态管理：> 70% ⚠️

## 注意事项

1. 每个测试应该独立，不依赖其他测试的状态
2. 使用 `beforeEach` 和 `afterEach` 清理测试状态
3. Mock 外部依赖（如 Tauri API、网络请求等）
4. 测试应该清晰描述被测试的行为

