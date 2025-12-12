use crate::job::TrainingJobId;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "type", rename_all = "snake_case")]
pub enum ProgressEvent {
    Started { job_id: TrainingJobId },
    Message { job_id: TrainingJobId, message: String },
    Step { job_id: TrainingJobId, step: u64, total: Option<u64> },
    Finished { job_id: TrainingJobId },
}

pub trait ProgressSink: Send + Sync {
    fn on_event(&self, event: ProgressEvent);
}

#[derive(Debug, Default)]
pub struct StdoutProgressSink;

impl ProgressSink for StdoutProgressSink {
    fn on_event(&self, event: ProgressEvent) {
        match event {
            ProgressEvent::Started { job_id } => println!("[train:{job_id}] started"),
            ProgressEvent::Message { job_id, message } => println!("[train:{job_id}] {message}"),
            ProgressEvent::Step { job_id, step, total } => {
                if let Some(total) = total {
                    println!("[train:{job_id}] step {step}/{total}");
                } else {
                    println!("[train:{job_id}] step {step}");
                }
            }
            ProgressEvent::Finished { job_id } => println!("[train:{job_id}] finished"),
        }
    }
}
