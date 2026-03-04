# Decisions

## 2026-03-04 视图切换中文化
- **变更内容**: 将 `src/components/Layout.tsx` 中的视图切换按钮标签从英文缩写改为中文
- **具体变更**:
  - timeline: 'T' → '时间轴' (label), 'TIME' → '时间轴' (full)
  - gallery: 'G' → '图库' (label), 'GALLERY' → '图库' (full)  
  - replay: 'R' → '回放' (label), 'REPLAY' → '回放' (full)
  - graph: 'K' → '图谱' (label), 'KNOWLEDGE' → '知识图谱' (full)
  - stats: 'S' → '统计' (label), 'STATS' → '统计' (full)
  - qa: 'Q' → '问答' (label), 'Q&A' → '问答' (full)
- **设计原则**: 
  - label 保持简短 (1-2字)，用于按钮显示
  - full 使用完整名称，用于工具提示
  - 保持视图 ID 不变，确保逻辑正确
  - 维持现有按钮样式和布局