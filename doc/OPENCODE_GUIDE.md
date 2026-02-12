# OpenCode 与 Oh My OpenCode 使用指南

## 〇、OpenCode 和 Oh My OpenCode（OMO）的关系

- **原生 OpenCode** 只有两种模式：**Plan**（只分析）和 **Build**（可改文件、跑命令），没有 Sisyphus、Atlas 等智能体。
- **Sisyphus / Prometheus / Hephaestus / Atlas** 等都是由 **Oh My OpenCode 插件**提供的；只有装了 OMO 插件才会出现这些名字。
- **只要装了 OMO 插件就会自动用上**：插件放在 `.opencode/plugin/` 或 `~/.config/opencode/plugin/`（或通过 `opencode.json` 的 `plugin` 数组指定 npm 包）后，OpenCode **启动时会自动加载**，无需再手动「启用」。所以如果你在界面上能看到 **Atlas** 或 **Sisyphus**，说明当前已经在用 OMO。
- 若你**不想用 OMO**、只想用原生 Plan/Build：需要从插件目录或配置中**移除/禁用** Oh My OpenCode 插件，重启 OpenCode 后就不会再有 Atlas 等智能体。
- **当前配置都是官方的吗？** 若你没有在 `opencode.json` 或 `oh-my-opencode.json` 里写过 `model` / `agents.xxx.model`，用的就是**官方/内置默认**或**当前会话在 TUI 里选的模型**。要让 OMO 各智能体用你自己的模型，需要在上述配置里显式指定，见下文「OMO 各智能体可以用自己的模型吗？」。

---

## 一、Tab 切换的智能体（Sisyphus / Hephaestus / Prometheus / Atlas …）

**前提**：以下 Tab 切换、智能体说明等，只在**已安装 OMO 插件**时适用；插件安装后**自动加载、无需输入任何命令**即可使用。

在 OMO 已加载时，**Tab** 是在**多个智能体**之间轮换，右下角会显示当前智能体名字。**Shift+Tab** 反向切换。

### Tab 可切换的智能体

| 智能体 | 说明 | 典型用途 |
|--------|------|----------|
| **Sisyphus** | 主智能体，统筹规划与执行，带 Todo 推进 | 默认选它即可，写代码、拆任务、做到完 |
| **Prometheus** | 规划智能体，只出方案、不直接改代码 | 先想清楚再干：出步骤、写计划，再交给 Sisyphus 执行 |
| **Hephaestus** | 深度执行智能体，目标导向、自主研究后动手 | 给一个目标就持续做到完，适合大块/复杂任务 |
| **Atlas** | 高层编排，通过 Atlas Hook 做整体协调 | 多步骤、多仓库或需要统一编排时 |
| （以及 Metis、Momus 等） | 其他专用智能体 | 按插件配置，可能还有规划顾问、评审等 |

- **Tab**：下一个智能体（如 Sisyphus → Prometheus → Hephaestus → Atlas → …）

### 四主智能体详细说明与适用场景

以下分别说明 Sisyphus、Prometheus、Hephaestus、Atlas 四个主智能体的定位、特点及最佳使用场景。

#### 1. Sisyphus（主智能体）

**定位**：默认主智能体，统筹规划与执行。

**特点**：

- 会拆任务、列 Todo、按步骤推进
- 可以改代码、执行命令
- 持续跟踪进度，被打断后也能继续
- 必要时会调用 Oracle、Librarian、Explore 等子智能体

**适用场景**：

- **日常开发**：改功能、修 bug、小重构
- **中等复杂度任务**：加新模块、改多文件、按步骤实现
- **先有粗略想法**：需要它帮忙拆成步骤并执行
- **默认选项**：不确定选谁时用 Sisyphus

**示例**：「给设置页加暗色主题开关」「把这个 API 改成支持分页」「修一下登录失败时的错误提示」

---

#### 2. Prometheus（规划智能体）

**定位**：只负责规划和拆解，不改代码、不执行。

**特点**：

- 专注分析和方案设计
- 输出步骤、架构思路、实现建议
- 不做编辑、不改文件、不跑命令

**适用场景**：

- **大任务先做方案**：先理清思路再动手
- **架构讨论**：如何拆分、如何扩展、技术选型
- **需求梳理**：把模糊需求变成清晰的实现步骤
- **配合 Sisyphus 执行**：先用 Prometheus 出计划，再交给 Sisyphus 按计划执行

**推荐流程**：切到 Prometheus → 描述需求 → 拿到方案 → 切回 Sisyphus → 输入 `/start-work` 按计划执行。

**示例**：「设计一个支持多租户的权限系统」「这个需求该怎么拆成任务」「帮我分析下这个模块的依赖和扩展点」

---

#### 3. Hephaestus（深度执行智能体）

**定位**：自主深度执行，给一个目标就持续做到完成。

**特点**：

- 更偏向「目标导向」而非「步骤导向」
- 会自己研究、查资料、试错，然后动手实现
- 适合长时间、多步骤、需要自主探索的任务

**适用场景**：

- **任务复杂、步骤多**：实现一个完整功能、接一个 SDK、做复杂重构
- **目标清晰但实现路径不清晰**：需要它自己查文档、调 API、试错
- **大块工作**：例如「接好支付」「实现 OAuth 登录」
- **不想频繁发指令**：希望它自动推进到完成

**与 Sisyphus 的差异**：Sisyphus 适合你带着步骤一起推进；Hephaestus 适合你给一个终点，它自己找路走过去。

**示例**：「把 Stripe 支付集成进来」「实现基于 JWT 的认证流程」「把这个旧模块迁移到新架构」

---

#### 4. Atlas（编排智能体）

**定位**：高层编排与协调，通过 Atlas Hook 做整体协调。

**特点**：

- 负责跨步骤、跨仓库的协调
- 通过 Hook 串联多个任务和流程
- 偏「指挥者」，而不是直接写代码

**适用场景**：

- **多步骤流水线**：构建 → 测试 → 部署
- **多仓库协作**：主仓库 + 子模块、多项目一起改
- **需要统一规划的大规模改动**：例如跨仓库的重命名、大重构
- **有预定义 Hook 或工作流**：用 Atlas 做整体编排

**与其它智能体的关系**：Sisyphus 偏单项目执行，Atlas 偏跨项目/跨流程的整体协调。

**示例**：「同时改 main 和 submodule，并保持同步」「按 CI 流程完成构建、测试和部署」「规划并协调一次跨 3 个仓库的大重构」

---

#### 场景速查表

| 场景 | 推荐智能体 |
|------|------------|
| 日常改代码、修 bug、小功能 | **Sisyphus** |
| 先要方案再写代码 | **Prometheus** → 再切 Sisyphus + `/start-work` |
| 大块、复杂、需要自主探索的任务 | **Hephaestus** |
| 多仓库、多步骤、流水线式工作 | **Atlas** |
| 不确定选谁 | **Sisyphus** |
| 想多智能体协同（含 Oracle、Librarian 等） | 消息里加 **`ulw`** 让主智能体调度 |

---

### 为什么一切到 Atlas 就变成 Kimi（或其他非预期模型）？

这是 **Oh My OpenCode 的已知行为**：Atlas 智能体有时会**忽略**你在 `oh-my-opencode.json` 里为它配置的模型，回退到「当前会话默认模型」或内置默认。若你当前会话/全局默认是 Zen 的 `kimi-k2.5-free`，切到 Atlas 时就会显示「Atlas · kimi-k2.5-free」。

**解决办法**：在配置里**显式指定** Atlas 要用的模型。

- **配置文件位置**（任选其一或同时存在，项目级会覆盖全局）：
  - 全局：`~/.config/opencode/oh-my-opencode.json`（Windows：`%USERPROFILE%\.config\opencode\oh-my-opencode.json`）
  - 项目级：项目根目录下 `.opencode/oh-my-opencode.json`
- **配置示例**（把 `provider/模型ID` 换成你实际要用的，如 `anthropic/claude-sonnet-4-5`、`openai/gpt-5.2-codex` 等）：

```json
{
  "agents": {
    "Atlas": { "model": "anthropic/claude-sonnet-4-5" },
    "atlas": { "model": "anthropic/claude-sonnet-4-5" }
  }
}
```

- 同时写 `"Atlas"` 和 `"atlas"` 可避免旧版本因**大小写**导致配置不生效的问题。
- 若配置后仍不生效，可升级 Oh My OpenCode 到最新版（相关 Issue：[#1006](https://github.com/code-yeongyu/oh-my-opencode/issues/1006)、[#641](https://github.com/code-yeongyu/oh-my-opencode/issues/641)）。

- **Shift+Tab**：上一个智能体

### OMO 各智能体可以用自己的模型吗？当前配置是官方的吗？

**可以。** OMO 的 Sisyphus、Prometheus、Hephaestus、Atlas 等都可以在配置里指定**你自己要用的模型**，不一定要用官方默认。

- **没配置时**：用的就是「官方/内置默认」或**当前会话在 TUI 里选的模型**（例如你在 `/models` 里选过 Zen 的 Kimi，各智能体就可能回退到该会话默认）。所以你现在看到的配置本质上就是「默认行为」。
- **要用自己的模型**，需要改两类配置（格式均为 `提供商ID/模型ID`，例如 `anthropic/claude-sonnet-4-5`、`openai/gpt-5.2-codex`、Zen 的 `opencode/xxx` 等，可用 TUI 的 **`/models`** 或 OpenCode 文档查看当前可用的 provider/model 列表）：

| 作用范围 | 配置文件 | 说明 |
|----------|----------|------|
| **全局/会话默认** | **`opencode.json`**（项目根或 `~/.config/opencode/opencode.json`） | 顶层的 `"model": "provider/model-id"` 决定默认模型；在 TUI 里 `/models` 切换的也是会话模型，会作为「未单独配置的智能体」的回退。 |
| **每个 OMO 智能体** | **`oh-my-opencode.json`**（`.opencode/oh-my-opencode.json` 或 `~/.config/opencode/oh-my-opencode.json`） | 在 `agents` 里为每个智能体写 `"model": "provider/model-id"`，该智能体就会尽量用你指定的模型。 |

**示例**：让 Sisyphus、Prometheus、Hephaestus、Atlas 分别用不同模型（按需改成你自己的 provider/model-id）：

```json
{
  "agents": {
    "Sisyphus": { "model": "anthropic/claude-sonnet-4-5" },
    "sisyphus": { "model": "anthropic/claude-sonnet-4-5" },
    "Prometheus": { "model": "openai/gpt-5.2-codex" },
    "prometheus": { "model": "openai/gpt-5.2-codex" },
    "Hephaestus": { "model": "anthropic/claude-sonnet-4-5" },
    "hephaestus": { "model": "anthropic/claude-sonnet-4-5" },
    "Atlas": { "model": "anthropic/claude-sonnet-4-5" },
    "atlas": { "model": "anthropic/claude-sonnet-4-5" }
  }
}
```

- 同时写首字母大写和小写（如 `"Atlas"` 和 `"atlas"`）可以避免旧版 OMO 因**大小写**导致配置不生效。
- **已知问题**：部分版本中某些智能体（如 Atlas、Explore）会忽略 `oh-my-opencode.json` 里的 `model`，回退到会话默认；若遇到，可尝试升级 Oh My OpenCode，或先在 **`opencode.json`** 里设好 `model` 作为全局/会话默认，再在 `oh-my-opencode.json` 里按需覆盖各智能体。

### 本指南采用的智能体与 Categories 模型配置（参考）

下面是一套按「执行省 token、规划/编排用强模型、混用 Codex / Claude / GLM」的配置，可直接抄进 `oh-my-opencode.json`（注意把 `wyzai`、`zhipu` 换成你自己的 provider ID，模型 ID 按你在 `/models` 里看到的为准）。

**Agents（智能体）**

| 智能体 | 用途 | 模型 |
|--------|------|------|
| Sisyphus | 执行、写代码 | zhipu/glm-4.7 |
| Prometheus | 规划、拆任务 | wyzai/claude-opus-4-5-20250929 |
| Hephaestus | 深度执行 | wyzai/claude-sonnet-4-5-20250929 |
| Atlas | 编排协调 | wyzai/claude-sonnet-4-5-20250929 |
| Oracle | 架构/设计 | wyzai/gpt-5.2-codex |
| Metis | 规划顾问 | wyzai/gpt-5.3-codex |
| explore | 搜代码 | wyzai/claude-haiku-4-5-20251001 |
| librarian | 查文档 | zhipu/glm-4.7 |
| momus | 评审等 | zhipu/glm-4.7 |
| multimodal-looker | 多模态 | wyzai/MiniMax-M2.1 |

**Categories（任务类型 → 模型）**

| Category | 模型 |
|----------|------|
| visual-engineering / artistry | wyzai/MiniMax-M2.1 |
| ultrabrain | wyzai/claude-opus-4-5-20250929 |
| deep | wyzai/claude-sonnet-4-5-20250929 |
| quick / unspecified-low | zhipu/glm-4.7 |
| unspecified-high | wyzai/gpt-5.2-codex |
| writing | wyzai/claude-haiku-4-5-20251001 |

配置路径：全局 `~/.config/opencode/oh-my-opencode.json`（Windows：`%USERPROFILE%\.config\opencode\oh-my-opencode.json`），或项目 `.opencode/oh-my-opencode.json`。

### 和 OpenCode 自带的 Plan/Build 关系

OpenCode 自带两种**模式**：**Plan**（只分析、不出手）和 **Build**（可改文件、跑命令）。  
在 OMO 里，你按 Tab 换的是**智能体**：选 **Prometheus** 相当于偏「规划」；选 **Sisyphus** 或 **Hephaestus** 相当于「构建/执行」。不同智能体背后会用到不同的模型和工具权限。

### 推荐用法（OMO 加载后）

1. 想**先要方案再写代码**：**Tab** 切到 **Prometheus** → 描述需求 → 看计划 → **Tab** 切回 **Sisyphus** → 输入 **`/start-work`** 按计划执行。
2. 想**丢一个目标让它做到完**：**Tab** 切到 **Hephaestus**，直接说目标。
3. **日常开发**：保持 **Sisyphus**；需要更强协作时在句子里加 **`ulw`** 启用全力模式，让主智能体自动调度子智能体。

---

## 二、OpenCode 斜杠命令（/xxx）

在 TUI 输入框输入 `/` 加命令名即可。

### 会话与连接

| 命令 | 说明 |
|------|------|
| `/connect` | 连接/添加提供商（Zen、Claude、OpenAI 等），输入 API Key |
| `/models` | 打开模型列表，切换当前使用的模型 |
| `/new` | 新建会话（别名：`/clear`） |
| `/sessions` | 列出会话并切换（别名：`/resume`、`/continue`） |
| `/exit` | 退出 OpenCode（别名：`/quit`、`/q`） |

**历史会话怎么删？** TUI 里**没有**「删除会话」的斜杠命令。要删掉某条历史会话只能：**(1)** 在会话列表（`/sessions` 或 **Ctrl+x l**）里看是否有删除入口（视版本而定）；**(2)** 或直接删文件：会话存在本机目录，例如 Windows 为 `%USERPROFILE%\.local\share\opencode\storage\session\<项目ID>\`，Linux/macOS 为 `~/.local/share/opencode/storage/session/<项目ID>/`，删除对应 `<sessionID>.json` 即删除该会话。删前请先退出 OpenCode 或切到其他会话，避免正在使用该会话时删文件。

### 对话与撤销

| 命令 | 说明 |
|------|------|
| `/undo` | 撤销上一条用户消息及其后的回复与文件改动 |
| `/redo` | 在 `/undo` 之后恢复刚才撤销的内容 |
| `/compact` | 压缩当前会话上下文（别名：`/summarize`） |
| `/thinking` | 开关「思考过程」块的显示（不改变模型是否推理） |

### 分享与导出

| 命令 | 说明 |
|------|------|
| `/share` | 生成当前会话的分享链接并复制到剪贴板 |
| `/unshare` | 取消分享 |
| `/export` | 将会话导出为 Markdown，用默认编辑器打开 |

### 项目与编辑

| 命令 | 说明 |
|------|------|
| `/init` | 为当前项目创建/更新 `AGENTS.md`，方便 AI 理解项目 |
| `/editor` | 用外部编辑器（由 `EDITOR` 环境变量指定）编写长消息 |
| `/details` | 开关工具执行详情显示 |
| `/help` | 显示帮助与快捷键 |
| `/theme` | 打开主题列表切换主题（文档里也写作 `/themes`） |

### 仅当已安装 OMO 插件时提供的命令

OMO 插件加载后会自动提供以下命令（无需额外启用）：

| 命令 | 说明 |
|------|------|
| `/start-work` | 在 Build 模式下，按当前规划执行并开始改代码 |

---

## 三、快捷键（Keybinds）

默认 **Leader 键**为 **Ctrl+x**：先按 `Ctrl+x`，再按下面的第二键。

### 常用

| 按键 | 作用 |
|------|------|
| **Ctrl+x** 然后 **m** | 打开模型列表（等同 `/models`） |
| **Ctrl+x** 然后 **n** | 新建会话 |
| **Ctrl+x** 然后 **q** | 退出 |
| **Ctrl+x** 然后 **h** | 帮助 |
| **Ctrl+x** 然后 **c** | 压缩会话 |
| **Ctrl+x** 然后 **u** | 撤销 |
| **Ctrl+x** 然后 **r** | 重做 |
| **Ctrl+x** 然后 **s** | 分享 / 状态视图（视版本） |
| **Ctrl+x** 然后 **x** | 导出会话 |
| **Ctrl+x** 然后 **e** | 打开外部编辑器写消息 |
| **Ctrl+x** 然后 **i** | 执行 `/init` |
| **Ctrl+x** 然后 **l** | 会话列表 |
| **Ctrl+x** 然后 **t** | 主题列表 |
| **Ctrl+x** 然后 **a** | 智能体列表（仅 OMO 加载时可见/可用） |
| **Tab** | 切换智能体（Sisyphus / Prometheus / Hephaestus / Atlas …，仅 OMO 加载时） |
| **Shift+Tab** | 反向切换智能体 |
| **Ctrl+t** | 切换模型 variant（如 low / high / max） |
| **Ctrl+p** | 打开**命令面板**（Commands），可搜索并执行下列命令 |
| **Esc** | 中断当前回复 / 关闭弹窗 |

### 命令面板（Ctrl+p）里有哪些

按 **Ctrl+p** 会打开 **Commands** 面板，支持搜索，**esc** 关闭。常见分组如下：

| 分类 | 命令 | 快捷键 | 说明 |
|------|------|--------|------|
| **Suggested** | Switch session | Ctrl+x l | 切换会话（会话列表） |
| | Switch model | Ctrl+x m | 切换模型 |
| **Prompt** | Stash prompt | — | 暂存当前输入内容 |
| | Skills | — | 技能 / 快捷能力 |
| **Session** | （及其他会话相关） | — | 新建、导出、分享等 |

面板里会按分类显示更多项，直接选或输入关键词搜索即可；每个命令右侧会显示对应快捷键（若有）。

| 按键 | 作用 |
|------|------|
| **Enter** | 发送消息 |
| **Shift+Enter** / **Ctrl+Enter** | 换行，不发送 |
| **Ctrl+c** | 清空输入 / 取消 |
| **Ctrl+v** | 粘贴 |
| **Page Up** / **Page Down** | 消息区域上下翻页 |
| **Ctrl+g** / **Home** | 跳到第一条消息 |
| **Ctrl+Alt+g** / **End** | 跳到最后一条消息 |

---

## 三.五、Variants（推理档位：low / medium / high）

**Variants** 是**同一模型**下的不同「推理档位」：控制模型花多少 token 做思考（reasoning），从而在**速度/成本**和**质量**之间取舍。

- **low**：推理少，回答快、省 token，适合简单问答、小改。
- **medium**：中等推理，平衡速度和效果。
- **high**：推理多，逻辑更稳、代码质量更好，更慢、更费。
- **max**（部分模型）：最大推理预算，适合最难的任务。

不同提供商的叫法略有差异，例如：

| 提供商 | 常见 Variants |
|--------|----------------|
| **Anthropic (Claude)** | `high`（默认）、`max` |
| **OpenAI** | `none`、`minimal`、`low`、`medium`、`high`、`xhigh` |
| **Google** | `low`、`high` |

在 TUI 里按 **Ctrl+t** 可在当前模型的多个 variant 之间轮换，右下角会显示当前档位（如 `max`、`high`）。

---

## 四、Oh My OpenCode 魔法词与智能体

**说明**：OMO 插件**安装即自动启用**，不需要输入任何命令来「打开 OMO」。下面说的魔法词 **`ulw`** 是在 OMO **已经启用**的前提下，用来开启「全力模式」的，不是用来启动 OMO 的。

### 魔法词（全力模式）

在**已启用 OMO** 的前提下，在任意一条消息里加上：

- **`ultrawork`** 或 **`ulw`**

会启用 OMO 的「**全力模式**」：多智能体协作、后台任务、深度探索，并尽量把任务做到完成。

示例：

```text
ulw 给设置页加一个暗色主题开关，并和现有主题联动
```

### 主要智能体（OMO 加载后可用 Tab 切换或自动调度）

| 智能体 | 用途 |
|--------|------|
| **Sisyphus** | 主智能体，统筹与写代码（Tab 可切到） |
| **Prometheus** | 规划与拆解任务，只出方案不改代码（Tab 可切到） |
| **Hephaestus** | 深度执行，给目标就做到完（Tab 可切到） |
| **Atlas** | 高层编排与协调（Tab 可切到） |
| **Oracle** | 架构、调试、设计决策（多由 Sisyphus 等调用） |
| **Librarian** | 查文档与开源实现（多由 Sisyphus 等调用） |
| **Explore** | 在仓库内快速搜代码（多由 Sisyphus 等调用） |

用 **Tab** 可手动切换到 Sisyphus、Hephaestus、Prometheus、Atlas 等；不切换时，在句子里加 **`ulw`** 会由主智能体自动调度子智能体（Oracle、Librarian、Explore 等），进入全力模式。

---

## 五、开发时如何选模型（省额度）

以下按任务选模型的建议在 **OMO 已加载**（或原生 OpenCode）时都适用。当前可用：**GLM 4.7**、**Codex 5.3 / 5.2**、**Claude 4.6 / 4.5**、**Doubao-Seed-Code**（额度较少）。若 Codex 5.3 分析一次就占不少额度，且有多项目，建议按任务分层用模型。

### 按任务选模型（推荐）

| 任务类型 | 推荐模型 | 说明 |
|----------|----------|------|
| **全量分析 / 架构梳理 / 大方案** | 少用 Codex 5.3 | 耗额大，留给「真需要最强推理」的少数会话；多项目时尤其省着用。 |
| **先出方案再写代码** | **Prometheus + Codex 5.2** 或 **Claude 4.5** | 规划用 5.2/4.5 足够，把 5.3/4.6 留给执行或难点。 |
| **日常实现、改代码、小重构** | **Codex 5.2** 或 **Claude 4.5** | 性价比好，多项目轮着用也不会很快见底。 |
| **难点调试、复杂逻辑、大重构** | **Codex 5.3** 或 **Claude 4.6** | 只在关键会话用，避免每个小需求都上 5.3。 |
| **简单修改、查代码、解释、中文问答** | **GLM 4.7** | 省额度，中文友好，适合大量轻量交互。 |
| **重复性、模板类、小改** | **Doubao-Seed-Code**（额度允许时）或 **GLM 4.7** | Doubao 额度少就优先用 GLM 兜底。 |

### 多项目时的习惯

- **新项目 / 第一次看代码**：用 **GLM 4.7** 或 **Codex 5.2** 先做「理解 + 小改」，确认值得深度投入再用 **5.3/4.6** 做一次集中分析或设计。
- **同一需求先规划再执行**：**Prometheus（5.2 或 4.5）** 出步骤 → **Sisyphus（5.2/4.5 或 5.3/4.6）** 执行，减少 5.3 的「试探性」长对话。
- **固定习惯**：**Ctrl+x m**（`/models`）按任务切换模型，而不是全程 5.3。

### 小结

- **Codex 5.3 / Claude 4.6**：留给少数「值得花额度」的会话（架构、难点、大重构）。
- **Codex 5.2 / Claude 4.5**：日常主力（方案 + 实现）。
- **GLM 4.7**：大量轻量工作（查代码、小改、中文、多项目试水）。
- **Doubao-Seed-Code**：额度允许时用于轻量、重复类任务。

---

## 六、推荐流程小结

1. **OMO**：安装插件后**自动启用**，无需输入任何命令；若不想用 OMO，需从插件目录或配置中移除该插件。
2. **首次**：在项目根目录执行 **`/init`**，生成/更新 `AGENTS.md`。
3. **日常**：直接描述需求；需要「做到完」或多智能体协作时，在句子里加 **`ulw`**（全力模式）。
4. **先规划再写代码**：**Tab** 切到 **Prometheus** → 说需求 → 看计划 → **Tab** 切回 **Sisyphus** → 输入 **`/start-work`**。
5. **换模型**：**Ctrl+x** → **m**，或输入 **`/models`**，按上面表格按任务选模型。
6. **连新 API**：**`/connect`** 或 **Ctrl+a**（在选模型界面里「Connect provider」）。

---

## 七、省 Token 建议

OpenCode（含 OMO）容易消耗大量 token，主要原因包括：推理档位高（high/max）、全力模式多智能体协作（`ulw`）、会话历史过长、自动读取大量项目文件。以下做法可显著降低消耗：

| 做法 | 说明 |
|------|------|
| **降推理档位** | 按 **Ctrl+t** 将 variant 从 `high`/`max` 调到 **low** 或 **medium**。简单问答、小改用 **low** 即可。 |
| **定期压缩上下文** | 输入 **`/compact`**（或 **Ctrl+x** → **c**）压缩当前会话，用摘要替代完整历史，后续请求更省。 |
| **长会话开新会话** | 任务告一段落后用 **`/new`** 开新会话，避免单会话拖得过长。 |
| **简单任务不用 ulw** | 小改、单文件、需求明确时直接描述即可（OMO 下用 Sisyphus 即可），不要加 `ulw`，避免触发全力模式与深度探索。 |
| **先规划再执行** | 大任务先 **Tab** 到 **Prometheus** 只要方案，再切回 **Sisyphus** 用 **`/start-work`** 执行，有时比一直让主智能体自己探索更省。 |

详见上文 **三.五、Variants** 与 **二、斜杠命令** 中的 `/compact`、`/new`。

---

## 八、参考链接

- [OpenCode 官方文档](https://opencode.ai/docs)
- [OpenCode TUI 命令](https://opencode.ai/docs/tui)
- [OpenCode 快捷键](https://opencode.ai/docs/keybinds)
- [OpenCode 模式（Plan/Build）](https://opencode.ai/docs/modes)
- [Oh My OpenCode 仓库](https://github.com/code-yeongyu/oh-my-opencode)
