use radium_training::{
    build_dataset, write_jsonl_dataset, ArtifactKind, DatasetBuildOptions, ProgressEvent,
    ProgressSink, Trainer, TrainerStatus, TrainingArtifact, TrainingError, TrainingJobId,
    TrainingJobSpec, TrainingManifest, TrainingMetrics, TrainingResult, TrainingLayout,
    TrainingObjective,
};

use async_trait::async_trait;
use serde::{Deserialize, Serialize};
use std::collections::{BTreeSet, HashMap};
use std::path::{Path, PathBuf};
use std::sync::{Arc, Mutex};

/// A minimal local trainer that produces a character-level bigram checkpoint
/// usable by the `BurnEngine`.
#[derive(Clone)]
pub struct BurnBigramTrainer {
    workspace_root: PathBuf,
    statuses: Arc<Mutex<HashMap<String, TrainerStatus>>>,
}

impl BurnBigramTrainer {
    #[must_use]
    pub fn new(workspace_root: PathBuf) -> Self {
        Self { workspace_root, statuses: Arc::new(Mutex::new(HashMap::new())) }
    }

    fn layout(&self) -> TrainingLayout {
        TrainingLayout::for_workspace_root(&self.workspace_root)
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
struct BigramCheckpoint {
    vocab: Vec<String>,
    transitions: Vec<Vec<f32>>,
}

fn build_bigram_checkpoint(text: &str) -> TrainingResult<BigramCheckpoint> {
    if text.is_empty() {
        return Err(TrainingError::Trainer("training text is empty".to_string()));
    }

    // Stable vocab ordering
    let mut set = BTreeSet::new();
    for ch in text.chars() {
        set.insert(ch);
    }
    let vocab: Vec<char> = set.into_iter().collect();
    let n = vocab.len();
    if n == 0 {
        return Err(TrainingError::Trainer("vocab is empty".to_string()));
    }

    let mut index = HashMap::new();
    for (i, ch) in vocab.iter().enumerate() {
        index.insert(*ch, i);
    }

    // Count transitions with Laplace smoothing
    let mut counts = vec![vec![1f32; n]; n];
    let mut prev: Option<usize> = None;
    for ch in text.chars() {
        let cur = *index
            .get(&ch)
            .ok_or_else(|| TrainingError::Trainer("failed to index char".to_string()))?;
        if let Some(p) = prev {
            counts[p][cur] += 1.0;
        }
        prev = Some(cur);
    }

    Ok(BigramCheckpoint {
        vocab: vocab.into_iter().map(|c| c.to_string()).collect(),
        transitions: counts,
    })
}

fn write_json<P: AsRef<Path>, T: Serialize>(path: P, value: &T) -> TrainingResult<()> {
    let json = serde_json::to_string_pretty(value)?;
    std::fs::write(path, json)?;
    Ok(())
}

#[async_trait]
impl Trainer for BurnBigramTrainer {
    fn id(&self) -> &'static str {
        "burn-bigram"
    }

    async fn prepare(&self, job: &TrainingJobSpec) -> TrainingResult<()> {
        job.validate()?;
        self.layout().ensure_job_dirs(&job.job_id)?;
        Ok(())
    }

    async fn run(&self, job: &TrainingJobSpec, progress: &dyn ProgressSink) -> TrainingResult<TrainingManifest> {
        job.validate()?;

        if job.objective != TrainingObjective::Sft {
            return Err(TrainingError::Trainer("BurnBigramTrainer only supports SFT objective".to_string()));
        }

        let job_id = job.job_id.clone();
        progress.on_event(ProgressEvent::Started { job_id: job_id.clone() });

        {
            if let Ok(mut s) = self.statuses.lock() {
                s.insert(job_id.0.clone(), TrainerStatus::Preparing);
            }
        }

        let layout = self.layout();
        layout.ensure_job_dirs(&job.job_id)?;

        // Build dataset
        progress.on_event(ProgressEvent::Message {
            job_id: job_id.clone(),
            message: "building dataset".to_string(),
        });

        let (dataset, dataset_id) = build_dataset(&job.dataset, &DatasetBuildOptions::default())?;

        let dataset_path = layout.dataset_jsonl_path(&job.job_id);
        write_jsonl_dataset(&dataset_path, &dataset)?;

        // Prepare training text (prompt + response)
        let mut corpus = String::new();
        for ex in &dataset {
            corpus.push_str(&ex.prompt);
            corpus.push('\n');
            corpus.push_str(&ex.response);
            corpus.push('\n');
        }

        // Train bigram checkpoint
        {
            if let Ok(mut s) = self.statuses.lock() {
                s.insert(job_id.0.clone(), TrainerStatus::Running);
            }
        }

        progress.on_event(ProgressEvent::Message {
            job_id: job_id.clone(),
            message: "training burn bigram checkpoint".to_string(),
        });

        let ckpt = build_bigram_checkpoint(&corpus)?;
        let ckpt_path = layout
            .checkpoints_dir(&job.job_id)
            .join("burn_bigram_checkpoint.json");
        write_json(&ckpt_path, &ckpt)?;

        let artifacts = vec![
            TrainingArtifact {
                kind: ArtifactKind::FullCheckpoint,
                path: ckpt_path.clone(),
                sha256: radium_training::artifacts::sha256_file(&ckpt_path)?,
            },
            TrainingArtifact {
                kind: ArtifactKind::DatasetJsonl,
                path: dataset_path.clone(),
                sha256: radium_training::artifacts::sha256_file(&dataset_path)?,
            },
        ];

        let manifest = TrainingManifest {
            job_id: job.job_id.clone(),
            created_at: chrono::Utc::now(),
            objective: job.objective.clone(),
            base_model: job.base_model.clone(),
            dataset_id,
            metrics: TrainingMetrics { train_loss: None, eval_loss: None, steps: None },
            artifacts,
        };

        let manifest_path = layout.job_manifest_path(&job.job_id);
        write_json(manifest_path, &manifest)?;

        {
            if let Ok(mut s) = self.statuses.lock() {
                s.insert(job_id.0.clone(), TrainerStatus::Finished);
            }
        }

        progress.on_event(ProgressEvent::Finished { job_id });
        Ok(manifest)
    }

    async fn status(&self, job_id: &TrainingJobId) -> TrainingResult<TrainerStatus> {
        Ok(self
            .statuses
            .lock()
            .ok()
            .and_then(|s| s.get(&job_id.0).cloned())
            .unwrap_or(TrainerStatus::Idle))
    }

    async fn cancel(&self, job_id: &TrainingJobId) -> TrainingResult<()> {
        if let Ok(mut s) = self.statuses.lock() {
            s.insert(job_id.0.clone(), TrainerStatus::Cancelled);
        }
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use radium_training::{DatasetSource, ModelSpec, StdoutProgressSink};
    use tempfile::TempDir;

    #[tokio::test]
    async fn test_burn_bigram_trainer_writes_checkpoint_and_manifest() {
        let temp = TempDir::new().unwrap();
        let ws = temp.path().to_path_buf();

        // Create a minimal workspace-like structure where TrainingLayout can write.
        std::fs::create_dir_all(ws.join(".radium/_internals/artifacts")).unwrap();

        let trainer = BurnBigramTrainer::new(ws.clone());
        let dataset_dir = ws.join("data");
        std::fs::create_dir_all(&dataset_dir).unwrap();
        std::fs::write(dataset_dir.join("a.txt"), "hello world\n".repeat(200)).unwrap();

        let mut job = TrainingJobSpec::new(
            ModelSpec { engine: "burn".to_string(), model_id: "burn-bigram".to_string() },
            TrainingObjective::Sft,
            DatasetSource::TextFiles { paths: vec![dataset_dir] },
        );
        job.hyperparams.max_seq_len = 256;

        trainer.prepare(&job).await.unwrap();
        let manifest = trainer.run(&job, &StdoutProgressSink).await.unwrap();

        let layout = TrainingLayout::for_workspace_root(&ws);
        let manifest_path = layout.job_manifest_path(&manifest.job_id);
        assert!(manifest_path.exists());
    }
}

