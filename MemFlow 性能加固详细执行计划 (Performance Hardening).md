# MemFlow 性能加固详细执行计划 (Performance Hardening)

本计划专注于解决 UIA 遍历可能导致的卡顿问题，以及优化存储效率。

## Phase 1: UIA 遍历性能熔断 (src-tauri/src/uia.rs)
**目标**: 防止在复杂 UI（如 IDE、浏览器）中因遍历过多节点导致主线程挂起。

### 1.1 引入受控遍历 (Controlled Traversal)
- **原理**: 废弃 `FindAll(TreeScope_Descendants)`（这是黑盒，不可控），改用 `TreeWalker` 手动递归。
- **逻辑**:
    ```rust
    fn walk_tree(element: &IUIAutomationElement, depth: u32, max_depth: u32, start_time: Instant) -> Result<String> {
        if depth > max_depth || start_time.elapsed().as_millis() > 200 { 
            return Ok(String::new()); // 超时或超深，立即截断
        }
        // ... 获取当前节点文本 ...
        // ... 遍历 FirstChild -> NextSibling ...
    }
    ```
- **实现步骤**:
    1.  修改 `get_window_text_content`，使用 `IUIAutomation::CreateTreeWalker`。
    2.  实现递归函数，硬限制深度为 **5 层**，超时时间 **200ms**。

## Phase 2: 图像存储 WebP 化 (src-tauri/src/recorder.rs)
**目标**: 将截图体积减少 70%，减少磁盘 I/O 压力。

### 2.1 切换编码格式
- **逻辑**:
    ```rust
    // 原逻辑: image.save_with_format(path, Png)
    // 新逻辑:
    use webp::{Encoder, WebPMemory};
    let encoder = Encoder::from_image(&screenshot).map_err(...)?;
    let memory = encoder.encode(80.0); // 80% 质量，兼顾清晰度与体积
    std::fs::write(&path_with_webp_ext, memory.as_bytes())?;
    ```
- **实现步骤**:
    1.  确认 `Cargo.toml` 包含 `webp` crate。
    2.  修改 `capture_and_save` 中的保存逻辑，文件扩展名改为 `.webp`。

## Phase 3: 智能混合去重 (src-tauri/src/recorder.rs)
**目标**: 解决“打字时画面没变但文本变了”导致的漏录问题。

### 3.1 引入文本状态追踪
- **逻辑**:
    - 新增全局状态: `static LAST_TEXT_HASH: Lazy<Mutex<Option<u64>>>`。
    - 计算当前 UIA 文本的 Hash (使用 `std::collections::hash_map::DefaultHasher`)。
- **修改去重判断**:
    ```rust
    let text_changed = current_text_hash != last_text_hash;
    let visual_changed = distance > DEDUP_HAMMING_THRESHOLD;
    
    if !text_changed && !visual_changed {
        return Ok(()); // 只有当画面和文本都没变时，才跳过
    }
    ```

**执行顺序**: 建议优先执行 **Phase 2 (WebP)**，因为收益最立竿见影且风险最小；然后是 **Phase 3**；最后攻坚 **Phase 1**。

**是否开始执行？**
