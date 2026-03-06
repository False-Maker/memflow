//! Vector database module - Tauri wrapper for memflow-core vector_db
//!
//! Re-exports from memflow_core::vector_db and provides the
//! generate_embedding function.

// Re-export everything from memflow_core vector_db
pub use memflow_core::vector_db::*;

use crate::app_config;
use anyhow::Result;
use std::path::PathBuf;
use std::pin::Pin;
use std::future::Future;

/// Generate embedding using local BGE model
/// This function always uses the local Chinese embedding model (BGE-small-zh-v1.5)
pub async fn generate_embedding(text: &str) -> Result<Vec<f32>> {
    // 创建本地 context 用于本地 embedding 模型
    struct LocalContext {
        resource_dir: PathBuf,
    }
    
    impl memflow_core::context::RuntimeContext for LocalContext {
        fn resource_dir(&self) -> PathBuf {
            self.resource_dir.clone()
        }
        
        fn app_dir(&self) -> PathBuf {
            PathBuf::from(".")
        }
        
        fn emit(&self, _event: &str, _payload: serde_json::Value) -> anyhow::Result<()> {
            Ok(())
        }

        fn analyze_for_proposals(
            &self,
            _context_text: &str,
        ) -> Pin<Box<dyn Future<Output = Result<memflow_core::context::AiAnalysisResult>> + Send + '_>> {
            Box::pin(async {
                // Not implemented for local embedding generation
                Ok(memflow_core::context::AiAnalysisResult { tasks: vec![] })
            })
        }
    }

    // 获取 resource directory
    // 优先级：用户配置的 data_directory > 可执行文件所在目录 > 当前目录
    let config = app_config::get_config().await;
    
    // 尝试从配置获取
    let configured_dir = config
        .ok()
        .and_then(|cfg| cfg.data_directory.clone())
        .map(|d| PathBuf::from(d).join("resources"));
    
    // 尝试获取可执行文件所在目录
    let exe_dir = std::env::current_exe()
        .ok()
        .and_then(|p| p.parent().map(|p| p.to_path_buf()));
    
    // 选择最合适的 resource directory
    let resource_dir = configured_dir
        .or_else(|| exe_dir.clone())
        .unwrap_or_else(|| PathBuf::from("."));
    
    tracing::debug!(
        "Using resource directory for embedding: {:?}, exe_dir: {:?}",
        resource_dir,
        exe_dir
    );

    let ctx = LocalContext {
        resource_dir,
    };

    // 使用本地中文模型生成向量
    match memflow_core::ai::embedding::embed_with_local_model(&ctx, text) {
        Ok(embedding) => {
            tracing::debug!(
                "使用本地模型生成向量成功，维度: {}",
                embedding.len()
            );
            Ok(embedding)
        }
        Err(e) => {
            tracing::error!(
                "本地模型生成失败: {}",
                e
            );
            // 返回错误而不是 fallback 到占位符
            Err(e)
        }
    }
}
