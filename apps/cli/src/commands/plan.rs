//! Plan command implementation.
//!
//! Generates structured plans from specification files.

use anyhow::{Context, bail};
use colored::Colorize;
use radium_core::{Iteration, Plan, PlanManifest, PlanStatus, PlanTask, RequirementId, Workspace};
use std::fs;
use std::path::{Path, PathBuf};

/// Execute the plan command.
///
/// Generates a structured plan from a specification file.
pub async fn execute(
    input: Option<String>,
    id: Option<String>,
    name: Option<String>,
) -> anyhow::Result<()> {
    println!("{}", "rad plan".bold().cyan());
    println!();

    // Get or discover workspace
    let workspace = Workspace::discover().context("Failed to discover workspace")?;

    // Ensure workspace structure exists
    workspace.ensure_structure().context("Failed to ensure workspace structure")?;

    // Get input
    let input_str = input
        .ok_or_else(|| anyhow::anyhow!("Input required: file path or specification content"))?;

    // Check if input is a file path
    let input_path = PathBuf::from(&input_str);
    let (spec_content, source_desc) = if input_path.exists() && input_path.is_file() {
        let content = fs::read_to_string(&input_path)
            .context(format!("Failed to read specification file: {}", input_str))?;
        (content, format!("File: {}", input_str))
    } else {
        // Treat as direct content
        (input_str, "Direct input".to_string())
    };

    println!("  Source: {}", source_desc.green());
    println!("  Size: {} bytes", spec_content.len().to_string().dimmed());
    println!();

    // Generate or use provided requirement ID
    let requirement_id = if let Some(id_str) = id {
        // Parse existing ID
        id_str.parse().context(format!("Invalid requirement ID format: {}", id_str))?
    } else {
        // Generate next ID
        RequirementId::next(workspace.root().join(".radium"))
            .context("Failed to generate requirement ID")?
    };

    println!("  Requirement ID: {}", requirement_id.to_string().green());

    // Determine folder name
    let folder_name = if let Some(custom_name) = name {
        format!("{}-{}", requirement_id, custom_name)
    } else {
        // Extract project name from spec file
        let project_name =
            extract_project_name(&spec_content).unwrap_or_else(|| "project".to_string());
        format!("{}-{}", requirement_id, slugify(&project_name))
    };

    println!("  Folder name: {}", folder_name.green());
    println!();

    // Create plan directory in backlog stage
    let plan_dir = workspace.root().join("radium").join("backlog").join(&folder_name);

    if plan_dir.exists() {
        bail!("Plan directory already exists: {}\nUse a different ID or name.", plan_dir.display());
    }

    println!("{}", "Creating plan structure...".bold());

    // Create plan directory structure
    create_plan_structure(&plan_dir).context("Failed to create plan structure")?;
    println!("  ✓ Created plan directories");

    // Copy specification file
    let spec_dest = plan_dir.join("specifications.md");
    fs::write(&spec_dest, spec_content.as_bytes()).context("Failed to write specifications.md")?;
    println!("  ✓ Copied specification file");

    // Generate basic plan from specification
    println!();
    println!("{}", "Generating plan...".bold());

    let plan = generate_basic_plan(&spec_content, &requirement_id, &folder_name)?;
    println!("  ✓ Generated {} iterations", plan.total_iterations);
    println!("  ✓ Generated {} tasks", plan.total_tasks);

    // Save plan.json
    let plan_json_path = plan_dir.join("plan.json");
    let plan_json = serde_json::to_string_pretty(&plan).context("Failed to serialize plan")?;
    fs::write(&plan_json_path, plan_json).context("Failed to write plan.json")?;
    println!("  ✓ Saved plan.json");

    // Generate plan manifest
    let manifest = generate_manifest(&plan, &spec_content);
    let manifest_path = plan_dir.join("plan").join("plan_manifest.json");
    let manifest_json =
        serde_json::to_string_pretty(&manifest).context("Failed to serialize manifest")?;
    fs::write(&manifest_path, manifest_json).context("Failed to write plan_manifest.json")?;
    println!("  ✓ Saved plan_manifest.json");

    println!();
    println!("{}", "Plan generated successfully!".green().bold());
    println!();
    println!("  Location: {}", plan_dir.display().to_string().cyan());
    println!("  Next step: {}", format!("rad craft {}", requirement_id).cyan());
    println!();

    Ok(())
}

/// Creates the plan directory structure.
fn create_plan_structure(plan_dir: &Path) -> anyhow::Result<()> {
    fs::create_dir_all(plan_dir)?;
    fs::create_dir_all(plan_dir.join("plan"))?;
    fs::create_dir_all(plan_dir.join("artifacts").join("architecture"))?;
    fs::create_dir_all(plan_dir.join("artifacts").join("tasks"))?;
    fs::create_dir_all(plan_dir.join("memory"))?;
    fs::create_dir_all(plan_dir.join("prompts"))?;
    Ok(())
}

/// Generates a basic plan from specification content.
fn generate_basic_plan(
    spec_content: &str,
    requirement_id: &RequirementId,
    folder_name: &str,
) -> anyhow::Result<Plan> {
    // Extract project name
    let project_name =
        extract_project_name(spec_content).unwrap_or_else(|| "Untitled Project".to_string());

    // Parse iterations and tasks from spec (simple parsing for now)
    let (iterations, total_tasks) = parse_spec_structure(spec_content);

    let mut plan = Plan::new(
        *requirement_id,
        project_name,
        folder_name.to_string(),
        "backlog".to_string(),
    );

    plan.total_iterations = iterations as u32;
    plan.total_tasks = total_tasks as u32;

    Ok(plan)
}

/// Generates a plan manifest with iterations and tasks.
fn generate_manifest(plan: &Plan, _spec_content: &str) -> PlanManifest {
    use std::collections::HashMap;

    // Generate basic iterations (for now, just create I1, I2, I3)
    let iterations = (1..=plan.total_iterations)
        .map(|i| Iteration {
            id: format!("I{}", i),
            number: i,
            name: format!("Iteration {}", i),
            description: Some(format!("Iteration {} tasks", i)),
            goal: Some(format!("Complete iteration {}", i)),
            tasks: vec![PlanTask {
                id: format!("I{}.T1", i),
                number: 1,
                title: format!("Task 1 for iteration {}", i),
                description: Some("Generated task".to_string()),
                completed: false,
                agent_id: None,
                dependencies: Vec::new(),
                acceptance_criteria: Vec::new(),
                metadata: HashMap::new(),
            }],
            status: PlanStatus::NotStarted,
            metadata: HashMap::new(),
        })
        .collect();

    let mut metadata = HashMap::new();
    metadata.insert("created_at".to_string(), serde_json::json!(plan.created_at.to_rfc3339()));
    metadata.insert("updated_at".to_string(), serde_json::json!(plan.updated_at.to_rfc3339()));
    metadata.insert("total_iterations".to_string(), serde_json::json!(plan.total_iterations));

    PlanManifest {
        requirement_id: plan.requirement_id,
        project_name: plan.project_name.clone(),
        iterations,
        metadata,
    }
}

/// Extracts project name from specification content.
fn extract_project_name(content: &str) -> Option<String> {
    // Look for first # header
    for line in content.lines() {
        let trimmed = line.trim();
        if trimmed.starts_with("# ") {
            return Some(trimmed[2..].trim().to_string());
        }
    }
    None
}

/// Parses specification structure to count iterations and tasks.
fn parse_spec_structure(content: &str) -> (usize, usize) {
    let mut iterations = 0;
    let mut tasks = 0;

    for line in content.lines() {
        let trimmed = line.trim();
        // Count ## headers as iterations
        if trimmed.starts_with("## ") {
            iterations += 1;
        }
        // Count - [ ] as tasks
        if trimmed.starts_with("- [ ]") || trimmed.starts_with("* [ ]") {
            tasks += 1;
        }
    }

    // Ensure at least 1 iteration
    if iterations == 0 {
        iterations = 1;
    }
    // Ensure at least 1 task per iteration
    if tasks == 0 {
        tasks = iterations;
    }

    (iterations, tasks)
}

/// Converts a string to a URL-friendly slug.
fn slugify(s: &str) -> String {
    s.to_lowercase()
        .chars()
        .map(|c| {
            if c.is_alphanumeric() {
                c
            } else if c.is_whitespace() || c == '-' || c == '_' {
                '-'
            } else {
                ' '
            }
        })
        .collect::<String>()
        .split_whitespace()
        .collect::<Vec<_>>()
        .join("-")
}
