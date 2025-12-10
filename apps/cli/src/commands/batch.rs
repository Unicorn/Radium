//! Batch command implementation.
//!
//! Executes agents with multiple prompts from input files in parallel.

use anyhow::{Context, bail};
use clap::Subcommand;
use std::path::PathBuf;

/// Batch command actions.
#[derive(Subcommand, Debug)]
pub enum BatchAction {
    /// Run batch execution with an agent
    Run {
        /// Agent ID to execute
        agent_id: String,

        /// Input file with prompts (line-delimited or JSON array)
        #[arg(long)]
        input_file: PathBuf,

        /// Maximum concurrent executions (default: 5, max: 20)
        #[arg(long, default_value = "5")]
        concurrency: usize,

        /// Stop on first error (default: continue)
        #[arg(long)]
        fail_fast: bool,

        /// Output directory for individual results
        #[arg(long)]
        output_dir: Option<PathBuf>,
    },
}

/// Execute batch command.
pub async fn execute(action: BatchAction) -> anyhow::Result<()> {
    match action {
        BatchAction::Run {
            agent_id,
            input_file,
            concurrency,
            fail_fast,
            output_dir,
        } => execute_run(agent_id, input_file, concurrency, fail_fast, output_dir).await,
    }
}

/// Execute batch run command.
async fn execute_run(
use colored::Colorize;
use radium_core::{
    batch::{
        parse_input_file, BatchInput, BatchProcessor, BatchProgressTracker, RetryPolicy,
        render_progress, render_summary,
    },
    context::ContextFileLoader,
    AgentDiscovery, PromptContext, PromptTemplate, Workspace,
};
use radium_models::ModelFactory;
use std::io::Write;
use std::path::PathBuf;
use std::sync::Arc;
use std::time::Duration;
use tokio::signal;

async fn execute_run(
    agent_id: String,
    input_file: PathBuf,
    concurrency: usize,
    fail_fast: bool,
    output_dir: Option<PathBuf>,
) -> anyhow::Result<()> {
    println!("{}", "rad batch run".bold().cyan());
    println!();

    // Validate concurrency
    if concurrency == 0 || concurrency > 20 {
        bail!("Concurrency must be between 1 and 20");
    }

    // Discover workspace
    let workspace = Workspace::discover().ok();
    let workspace_root = workspace
        .as_ref()
        .map(|w| w.root().to_path_buf())
        .unwrap_or_else(|| std::env::current_dir().unwrap_or_else(|_| PathBuf::from(".")));

    // Load context files if available
    let loader = ContextFileLoader::new(&workspace_root);
    let current_dir = std::env::current_dir().unwrap_or_else(|| workspace_root.clone());
    let context_files = loader.load_hierarchical(&current_dir).unwrap_or_default();

    // Discover agents
    println!("  {}", "Discovering agents...".dimmed());
    let discovery = AgentDiscovery::new();
    let agents = discovery.discover_all().context("Failed to discover agents")?;

    if agents.is_empty() {
        bail!("No agents found. Place agent configs in ./agents/ or ~/.radium/agents/");
    }

    // Find the requested agent
    let agent = agents
        .get(&agent_id)
        .ok_or_else(|| anyhow::anyhow!("Agent not found: {}", agent_id))?;

    println!("  {} Agent: {}", "✓".green(), agent_id.cyan());
    println!("  {} Input file: {}", "✓".green(), input_file.display());

    // Parse input file
    println!();
    println!("  {}", "Parsing input file...".dimmed());
    let inputs = parse_input_file(&input_file)
        .with_context(|| format!("Failed to parse input file: {}", input_file.display()))?;

    println!("  {} Loaded {} prompts", "✓".green(), inputs.len());

    // Create output directory if specified
    if let Some(ref dir) = output_dir {
        std::fs::create_dir_all(dir)
            .with_context(|| format!("Failed to create output directory: {}", dir.display()))?;
        println!("  {} Output directory: {}", "✓".green(), dir.display());
    }

    // Load agent prompt template
    let prompt_content = std::fs::read_to_string(&agent.prompt_path)
        .with_context(|| format!("Failed to read prompt file: {}", agent.prompt_path.display()))?;
    let template = PromptTemplate::from_string(prompt_content);

    // Create batch processor
    let timeout = Duration::from_secs(300); // 5 minutes default
    let retry_policy = RetryPolicy::default();
    let processor = BatchProcessor::new(concurrency, timeout, retry_policy);

    // Create progress tracker
    let mut progress_tracker = BatchProgressTracker::new(inputs.len());
    let progress_callback: Arc<dyn Fn(usize, usize, usize, usize, usize) + Send + Sync> =
        Arc::new({
            let mut tracker = progress_tracker.clone();
            move |index, completed, active, successful, failed| {
                tracker.update(index, completed, active, successful, failed);
                let _ = render_progress(&tracker, &agent_id);
            }
        });

    // Setup Ctrl+C handler
    let mut ctrl_c_stream = signal::ctrl_c();
    let cancelled = Arc::new(tokio::sync::Mutex::new(false));

    // Spawn cancellation handler
    let cancelled_clone = Arc::clone(&cancelled);
    tokio::spawn(async move {
        if ctrl_c_stream.recv().await.is_some() {
            *cancelled_clone.lock().await = true;
            println!("\n{} Cancellation requested, waiting for active requests...", "⚠".yellow());
        }
    });

    println!();
    println!("{}", "Starting batch execution...".bold());
    println!();

    // Define processor function - need to capture index
    let agent_id_clone = agent_id.clone();
    let template_clone = template.clone();
    let context_files_clone = context_files.clone();
    let output_dir_clone = output_dir.clone();
    let cancelled_check = Arc::clone(&cancelled);
    let inputs_clone = inputs.clone();

    // Create a map of inputs to indices for saving files
    let input_to_index: std::collections::HashMap<_, _> = inputs
        .iter()
        .enumerate()
        .map(|(i, input)| {
            // Use prompt as key (simple approach)
            (input.prompt.clone(), i)
        })
        .collect();

    let processor_fn = move |input: BatchInput| {
        let agent_id = agent_id_clone.clone();
        let template = template_clone.clone();
        let context_files = context_files_clone.clone();
        let output_dir = output_dir_clone.clone();
        let cancelled = cancelled_check.clone();
        let input_to_index = input_to_index.clone();
        let index = input_to_index.get(&input.prompt).copied().unwrap_or(0);

        async move {
            // Check if cancelled
            if *cancelled.lock().await {
                return Err("Cancelled".to_string());
            }

            // Render prompt
            let mut context = PromptContext::new();
            context.set("user_input", input.prompt.clone());
            if let Some(ctx) = input.context.clone() {
                context.set("context", ctx);
            }
            if !context_files.is_empty() {
                context.set("context_files", context_files.clone());
            }

            let rendered = template.render(&context)?;

            // Create model and execute
            let engine = "mock"; // Default for now
            let model = ModelFactory::create_from_str(engine, None)
                .map_err(|e| format!("Failed to create model: {}", e))?;

            let response = model
                .generate(&rendered, None)
                .await
                .map_err(|e| format!("Model generation failed: {}", e))?;

            let result_text = response.text().unwrap_or_default();

            // Return result with metadata for saving later
            Ok(serde_json::json!({
                "index": index,
                "prompt": input.prompt,
                "response": result_text,
                "context": input.context,
            }).to_string())
        }
    };

    // Process batch
    let result = processor
        .process_batch(inputs, processor_fn, Some(progress_callback))
        .await;

    // Wait for active requests to complete (with timeout)
    if *cancelled.lock().await {
        tokio::time::sleep(Duration::from_secs(30)).await;
    }

    println!();

    // Update final progress
    progress_tracker.completed = result.total_items();
    progress_tracker.successful = result.successful.len();
    progress_tracker.failed = result.failed.len();

    // Render summary
    render_summary(&progress_tracker, &result.failed, output_dir.as_deref())?;

    // Handle failures
    if !result.failed.is_empty() {
        if fail_fast {
            bail!("Batch execution failed with {} errors (fail-fast mode)", result.failed.len());
        } else {
            eprintln!(
                "\n{} {} requests failed",
                "⚠".yellow(),
                result.failed.len()
            );
        }
    }

    Ok(())
}

