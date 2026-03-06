use anyhow::Result;
use image::{imageops, DynamicImage, GenericImageView, Luma};
use std::path::Path;

/// OCR 质量评估结果。
///
/// - `cer`: 字符级错误率（0.0 越接近表示越好）。
/// - `wer`: 词级错误率（0.0 越接近表示越好）。
/// - `score`: 综合评分，简单归一化到 `[0, 1]` 区间，1.0 表示完美匹配。
#[derive(Debug, Clone, Copy)]
pub struct OcrQuality {
    pub cer: f64,
    pub wer: f64,
    pub score: f64,
}

/// 终端/代码类截图的预处理。
///
/// 典型步骤（与架构文档中 `ocr_enhance` 能力保持一致）：
/// - 灰度化：降低色彩噪声，对终端/IDE 截图足够。
/// - 轻微对比度增强：提升前景/背景对比度，便于 OCR 引擎识别。
/// - 简单二值化：针对背景较干净的终端场景压低噪声（可视为启发式增强）。
/// - 条件性缩放到指定宽度，保持纵横比。
/// - 转为 PNG 字节（供上层写入临时文件，再交给 OCR 引擎）。
///
/// 返回：
/// - Ok(None)  表示无需预处理，沿用原图
/// - Ok(Some(png_bytes)) 表示返回预处理后的 PNG 图像
pub fn preprocess_terminal_image(
    src_path: &Path,
    target_width: u32,
    max_pixels: u64,
) -> Result<Option<Vec<u8>>> {
    let img = image::open(src_path)?;
    let (w, h) = img.dimensions();
    let pixels = w as u64 * h as u64;

    // 图像尺寸已经足够小，且像素数不大，直接跳过预处理，避免额外开销。
    if w <= target_width && pixels <= max_pixels {
        return Ok(None);
    }

    // 计算新尺寸，保持纵横比
    let new_w = target_width.max(1).min(w);
    let ratio = new_w as f64 / w.max(1) as f64;
    let new_h = ((h as f64) * ratio).round().max(1.0) as u32;

    // 1) 灰度化
    let gray = img.grayscale();

    // 2) 轻微对比度增强（经验值：10.0 足以压低背景、拉高前景）
    let contrasted: DynamicImage =
        image::DynamicImage::ImageRgba8(imageops::contrast(&gray, 10.0));

    // 3) 简单二值化：阈值法对大部分终端/IDE 截图足够
    let mut luma = contrasted.to_luma8();
    let threshold: u8 = 160;
    for p in luma.pixels_mut() {
        let v = p[0];
        *p = if v > threshold {
            Luma([255])
        } else {
            Luma([0])
        };
    }

    // 4) 双线性缩放到目标宽度
    let binarized = DynamicImage::ImageLuma8(luma);
    let resized = binarized.resize_exact(new_w, new_h, imageops::FilterType::Triangle);

    // 5) 编码为 PNG 字节
    let mut buf: Vec<u8> = Vec::new();
    {
        use std::io::Cursor;
        let mut cursor = Cursor::new(&mut buf);
        resized.write_to(&mut cursor, image::ImageFormat::Png)?;
    }

    Ok(Some(buf))
}

/// 对疑似代码类 OCR 文本做后处理：
/// - 统一换行
/// - 去除行尾多余空白
/// - 修正常见的全角符号和括号
pub fn postprocess_terminal_text(text: &str) -> String {
    let normalized = text.replace("\r\n", "\n").replace('\r', "\n");

    let mut lines = Vec::new();
    for line in normalized.lines() {
        // 保留前导缩进，只去除尾部空格/Tab
        let trimmed_end = line
            .trim_end_matches(|c| c == ' ' || c == '\t')
            .to_string();

        let fixed_punctuation = trimmed_end
            // 全角括号/花括号/中括号
            .replace('（', "(")
            .replace('）', ")")
            .replace('［', "[")
            .replace('］', "]")
            .replace('｛', "{")
            .replace('｝', "}")
            // 全角标点转半角
            .replace('！', "!")
            .replace('；', ";")
            .replace('，', ",")
            .replace('：', ":")
            .replace('“', "\"")
            .replace('”', "\"")
            .replace('‘', "'")
            .replace('’', "'");

        lines.push(fixed_punctuation);
    }

    lines.join("\n")
}

/// 简单启发式判断文本是否“像代码”
pub fn is_likely_code(text: &str) -> bool {
    let lines: Vec<&str> = text.lines().collect();
    if lines.is_empty() {
        return false;
    }

    let mut score = 0i32;

    for line in &lines {
        let trimmed = line.trim_start();
        if trimmed.is_empty() {
            continue;
        }

        // 常见代码结构与符号
        if trimmed.starts_with("//")
            || trimmed.starts_with("#include")
            || trimmed.starts_with("using ")
        {
            score += 2;
        }
        if trimmed.starts_with("fn ")
            || trimmed.starts_with("pub fn")
            || trimmed.starts_with("def ")
            || trimmed.starts_with("class ")
            || trimmed.contains(" => ")
        {
            score += 3;
        }
        if trimmed.contains('{') || trimmed.contains('}') {
            score += 1;
        }
        if trimmed.ends_with('{') || trimmed.ends_with(';') {
            score += 1;
        }
        if trimmed.contains("let ")
            || trimmed.contains("var ")
            || trimmed.contains("const ")
        {
            score += 1;
        }
    }

    score >= 3 && lines.len() >= 2
}

/// 粗略的语言检测，仅用于日志与调试
pub fn detect_language(text: &str) -> Option<String> {
    let lower = text.to_lowercase();

    if lower.contains("fn main()")
        || lower.contains("use std::")
        || lower.contains("pub struct")
    {
        return Some("rust".to_string());
    }

    if lower.contains("def ")
        && (lower.contains("import ")
            || lower.contains("from ")
            || lower.contains("self"))
    {
        return Some("python".to_string());
    }

    if lower.contains("console.log")
        || lower.contains("function ")
        || lower.contains("=>")
    {
        return Some("javascript".to_string());
    }

    if lower.contains("#include")
        || lower.contains("std::")
        || lower.contains("::cout")
    {
        return Some("cpp".to_string());
    }

    if lower.contains("public class")
        || lower.contains("System.out.println")
    {
        return Some("java".to_string());
    }

    None
}

/// 计算字符级错误率 CER（Character Error Rate）。
///
/// 定义：`CER = edit_distance(chars(reference), chars(hypothesis)) / len(reference)`。
/// 当参考文本为空时，返回 0 以避免除零和 NaN。
pub fn calculate_cer(reference: &str, hypothesis: &str) -> f64 {
    if reference.is_empty() {
        return 0.0;
    }

    let ref_chars: Vec<char> = reference.chars().collect();
    let hyp_chars: Vec<char> = hypothesis.chars().collect();
    let dist = levenshtein_distance(&ref_chars, &hyp_chars);

    dist as f64 / ref_chars.len() as f64
}

/// 计算词级错误率 WER（Word Error Rate）。
///
/// 定义：`WER = edit_distance(words(reference), words(hypothesis)) / len(words(reference))`。
/// 当参考文本中不包含任何“词”时，返回 0。
pub fn calculate_wer(reference: &str, hypothesis: &str) -> f64 {
    let ref_words: Vec<&str> = reference.split_whitespace().collect();
    if ref_words.is_empty() {
        return 0.0;
    }
    let hyp_words: Vec<&str> = hypothesis.split_whitespace().collect();
    let dist = levenshtein_distance(&ref_words, &hyp_words);

    dist as f64 / ref_words.len() as f64
}

fn levenshtein_distance<T: PartialEq>(a: &[T], b: &[T]) -> usize {
    let len_a = a.len();
    let len_b = b.len();

    if len_a == 0 {
        return len_b;
    }
    if len_b == 0 {
        return len_a;
    }

    let mut dp: Vec<Vec<usize>> = vec![vec![0; len_b + 1]; len_a + 1];

    for i in 0..=len_a {
        dp[i][0] = i;
    }
    for j in 0..=len_b {
        dp[0][j] = j;
    }

    for i in 1..=len_a {
        for j in 1..=len_b {
            let cost = if a[i - 1] == b[j - 1] { 0 } else { 1 };

            dp[i][j] = std::cmp::min(
                std::cmp::min(dp[i - 1][j] + 1, dp[i][j - 1] + 1),
                dp[i - 1][j - 1] + cost,
            );
        }
    }

    dp[len_a][len_b]
}

/// 综合评估 OCR 质量，返回 CER / WER 以及一个简单的整体评分。
///
/// 设计目标：
/// - 调用方可以只关心一个 `score`（0~1），同时在需要时查看 CER / WER 细节。
/// - 当前实现使用最简单的线性组合，后续如需更精细的权重或分段函数可以在不破坏 API 的前提下调整内部逻辑。
pub fn evaluate_ocr_quality(reference: &str, hypothesis: &str) -> OcrQuality {
    let cer = calculate_cer(reference, hypothesis);
    let wer = calculate_wer(reference, hypothesis);

    // 简单线性归一化：
    // - 将 CER/WER 限制在 [0, 1] 内（即视 >100% 的错误率为最差情况）。
    // - 取平均后做 1 - x 得到“越大越好”的分值。
    let cer_clamped = cer.clamp(0.0, 1.0);
    let wer_clamped = wer.clamp(0.0, 1.0);
    let avg_error = (cer_clamped + wer_clamped) / 2.0;
    let score = (1.0 - avg_error).clamp(0.0, 1.0);

    OcrQuality { cer, wer, score }
}
