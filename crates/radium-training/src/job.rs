use crate::dataset::DatasetSource;
use crate::error::{TrainingError, TrainingResult};
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

/// Identifier for a training job.
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct TrainingJobId(pub String);

impl TrainingJobId {
    #[must_use]
    pub fn new() -> Self {
        Self(Uuid::new_v4().to_string())
    }
}

impl std::fmt::Display for TrainingJobId {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        self.0.fmt(f)
    }
}

/// Backend-agnostic model reference.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ModelSpec {
    /// Engine/provider identifier (e.g., "burn", "aws-bedrock")
    pub engine: String,
    /// Model ID/name (engine-specific)
    pub model_id: String,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum TrainingObjective {
    Sft,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct TrainingResources {
    pub device: TrainingDevice,
    pub max_steps: Option<u64>,
    pub max_seconds: Option<u64>,
}

impl Default for TrainingResources {
    fn default() -> Self {
        Self { device: TrainingDevice::Auto, max_steps: None, max_seconds: None }
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum TrainingDevice {
    Auto,
    Cpu,
    Cuda,
    Metal,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TrainingJobSpec {
    pub job_id: TrainingJobId,
    pub created_at: DateTime<Utc>,
    pub base_model: ModelSpec,
    pub objective: TrainingObjective,
    pub dataset: DatasetSource,
    pub hyperparams: TrainingHyperParams,
    pub resources: TrainingResources,
}

impl TrainingJobSpec {
    #[must_use]
    pub fn new(base_model: ModelSpec, objective: TrainingObjective, dataset: DatasetSource) -> Self {
        Self {
            job_id: TrainingJobId::new(),
            created_at: Utc::now(),
            base_model,
            objective,
            dataset,
            hyperparams: TrainingHyperParams::default(),
            resources: TrainingResources::default(),
        }
    }

    pub fn validate(&self) -> TrainingResult<()> {
        if self.base_model.engine.trim().is_empty() {
            return Err(TrainingError::InvalidSpec("base_model.engine is required".to_string()));
        }
        if self.base_model.model_id.trim().is_empty() {
            return Err(TrainingError::InvalidSpec("base_model.model_id is required".to_string()));
        }
        self.hyperparams.validate()?;
        Ok(())
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TrainingHyperParams {
    pub seed: u64,
    pub epochs: u32,
    pub learning_rate: f64,
    pub batch_size: u32,
    pub max_seq_len: u32,
}

impl Default for TrainingHyperParams {
    fn default() -> Self {
        Self { seed: 42, epochs: 1, learning_rate: 2e-5, batch_size: 1, max_seq_len: 2048 }
    }
}

impl TrainingHyperParams {
    pub fn validate(&self) -> TrainingResult<()> {
        if self.epochs == 0 {
            return Err(TrainingError::InvalidSpec("epochs must be >= 1".to_string()));
        }
        if !(self.learning_rate.is_finite()) || self.learning_rate <= 0.0 {
            return Err(TrainingError::InvalidSpec("learning_rate must be > 0".to_string()));
        }
        if self.batch_size == 0 {
            return Err(TrainingError::InvalidSpec("batch_size must be >= 1".to_string()));
        }
        if self.max_seq_len == 0 {
            return Err(TrainingError::InvalidSpec("max_seq_len must be >= 1".to_string()));
        }
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_job_spec_validate_requires_base_model_fields() {
        let spec = TrainingJobSpec::new(
            ModelSpec { engine: "".to_string(), model_id: "".to_string() },
            TrainingObjective::Sft,
            DatasetSource::Jsonl { path: std::path::PathBuf::from("x.jsonl") },
        );
        assert!(spec.validate().is_err());
    }
}
