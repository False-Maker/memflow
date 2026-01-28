//! Prompt 模板引擎 - 借鉴 Dify 的变量注入思想
//!
//! 支持 `{{variable}}` 语法的动态变量替换，实现 Prompt 逻辑与代码逻辑的解耦。

use std::collections::HashMap;
use regex::Regex;
use once_cell::sync::Lazy;

/// 变量匹配正则：匹配 `{{variable_name}}` 格式
static VARIABLE_REGEX: Lazy<Regex> = Lazy::new(|| {
    Regex::new(r"\{\{(\w+)\}\}").expect("Invalid regex pattern")
});

/// Prompt 模板结构体
/// 
/// # Example
/// ```ignore
/// use std::collections::HashMap;
/// use memflow_core::ai::prompt_engine::PromptTemplate;
/// 
/// let template = PromptTemplate::new("基于以下上下文回答：\n{{context}}\n用户问题：{{query}}");
/// let mut vars = HashMap::new();
/// vars.insert("context".to_string(), "今天天气很好".to_string());
/// vars.insert("query".to_string(), "今天天气怎么样？".to_string());
/// let result = template.render(&vars);
/// ```
#[derive(Debug, Clone)]
pub struct PromptTemplate {
    template: String,
}

impl PromptTemplate {
    /// 创建新的 Prompt 模板
    pub fn new(template: &str) -> Self {
        Self {
            template: template.to_string(),
        }
    }

    /// 从字符串创建模板
    pub fn from_string(template: String) -> Self {
        Self { template }
    }

    /// 渲染模板，将所有 `{{variable}}` 替换为对应的值
    /// 
    /// 如果变量未找到，保留原始占位符
    pub fn render(&self, variables: &HashMap<String, String>) -> String {
        let mut result = self.template.clone();
        
        for cap in VARIABLE_REGEX.captures_iter(&self.template) {
            let full_match = cap.get(0).unwrap().as_str();
            let var_name = cap.get(1).unwrap().as_str();
            
            if let Some(value) = variables.get(var_name) {
                result = result.replace(full_match, value);
            }
        }
        
        result
    }

    /// 渲染模板，未找到的变量替换为默认值
    pub fn render_with_default(&self, variables: &HashMap<String, String>, default: &str) -> String {
        let mut result = self.template.clone();
        
        for cap in VARIABLE_REGEX.captures_iter(&self.template) {
            let full_match = cap.get(0).unwrap().as_str();
            let var_name = cap.get(1).unwrap().as_str();
            
            let value = variables.get(var_name).map(|s| s.as_str()).unwrap_or(default);
            result = result.replace(full_match, value);
        }
        
        result
    }

    /// 提取模板中的所有变量名
    pub fn extract_variables(&self) -> Vec<String> {
        VARIABLE_REGEX
            .captures_iter(&self.template)
            .map(|cap| cap.get(1).unwrap().as_str().to_string())
            .collect()
    }

    /// 检查是否包含指定变量
    pub fn has_variable(&self, name: &str) -> bool {
        let pattern = format!("{{{{{}}}}}", name);
        self.template.contains(&pattern)
    }

    /// 获取原始模板字符串
    pub fn raw(&self) -> &str {
        &self.template
    }
}

/// 快速构建变量 HashMap 的宏
#[macro_export]
macro_rules! prompt_vars {
    ($($key:expr => $value:expr),* $(,)?) => {{
        let mut map = std::collections::HashMap::new();
        $(
            map.insert($key.to_string(), $value.to_string());
        )*
        map
    }};
}

/// 预定义的系统 Prompt 模板
pub mod templates {
    use super::PromptTemplate;

    /// RAG 问答模板
    pub fn rag_qa() -> PromptTemplate {
        PromptTemplate::new(
            "基于以下上下文回答用户问题。如果无法从上下文中找到答案，请明确说明。\n\n\
             ## 上下文\n{{context}}\n\n\
             ## 用户问题\n{{query}}\n\n\
             ## 回答要求\n\
             - 回答应简洁明了\n\
             - 引用上下文中的具体信息\n\
             - 如果信息不足，请说明"
        )
    }

    /// 活动分析模板
    pub fn activity_analysis() -> PromptTemplate {
        PromptTemplate::new(
            "分析以下桌面活动记录，识别用户的主要任务和工作模式。\n\n\
             ## 活动记录\n{{activities}}\n\n\
             ## 时间范围\n{{time_range}}\n\n\
             ## 分析要求\n\
             - 识别主要任务/项目\n\
             - 总结工作模式\n\
             - 提取关键文件和链接"
        )
    }

    /// 意图解析模板
    pub fn intent_parser() -> PromptTemplate {
        PromptTemplate::new(
            "解析用户查询的意图，提取搜索参数。\n\n\
             用户查询：{{query}}\n\n\
             返回 JSON 格式的过滤参数。"
        )
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_simple_render() {
        let template = PromptTemplate::new("Hello, {{name}}!");
        let mut vars = HashMap::new();
        vars.insert("name".to_string(), "World".to_string());
        
        assert_eq!(template.render(&vars), "Hello, World!");
    }

    #[test]
    fn test_multiple_variables() {
        let template = PromptTemplate::new("{{greeting}}, {{name}}! Today is {{day}}.");
        let mut vars = HashMap::new();
        vars.insert("greeting".to_string(), "Hi".to_string());
        vars.insert("name".to_string(), "Alice".to_string());
        vars.insert("day".to_string(), "Monday".to_string());
        
        assert_eq!(template.render(&vars), "Hi, Alice! Today is Monday.");
    }

    #[test]
    fn test_missing_variable_preserved() {
        let template = PromptTemplate::new("Hello, {{name}}! Your id is {{id}}.");
        let mut vars = HashMap::new();
        vars.insert("name".to_string(), "Bob".to_string());
        
        assert_eq!(template.render(&vars), "Hello, Bob! Your id is {{id}}.");
    }

    #[test]
    fn test_render_with_default() {
        let template = PromptTemplate::new("Hello, {{name}}! Your id is {{id}}.");
        let mut vars = HashMap::new();
        vars.insert("name".to_string(), "Bob".to_string());
        
        assert_eq!(template.render_with_default(&vars, "N/A"), "Hello, Bob! Your id is N/A.");
    }

    #[test]
    fn test_extract_variables() {
        let template = PromptTemplate::new("{{context}}\n{{query}}\n{{context}}");
        let vars = template.extract_variables();
        
        // 注意：重复的变量会被多次提取
        assert_eq!(vars.len(), 3);
        assert!(vars.contains(&"context".to_string()));
        assert!(vars.contains(&"query".to_string()));
    }

    #[test]
    fn test_has_variable() {
        let template = PromptTemplate::new("Hello, {{name}}!");
        
        assert!(template.has_variable("name"));
        assert!(!template.has_variable("unknown"));
    }

    #[test]
    fn test_chinese_content() {
        let template = PromptTemplate::new("基于以下上下文回答：\n{{context}}\n用户问题：{{query}}");
        let mut vars = HashMap::new();
        vars.insert("context".to_string(), "今天天气晴朗，温度25度".to_string());
        vars.insert("query".to_string(), "今天天气怎么样？".to_string());
        
        let result = template.render(&vars);
        assert!(result.contains("今天天气晴朗"));
        assert!(result.contains("今天天气怎么样？"));
    }

    #[test]
    fn test_rag_template() {
        let template = templates::rag_qa();
        assert!(template.has_variable("context"));
        assert!(template.has_variable("query"));
    }
}
