//! Workflow management commands

use clap::Subcommand;
use radium_core::radium_client::RadiumClient;
use tonic::Request;

/// Workflow management subcommands
#[derive(Subcommand, Debug)]
pub enum WorkflowCommand {
    /// Create a new workflow
    Create {
        /// Workflow ID
        #[arg(short, long)]
        id: Option<String>,
        /// Workflow name
        #[arg(short, long)]
        name: Option<String>,
        /// Workflow description
        #[arg(short, long)]
        description: Option<String>,
    },
    /// List all workflows
    List,
    /// Get workflow details
    Get {
        /// Workflow ID
        id: String,
    },
    /// Execute a workflow
    Execute {
        /// Workflow ID
        id: String,
        /// Execute steps in parallel when possible
        #[arg(short, long, default_value = "false")]
        parallel: bool,
    },
    /// Update a workflow
    Update {
        /// Workflow ID
        id: String,
        /// Workflow name
        #[arg(short, long)]
        name: Option<String>,
        /// Workflow description
        #[arg(short, long)]
        description: Option<String>,
    },
    /// Delete a workflow
    Delete {
        /// Workflow ID
        id: String,
    },
}

/// Execute workflow command
pub async fn execute_workflow_command(
    command: WorkflowCommand,
    client: &mut RadiumClient<tonic::transport::Channel>,
) -> Result<(), Box<dyn std::error::Error>> {
    match command {
        WorkflowCommand::Create { id, name, description } => {
            create_workflow(client, id, name, description).await
        }
        WorkflowCommand::List => list_workflows(client).await,
        WorkflowCommand::Get { id } => get_workflow(client, id).await,
        WorkflowCommand::Execute { id, parallel } => execute_workflow(client, id, parallel).await,
        WorkflowCommand::Update { id, name, description } => {
            update_workflow(client, id, name, description).await
        }
        WorkflowCommand::Delete { id } => delete_workflow(client, id).await,
    }
}

async fn create_workflow(
    client: &mut RadiumClient<tonic::transport::Channel>,
    id: Option<String>,
    name: Option<String>,
    description: Option<String>,
) -> Result<(), Box<dyn std::error::Error>> {
    use inquire::Text;

    let workflow_id = id
        .unwrap_or_else(|| Text::new("Workflow ID:").prompt().expect("Failed to read workflow ID"));

    let workflow_name = name.unwrap_or_else(|| {
        Text::new("Workflow name:").prompt().expect("Failed to read workflow name")
    });

    let workflow_description = description.unwrap_or_else(|| {
        Text::new("Workflow description:").prompt().expect("Failed to read workflow description")
    });

    let workflow = radium_core::proto::Workflow {
        id: workflow_id.clone(),
        name: workflow_name,
        description: workflow_description,
        steps: vec![],
        state: "\"idle\"".to_string(),
        created_at: chrono::Utc::now().to_rfc3339(),
        updated_at: chrono::Utc::now().to_rfc3339(),
    };

    let request =
        Request::new(radium_core::proto::CreateWorkflowRequest { workflow: Some(workflow) });
    let response = client.create_workflow(request).await?;

    println!("Workflow created with ID: {}", response.into_inner().workflow_id);
    Ok(())
}

async fn list_workflows(
    client: &mut RadiumClient<tonic::transport::Channel>,
) -> Result<(), Box<dyn std::error::Error>> {
    use tabled::{Table, Tabled};

    #[derive(Tabled)]
    struct WorkflowRow {
        id: String,
        name: String,
        description: String,
        steps: usize,
        state: String,
        created_at: String,
    }

    let request = Request::new(radium_core::proto::ListWorkflowsRequest {});
    let response = client.list_workflows(request).await?;
    let workflows = response.into_inner().workflows;

    let rows: Vec<WorkflowRow> = workflows
        .into_iter()
        .map(|w| WorkflowRow {
            id: w.id,
            name: w.name,
            description: w.description,
            steps: w.steps.len(),
            state: w.state,
            created_at: w.created_at,
        })
        .collect();

    println!("{}", Table::new(rows));
    Ok(())
}

async fn get_workflow(
    client: &mut RadiumClient<tonic::transport::Channel>,
    id: String,
) -> Result<(), Box<dyn std::error::Error>> {
    use colored::*;

    let request = Request::new(radium_core::proto::GetWorkflowRequest { workflow_id: id.clone() });
    let response = client.get_workflow(request).await?;
    let workflow = response.into_inner().workflow;

    if let Some(workflow) = workflow {
        println!("{} {}", "Workflow ID:".bright_cyan(), workflow.id.bright_white());
        println!("{} {}", "Name:".bright_cyan(), workflow.name.bright_white());
        println!("{} {}", "Description:".bright_cyan(), workflow.description.bright_white());
        println!("{} {}", "State:".bright_cyan(), workflow.state.bright_white());
        println!("{} {}", "Steps:".bright_cyan(), workflow.steps.len());
        for (idx, step) in workflow.steps.iter().enumerate() {
            println!(
                "  {} {}: {} (Task: {})",
                "Step".bright_cyan(),
                idx + 1,
                step.name.bright_white(),
                step.task_id.bright_white()
            );
        }
        println!("{} {}", "Created:".bright_cyan(), workflow.created_at.bright_white());
        println!("{} {}", "Updated:".bright_cyan(), workflow.updated_at.bright_white());
    } else {
        eprintln!("{} Workflow not found: {}", "Error:".red().bold(), id);
        return Err("Workflow not found".into());
    }

    Ok(())
}

async fn execute_workflow(
    client: &mut RadiumClient<tonic::transport::Channel>,
    id: String,
    parallel: bool,
) -> Result<(), Box<dyn std::error::Error>> {
    use colored::*;

    let request = Request::new(radium_core::proto::ExecuteWorkflowRequest {
        workflow_id: id.clone(),
        use_parallel: parallel,
    });
    let response = client.execute_workflow(request).await?;
    let result = response.into_inner();

    if result.success {
        println!(
            "{} Workflow executed: {} (Execution ID: {})",
            "Success:".green().bold(),
            id,
            result.execution_id
        );
        println!("{} {}", "Final State:".bright_cyan(), result.final_state.bright_white());
    } else {
        eprintln!(
            "{} Workflow execution failed: {}",
            "Error:".red().bold(),
            result.error.unwrap_or_else(|| "Unknown error".to_string())
        );
        return Err("Workflow execution failed".into());
    }

    Ok(())
}

async fn update_workflow(
    client: &mut RadiumClient<tonic::transport::Channel>,
    id: String,
    name: Option<String>,
    description: Option<String>,
) -> Result<(), Box<dyn std::error::Error>> {
    use inquire::Text;

    // First get the current workflow
    let get_request =
        Request::new(radium_core::proto::GetWorkflowRequest { workflow_id: id.clone() });
    let get_response = client.get_workflow(get_request).await?;
    let mut workflow =
        get_response.into_inner().workflow.ok_or_else(|| "Workflow not found".to_string())?;

    // Update fields if provided
    if let Some(new_name) = name {
        workflow.name = new_name;
    } else {
        let new_name = Text::new("Workflow name:")
            .with_default(&workflow.name)
            .prompt()
            .expect("Failed to read workflow name");
        workflow.name = new_name;
    }

    if let Some(new_description) = description {
        workflow.description = new_description;
    } else {
        let new_description = Text::new("Workflow description:")
            .with_default(&workflow.description)
            .prompt()
            .expect("Failed to read workflow description");
        workflow.description = new_description;
    }

    workflow.updated_at = chrono::Utc::now().to_rfc3339();

    let request =
        Request::new(radium_core::proto::UpdateWorkflowRequest { workflow: Some(workflow) });
    let response = client.update_workflow(request).await?;

    println!("Workflow updated: {}", response.into_inner().workflow_id);
    Ok(())
}

async fn delete_workflow(
    client: &mut RadiumClient<tonic::transport::Channel>,
    id: String,
) -> Result<(), Box<dyn std::error::Error>> {
    use colored::*;
    use inquire::Confirm;

    let confirmed = Confirm::new(&format!("Are you sure you want to delete workflow '{}'?", id))
        .with_default(false)
        .prompt()
        .expect("Failed to read confirmation");

    if !confirmed {
        println!("{}", "Deletion cancelled.".yellow());
        return Ok(());
    }

    let request =
        Request::new(radium_core::proto::DeleteWorkflowRequest { workflow_id: id.clone() });
    let response = client.delete_workflow(request).await?;

    if response.into_inner().success {
        println!("{} Workflow deleted: {}", "Success:".green().bold(), id);
    } else {
        eprintln!("{} Failed to delete workflow: {}", "Error:".red().bold(), id);
        return Err("Failed to delete workflow".into());
    }

    Ok(())
}
