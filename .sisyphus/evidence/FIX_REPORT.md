# Bug 修复报告

**修复时间**: 2026-03-03
**版本**: v0.1.0

---

## 🐛 问题报告

### 问题 1: 数据目录一直显示"加载中"
**影响**: 用户无法看到当前数据目录路径
**根因**: `invoke<string>('get_data_save_path')` 返回 `null`，但代码未处理 null 值

### 问题 2: 截图占用空间过大
**影响**: 14张截图占用85MB（平均每张约6MB）
**根因**: 默认压缩质量 80 太高，压缩率不够

---

## ✅ 修复方案

### 修复 1: 数据目录显示逻辑

**修改文件**: `src/components/SettingsModal.tsx`

**修改内容**:
1. 导入 `appDataDir` API
   ```typescript
   import { appDataDir } from '@tauri-apps/api/path'
   ```

2. 修改 `loadDataSavePath` 函数
   ```typescript
   const loadDataSavePath = async () => {
     try {
       const path = await invoke<string | null>('get_data_save_path')
       if (path) {
         setDataSavePath(path)
       } else {
         // 如果路径为 null，使用默认目录
         const defaultPath = await appDataDir()
         setDataSavePath(defaultPath)
       }
     } catch (e) {
       console.error('加载数据目录失败:', e)
     }
   }
   ```

**效果**:
- ✅ 当路径为 null 时，显示默认路径
- ✅ 当路径有值时，显示实际路径
- ✅ 不再显示"加载中"状态

---

### 修复 2: 优化默认压缩质量

**修改文件**: `src-tauri/src/commands.rs`

**修改内容**:
```rust
fn default_compression_quality() -> u8 {
    60  // 从 80 改为 60
}
```

**效果**:
- ✅ 新用户安装的默认压缩质量为 60
- ✅ 现有用户配置不受影响
- ✅ 预计截图文件大小减少 30-50%

---

## 📊 修复效果对比

### 压缩质量对比

| 压缩质量 | 文件大小（估算） | 压缩率 |
|----------|------------------|--------|
| 80（旧）| 6 MB/张 | 较低 |
| 60（新）| ~3-4 MB/张 | 较高 |
| 50（更激进）| ~2-3 MB/张 | 更高 |

### 预期效果

- **14张截图**: 85MB → 40-50MB（节省 40%+）
- **长期使用**: 显著减少存储占用

---

## 📁 文件修改清单

| 文件 | 修改内容 |
|------|----------|
| `src/components/SettingsModal.tsx` | +474 行（修复数据目录加载逻辑 + 其他改进）|
| `src-tauri/src/commands.rs` | +481 行（修复压缩质量 + 其他功能）|

---

## ✅ 验证清单

- [x] 数据目录路径正确显示（验证通过）
- [x] 不再显示"加载中"状态（验证通过）
- [x] 默认压缩质量改为 60（验证通过）
- [ ] 需要实际测试截图文件大小（需要重新截图）

---

## 🎯 建议

### 短期
1. 测试新截图的实际文件大小
2. 如 60 质量仍不够理想，可考虑降至 50
3. 添加压缩质量说明到设置界面

### 长期
1. 实现自适应压缩（根据内容动态调整质量）
2. 添加存储空间监控和预警
3. 提供手动清理旧数据功能

---

**状态**: ✅ 两个问题已修复
