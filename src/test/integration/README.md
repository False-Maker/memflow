# 集成测试

这个目录包含应用程序的集成测试。

## 测试目的

集成测试用于验证多个组件之间的交互，确保它们能够正确地协同工作。

## 运行集成测试

```bash
# 运行所有集成测试
pnpm test:integration

# 运行特定测试文件
pnpm test:unit src/test/integration/AppContext.integration.test.tsx
```

## 测试覆盖范围

- **AppContext Integration Tests**: 测试 AppContext 的完整工作流程
  - 完整的录制流程（启动到停止）
  - 活动生命周期（创建到 OCR 更新）
  - 搜索流程
  - 配置管理
  - 视图切换
  - 错误处理

## 编写新的集成测试

1. 在 `src/test/integration/` 目录中创建新的测试文件
2. 使用 `*.integration.test.tsx` 命名约定
3. 测试应该关注组件之间的交互，而不是单个组件的功能
4. 使用 Mock 来模拟外部依赖（如 Tauri API）

## 最佳实践

- 测试应该独立且可重复运行
- 使用 `beforeEach` 和 `afterEach` 来设置和清理测试环境
- 测试应该覆盖正常流程和错误情况
- 保持测试简洁，专注于验证交互行为

