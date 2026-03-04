//! OCR processing module for MemFlow Core
//!
//! This module provides OCR functionality for activity records.
//! Supports multiple OCR backends:
//! - RapidOCR API (local HTTP service)
//! - Tesseract (system binary)

use anyhow::{Context, Result};
use std::path::Path;
use std::time::Instant;

/// OCR result
#[derive(Debug, Clone)]
pub struct OcrResult {
    /// Extracted text
    pub text: String,
    /// Confidence score (0-1)
    pub confidence: f64,
    /// Processing time in milliseconds
    pub processing_time_ms: u64,
}

/// OCR engine type
#[derive(Debug, Clone, Default)]
pub enum OcrEngineType {
    /// RapidOCR HTTP API
    #[default]
    RapidOCR,
    /// Tesseract CLI
    Tesseract,
}

/// OCR configuration
#[derive(Debug, Clone)]
pub struct OcrConfig {
    /// Engine type
    pub engine: OcrEngineType,
    /// RapidOCR API URL
    pub rapidocr_url: String,
    /// Tesseract command path
    pub tesseract_cmd: String,
    /// Tesseract language
    pub tesseract_lang: String,
}

impl Default for OcrConfig {
    fn default() -> Self {
        Self {
            engine: OcrEngineType::RapidOCR,
            rapidocr_url: "http://127.0.0.1:9003/ocr".to_string(),
            tesseract_cmd: "tesseract".to_string(),
            tesseract_lang: "eng+chi_sim".to_string(),
        }
    }
}

/// Process an image and extract text using OCR
pub async fn process_ocr(image_path: &str) -> Result<OcrResult> {
    process_ocr_with_config(image_path, &OcrConfig::default()).await
}

/// Process an image with custom OCR configuration
pub async fn process_ocr_with_config(image_path: &str, config: &OcrConfig) -> Result<OcrResult> {
    match config.engine {
        OcrEngineType::RapidOCR => process_rapidocr(image_path, &config.rapidocr_url).await,
        OcrEngineType::Tesseract => process_tesseract(image_path, &config.tesseract_cmd, &config.tesseract_lang).await,
    }
}

/// Process OCR using RapidOCR HTTP API
async fn process_rapidocr(image_path: &str, api_url: &str) -> Result<OcrResult> {
    let start = Instant::now();

    let image_path = Path::new(image_path);
    if !image_path.exists() {
        return Err(anyhow::anyhow!("Image file does not exist: {}", image_path.display()));
    }

    let image_bytes = tokio::fs::read(image_path)
        .await
        .context(format!("Failed to read image: {}", image_path.display()))?;

    let filename = image_path
        .file_name()
        .and_then(|n| n.to_str())
        .unwrap_or("image.png")
        .to_string();

    let ext = image_path
        .extension()
        .and_then(|e| e.to_str())
        .unwrap_or("")
        .to_lowercase();

    let mime = match ext.as_str() {
        "png" => "image/png",
        "jpg" | "jpeg" => "image/jpeg",
        "webp" => "image/webp",
        _ => "application/octet-stream",
    };

    // Build multipart form
    let part = reqwest::multipart::Part::bytes(image_bytes)
        .file_name(filename)
        .mime_str(mime)
        .context("Failed to create multipart part")?;

    let form = reqwest::multipart::Form::new().part("image", part);

    let client = reqwest::Client::builder()
        .timeout(std::time::Duration::from_secs(30))
        .no_proxy()
        .build()
        .context("Failed to create HTTP client")?;

    let response = client
        .post(api_url)
        .multipart(form)
        .send()
        .await
        .context(format!("OCR request failed, service: {}", api_url))?;

    if !response.status().is_success() {
        return Err(anyhow::anyhow!(
            "OCR service returned error: {} - {}",
            response.status(),
            response.text().await.unwrap_or_default()
        ));
    }

    let result: serde_json::Value = response
        .json()
        .await
        .context("Failed to parse OCR response JSON")?;

    // Extract text - rapidocr_api returns: {"0": {"rec_txt": "text", "dt_boxes": [...], "score": "0.9"}, ...}
    let mut texts = Vec::new();
    let mut total_confidence = 0.0;
    let mut count = 0;

    if let Some(obj) = result.as_object() {
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
                if let Some(score) = item.get("score").and_then(|v| v.as_str()) {
                    if let Ok(s) = score.parse::<f64>() {
                        total_confidence += s;
                        count += 1;
                    }
                }
            }
        }
    }

    let text = texts.join("\n");
    let confidence = if count > 0 {
        total_confidence / count as f64
    } else {
        0.0
    };

    Ok(OcrResult {
        text,
        confidence,
        processing_time_ms: start.elapsed().as_millis() as u64,
    })
}

/// Process OCR using Tesseract CLI
async fn process_tesseract(image_path: &str, cmd: &str, lang: &str) -> Result<OcrResult> {
    let start = Instant::now();

    let image_path = Path::new(image_path);
    if !image_path.exists() {
        return Err(anyhow::anyhow!("Image file does not exist: {}", image_path.display()));
    }

    // Run tesseract using tokio::task::spawn_blocking
    let image_path_str = image_path.to_string_lossy().to_string();
    let cmd_str = cmd.to_string();
    let lang_str = lang.to_string();

    let output = tokio::task::spawn_blocking(move || {
        std::process::Command::new(&cmd_str)
            .args([
                &image_path_str,
                "stdout",
                "-l",
                &lang_str,
            ])
            .output()
    })
    .await??;

    if !output.status.success() {
        return Err(anyhow::anyhow!(
            "Tesseract failed: {}",
            String::from_utf8_lossy(&output.stderr)
        ));
    }

    let text = String::from_utf8_lossy(&output.stdout).to_string();

    Ok(OcrResult {
        text,
        confidence: 0.0, // Tesseract doesn't provide confidence in stdout mode
        processing_time_ms: start.elapsed().as_millis() as u64,
    })
}

/// Check if OCR is available (any backend)
pub fn is_ocr_available() -> bool {
    is_rapidocr_available() || is_tesseract_available()
}

/// Check if RapidOCR service is available
pub fn is_rapidocr_available() -> bool {
    // Try to connect to the RapidOCR service
    let url = "http://127.0.0.1:9003/docs";
    match reqwest::blocking::get(url) {
        Ok(response) => response.status().is_success(),
        Err(_) => false,
    }
}

/// Check if Tesseract is available on the system
pub fn is_tesseract_available() -> bool {
    match std::process::Command::new("tesseract").arg("--version").output() {
        Ok(output) => output.status.success(),
        Err(_) => false,
    }
}

/// Get OCR engine info
pub fn get_ocr_engine_info() -> OcrEngineInfo {
    OcrEngineInfo {
        rapidocr_available: is_rapidocr_available(),
        tesseract_available: is_tesseract_available(),
    }
}

/// OCR engine information
#[derive(Debug, Clone)]
pub struct OcrEngineInfo {
    pub rapidocr_available: bool,
    pub tesseract_available: bool,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_ocr_config_default() {
        let config = OcrConfig::default();
        assert!(matches!(config.engine, OcrEngineType::RapidOCR));
    }

    #[test]
    fn test_ocr_engine_info() {
        let info = get_ocr_engine_info();
        println!("RapidOCR available: {}", info.rapidocr_available);
        println!("Tesseract available: {}", info.tesseract_available);
    }
}
