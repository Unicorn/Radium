//! Training command implementation.

use crate::commands::types::{AwsTrainCommand, TrainCommand};
use anyhow::{Context, Result};
use colored::Colorize;
use radium_core::workspace::Workspace;
use radium_core::training::BurnBigramTrainer;
use radium_training::Trainer;
use serde_json::json;
use std::path::PathBuf;

pub async fn execute(command: TrainCommand) -> Result<()> {
    match command {
        TrainCommand::List { json: json_output } => list_trained_models(json_output).await,
        TrainCommand::Bigram { text_dir, json: json_output } => train_bigram(text_dir, json_output).await,
        TrainCommand::Aws(command) => aws_dispatch(command).await,
    }
}

async fn list_trained_models(json_output: bool) -> Result<()> {
    let workspace = Workspace::discover().context("Failed to discover workspace. Run `rad init` first.")?;
    let models = radium_training::discover_trained_models(workspace.root())
        .context("Failed to discover trained models")?;

    if json_output {
        let out: Vec<_> = models
            .into_iter()
            .map(|m| {
                json!({
                    "id": m.trained_model_id,
                    "engine": m.engine_id,
                    "checkpoint_path": m.checkpoint_path,
                    "job_id": m.manifest.job_id.0,
                    "created_at": m.manifest.created_at,
                    "objective": format!("{:?}", m.manifest.objective),
                    "base_model": m.manifest.base_model,
                    "dataset_id": m.manifest.dataset_id.0,
                })
            })
            .collect();
        println!("{}", serde_json::to_string_pretty(&out)?);
        return Ok(());
    }

    println!();
    println!("{}", format!("Trained Models ({})", models.len()).bold().cyan());
    println!();

    if models.is_empty() {
        println!("  {}", "No trained models found for this workspace.".dimmed());
        println!();
        println!("  {}", "Tip: run a local training job to produce a checkpoint, then use it via `rad step --engine burn --model trained:<job_id>`.".dimmed());
        return Ok(());
    }

    println!("{:<28} {:<8} {}", "ID", "Engine", "Checkpoint");
    println!("{}", "─".repeat(90));
    for m in models {
        println!(
            "{:<28} {:<8} {}",
            m.trained_model_id.cyan(),
            m.engine_id.dimmed(),
            m.checkpoint_path.display().to_string().dimmed()
        );
    }
    println!();
    Ok(())
}

async fn train_bigram(text_dirs: Vec<PathBuf>, json_output: bool) -> Result<()> {
    let workspace = Workspace::discover().context("Failed to discover workspace. Run `rad init` first.")?;

    // Ensure the training artifact root exists.
    std::fs::create_dir_all(workspace.root().join(".radium/_internals/artifacts/training"))?;

    let trainer = BurnBigramTrainer::new(workspace.root().to_path_buf());
    let job = radium_training::TrainingJobSpec::new(
        radium_training::ModelSpec { engine: "burn".to_string(), model_id: "burn-bigram".to_string() },
        radium_training::TrainingObjective::Sft,
        radium_training::DatasetSource::TextFiles { paths: text_dirs },
    );

    trainer.prepare(&job).await?;
    let manifest = trainer.run(&job, &radium_training::StdoutProgressSink).await?;

    if json_output {
        println!("{}", serde_json::to_string_pretty(&manifest)?);
        return Ok(());
    }

    let trained_id = radium_training::trained_model_id_for_job(&manifest.job_id.0);
    println!();
    println!("{}", "Local training complete".bold().green());
    println!("  Job: {}", manifest.job_id.0.cyan());
    println!("  Use: {}", format!("rad step --engine burn --model {}", trained_id).dimmed());
    println!();
    Ok(())
}

#[cfg(not(feature = "aws-backend"))]
async fn aws_dispatch(_cmd: AwsTrainCommand) -> Result<()> {
    anyhow::bail!(
        "AWS training backend is not enabled in this build. Rebuild with the `aws-backend` feature."
    )
}

#[cfg(feature = "aws-backend")]
async fn aws_dispatch(cmd: AwsTrainCommand) -> Result<()> {
    match cmd {
        AwsTrainCommand::Bootstrap { region, bucket, config_path } => aws_bootstrap(region, bucket, config_path).await,
        AwsTrainCommand::Deploy { config_path } => aws_deploy(config_path).await,
    }
}

#[cfg(feature = "aws-backend")]
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
struct AwsTrainingConfig {
    region: String,
    s3_bucket: String,
}

#[cfg(feature = "aws-backend")]
fn default_aws_config_path(workspace: &Workspace) -> PathBuf {
    workspace.radium_dir().join("aws").join("training").join("config.json")
}

#[cfg(feature = "aws-backend")]
async fn aws_bootstrap(region: Option<String>, bucket: Option<String>, config_path: Option<PathBuf>) -> Result<()> {
    let workspace = Workspace::discover().context("Failed to discover workspace. Run `rad init` first.")?;
    let path = config_path.unwrap_or_else(|| default_aws_config_path(&workspace));
    if let Some(parent) = path.parent() {
        std::fs::create_dir_all(parent)?;
    }

    let region = region.unwrap_or_else(|| "us-east-1".to_string());
    let bucket = bucket.unwrap_or_else(|| format!("radium-training-{}", uuid::Uuid::new_v4()));

    let cfg = AwsTrainingConfig { region, s3_bucket: bucket };
    std::fs::write(&path, serde_json::to_string_pretty(&cfg)?)?;

    println!();
    println!("{}", "AWS training backend bootstrap".bold().cyan());
    println!("  Wrote: {}", path.display().to_string().dimmed());
    println!();
    println!("  {}", "Next: run `rad train aws deploy` to validate prerequisites and print the deployment plan.".dimmed());
    println!();
    Ok(())
}

#[cfg(feature = "aws-backend")]
async fn aws_deploy(config_path: Option<PathBuf>) -> Result<()> {
    let workspace = Workspace::discover().context("Failed to discover workspace. Run `rad init` first.")?;
    let path = config_path.unwrap_or_else(|| default_aws_config_path(&workspace));
    let cfg: AwsTrainingConfig = serde_json::from_slice(&std::fs::read(&path).with_context(|| {
        format!("Failed to read AWS training config: {}", path.display())
    })?)?;

    // v1 scaffold: verify `aws` CLI exists and print a deploy plan.
    let aws_ok = std::process::Command::new("aws")
        .arg("--version")
        .output()
        .is_ok();
    if !aws_ok {
        anyhow::bail!("AWS CLI not found on PATH. Install `awscli` and re-run.");
    }

    println!();
    println!("{}", "AWS training backend deploy (scaffold)".bold().cyan());
    println!("  Region: {}", cfg.region.cyan());
    println!("  S3 bucket: {}", cfg.s3_bucket.cyan());
    println!();
    println!("  {}", "This is a scaffold command (no infrastructure is created yet).".yellow());
    println!("  {}", "Planned resources for Phase 3: S3 (artifacts), IAM role/policy, SageMaker training job templates, optional Bedrock adapter targets.".dimmed());
    println!();
    Ok(())
}

