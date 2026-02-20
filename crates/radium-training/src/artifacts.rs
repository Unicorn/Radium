use crate::dataset::DatasetId;
use crate::error::{TrainingError, TrainingResult};
use crate::job::{ModelSpec, TrainingJobId, TrainingObjective};
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};
use std::path::{Path, PathBuf};

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum ArtifactKind {
    FullCheckpoint,
    Adapter,
    Tokenizer,
    Config,
    Metrics,
    DatasetJsonl,
    Other,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TrainingArtifact {
    pub kind: ArtifactKind,
    pub path: PathBuf,
    pub sha256: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct TrainingMetrics {
    pub train_loss: Option<f64>,
    pub eval_loss: Option<f64>,
    pub steps: Option<u64>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TrainingManifest {
    pub job_id: TrainingJobId,
    pub created_at: DateTime<Utc>,
    pub objective: TrainingObjective,
    pub base_model: ModelSpec,
    pub dataset_id: DatasetId,
    #[serde(default)]
    pub metrics: TrainingMetrics,
    pub artifacts: Vec<TrainingArtifact>,
}

pub fn sha256_file(path: &Path) -> TrainingResult<String> {
    let bytes = std::fs::read(path)?;
    let mut hasher = Sha256::new();
    hasher.update(&bytes);
    Ok(hex::encode(hasher.finalize()))
}

pub fn make_artifact(kind: ArtifactKind, path: PathBuf) -> TrainingResult<TrainingArtifact> {
    if !path.exists() {
        return Err(TrainingError::Artifact(format!(
            "artifact path does not exist: {}",
            path.display()
        )));
    }

    let hash = sha256_file(&path)?;
    Ok(TrainingArtifact { kind, path, sha256: hash })
}
