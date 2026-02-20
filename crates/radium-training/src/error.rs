use thiserror::Error;

pub type TrainingResult<T> = std::result::Result<T, TrainingError>;

#[derive(Debug, Error)]
pub enum TrainingError {
    #[error("invalid training job spec: {0}")]
    InvalidSpec(String),

    #[error("dataset error: {0}")]
    Dataset(String),

    #[error("artifact error: {0}")]
    Artifact(String),

    #[error("trainer error: {0}")]
    Trainer(String),

    #[error(transparent)]
    Io(#[from] std::io::Error),

    #[error(transparent)]
    Json(#[from] serde_json::Error),

    #[error(transparent)]
    Other(#[from] anyhow::Error),
}
