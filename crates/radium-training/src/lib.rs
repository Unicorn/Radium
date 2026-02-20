//! Radium Training
//!
//! Backend-agnostic training primitives for:
//! - Defining training jobs (`TrainingJobSpec`)
//! - Representing datasets and examples
//! - Writing training artifacts + manifests
//! - Implementing training backends (`Trainer`)

pub mod artifacts;
pub mod builders;
pub mod dataset;
pub mod error;
pub mod job;
pub mod layout;
pub mod progress;
pub mod registry;
pub mod trainer;

pub use artifacts::{ArtifactKind, TrainingArtifact, TrainingManifest, TrainingMetrics};
pub use builders::{build_dataset, read_jsonl_dataset, write_jsonl_dataset, DatasetBuildOptions};
pub use dataset::{Dataset, DatasetId, DatasetSource, TrainingExample};
pub use error::{TrainingError, TrainingResult};
pub use job::{ModelSpec, TrainingJobId, TrainingJobSpec, TrainingObjective, TrainingResources};
pub use layout::TrainingLayout;
pub use progress::{ProgressEvent, ProgressSink, StdoutProgressSink};
pub use registry::{discover_trained_models, resolve_trained_model_checkpoint, trained_model_id_for_job, TrainedModelEntry};
pub use trainer::{Trainer, TrainerStatus};

