use anyhow::Result;

/// OCR 引擎接口
pub trait OcrEngine: Send + Sync {
    /// 识别图像中的文本
    fn recognize(&self, image_path: &str) -> Result<String>;

    /// 返回引擎名称
    fn name(&self) -> &str;
}
