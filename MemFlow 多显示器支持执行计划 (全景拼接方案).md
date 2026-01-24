# MemFlow 多显示器支持执行计划 (全景拼接方案)

本计划旨在解决当前仅录制主屏幕的局限，通过**全景拼接 (Panorama Stitching)** 技术，将所有显示器的画面合并为一张全景图进行处理。此方案无需修改数据库结构，兼容性最强，且能完整保留用户的“全景工作台”上下文。

## 1. 图像采集层：全景拼接 (src-tauri/src/recorder.rs)
- **目标**: 捕获所有活动显示器，按物理位置拼接为一张 `DynamicImage`。
- **逻辑**:
    1.  获取所有显示器 `xcap::Monitor::all()`。
    2.  计算总画布大小（覆盖所有显示器的 bounding box）。
    3.  创建一个空的 RGBA 画布。
    4.  遍历每个显示器，`capture_image()` 并按其 `x, y` 坐标贴到画布上。
- **代码变更**: 重写 `capture_screen` 函数。
    ```rust
    fn capture_all_screens() -> Result<DynamicImage> {
        let monitors = xcap::Monitor::all()?;
        // 1. 计算画布边界
        let min_x = monitors.iter().map(|m| m.x()).min().unwrap_or(0);
        let min_y = monitors.iter().map(|m| m.y()).min().unwrap_or(0);
        let max_x = monitors.iter().map(|m| m.x() + m.width() as i32).max().unwrap_or(1920);
        let max_y = monitors.iter().map(|m| m.y() + m.height() as i32).max().unwrap_or(1080);
        
        let width = (max_x - min_x) as u32;
        let height = (max_y - min_y) as u32;
        
        // 2. 创建画布
        let mut panorama = RgbaImage::new(width, height);
        
        // 3. 拼接
        for monitor in monitors {
            let img = monitor.capture_image()?;
            // 转换 xcap image 到 image::RgbaImage 并贴图
            // 注意坐标系转换 (monitor.x - min_x, monitor.y - min_y)
            overlay(&mut panorama, &img, (monitor.x() - min_x) as u32, ...);
        }
        Ok(DynamicImage::ImageRgba8(panorama))
    }
    ```

## 2. 性能优化层：并行采集 (Async Capture)
- **隐患**: 串行截图会导致延迟累积（3个屏幕可能需要 300ms）。
- **优化**: 使用 `rayon` 并行迭代器或 `std::thread` 并行采集所有屏幕，然后再拼接。

## 3. 智能处理适配
- **UIA**: `get_foreground_text` 依然有效，因为它基于 Window Handle，无论窗口在哪个屏幕都能获取。
- **OCR**: RapidOCR 能够处理大图，但耗时会增加。建议在 `OcrConfig` 中开启多线程支持。
- **存储**: WebP 对大片黑色区域（非矩形拼接产生的空白）压缩极佳，文件体积不会成倍增加。

**执行确认**:
此方案将立即启用全屏录制，无需迁移数据库。是否执行？
