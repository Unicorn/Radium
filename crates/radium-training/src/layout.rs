use crate::error::TrainingResult;
use crate::job::TrainingJobId;
use std::path::{Path, PathBuf};

/// Filesystem layout for training artifacts inside a workspace.
///
/// Default layout is under `.radium/_internals/artifacts/training/<job_id>/...`
#[derive(Debug, Clone)]
pub struct TrainingLayout {
    root: PathBuf,
}

impl TrainingLayout {
    #[must_use]
    pub fn new(root: PathBuf) -> Self {
        Self { root }
    }

    /// Create a layout rooted in a Radium workspace root.
    #[must_use]
    pub fn for_workspace_root(workspace_root: &Path) -> Self {
        Self::new(
            workspace_root
                .join(".radium")
                .join("_internals")
                .join("artifacts")
                .join("training"),
        )
    }

    #[must_use]
    pub fn root(&self) -> &Path {
        &self.root
    }

    #[must_use]
    pub fn job_dir(&self, job_id: &TrainingJobId) -> PathBuf {
        self.root.join(job_id.0.as_str())
    }

    #[must_use]
    pub fn job_manifest_path(&self, job_id: &TrainingJobId) -> PathBuf {
        self.job_dir(job_id).join("training_manifest.json")
    }

    #[must_use]
    pub fn dataset_jsonl_path(&self, job_id: &TrainingJobId) -> PathBuf {
        self.job_dir(job_id).join("dataset.jsonl")
    }

    #[must_use]
    pub fn checkpoints_dir(&self, job_id: &TrainingJobId) -> PathBuf {
        self.job_dir(job_id).join("checkpoints")
    }

    pub fn ensure_job_dirs(&self, job_id: &TrainingJobId) -> TrainingResult<()> {
        std::fs::create_dir_all(self.job_dir(job_id))?;
        std::fs::create_dir_all(self.checkpoints_dir(job_id))?;
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::TempDir;

    #[test]
    fn test_layout_paths() {
        let temp = TempDir::new().unwrap();
        let layout = TrainingLayout::for_workspace_root(temp.path());
        let id = TrainingJobId("job-1".to_string());

        assert!(layout.root().to_string_lossy().contains(".radium"));
        assert!(layout.job_dir(&id).to_string_lossy().contains("job-1"));
    }
}
