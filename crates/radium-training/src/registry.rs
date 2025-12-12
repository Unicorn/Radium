use crate::artifacts::ArtifactKind;
use crate::error::{TrainingError, TrainingResult};
use crate::layout::TrainingLayout;
use crate::artifacts::TrainingManifest;
use std::path::{Path, PathBuf};

/// A discovered trained model entry.
///
/// v1: these are training jobs that produced a local checkpoint usable by an engine
/// (currently the Burn bigram checkpoint JSON).
#[derive(Debug, Clone)]
pub struct TrainedModelEntry {
    /// Stable identifier used in CLI overrides (e.g. `trained:<job_id>`).
    pub trained_model_id: String,
    /// The underlying engine that can execute this model (e.g. `burn`).
    pub engine_id: String,
    /// Path to the checkpoint to pass to the engine, if applicable.
    pub checkpoint_path: PathBuf,
    /// The job manifest for details/metadata.
    pub manifest: TrainingManifest,
}

#[must_use]
pub fn trained_model_id_for_job(job_id: &str) -> String {
    format!("trained:{job_id}")
}

fn read_manifest(path: &Path) -> TrainingResult<TrainingManifest> {
    let bytes = std::fs::read(path)?;
    Ok(serde_json::from_slice::<TrainingManifest>(&bytes)?)
}

/// Discover trained models by scanning `.radium/_internals/artifacts/training/*/training_manifest.json`.
pub fn discover_trained_models(workspace_root: &Path) -> TrainingResult<Vec<TrainedModelEntry>> {
    let layout = TrainingLayout::for_workspace_root(workspace_root);
    let mut out = Vec::new();

    let dir = match std::fs::read_dir(layout.root()) {
        Ok(d) => d,
        Err(e) if e.kind() == std::io::ErrorKind::NotFound => return Ok(out),
        Err(e) => return Err(e.into()),
    };

    for entry in dir {
        let entry = entry?;
        let job_dir = entry.path();
        if !job_dir.is_dir() {
            continue;
        }
        let manifest_path = job_dir.join("training_manifest.json");
        if !manifest_path.exists() {
            continue;
        }
        let manifest = read_manifest(&manifest_path)?;
        let job_id = manifest.job_id.0.clone();

        // Heuristic engine selection:
        // - If a FullCheckpoint artifact ends with `.json`, assume burn bigram checkpoint.
        let ckpt = manifest
            .artifacts
            .iter()
            .find(|a| a.kind == ArtifactKind::FullCheckpoint)
            .map(|a| a.path.clone())
            .ok_or_else(|| {
                TrainingError::Artifact(format!(
                    "training manifest for job {} has no FullCheckpoint artifact",
                    job_id
                ))
            })?;

        out.push(TrainedModelEntry {
            trained_model_id: trained_model_id_for_job(&job_id),
            engine_id: "burn".to_string(),
            checkpoint_path: ckpt,
            manifest,
        });
    }

    Ok(out)
}

/// Resolve a `trained:<job_id>` model spec into a concrete checkpoint path.
pub fn resolve_trained_model_checkpoint(workspace_root: &Path, trained_model_id: &str) -> TrainingResult<PathBuf> {
    let job_id = trained_model_id
        .strip_prefix("trained:")
        .ok_or_else(|| TrainingError::InvalidSpec(format!("invalid trained model id: {trained_model_id}")))?;

    let layout = TrainingLayout::for_workspace_root(workspace_root);
    let manifest_path = layout.root().join(job_id).join("training_manifest.json");
    if !manifest_path.exists() {
        return Err(TrainingError::InvalidSpec(format!(
            "trained model not found (missing manifest): {trained_model_id}"
        )));
    }
    let manifest = read_manifest(&manifest_path)?;

    let ckpt = manifest
        .artifacts
        .iter()
        .find(|a| a.kind == ArtifactKind::FullCheckpoint)
        .map(|a| a.path.clone())
        .ok_or_else(|| TrainingError::Artifact("no FullCheckpoint artifact in manifest".to_string()))?;

    Ok(ckpt)
}

