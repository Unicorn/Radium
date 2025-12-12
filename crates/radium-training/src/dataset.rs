use crate::error::{TrainingError, TrainingResult};
use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};
use std::path::PathBuf;

/// Stable identifier for a dataset (content hash).
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct DatasetId(pub String);

/// A single training example for SFT-style training.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TrainingExample {
    pub prompt: String,
    pub response: String,
    #[serde(default)]
    pub metadata: serde_json::Value,
}

pub type Dataset = Vec<TrainingExample>;

/// Where dataset comes from (builder responsibility).
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "type", rename_all = "snake_case")]
pub enum DatasetSource {
    /// Build dataset from repo scanning / code analysis.
    RepoScan {
        root: PathBuf,
        #[serde(default)]
        depth: ScanDepth,
    },
    /// Build dataset from local text files/directories.
    TextFiles {
        paths: Vec<PathBuf>,
    },
    /// Use an existing JSONL dataset (each line is a `TrainingExample`).
    Jsonl {
        path: PathBuf,
    },
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum ScanDepth {
    Quick,
    Full,
}

impl Default for ScanDepth {
    fn default() -> Self {
        Self::Quick
    }
}

pub fn compute_dataset_id(examples: &[TrainingExample]) -> TrainingResult<DatasetId> {
    let mut hasher = Sha256::new();

    for ex in examples {
        let bytes = serde_json::to_vec(ex)?;
        hasher.update(bytes);
        hasher.update(b"\n");
    }

    Ok(DatasetId(hex::encode(hasher.finalize())))
}

pub fn validate_examples(examples: &[TrainingExample]) -> TrainingResult<()> {
    if examples.is_empty() {
        return Err(TrainingError::Dataset("dataset must not be empty".to_string()));
    }
    for (idx, ex) in examples.iter().enumerate() {
        if ex.prompt.trim().is_empty() {
            return Err(TrainingError::Dataset(format!("example[{idx}] prompt is empty")));
        }
        if ex.response.trim().is_empty() {
            return Err(TrainingError::Dataset(format!("example[{idx}] response is empty")));
        }
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_validate_examples_rejects_empty() {
        let examples: Vec<TrainingExample> = vec![];
        assert!(validate_examples(&examples).is_err());
    }

    #[test]
    fn test_compute_dataset_id_stable_for_same_content() {
        let examples = vec![
            TrainingExample {
                prompt: "p1".to_string(),
                response: "r1".to_string(),
                metadata: serde_json::json!({"a": 1}),
            },
            TrainingExample {
                prompt: "p2".to_string(),
                response: "r2".to_string(),
                metadata: serde_json::json!({}),
            },
        ];

        let id1 = compute_dataset_id(&examples).unwrap();
        let id2 = compute_dataset_id(&examples).unwrap();
        assert_eq!(id1, id2);
    }
}
