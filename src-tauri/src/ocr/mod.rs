pub mod ocr_engine;
pub mod rapidocr;
pub mod service;

use crate::ocr::ocr_engine::OcrEngine;
use anyhow::Result;
use std::path::PathBuf;

/// OCR 配置
#[derive(Debug, Clone)]
pub struct OcrConfig {
    /// OCR 引擎类型
    pub engine: String,
    /// 是否启用 OCR
    pub enabled: bool,
    /// 资源目录（用于查找可执行文件和模型）
    pub resource_dir: Option<PathBuf>,
    /// 是否启用 PII 脱敏
    pub redaction_enabled: bool,
    /// PII 脱敏级别 ("basic" | "strict")
    pub redaction_level: String,
}

impl OcrConfig {
    pub fn new(engine: impl Into<String>) -> Self {
        Self {
            engine: engine.into(),
            enabled: true,
            resource_dir: None,
            redaction_enabled: true,
            redaction_level: "basic".to_string(),
        }
    }

    pub fn with_resource_dir(mut self, dir: PathBuf) -> Self {
        self.resource_dir = Some(dir);
        self
    }

    pub fn with_redaction(mut self, enabled: bool) -> Self {
        self.redaction_enabled = enabled;
        self
    }

    pub fn with_redaction_level(mut self, level: impl Into<String>) -> Self {
        self.redaction_level = level.into();
        self
    }
}

/// 处理图像并进行 OCR 识别
pub async fn process_image(image_path: &str, config: OcrConfig) -> Result<String> {
    let image_path = image_path.to_string();
    let redaction_enabled = config.redaction_enabled;
    let redaction_level = config.redaction_level.clone();

    // 选择 OCR 引擎
    let ocr_engine: Box<dyn OcrEngine> = match config.engine.as_str() {
        "rapidocr" => {
            tracing::info!("使用 RapidOCR 引擎");

            let engine = if let Some(ref resource_dir) = config.resource_dir {
                rapidocr::RapidOcrEngine::with_resource_dir(resource_dir.clone())?
            } else {
                rapidocr::RapidOcrEngine::new()?
            };

            Box::new(engine)
        }
        _ => {
            return Err(anyhow::anyhow!("不支持的 OCR 引擎: {}", config.engine));
        }
    };

    tracing::info!("使用 OCR 引擎: {}", ocr_engine.name());

    // 执行 OCR (Async)
    let text = ocr_engine.recognize(&image_path).await?;

    // PII 脱敏
    let text = if redaction_enabled {
        mask_pii(&text, &redaction_level)
    } else {
        text
    };

    Ok(text)
}

/// PII 脱敏处理
fn mask_pii(text: &str, level: &str) -> String {
    use regex::Regex;

    let mut result = text.to_string();
    let is_strict = level == "strict";

    // --- Basic Redaction (Always applied if enabled) ---

    // 邮箱脱敏
    if let Ok(re) = Regex::new(r"\b[A-Za-z0-9._%+-]+@[A-Za-z0-9.-]+\.[A-Za-z]{2,}\b") {
        result = re.replace_all(&result, "[EMAIL_REDACTED]").to_string();
    }

    // 手机号脱敏（11位）
    if let Ok(re) = Regex::new(r"\b1[3-9]\d{9}\b") {
        result = re
            .replace_all(&result, |caps: &regex::Captures| {
                let phone = &caps[0];
                format!("{}****{}", &phone[..3], &phone[7..])
            })
            .to_string();
    }

    // 身份证号脱敏（18位）
    if let Ok(re) = Regex::new(r"\b\d{17}[\dXx]\b") {
        result = re
            .replace_all(&result, |caps: &regex::Captures| {
                let id = &caps[0];
                format!("{}****{}", &id[..6], &id[14..])
            })
            .to_string();
    }

    // 银行卡号脱敏（16-19位）
    if let Ok(re) = Regex::new(r"\b\d{16,19}\b") {
        result = re
            .replace_all(&result, |caps: &regex::Captures| {
                let card = &caps[0];
                format!("{}****{}", &card[..4], &card[card.len() - 4..])
            })
            .to_string();
    }

    if is_strict {
        // --- Strict Redaction ---

        // IPv4 Address
        if let Ok(re) = Regex::new(
            r"\b(?:(?:25[0-5]|2[0-4][0-9]|[01]?[0-9][0-9]?)\.){3}(?:25[0-5]|2[0-4][0-9]|[01]?[0-9][0-9]?)\b",
        ) {
            result = re.replace_all(&result, "[IP_REDACTED]").to_string();
        }

        // MAC Address
        if let Ok(re) = Regex::new(r"\b([0-9A-Fa-f]{2}[:-]){5}([0-9A-Fa-f]{2})\b") {
            result = re.replace_all(&result, "[MAC_REDACTED]").to_string();
        }

        // Currency Amounts (Simple)
        if let Ok(re) = Regex::new(r"(?:[¥$€£]|CNY|USD)\s*\d+(?:,\d{3})*(?:\.\d+)?") {
            result = re.replace_all(&result, "[MONEY_REDACTED]").to_string();
        }

        // Any remaining long number sequences (>6 digits)
        // This is aggressive and might catch non-PII, but that's the point of "strict"
        if let Ok(re) = Regex::new(r"\b\d{7,}\b") {
            result = re.replace_all(&result, "[NUMBER_REDACTED]").to_string();
        }
    }

    result
}
