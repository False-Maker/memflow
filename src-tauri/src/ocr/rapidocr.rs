use crate::ocr::ocr_engine::OcrEngine;
use anyhow::{Context, Result};
use async_trait::async_trait;
use reqwest::multipart;
use std::path::Path;

/// RapidOCR API 服务地址
const DEFAULT_API_URL: &str = "http://127.0.0.1:9003/ocr";

pub struct RapidOcrEngine {
    api_url: String,
    client: reqwest::Client,
}

impl RapidOcrEngine {
    /// 创建新的 RapidOCR 引擎实例（HTTP API 版本）
    pub fn new() -> Result<Self> {
        let api_url =
            std::env::var("RAPIDOCR_API_URL").unwrap_or_else(|_| DEFAULT_API_URL.to_string());

        let client = reqwest::Client::builder()
            .timeout(std::time::Duration::from_secs(30))
            .no_proxy()
            .build()
            .context("创建 HTTP 客户端失败")?;

        Ok(Self { api_url, client })
    }

    /// 从资源目录创建（保持 API 兼容性）
    pub fn with_resource_dir(_resource_dir: std::path::PathBuf) -> Result<Self> {
        // API 版本不需要资源目录，直接创建默认实例
        Self::new()
    }

    /// 设置 API 地址
    pub fn with_api_url(mut self, url: String) -> Self {
        self.api_url = url;
        self
    }

    /// 检查 OCR 服务是否可用
    pub async fn is_service_available(&self) -> bool {
        // 尝试请求服务的 docs 页面
        let docs_url = self.api_url.replace("/ocr", "/docs");
        self.client
            .get(&docs_url)
            .timeout(std::time::Duration::from_secs(2))
            .send()
            .await
            .is_ok()
    }
}

#[async_trait]
impl OcrEngine for RapidOcrEngine {
    async fn recognize(&self, image_path: &str) -> Result<String> {
        // 验证图片文件存在
        let image_path = Path::new(image_path);
        if !image_path.exists() {
            return Err(anyhow::anyhow!("图片文件不存在: {}", image_path.display()));
        }

        // 读取图片文件 (async read would be better, but file is local, spawn_blocking or std::fs::read is okay for small files)
        // However, better to use tokio::fs if we are fully async, or just std::fs::read since it's fast on SSD usually.
        // For correctness in async context, let's use tokio::fs or just keep std::fs::read but wrap if needed.
        // reqwest multipart expects a stream or bytes.
        let image_bytes = std::fs::read(image_path)
            .with_context(|| format!("读取图片失败: {}", image_path.display()))?;

        // 获取文件名
        let filename = image_path
            .file_name()
            .and_then(|n| n.to_str())
            .unwrap_or("image.png")
            .to_string();

        // 构建 multipart 表单
        // reqwest async multipart
        let part = multipart::Part::bytes(image_bytes)
            .file_name(filename)
            .mime_str("image/png")?;

        let form = multipart::Form::new().part("image", part);

        tracing::debug!("发送 OCR 请求到: {}", self.api_url);

        // 发送请求
        let response = self
            .client
            .post(&self.api_url)
            .multipart(form)
            .send()
            .await
            .with_context(|| format!("OCR 请求失败，服务地址: {}", self.api_url))?;

        if !response.status().is_success() {
            return Err(anyhow::anyhow!(
                "OCR 服务返回错误: {} - {}",
                response.status(),
                response.text().await.unwrap_or_default()
            ));
        }

        // 解析响应
        let result: serde_json::Value = response.json().await.context("解析 OCR 响应 JSON 失败")?;

        // 提取文本
        // rapidocr_api 返回格式: {"0": {"rec_txt": "文本", "dt_boxes": [...], "score": "0.9"}, ...}
        let mut texts = Vec::new();

        if let Some(obj) = result.as_object() {
            // 按 key 排序（"0", "1", "2"...）
            let mut keys: Vec<_> = obj.keys().collect();
            keys.sort_by(|a, b| {
                a.parse::<i32>()
                    .unwrap_or(0)
                    .cmp(&b.parse::<i32>().unwrap_or(0))
            });

            for key in keys {
                if let Some(item) = obj.get(key) {
                    if let Some(text) = item.get("rec_txt").and_then(|v| v.as_str()) {
                        texts.push(text.to_string());
                    }
                }
            }
        }

        Ok(texts.join("\n"))
    }

    fn name(&self) -> &str {
        "RapidOCR (API)"
    }
}
