//! NLP 模块 - 关键词提取与文本分析
//!
//! 使用 jieba-rs 进行中文分词，支持停用词过滤和关键词提取

use once_cell::sync::Lazy;
use std::collections::HashSet;

/// 全局 Jieba 分词器实例（延迟初始化）
static JIEBA: Lazy<jieba_rs::Jieba> = Lazy::new(|| {
    tracing::info!("初始化 Jieba 分词器...");
    jieba_rs::Jieba::new()
});

/// 中文停用词列表
static CHINESE_STOPWORDS: Lazy<HashSet<&'static str>> = Lazy::new(|| {
    let words = [
        // 常用虚词
        "的", "了", "和", "是", "就", "都", "而", "及", "与", "着",
        "或", "一个", "没有", "我们", "你们", "他们", "它们", "这个", "那个",
        "这些", "那些", "自己", "什么", "哪些", "怎么", "如何", "为什么",
        "因为", "所以", "但是", "然而", "如果", "虽然", "即使", "只要",
        "不过", "而且", "并且", "或者", "可以", "可能", "应该", "需要",
        "必须", "已经", "正在", "将要", "曾经", "一直", "总是", "经常",
        "有时", "偶尔", "从不", "很", "非常", "十分", "相当", "比较",
        "更加", "最", "太", "越", "再", "又", "也", "还", "才", "刚",
        "已", "将", "要", "会", "能", "得", "地", "之", "以", "于",
        "在", "到", "从", "向", "对", "把", "被", "给", "让", "使",
        // 数字和量词
        "一", "二", "三", "四", "五", "六", "七", "八", "九", "十",
        "百", "千", "万", "亿", "个", "只", "条", "件", "本", "份",
        "次", "遍", "回", "趟", "下", "种", "类", "些", "点",
        // 代词
        "我", "你", "他", "她", "它", "这", "那", "哪", "谁", "其",
        // 标点和特殊字符对应的词
        "nbsp", "quot", "amp", "lt", "gt",
        // 英文常用停用词
        "the", "a", "an", "is", "are", "was", "were", "be", "been",
        "being", "have", "has", "had", "do", "does", "did", "will",
        "would", "could", "should", "may", "might", "must", "shall",
        "can", "need", "dare", "ought", "used", "to", "of", "in",
        "for", "on", "with", "at", "by", "from", "as", "into", "through",
        "during", "before", "after", "above", "below", "between", "under",
        "again", "further", "then", "once", "here", "there", "when",
        "where", "why", "how", "all", "each", "few", "more", "most",
        "other", "some", "such", "no", "nor", "not", "only", "own",
        "same", "so", "than", "too", "very", "just", "and", "but",
        "if", "or", "because", "until", "while", "this", "that",
        "these", "those", "am", "it", "its", "he", "she", "they",
        "them", "his", "her", "their", "what", "which", "who", "whom",
        // 常见无意义词
        "com", "www", "http", "https", "html", "htm", "php", "asp",
        "org", "net", "edu", "gov", "cn", "jpg", "png", "gif", "pdf",
    ];
    words.into_iter().collect()
});

/// 关键词提取选项
#[derive(Debug, Clone)]
pub struct KeywordOptions {
    /// 最大关键词数量
    pub max_keywords: usize,
    /// 最小词长度（字符数）
    pub min_word_len: usize,
    /// 是否过滤停用词
    pub filter_stopwords: bool,
    /// 是否过滤纯数字
    pub filter_numbers: bool,
}

impl Default for KeywordOptions {
    fn default() -> Self {
        Self {
            max_keywords: 10,
            min_word_len: 2,
            filter_stopwords: true,
            filter_numbers: true,
        }
    }
}

/// 使用 jieba 分词并提取关键词
pub fn extract_keywords(text: &str, options: Option<KeywordOptions>) -> Vec<String> {
    let opts = options.unwrap_or_default();
    
    if text.trim().is_empty() {
        return vec![];
    }
    
    // 使用 jieba 进行分词
    let words = JIEBA.cut(text, false);
    
    // 统计词频
    let mut word_freq: std::collections::HashMap<String, usize> = std::collections::HashMap::new();
    
    for word in words {
        let word = word.trim();
        
        // 长度过滤
        if word.chars().count() < opts.min_word_len {
            continue;
        }
        
        // 停用词过滤
        if opts.filter_stopwords && CHINESE_STOPWORDS.contains(word.to_lowercase().as_str()) {
            continue;
        }
        
        // 纯数字过滤
        if opts.filter_numbers && word.chars().all(|c| c.is_ascii_digit()) {
            continue;
        }
        
        // 过滤纯标点符号
        if word.chars().all(|c| !c.is_alphanumeric()) {
            continue;
        }
        
        *word_freq.entry(word.to_string()).or_insert(0) += 1;
    }
    
    // 按词频排序
    let mut sorted_words: Vec<(String, usize)> = word_freq.into_iter().collect();
    sorted_words.sort_by(|a, b| b.1.cmp(&a.1));
    
    // 取前 N 个关键词
    sorted_words
        .into_iter()
        .take(opts.max_keywords)
        .map(|(word, _)| word)
        .collect()
}

/// 使用 TF-IDF 算法提取关键词（更高质量）
/// 注意：jieba-rs 0.7 版本中 TF-IDF 功能需要额外配置，这里使用词频排序作为替代
pub fn extract_keywords_tfidf(text: &str, top_k: usize) -> Vec<String> {
    if text.trim().is_empty() {
        return vec![];
    }
    
    // 使用 jieba 分词并按词频排序（模拟 TF-IDF 效果）
    let words = JIEBA.cut_for_search(text, true);
    
    // 统计词频
    let mut word_freq: std::collections::HashMap<String, usize> = std::collections::HashMap::new();
    
    for word in words {
        let word = word.trim();
        
        // 长度过滤（至少2个字符）
        if word.chars().count() < 2 {
            continue;
        }
        
        // 停用词过滤
        if CHINESE_STOPWORDS.contains(word.to_lowercase().as_str()) {
            continue;
        }
        
        // 过滤纯标点符号和数字
        if word.chars().all(|c| !c.is_alphanumeric()) {
            continue;
        }
        if word.chars().all(|c| c.is_ascii_digit()) {
            continue;
        }
        
        *word_freq.entry(word.to_string()).or_insert(0) += 1;
    }
    
    // 按词频排序并取前 N 个
    let mut sorted_words: Vec<(String, usize)> = word_freq.into_iter().collect();
    sorted_words.sort_by(|a, b| b.1.cmp(&a.1));
    
    sorted_words
        .into_iter()
        .take(top_k)
        .map(|(word, _)| word)
        .collect()
}

/// 判断文本是否主要为中文
pub fn is_chinese_text(text: &str) -> bool {
    let chinese_count = text.chars().filter(|c| {
        matches!(*c as u32, 0x4E00..=0x9FFF | 0x3400..=0x4DBF | 0x20000..=0x2A6DF)
    }).count();
    
    let total_alpha = text.chars().filter(|c| c.is_alphabetic()).count();
    
    if total_alpha == 0 {
        return false;
    }
    
    chinese_count as f64 / total_alpha as f64 > 0.3
}

/// 提取文本中的专有名词（简单实现）
pub fn extract_named_entities(text: &str) -> Vec<String> {
    let mut entities = Vec::new();
    
    // 提取可能的文件路径
    let path_patterns = [
        r#"[A-Za-z]:\\[^\s<>:"|?*]+"#,  // Windows 路径
        r#"/[^\s<>:"|?*]+"#,            // Unix 路径
    ];
    
    for pattern in &path_patterns {
        if let Ok(re) = regex::Regex::new(pattern) {
            for cap in re.find_iter(text) {
                let path = cap.as_str();
                // 过滤太短的路径
                if path.len() > 5 {
                    entities.push(path.to_string());
                }
            }
        }
    }
    
    // 提取 URL
    if let Ok(url_re) = regex::Regex::new(r#"https?://[^\s<>"]+"#) {
        for cap in url_re.find_iter(text) {
            entities.push(cap.as_str().to_string());
        }
    }
    
    entities
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_extract_keywords_chinese() {
        let text = "今天我在使用 Visual Studio Code 编写 Rust 程序";
        let keywords = extract_keywords(text, None);
        
        assert!(!keywords.is_empty());
        // 应该包含有意义的词
        assert!(keywords.iter().any(|k| k.contains("Visual") || k.contains("Code") || k.contains("Rust")));
    }

    #[test]
    fn test_extract_keywords_english() {
        let text = "The quick brown fox jumps over the lazy dog while programming";
        let keywords = extract_keywords(text, None);
        
        assert!(!keywords.is_empty());
    }

    #[test]
    fn test_stopword_filtering() {
        let text = "的的的了了了是是是";
        let keywords = extract_keywords(text, None);
        
        // 应该过滤掉所有停用词
        assert!(keywords.is_empty());
    }

    #[test]
    fn test_is_chinese_text() {
        assert!(is_chinese_text("这是中文文本"));
        assert!(!is_chinese_text("This is text"));
        assert!(is_chinese_text("这是中文混合"));
    }
}
