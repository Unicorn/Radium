//! Burn engine provider implementation.
//!
//! This is a **single-binary local** engine intended as the foundation for local
//! inference/training without external services.
//!
//! v1: a minimal Bigram LM checkpoint format.

use crate::engines::engine_trait::{Engine, EngineMetadata, ExecutionRequest, ExecutionResponse};
use crate::engines::error::{EngineError, Result};
use async_trait::async_trait;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::path::{Path, PathBuf};

#[derive(Debug, Clone, Serialize, Deserialize)]
struct BigramCheckpoint {
    /// Ordered vocabulary. Each entry should be a single Unicode scalar value encoded as a string.
    vocab: Vec<String>,
    /// Transition scores: transitions[current_id][next_id] = score.
    transitions: Vec<Vec<f32>>,
}

impl BigramCheckpoint {
    fn validate(&self) -> Result<()> {
        if self.vocab.is_empty() {
            return Err(EngineError::InvalidConfig("burn checkpoint vocab is empty".to_string()));
        }
        if self.transitions.len() != self.vocab.len() {
            return Err(EngineError::InvalidConfig(
                "burn checkpoint transitions must be square (rows != vocab)".to_string(),
            ));
        }
        for row in &self.transitions {
            if row.len() != self.vocab.len() {
                return Err(EngineError::InvalidConfig(
                    "burn checkpoint transitions must be square (cols != vocab)".to_string(),
                ));
            }
        }
        Ok(())
    }

    fn vocab_char_at(&self, id: usize) -> Option<char> {
        self.vocab.get(id).and_then(|s| s.chars().next())
    }

    fn id_for_char(&self, ch: char) -> Option<usize> {
        self.vocab.iter().position(|s| s.chars().next() == Some(ch))
    }

    fn next_id_argmax(&self, current_id: usize) -> Result<usize> {
        let row = self.transitions.get(current_id).ok_or_else(|| {
            EngineError::ExecutionError("invalid current token id".to_string())
        })?;

        // v1: keep the engine single-binary and deterministic.
        // We don't require tensor ops yet for this minimal bigram model.
        let mut best_i = 0usize;
        let mut best_v = f32::NEG_INFINITY;
        for (i, v) in row.iter().copied().enumerate() {
            if v > best_v {
                best_v = v;
                best_i = i;
            }
        }
        Ok(best_i)
    }
}

/// Burn engine implementation.
///
/// **Model selection**: `request.model` must be a path to a checkpoint json, or the literal
/// `"burn-bigram"` (in which case `RADIUM_BURN_BIGRAM_CHECKPOINT` must point to a file).
pub struct BurnEngine {
    metadata: EngineMetadata,
}

impl BurnEngine {
    #[must_use]
    pub fn new() -> Self {
        let metadata = EngineMetadata::new(
            "burn".to_string(),
            "Burn (Local)".to_string(),
            "Single-binary local engine powered by Burn tensors".to_string(),
        )
        .with_auth_required(false)
        .with_models(vec!["burn-bigram".to_string()]);

        Self { metadata }
    }

    fn resolve_checkpoint_path(&self, model: &str) -> Result<PathBuf> {
        if model == "burn-bigram" {
            let path = std::env::var("RADIUM_BURN_BIGRAM_CHECKPOINT").map_err(|_| {
                EngineError::InvalidConfig(
                    "RADIUM_BURN_BIGRAM_CHECKPOINT is required when using model 'burn-bigram'"
                        .to_string(),
                )
            })?;
            return Ok(PathBuf::from(path));
        }

        Ok(PathBuf::from(model))
    }

    fn load_checkpoint(path: &Path) -> Result<BigramCheckpoint> {
        let bytes = std::fs::read(path).map_err(|e| {
            EngineError::ExecutionError(format!(
                "failed to read burn checkpoint {}: {e}",
                path.display()
            ))
        })?;
        let ckpt: BigramCheckpoint = serde_json::from_slice(&bytes).map_err(|e| {
            EngineError::ExecutionError(format!(
                "failed to parse burn checkpoint {}: {e}",
                path.display()
            ))
        })?;
        ckpt.validate()?;
        Ok(ckpt)
    }

    fn generate(&self, ckpt: &BigramCheckpoint, prompt: &str, max_tokens: usize) -> Result<String> {
        let mut current_id = prompt
            .chars()
            .rev()
            .find_map(|ch| ckpt.id_for_char(ch))
            .unwrap_or(0);

        let mut out = String::new();
        for _ in 0..max_tokens {
            let next_id = ckpt.next_id_argmax(current_id)?;
            let ch = ckpt.vocab_char_at(next_id).unwrap_or('?');
            out.push(ch);
            current_id = next_id;
        }
        Ok(out)
    }
}

#[async_trait]
impl Engine for BurnEngine {
    fn metadata(&self) -> &EngineMetadata {
        &self.metadata
    }

    async fn is_available(&self) -> bool {
        true
    }

    async fn is_authenticated(&self) -> Result<bool> {
        Ok(true)
    }

    async fn execute(&self, request: ExecutionRequest) -> Result<ExecutionResponse> {
        let start = std::time::Instant::now();
        let ckpt_path = self.resolve_checkpoint_path(&request.model)?;

        if !ckpt_path.exists() {
            return Err(EngineError::ExecutionError(format!(
                "burn checkpoint not found: {}",
                ckpt_path.display()
            )));
        }

        let ckpt = Self::load_checkpoint(&ckpt_path)?;
        let max_tokens = request.max_tokens.unwrap_or(256);
        let generated = self.generate(&ckpt, &request.prompt, max_tokens)?;

        Ok(ExecutionResponse {
            content: generated,
            usage: None,
            model: request.model,
            raw: None,
            execution_duration: Some(start.elapsed()),
            metadata: Some(HashMap::from([(
                "checkpoint_path".to_string(),
                serde_json::Value::String(ckpt_path.display().to_string()),
            )])),
        })
    }

    fn default_model(&self) -> String {
        "burn-bigram".to_string()
    }
}

impl Default for BurnEngine {
    fn default() -> Self {
        Self::new()
    }
}
