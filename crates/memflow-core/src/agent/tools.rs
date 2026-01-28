//! 工具抽象层 - 借鉴 Dify 的 Tool/Plugin 系统
//!
//! 将硬编码的 AutomationStep 升级为通用的 Tool Trait，
//! 实现即插即用的工具扩展机制。

use anyhow::Result;
use async_trait::async_trait;
use serde_json::Value;
use std::collections::HashMap;
use std::sync::Arc;

/// 通用工具 Trait - 供 Agent/LLM 动态选择和执行
/// 
/// # 设计理念 (Ref: Dify Tools/Plugin System)
/// - Agent 不是死板的脚本，而是动态选择工具
/// - 每个工具有名称和描述，供 LLM 决策使用
/// - 工具执行接收 JSON 参数，返回 JSON 结果
#[async_trait]
pub trait Tool: Send + Sync {
    /// 工具名称（唯一标识）
    fn name(&self) -> &str;
    
    /// 工具描述（供 LLM 理解用途）
    fn description(&self) -> &str;
    
    /// 参数 Schema 描述（JSON Schema 格式，可选）
    fn parameters_schema(&self) -> Option<Value> {
        None
    }
    
    /// 执行工具
    async fn execute(&self, args: Value) -> Result<Value>;
}

/// 工具注册表 - 管理所有可用工具
pub struct ToolRegistry {
    tools: HashMap<String, Arc<dyn Tool>>,
}

impl Default for ToolRegistry {
    fn default() -> Self {
        Self::new()
    }
}

impl ToolRegistry {
    pub fn new() -> Self {
        Self {
            tools: HashMap::new(),
        }
    }

    /// 注册工具
    pub fn register(&mut self, tool: Arc<dyn Tool>) {
        self.tools.insert(tool.name().to_string(), tool);
    }

    /// 获取工具
    pub fn get(&self, name: &str) -> Option<Arc<dyn Tool>> {
        self.tools.get(name).cloned()
    }

    /// 列出所有工具（供 LLM 选择）
    pub fn list_tools(&self) -> Vec<ToolInfo> {
        self.tools
            .values()
            .map(|t| ToolInfo {
                name: t.name().to_string(),
                description: t.description().to_string(),
                parameters: t.parameters_schema(),
            })
            .collect()
    }

    /// 生成工具列表的 JSON 描述（供 LLM 理解）
    pub fn to_tool_descriptions(&self) -> String {
        let tools: Vec<_> = self.list_tools();
        let mut desc = String::from("可用工具列表：\n");
        for tool in tools {
            desc.push_str(&format!("- {}: {}\n", tool.name, tool.description));
        }
        desc
    }
}

/// 工具信息结构体
#[derive(Debug, Clone)]
pub struct ToolInfo {
    pub name: String,
    pub description: String,
    pub parameters: Option<Value>,
}

// ============================================
// 内置工具实现
// ============================================

/// 打开 URL 工具
pub struct OpenUrlTool;

#[async_trait]
impl Tool for OpenUrlTool {
    fn name(&self) -> &str {
        "open_url"
    }

    fn description(&self) -> &str {
        "在默认浏览器中打开指定的 URL 链接"
    }

    fn parameters_schema(&self) -> Option<Value> {
        Some(serde_json::json!({
            "type": "object",
            "properties": {
                "url": {
                    "type": "string",
                    "description": "要打开的 URL 地址"
                }
            },
            "required": ["url"]
        }))
    }

    async fn execute(&self, args: Value) -> Result<Value> {
        let url = args["url"]
            .as_str()
            .ok_or_else(|| anyhow::anyhow!("缺少 url 参数"))?;
        
        // 使用系统默认方式打开 URL
        #[cfg(target_os = "windows")]
        {
            std::process::Command::new("cmd")
                .args(["/C", "start", "", url])
                .spawn()?;
        }
        
        Ok(serde_json::json!({
            "status": "success",
            "url": url
        }))
    }
}

/// 打开文件工具
pub struct OpenFileTool;

#[async_trait]
impl Tool for OpenFileTool {
    fn name(&self) -> &str {
        "open_file"
    }

    fn description(&self) -> &str {
        "使用系统默认应用打开指定的文件路径"
    }

    fn parameters_schema(&self) -> Option<Value> {
        Some(serde_json::json!({
            "type": "object",
            "properties": {
                "path": {
                    "type": "string",
                    "description": "文件的绝对路径"
                }
            },
            "required": ["path"]
        }))
    }

    async fn execute(&self, args: Value) -> Result<Value> {
        let path = args["path"]
            .as_str()
            .ok_or_else(|| anyhow::anyhow!("缺少 path 参数"))?;
        
        #[cfg(target_os = "windows")]
        {
            std::process::Command::new("cmd")
                .args(["/C", "start", "", path])
                .spawn()?;
        }
        
        Ok(serde_json::json!({
            "status": "success",
            "path": path
        }))
    }
}

/// 打开应用程序工具
pub struct OpenAppTool;

#[async_trait]
impl Tool for OpenAppTool {
    fn name(&self) -> &str {
        "open_app"
    }

    fn description(&self) -> &str {
        "启动指定路径的应用程序"
    }

    fn parameters_schema(&self) -> Option<Value> {
        Some(serde_json::json!({
            "type": "object",
            "properties": {
                "path": {
                    "type": "string",
                    "description": "应用程序的绝对路径"
                }
            },
            "required": ["path"]
        }))
    }

    async fn execute(&self, args: Value) -> Result<Value> {
        let path = args["path"]
            .as_str()
            .ok_or_else(|| anyhow::anyhow!("缺少 path 参数"))?;
        
        #[cfg(target_os = "windows")]
        {
            std::process::Command::new(path).spawn()?;
        }
        
        Ok(serde_json::json!({
            "status": "success",
            "path": path
        }))
    }
}

/// 复制到剪贴板工具
pub struct CopyToClipboardTool;

#[async_trait]
impl Tool for CopyToClipboardTool {
    fn name(&self) -> &str {
        "copy_to_clipboard"
    }

    fn description(&self) -> &str {
        "将指定文本复制到系统剪贴板"
    }

    fn parameters_schema(&self) -> Option<Value> {
        Some(serde_json::json!({
            "type": "object",
            "properties": {
                "text": {
                    "type": "string",
                    "description": "要复制的文本内容"
                }
            },
            "required": ["text"]
        }))
    }

    async fn execute(&self, args: Value) -> Result<Value> {
        let text = args["text"]
            .as_str()
            .ok_or_else(|| anyhow::anyhow!("缺少 text 参数"))?;
        
        let mut clipboard = arboard::Clipboard::new()
            .map_err(|e| anyhow::anyhow!("初始化剪贴板失败: {}", e))?;
        clipboard
            .set_text(text.to_string())
            .map_err(|e| anyhow::anyhow!("写入剪贴板失败: {}", e))?;
        
        Ok(serde_json::json!({
            "status": "success",
            "text_length": text.len()
        }))
    }
}

/// 创建笔记工具
pub struct CreateNoteTool {
    notes_dir: Option<std::path::PathBuf>,
}

impl CreateNoteTool {
    pub fn new(notes_dir: Option<std::path::PathBuf>) -> Self {
        Self { notes_dir }
    }
}

#[async_trait]
impl Tool for CreateNoteTool {
    fn name(&self) -> &str {
        "create_note"
    }

    fn description(&self) -> &str {
        "创建或追加内容到 Markdown 笔记文件"
    }

    fn parameters_schema(&self) -> Option<Value> {
        Some(serde_json::json!({
            "type": "object",
            "properties": {
                "content": {
                    "type": "string",
                    "description": "笔记内容（Markdown 格式）"
                },
                "filename": {
                    "type": "string",
                    "description": "可选的文件名，默认为 memflow_notes.md"
                }
            },
            "required": ["content"]
        }))
    }

    async fn execute(&self, args: Value) -> Result<Value> {
        use std::io::Write;
        
        let content = args["content"]
            .as_str()
            .ok_or_else(|| anyhow::anyhow!("缺少 content 参数"))?;
        
        let filename = args["filename"]
            .as_str()
            .unwrap_or("memflow_notes.md");
        
        let notes_path = if let Some(ref dir) = self.notes_dir {
            dir.join(filename)
        } else {
            let doc_dir = dirs::document_dir()
                .ok_or_else(|| anyhow::anyhow!("无法获取文档目录"))?;
            doc_dir.join(filename)
        };
        
        // 确保父目录存在
        if let Some(parent) = notes_path.parent() {
            std::fs::create_dir_all(parent)?;
        }
        
        let mut file = std::fs::OpenOptions::new()
            .create(true)
            .append(true)
            .open(&notes_path)?;
        
        let timestamp = chrono::Local::now().format("%Y-%m-%d %H:%M:%S");
        writeln!(file, "\n---\n*记录于 {}*\n", timestamp)?;
        writeln!(file, "{}", content)?;
        
        Ok(serde_json::json!({
            "status": "success",
            "path": notes_path.to_string_lossy()
        }))
    }
}

/// 创建默认工具注册表（包含所有内置工具）
pub fn create_default_registry() -> ToolRegistry {
    let mut registry = ToolRegistry::new();
    
    registry.register(Arc::new(OpenUrlTool));
    registry.register(Arc::new(OpenFileTool));
    registry.register(Arc::new(OpenAppTool));
    registry.register(Arc::new(CopyToClipboardTool));
    registry.register(Arc::new(CreateNoteTool::new(None)));
    
    registry
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_tool_registry() {
        let registry = create_default_registry();
        
        assert!(registry.get("open_url").is_some());
        assert!(registry.get("open_file").is_some());
        assert!(registry.get("open_app").is_some());
        assert!(registry.get("copy_to_clipboard").is_some());
        assert!(registry.get("create_note").is_some());
        assert!(registry.get("unknown").is_none());
    }

    #[test]
    fn test_list_tools() {
        let registry = create_default_registry();
        let tools = registry.list_tools();
        
        assert_eq!(tools.len(), 5);
    }

    #[test]
    fn test_tool_descriptions() {
        let registry = create_default_registry();
        let desc = registry.to_tool_descriptions();
        
        assert!(desc.contains("open_url"));
        assert!(desc.contains("open_file"));
        assert!(desc.contains("创建"));
    }
}
