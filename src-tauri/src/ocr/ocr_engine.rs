use anyhow::Result;
use async_trait::async_trait;

/// OCR 引擎接口
#[async_trait]
pub trait OcrEngine: Send + Sync {
    /// 识别图像中的文本
    async fn recognize(&self, image_path: &str) -> Result<String>;

    /// 返回引擎名称
    fn name(&self) -> &str;
}
