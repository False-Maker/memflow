//! Local embedding model management shared across frontends.
//!
//! This module owns the lifecycle of the on-device embedding model (fastembed),
//! and exposes a simple API for generating embeddings. It is intentionally
//! decoupled from any specific binary (Tauri / MCP) so both can reuse it.
//!
//! # ONNX Runtime Requirement
//! This module requires `onnxruntime.dll` to be available. Download from:
//! https://github.com/microsoft/onnxruntime/releases/tag/v1.24.1
//!
//! Place the DLL in one of:
//! - Project's `src-tauri/resources/` directory
//! - Same directory as the executable
//! - System PATH

use crate::context::RuntimeContext;
use crate::vector_db::EMBEDDING_DIM;
use anyhow::{anyhow, Context, Result};
use fastembed::{InitOptions, TextEmbedding, EmbeddingModel};
use once_cell::sync::OnceCell;
use std::sync::Mutex;
use tracing::{info, warn};

/// Global embedding model instance, initialized on first use.
static EMBEDDING_MODEL: OnceCell<Mutex<TextEmbedding>> = OnceCell::new();

/// Store initialization error if it fails
static INIT_ERROR: OnceCell<String> = OnceCell::new();

/// Get or initialize the global embedding model using the provided runtime context.
///
/// The context is used to resolve the resource directory and model cache path.
/// If initialization fails (e.g., ONNX DLL not found), returns an error instead of panicking.
fn get_or_init_model(ctx: &impl RuntimeContext) -> Result<&'static Mutex<TextEmbedding>> {
    // First check if we have an initialization error stored
    if let Some(err_msg) = INIT_ERROR.get() {
        return Err(anyhow!("{}", err_msg));
    }

    // Try to get existing model
    if let Some(model) = EMBEDDING_MODEL.get() {
        return Ok(model);
    }

    // Attempt initialization
    let result = init_model_inner(ctx);

    match result {
        Ok(model) => {
            // Successfully initialized, store in OnceCell
            let _ = EMBEDDING_MODEL.set(model);
            Ok(EMBEDDING_MODEL.get().unwrap())
        }
        Err(e) => {
            // Store error to avoid retry
            let err_msg = format!(
                "Embedding model initialization failed: {}. \
                 Please ensure onnxruntime.dll v1.24.1 is available.",
                e
            );
            let _ = INIT_ERROR.set(err_msg.clone());
            Err(anyhow!(err_msg))
        }
    }
}

/// Inner function to initialize the model
fn init_model_inner(ctx: &impl RuntimeContext) -> Result<Mutex<TextEmbedding>> {
    let resource_dir = ctx.resource_dir();
    let model_dir = resource_dir.join("models");

    info!(
        "Initializing local embedding model BGESmallENV15Q at {:?}",
        model_dir
    );

    let model_opts = InitOptions::new(EmbeddingModel::BGESmallENV15Q)
        .with_cache_dir(model_dir)
        .with_show_download_progress(false);

    let model = TextEmbedding::try_new(model_opts)
        .context("Failed to initialize fastembed TextEmbedding model")?;

    Ok(Mutex::new(model))
}

/// Generate an embedding vector for the given text using the local model.
///
/// On success, returns a vector of size `EMBEDDING_DIM`. Callers can decide how
/// to handle failures (e.g. fall back to a placeholder embedding).
pub fn embed_with_local_model(ctx: &impl RuntimeContext, text: &str) -> Result<Vec<f32>> {
    let model_lock = get_or_init_model(ctx)?;
    
    // Handle poisoned mutex gracefully
    let mut model = match model_lock.lock() {
        Ok(m) => m,
        Err(_) => {
            return Err(anyhow!(
                "Embedding model mutex is poisoned. Please restart the application."
            ));
        }
    };

    let embeddings = model
        .embed(vec![text], None)
        .context("fastembed embedding generation failed")?;

    let vec = embeddings
        .into_iter()
        .next()
        .ok_or_else(|| anyhow!("fastembed returned empty embedding list"))?;

    if vec.len() != EMBEDDING_DIM {
        warn!(
            "Embedding dimension mismatch: expected {}, got {}",
            EMBEDDING_DIM,
            vec.len()
        );
    }

    Ok(vec)
}

/// Check whether the local embedding model has been initialized.
pub fn is_model_initialized() -> bool {
    EMBEDDING_MODEL.get().is_some()
}

/// Check if model initialization failed previously
pub fn has_init_error() -> bool {
    INIT_ERROR.get().is_some()
}
