use crate::artifacts::TrainingManifest;
use crate::error::TrainingResult;
use crate::job::{TrainingJobId, TrainingJobSpec};
use crate::progress::ProgressSink;
use async_trait::async_trait;

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum TrainerStatus {
    Idle,
    Preparing,
    Running,
    Finished,
    Failed(String),
    Cancelled,
}

#[async_trait]
pub trait Trainer: Send + Sync {
    fn id(&self) -> &'static str;

    async fn prepare(&self, job: &TrainingJobSpec) -> TrainingResult<()>;

    async fn run(
        &self,
        job: &TrainingJobSpec,
        progress: &dyn ProgressSink,
    ) -> TrainingResult<TrainingManifest>;

    async fn status(&self, job_id: &TrainingJobId) -> TrainingResult<TrainerStatus>;

    async fn cancel(&self, job_id: &TrainingJobId) -> TrainingResult<()>;
}
