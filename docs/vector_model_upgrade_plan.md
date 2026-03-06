# 向量模型修复与升级方案

## 一、问题分析

### 1.1 当前状态

| 项目 | 状态 |
|-----|------|
| 设置面板 | 有 embedding 模型配置项 |
| API Key 存储 | 有单独的存储逻辑 |
| 向量生成 | 实际用的是占位符向量 |

### 1.2 问题根因

```
设置面板 → embedding_model: "BGE-small-zh-v1.5"
         → embedding_api_key: "sk-xxx"
              ↓
后端代码 → 读取的是 chat_api_key，不是 embedding_api_key
              ↓
向量生成 → API Key 获取失败 → 降级用占位符向量
```

**核心问题**：代码逻辑错误，配置了没用上。

### 1.3 历史向量统计

- 总向量数：49
- 有 OCR 文本的活动：26
- 向量维度：384（BGE-small 标准）
- 创建时间：2026-03-05

---

## 二、解决方案

### 2.1 修复步骤

#### 步骤1：删除共享 API Key 功能

**问题**：向量模型的 API Key 和 Chat 的 API Key 混在一起。

**解决**：让向量模型独立使用自己的 API Key。

**涉及文件**：
- `frontend/src/components/SettingsModal.tsx` - 删除共享开关 UI
- `crates/memflow-core/src/config.rs` - 删除共享逻辑
- `crates/memflow-persistence/src/key_storage.rs` - 独立存储

#### 步骤2：修复 API Key 读取逻辑

**问题**：向量生成时读取的是 `chat` 服务的 key。

**解决**：改成读取 `embedding` 服务的 key。

**涉及文件**：
- `crates/memflow-core/src/ai/embedding.rs` - 修复 key 获取逻辑

#### 步骤3：配置验证

**问题**：不确定配置是否生效。

**解决**：添加日志或 UI 显示向量状态。

---

### 2.2 模型升级

#### 方案A：本地模型（推荐）

| 项目 | 当前 | 升级后 |
|-----|------|-------|
| 模型 | BGE-small-en-v1.5 | BGE-small-zh-v1.5 |
| 语言 | 英文 | 中文 |
| 维度 | 384 | 384 |
| 模型大小 | ~70MB | ~70MB |

**改动**：1行代码

```rust
// crates/memflow-core/src/ai/embedding.rs
// 修改前
EmbeddingModel::BGESmallENV15

// 修改后
EmbeddingModel::BGESmallZHV15
```

#### 方案B：升维方案

| 项目 | 当前 | 升级后 |
|-----|------|-------|
| 模型 | BGE-small-en-v1.5 | BGE-base-zh-v1.5 |
| 语言 | 英文 | 中文 |
| 维度 | 384 | 768 |
| 模型大小 | ~70MB | ~170MB |

**改动**：
1. 改代码：换模型
2. 改数据库：向量字段从 BLOB(384) 改成 BLOB(768)
3. 重新生成：所有历史数据重新做 embedding

#### 方案C：云端 API

| 项目 | 说明 |
|-----|------|
| 提供商 | 智谱 AI |
| 模型 | embedding-3 |
| 维度 | 1024/1536 |
| 费用 | ¥1-3/百万字符 |

**改动**：
1. 改代码：切换到云端模式
2. 配 API Key
3. 改数据库：向量字段升维

---

## 三、实施计划

### 3.1 阶段一：修复 Bug（必做）

| 序号 | 任务 | 涉及文件 | 预估时间 |
|-----|------|---------|---------|
| 1 | 删除共享 API Key 配置 | SettingsModal.tsx, config.rs | 10min |
| 2 | 修复 embedding key 读取 | embedding.rs | 10min |
| 3 | 测试验证 | - | 10min |

### 3.2 阶段二：模型升级（必做）

| 序号 | 任务 | 涉及文件 | 预估时间 |
|-----|------|---------|---------|
| 1 | 改模型为中文版 | embedding.rs | 5min |
| 2 | 重新生成历史向量 | - | 5min |
| 3 | 验证搜索效果 | - | 10min |

### 3.3 阶段三：可选优化

| 序号 | 任务 | 说明 |
|-----|------|------|
| 1 | 升维到 768 | 需要改 DB schema |
| 2 | 接入云端 API | 效果好，收费 |
| 3 | 模型可配置 | 用户可自选模型 |

---

## 四、技术细节

### 4.1 向量生成流程

```
用户活动（OCR文本）
       ↓
检查配置（embedding_model, embedding_api_key）
       ↓
  ┌────┴────┐
  ↓         ↓
有 Key    无 Key
  ↓         ↓
调用云端    使用本地模型
API         (fastembed)
  ↓         ↓
  └────┬────┘
       ↓
   获取向量
       ↓
   存储到 SQLite
```

### 4.2 当前配置结构

```json
{
  "embedding_model": "BGE-small-zh-v1.5",
  "embedding_api_key": "sk-xxx",  // 独立存储在 Windows 凭据管理器
  "chat_api_key": "sk-xxx",       // 另一个 key
}
```

### 4.3 数据库向量表

```sql
CREATE TABLE activities (
    id TEXT PRIMARY KEY,
    ocr_text TEXT,
    embedding BLOB(384),  -- 384维向量
    created_at TIMESTAMP
);
```

---

## 五、风险与注意事项

### 5.1 数据风险

| 风险 | 应对 |
|-----|------|
| 向量重新生成 | 先备份数据库 |
| 维度变更 | 需要迁移脚本 |

### 5.2 兼容性问题

| 问题 | 解决方案 |
|-----|---------|
| fastembed 版本 | 当前用 5.8.1，稳定 |
| ONNX 运行时 | 用 ort 2.0，需要复制 DLL |

### 5.3 验证方法

```bash
# 1. 查看向量统计
curl http://localhost:51888/vector_stats

# 2. 手动触发向量生成
# （需要 UI 操作或 API）

# 3. 检查日志
tail -f memflow.log | grep -i embed
```

---

## 六、预期效果

### 6.1 修复后

- 设置面板的 embedding 配置能真正生效
- 向量搜索能正常工作

### 6.2 升级中文模型后

- 中文搜索效果显著提升
- 本地模型，无需 API Key

### 6.3 升级到云端后（可选）

- 向量质量最高
- 需要付费

---

## 七、决策确认

请确认以下选项：

1. **阶段一**：是否立即执行修复？
2. **阶段二**：选择哪个方案？
   - [ ] A. 本地中文模型（推荐，快速见效）
   - [ ] B. 升维到 768
   - [ ] C. 接入云端 API
3. **阶段三**：是否需要？

---

*文档版本：v1.0*
*最后更新：2026-03-06*
