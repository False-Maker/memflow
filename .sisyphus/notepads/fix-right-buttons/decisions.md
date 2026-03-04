# 修复右上角按钮点击问题

## 问题描述
从 LayoutProps 接口定义中移除了 onOpenFeedback 属性，解决了接口不匹配导致的 TypeScript 类型错误。

## 问题分析
- App.tsx 只传递了 3 个回调函数给 Layout 组件
- Layout.tsx 的 LayoutProps 接口定义中包含了 onOpenFeedback 属性
- 这导致 TypeScript 类型检查失败，无法找到 onOpenFeedback 属性

## 解决方案
从 LayoutProps 接口和组件参数中移除 onOpenFeedback 属性。

## 文件修改
- `src/components/Layout.tsx`: 移除 onOpenFeedback 参数

## 验证结果
- ✅ TypeScript 类型检查通过
- ✅ 按钮可以正常点击
- ✅ 接口定义与使用一致

## 影响
- 无功能影响，仅移除了未使用的接口定义
- 保持代码清洁和类型安全