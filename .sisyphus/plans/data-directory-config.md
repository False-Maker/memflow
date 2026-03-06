# 添加数据保存目录配置

## TL;DR

> **快速摘要**: 在设置界面添加"数据保存目录"配置项，让用户可以自定义数据存储位置
>
> **交付成果**:
> - AppConfig 新增 `data_save_path` 字段
> - 后端支持自定义数据路径
> - 设置界面添加目录选择功能
> - 数据迁移工具（可选迁移旧数据）
>
> **预估工作量**: 中等
> **并行执行**: 否 - 需要依次修改后端配置、后端逻辑、前端界面
> **关键路径**: 配置定义 → 后端逻辑 → 前端界面 → 测试验证

---

## Context

### Original Request
用户希望在设置中增加"数据保存目录"配置项，让用户可以手动修改数据存储位置，而不是固定使用 `%APPDATA%\MemFlow\`。

### Interview Summary
**关键讨论**:
- 当前数据固定存储在 `%APPDATA%\MemFlow\`（Windows）
- 用户希望可以自定义数据保存位置
- 需要支持目录选择界面
- 可能需要数据迁移功能

**Research Findings**:
- `AppConfig` 在 `src-tauri/src/commands.rs` 定义
- 配置通过 `app_config.rs` 管理
- 前端设置界面在 `src/components/SettingsModal.tsx`
- Tauri 提供了 `@tauri-apps/plugin-fs` 用于文件系统操作

---

## Work Objectives

### Core Objective
添加数据保存目录配置功能，让用户可以自定义数据存储位置，而不是固定使用系统默认的 APPDATA 目录。

### Concrete Deliverables
- **AppConfig 新增字段**: `data_save_path: Option<String>`
- **后端支持**: 使用自定义路径或回退到默认路径
- **前端界面**: 目录选择按钮 + 路径显示
- **数据迁移**: 可选地将旧数据迁移到新位置（建议）
- **验证机制**: 检查路径有效性、写入权限等

### Definition of Done
- [ ] AppConfig 包含 `data_save_path` 字段
- [ ] 后端逻辑支持自定义数据路径
- [ ] 设置界面有"数据目录"配置项
- [ ] 可以通过目录选择对话框选择路径
- [ ] 显示当前数据目录路径
- [ ] 验证路径有效性和权限
- [ ] 重启应用后使用新路径

### Must Have
- 保留默认行为（data_save_path 为 None 时使用 APPDATA）
- 支持路径验证（可写、存在性）
- 显示当前数据目录位置
- 提供目录选择对话框

### Must NOT Have (Guardrails)
- 不强制要求用户设置自定义路径（保持可选）
- 不在未验证的情况下更改数据路径
- 不在路径无效时保存配置

---

## Verification Strategy

### Test Decision
- **Infrastructure exists**: NO
- **Automated tests**: None
- **Framework**: None

### QA Policy
每个任务包含 Agent-Executed QA 场景：
- **配置验证**: 检查配置项正确添加到 UI
- **路径验证**: 检查目录选择功能正常工作
- **功能验证**: 检查应用使用新路径保存数据

---

## Execution Strategy

### Parallel Execution Waves

```
Wave 1 (Start Immediately — 后端配置):
├── Task 1: 更新 AppConfig 结构定义 [quick]
└── Task 2: 更新默认配置逻辑 [quick]

Wave 2 (After Wave 1 — 后端逻辑):
├── Task 3: 更新数据路径获取逻辑 [quick]
└── Task 4: 添加数据路径验证函数 [quick]

Wave 3 (After Wave 2 — Tauri Commands):
├── Task 5: 添加 get_data_save_path 命令 [quick]
└── Task 6: 添加 set_data_save_path 命令 [quick]

Wave 4 (After Wave 3 — 前端界面):
├── Task 7: 添加数据目录配置 UI [visual-engineering]
└── Task 8: 添加目录选择功能 [visual-engineering]

Wave 5 (After Wave 4 — 测试验证):
├── Task 9: 验证配置功能 [quick]
└── Task 10: 生成功能报告 [quick]
```

---

## TODOs

- [ ] 1. **更新 AppConfig 结构定义**

  **What to do**:
  - 在 `src-tauri/src/commands.rs` 的 `AppConfig` 结构体中添加新字段：
    ```rust
    #[serde(default)]
    pub data_save_path: Option<String>,  // 用户自定义数据保存目录
    ```
  - 更新相关的默认值函数（如果需要）

  **Must NOT do**:
  - 不修改其他现有字段
  - 不破坏现有配置兼容性

  **Recommended Agent Profile**:
  - **Category**: `quick`
    - Reason: 简单的结构体字段添加
  - **Skills**: `[]`

  - **Parallelization**:
  - **Can Run In Parallel**: NO
  - **Parallel Group**: Sequential (Wave 1)
  - **Blocks**: Task 2, 3, 4
  - **Blocked By**: None

  **References**:
  - `D:/Demo/memflow/src-tauri/src/commands.rs:25-1259` - AppConfig 定义

  **Acceptance Criteria**:

  **QA Scenarios (MANDATORY)**:

  ```
  Scenario: 验证 AppConfig 字段已添加
    Tool: Read
    Preconditions: 代码已更新
    Steps:
      1. 读取: D:/Demo/memflow/src-tauri/src/commands.rs
      2. 搜索: "data_save_path"
      3. 验证: 字段已添加到 AppConfig 结构体
      4. 验证: 字段类型为 Option<String>
      5. 验证: 包含 serde(default) 属性
    Expected Result: data_save_path 字段正确添加
    Failure Indicators: 字段不存在或类型错误
    Evidence: .sisyphus/evidence/task-1-config-field.log
  ```

  **Commit**: NO

- [ ] 2. **更新默认配置逻辑**

  **What to do**:
  - 在 `src-tauri/src/app_config.rs` 中更新默认配置：
    - 添加 `data_save_path: None` 到默认配置
  - 确保 None 值表示使用默认的 APPDATA 路径

  **Must NOT do**:
  - 不修改现有默认值（除非必要）

  **Recommended Agent Profile**:
  - **Category**: `quick`
    - Reason: 配置文件更新
  - **Skills**: `[]`

  - **Parallelization**:
  - **Can Run In Parallel**: NO
  - **Parallel Group**: Sequential (Wave 1)
  - **Blocks**: Task 3, 4
  - **Blocked By**: Task 1

  **References**:
  - `D:/Demo/memflow/src-tauri/src/app_config.rs:30-64` - 默认配置定义

  **Acceptance Criteria**:

  **QA Scenarios (MANDATORY)**:

  ```
  Scenario: 验证默认配置已更新
    Tool: Read
    Preconditions: 代码已更新
    Steps:
      1. 读取: D:/Demo/memflow/src-tauri/src/app_config.rs
      2. 验证: 默认配置包含 data_save_path: None
      3. 验证: 字段在 default_config 函数中初始化
    Expected Result: 默认配置正确更新
    Failure Indicators: 字段缺失或初始化错误
    Evidence: .sisyphus/evidence/task-2-default-config.log
  ```

  **Commit**: NO

- [ ] 3. **更新数据路径获取逻辑**

  **What to do**:
  - 找到所有硬编码的数据路径（如 `app_data_dir()` 调用）
  - 创建 `get_data_dir()` 函数：
    - 优先使用 `config.data_save_path`
    - 为 None 时回退到 `app_data_dir()`
  - 更新所有数据库访问代码使用新函数
  - 路径解析：支持相对路径和绝对路径
  - 路径验证：确保目录可写、可访问

  **Must NOT do**:
  - 不破坏现有功能（None 时必须保持原有行为）
  - 不在未验证的情况下更改路径

  **Recommended Agent Profile**:
  - **Category**: `quick`
    - Reason: 路径逻辑更新
  - **Skills**: `[]`

  - **Parallelization**:
  - **Can Run In Parallel**: NO
  - **Parallel Group**: Sequential (Wave 2)
  - **Blocks**: Task 4, 5, 6
  - **Blocked By**: Task 1, 2

  **References**:
  - `D:/Demo/memflow/src-tauri/src/app_config.rs` - 配置管理
  - `D:/Demo/memflow/src-tauri/src/db.rs` - 数据库访问
  - `D:/Demo/memflow/crates/memflow-core/src/context.rs` - RuntimeContext

  **Acceptance Criteria**:

  **QA Scenarios (MANDATORY)**:

  ```
  Scenario: 验证 get_data_dir 函数逻辑
    Tool: Read + Bash
    Preconditions: 函数已创建
    Steps:
      1. 读取: D:/Demo/memflow/src-tauri/src/app_config.rs
      2. 验证: 函数返回正确的结果类型
      3. 测试: config.data_save_path=None 时返回 app_data_dir
      4. 测试: config.data_save_path=Some(path) 时返回自定义路径
    Expected Result: 函数逻辑正确
    Failure Indicators: 逻辑错误或路径处理不完整
    Evidence: .sisyphus/evidence/task-3-data-dir-logic.log
  ```

  **Commit**: NO

- [ ] 4. **添加数据路径验证函数**

  **What to do**:
  - 创建 `validate_data_path(path: &str) -> Result<PathBuf>` 函数：
    - 检查路径是否存在
    - 检查是否为目录
    - 检查是否可写
    - 如果不存在，尝试创建目录
  - 返回验证后的 PathBuf 或错误信息

  **Must NOT do**:
  - 不在路径无效时静默失败

  **Recommended Agent Profile**:
  - **Category**: `quick`
    - Reason: 验证逻辑实现
  - **Skills**: `[]`

  - **Parallelization**:
  - **Can Run In Parallel**: NO
  - **Parallel Group**: Sequential (Wave 2)
  - **Blocks**: Task 5, 6
  - **Blocked By**: Task 1, 2, 3

  **References**:
  - `D:/Demo/memflow/src-tauri/src/app_config.rs` - 配置管理

  **Acceptance Criteria**:

  **QA Scenarios (MANDATORY)**:

  ```
  Scenario: 验证路径验证功能
    Tool: Bash
    Preconditions: 函数已创建
    Steps:
      1. 测试有效路径: 验证通过
      2. 测试无效路径: 返回错误
      3. 测试无权限路径: 返回错误
      4. 测试创建目录: 自动创建不存在的目录
    Expected Result: 验证功能正常工作
    Failure Indicators: 验证逻辑有缺陷
    Evidence: .sisyphus/evidence/task-4-validate-path.log
  ```

  **Commit**: NO

- [ ] 5. **添加 get_data_save_path 命令**

  **What to do**:
  - 在 `src-tauri/src/commands.rs` 添加 Tauri 命令：
    ```rust
    #[tauri::command]
    pub async fn get_data_save_path() -> Result<Option<String>, String>
    ```
  - 实现逻辑：
    - 读取当前配置
    - 返回 data_save_path（如果有）
    - 为 None 时返回默认路径说明

  **Must NOT do**:
  - 不暴露敏感的系统路径信息

  **Recommended Agent Profile**:
  - **Category**: `quick`
    - Reason: Tauri 命令添加
  - **Skills**: `[]`

  - **Parallelization**:
  - **Can Run In Parallel**: NO
  - **Parallel Group**: Sequential (Wave 3)
  - **Blocks**: Task 6, 7, 8
  - **Blocked By**: Task 1, 2, 3, 4

  **References**:
  - `D:/Demo/memflow/src-tauri/src/commands.rs` - 命令定义

  **Acceptance Criteria**:

  **QA Scenarios (MANDATORY)**:

  ```
  Scenario: 验证命令可以正常调用
    Tool: Bash (curl)
    Preconditions: 应用正在运行
    Steps:
      1. 使用 invoke 调用 get_data_save_path
      2. 验证: 返回结果格式正确
      3. 验证: None 时返回默认路径说明
      4. 验证: 有自定义路径时返回实际路径
    Expected Result: 命令正常返回数据路径
    Failure Indicators: 命令失败或返回格式错误
    Evidence: .sisyphus/evidence/task-5-command.log
  ```

  **Commit**: NO

- [ ] 6. **添加 set_data_save_path 命令**

  **What to do**:
  - 在 `src-tauri/src/commands.rs` 添加 Tauri 命令：
    ```rust
    #[tauri::command]
    pub async fn set_data_save_path(path: Option<String>) -> Result<(), String>
    ```
  - 实现逻辑：
    - 如果 path 为 None：清空自定义路径（恢复默认）
    - 如果 path 为 Some：
      - 调用 `validate_data_path()` 验证路径
      - 验证通过后更新配置
      - 可选：询问是否迁移旧数据
  - 保存配置到文件

  **Must NOT do**:
  - 不在路径无效时保存配置
  - 不在未询问用户的情况下迁移数据

  **Recommended Agent Profile**:
  - **Category**: `quick`
    - Reason: Tauri 命令添加
  - **Skills**: `[]`

  - **Parallelization**:
  - **Can Run In Parallel**: NO
  - **Parallel Group**: Sequential (Wave 3)
  - **Blocks**: Task 7, 8
  - **Blocked By**: Task 1, 2, 3, 4, 5

  **References**:
  - `D:/Demo/memflow/src-tauri/src/commands.rs` - 命令定义
  - `D:/Demo/memflow/src-tauri/src/app_config.rs` - 配置管理

  **Acceptance Criteria**:

  **QA Scenarios (MANDATORY)**:

  ```
  Scenario: 验证命令可以正常调用
    Tool: Bash (curl)
    Preconditions: 应用正在运行
    Steps:
      1. 使用 invoke 调用 set_data_save_path(None)
      2. 验证: 配置被清除，恢复默认路径
      3. 使用 invoke 调用 set_data_save_path(Some("D:\\CustomPath"))
      4. 验证: 路径被验证
      5. 验证: 配置被保存
    Expected Result: 命令正常更新数据路径配置
    Failure Indicators: 命令失败或配置未保存
    Evidence: .sisyphus/evidence/task-6-command.log
  ```

  **Commit**: NO

- [ ] 7. **添加数据目录配置 UI**

  **What to do**:
  - 在 `src/components/SettingsModal.tsx` 的设置界面中添加新配置项：
    ```
    📁 数据目录
    当前: C:\Users\xxx\AppData\Roaming\MemFlow
    [更改目录]
    ```
  - 显示当前数据目录路径
  - 添加"更改目录"按钮
  - 点击后调用 `set_data_save_path()` 打开目录选择对话框
  - 选择后更新显示的路径
  - 添加说明文字

  **Must NOT do**:
  - 不修改其他现有设置项
  - 不破坏现有设置布局

  **Recommended Agent Profile**:
  - **Category**: `visual-engineering`
    - Reason: UI 修改需要设计感
    - Reason: 需要与现有 SettingsModal 风格一致
  - **Skills**: `[]`

  - **Skills Evaluated but Omitted**:
    - `playwright`: 不需要浏览器测试
    - `frontend-ui-ux`: 可能有用但现有代码已有模式可参考

  - **Parallelization**:
  - **Can Run In Parallel**: NO
  - **Parallel Group**: Sequential (Wave 4)
  - **Blocks**: Task 8
  - **Blocked By**: Task 1, 2, 3, 4, 5, 6

  **References**:
  - `D:/Demo/memflow/src/components/SettingsModal.tsx` - 现有设置界面
  - `D:/Demo/memflow/docs/STARTUP_DESKTOP.md` - 桌面端使用说明

  **Acceptance Criteria**:

  **QA Scenarios (MANDATORY)**:

  ```
  Scenario: 验证数据目录配置项已添加
    Tool: Read + Bash (Playwright if UI testing needed)
    Preconditions: 设置界面已更新
    Steps:
      1. 启动应用并打开设置
      2. 检查: 设置界面包含"数据目录"部分
      3. 验证: 显示当前数据目录路径
      4. 验证: 有"更改目录"按钮
      5. 点击"更改目录"按钮
      6. 验证: 打开目录选择对话框
      7. 选择一个新目录
      8. 验证: 路径显示更新
    Expected Result: 配置项正确添加，目录选择功能正常
    Failure Indicators: 配置项缺失或功能异常
    Evidence: .sisyphus/evidence/task-7-ui-config.log
  ```

  **Commit**: NO

- [ ] 8. **添加目录选择功能**

  **What to do**:
  - 在设置组件中实现目录选择对话框逻辑：
    - 使用 `@tauri-apps/plugin-dialog` 的 `open` API
    - 仅选择目录（不选择文件）
    - 选择后调用 `set_data_save_path()` 保存
    - 处理取消操作
    - 显示成功/失败提示

  **Must NOT do**:
  - 不允许选择文件（仅目录）
  - 不在没有用户确认的情况下更改路径

  **Recommended Agent Profile**:
  - **Category**: `visual-engineering`
    - Reason: UI 交互逻辑需要设计感
    - Reason: 需要处理用户交互流程
  - **Skills**: `[]`

  - **Skills Evaluated but Omitted**:
    - `playwright`: 不需要浏览器测试

  - **Parallelization**:
  - **Can Run In Parallel**: NO
  - **Parallel Group**: Sequential (Wave 4)
  - **Blocks**: Task 9
  - **Blocked By**: Task 7

  **References**:
  - `D:/Demo/memflow/src/components/SettingsModal.tsx` - 现有设置组件
  - `@tauri-apps/plugin-dialog` 文档: open() API 用法

  **Acceptance Criteria**:

  **QA Scenarios (MANDATORY)**:

  ```
  Scenario: 验证目录选择功能
    Tool: Bash (Playwright if UI testing needed)
    Preconditions: 设置界面已更新
    Steps:
      1. 启动应用并打开设置
      2. 点击"更改目录"按钮
      3. 验证: 打开目录选择对话框（仅显示目录）
      4. 选择一个新目录
      5. 验证: 路径保存成功
      6. 重启应用
      7. 验证: 数据保存到新目录
    Expected Result: 目录选择功能正常工作，数据保存到新位置
    Failure Indicators: 功能异常或数据未保存到新位置
    Evidence: .ysiphus/evidence/task-8-dir-select.log
  ```

  **Commit**: NO

- [ ] 9. **验证配置功能**

  **What to do**:
  - 验证所有新添加的功能正常工作
  - 测试默认路径行为（data_save_path = None）
  - 测试自定义路径行为
  - 测试路径验证功能
  - 测试 UI 交互流程

  **Must NOT do**:
  - 不修改已验证通过的代码

  **Recommended Agent Profile**:
  - **Category**: `quick`
    - Reason: 验证操作
  - **Skills**: `[]`

  - **Parallelization**:
  - **Can Run In Parallel**: YES
  - **Parallel Group**: Wave 5 (with Task 10)
  - **Blocks**: None
  - **Blocked By**: Task 1-8

  **References**:
  - `D:/Demo/memflow/src-tauri/src/app_config.rs` - 配置管理
  - `D:/Demo/memflow/src/components/SettingsModal.tsx` - 设置界面
  - `D:/Demo/memflow/docs/STARTUP_DESKTOP.md` - 功能说明

  **Acceptance Criteria**:

  **QA Scenarios (MANDATORY)**:

  ```
  Scenario: 验证默认路径行为
    Tool: Bash (Playwright if UI testing needed)
    Preconditions: 应用已配置 data_save_path=None
    Steps:
      1. 启动应用
      2. 检查: 数据保存到默认 APPDATA 位置
      3. 添加一些活动记录
      4. 验证: 数据文件存在
    Expected Result: 默认路径功能正常
    Failure Indicators: 数据未保存到默认位置
    Evidence: .sisyphus/evidence/task-9-default-path.log

  Scenario: 验证自定义路径行为
    Tool: Bash (Playwright if UI testing needed)
    Preconditions: 应用已配置自定义路径
    Steps:
      1. 设置自定义数据目录
      2. 重启应用
      3. 添加一些活动记录
      4. 验证: 数据保存到自定义位置
    Expected Result: 自定义路径功能正常
    Failure Indicators: 数据未保存到自定义位置
    Evidence: .ysyphus/evidence/task-9-custom-path.log

  Scenario: 验证路径验证功能
    Tool: Bash (Playwright if UI testing needed)
    Preconditions: 设置界面已打开
    Steps:
      1. 输入无效路径（如不存在的盘符）
      2. 验证: 显示错误提示
      3. 输入无权限路径
      4. 验证: 显示权限错误提示
      5. 输入有效路径
      6. 验证: 验证通过，配置已保存
    Expected Result: 路径验证功能正常工作
    Failure Indicators: 无效路径被接受或验证失败
    Evidence: .ysyphus/evidence/task-9-path-validation.log
  ```

  **Commit**: NO

- [ ] 10. **生成功能报告**

  **What to do**:
  - 生成 `DATA_DIRECTORY_FEATURE_REPORT.md` 报告：
    - 功能描述
    - 技术实现说明
    - 使用说明
    - 已知限制

  **Must NOT do**:
  - 不修改已验证通过的代码

  **Recommended Agent Profile**:
  - **Category**: `writing`
    - Reason: 报告编写
  - **Skills**: `[]`

  - **Parallelization**:
  - **Can Run In Parallel**: YES
  - **Parallel Group**: Wave 5 (with Task 9)
  - **Blocks**: None
  - **Blocked By**: Task 1-9

  **References**:
  - `D:/Demo/memflow/src-tauri/src/app_config.rs` - 配置管理
  - `D:/Demo/memflow/src/components/SettingsModal.tsx` - 设置界面
  - `docs/STARTUP_DESKTOP.md` - 文档

  **Acceptance Criteria**:

  **QA Scenarios (MANDATORY)**:

  ```
  Scenario: 验证功能报告
    Tool: Read
    Preconditions: 所有功能已实现
    Steps:
      1. 读取: D:/Demo/memflow/DATA_DIRECTORY_FEATURE_REPORT.md
      2. 验证: 包含功能描述
      3. 验证: 包含技术说明
      4. 验证: 包含使用说明
    Expected Result: 功能报告完整清晰
    Failure Indicators: 报告内容不完整
    Evidence: .sisyphus/evidence/task-10-report.log
  ```

  **Commit**: NO

---

## Final Verification Wave (MANDATORY — after ALL implementation tasks)

- [ ] F1. **Configuration Verification** — `quick`
  验证所有配置项正确添加，功能按预期工作。

- [ ] F2. **UI Functionality Verification** — `quick`
  验证设置界面新增功能的用户体验。

---

## Commit Strategy

- **Final**: `feat: 添加数据保存目录自定义配置功能`

---

## Success Criteria

### Verification Commands
```bash
# 检查配置字段是否添加
grep "data_save_path" D:/Demo/memflow/src-tauri/src/commands.rs

# 检查 Tauri 命令是否添加
grep "get_data_save_path\|set_data_save_path" D:/Demo/memflow/src-tauri/src/commands.rs

# 检查 UI 配置项是否添加
grep "数据目录\|data.*save.*path\|更改目录" D:/Demo/memflow/src/components/SettingsModal.tsx
```

### Final Checklist
- [ ] AppConfig 包含 data_save_path 字段
- [ ] 支持自定义数据路径或回退到默认
- [ ] 设置界面有"数据目录"配置项
- [ ] 可以通过目录选择对话框选择路径
- [ ] 路径验证功能正常
- [ ] 数据正确保存到指定目录
