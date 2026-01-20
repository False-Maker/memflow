# 测试指南

本文档介绍了项目的测试策略、运行方法和最佳实践。

## 测试结构

```
src/
├── components/          # 组件测试
│   ├── QnA.test.tsx
│   ├── Timeline.test.tsx
│   └── MessageRating.test.tsx
├── contexts/            # 上下文测试
│   └── AppContext.test.tsx
├── test/                # 测试配置和集成测试
│   ├── setupTests.ts
│   ├── integration/     # 集成测试
│   │   └── AppContext.integration.test.tsx
│   └── README.md
└── utils/               # 工具函数测试
    └── imageLoader.test.ts
```

## 运行测试

### 单元测试

```bash
# 运行所有单元测试
pnpm test:unit

# 监听模式
pnpm test:unit:watch

# UI 模式
pnpm test:unit:ui

# 生成覆盖率报告
pnpm test:unit:coverage
```

### 集成测试

```bash
# 运行所有集成测试
pnpm test:integration

# 运行所有测试（单元 + 集成）
pnpm test:all
```

### Rust 测试

```bash
# 运行 Rust 测试
pnpm test:rust

# 监听模式
pnpm test:watch
```

## 覆盖率目标

- **总体覆盖率**: ≥ 70%
- **AppContext**: ≥ 70% (当前目标)
- **组件**: ≥ 65%
- **工具函数**: ≥ 80%

## 测试类型

### 1. 单元测试

测试单个组件或函数的独立功能。

**示例**:
```typescript
describe('QnA Component', () => {
  it('应该发送消息并显示回复', async () => {
    // 测试实现
  })
})
```

### 2. 集成测试

测试多个组件之间的交互。

**示例**:
```typescript
describe('AppContext Integration Tests', () => {
  it('应该完成从启动到停止录制的完整流程', async () => {
    // 测试实现
  })
})
```

### 3. E2E 测试（规划中）

使用 Playwright 进行端到端测试。

```bash
pnpm test:e2e
```

## 测试最佳实践

### 1. 测试命名

- 使用描述性的测试名称
- 使用中文描述（与项目代码风格一致）
- 遵循 "应该..." 的模式

### 2. Mock 策略

- Mock 所有外部依赖（Tauri API、网络请求等）
- 在 `setupTests.ts` 中配置全局 Mock
- 在测试文件中覆盖特定的 Mock 行为

### 3. 测试结构

```typescript
describe('组件名称', () => {
  beforeEach(() => {
    // 设置测试环境
  })

  afterEach(() => {
    // 清理测试环境
  })

  describe('功能模块', () => {
    it('应该...', async () => {
      // 测试实现
    })
  })
})
```

### 4. 异步测试

使用 `waitFor` 等待异步操作完成：

```typescript
await waitFor(() => {
  expect(screen.getByText('预期文本')).toBeInTheDocument()
})
```

### 5. 用户交互测试

使用 `@testing-library/user-event` 模拟用户操作：

```typescript
const user = userEvent.setup()
await user.type(input, '文本')
await user.click(button)
```

## CI/CD 集成

GitHub Actions 工作流会自动运行测试：

1. **前端测试**: 运行单元测试和类型检查
2. **Rust 测试**: 运行后端测试
3. **集成测试**: 运行集成测试套件
4. **覆盖率检查**: 验证覆盖率是否达到目标

工作流文件: `.github/workflows/ci.yml`

## 编写新测试

### 为新组件添加测试

1. 在组件同级目录创建 `ComponentName.test.tsx`
2. 导入必要的测试工具和组件
3. 设置 Mock（如需要）
4. 编写测试用例

### 为新功能添加集成测试

1. 在 `src/test/integration/` 创建测试文件
2. 使用 `*.integration.test.tsx` 命名约定
3. 测试组件之间的交互
4. 验证完整的工作流程

## 调试测试

### 使用 Vitest UI

```bash
pnpm test:unit:ui
```

### 运行单个测试文件

```bash
pnpm test:unit src/components/QnA.test.tsx
```

### 使用 VS Code

安装 Vitest 扩展，可以在编辑器中直接运行测试。

## 常见问题

### Q: 测试失败但功能正常？

A: 检查 Mock 是否正确设置，特别是 Tauri API 的 Mock。

### Q: 覆盖率没有达到目标？

A: 
1. 运行 `pnpm test:unit:coverage` 查看详细报告
2. 检查未覆盖的代码行
3. 添加相应的测试用例

### Q: 集成测试太慢？

A: 
1. 确保 Mock 了所有外部依赖
2. 使用 `vi.useFakeTimers()` 跳过时间相关的等待
3. 减少不必要的异步等待

## 测试资源

- [Vitest 文档](https://vitest.dev/)
- [Testing Library 文档](https://testing-library.com/)
- [React Testing Best Practices](https://kentcdodds.com/blog/common-mistakes-with-react-testing-library)

## 更新日志

- **2024-01-XX**: 添加 AppContext 集成测试
- **2024-01-XX**: 添加 QnA 和 Timeline 组件测试
- **2024-01-XX**: 创建 CI/CD 工作流
- **2024-01-XX**: 设置覆盖率阈值
