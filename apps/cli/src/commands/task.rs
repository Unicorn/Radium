//! Task management commands

use clap::Subcommand;
use radium_core::radium_client::RadiumClient;
use tonic::Request;

/// Task management subcommands
#[derive(Subcommand, Debug)]
pub enum TaskCommand {
    /// List all tasks
    List,
    /// Get task details
    Get {
        /// Task ID
        id: String,
    },
    /// Create a new task
    Create {
        /// Task ID
        #[arg(short, long)]
        id: Option<String>,
        /// Task name
        #[arg(short, long)]
        name: Option<String>,
        /// Task description
        #[arg(short, long)]
        description: Option<String>,
        /// Agent ID
        #[arg(short, long)]
        agent_id: Option<String>,
        /// Input JSON
        #[arg(short, long)]
        input_json: Option<String>,
    },
    /// Cancel a running task
    Cancel {
        /// Task ID
        id: String,
    },
    /// Resume a cancelled task
    Resume {
        /// Task ID
        id: String,
    },
}

/// Execute task command
pub async fn execute_task_command(
    command: TaskCommand,
    client: &mut RadiumClient<tonic::transport::Channel>,
) -> Result<(), Box<dyn std::error::Error>> {
    match command {
        TaskCommand::List => list_tasks(client).await,
        TaskCommand::Get { id } => get_task(client, id).await,
        TaskCommand::Create { id, name, description, agent_id, input_json } => {
            create_task(client, id, name, description, agent_id, input_json).await
        }
        TaskCommand::Cancel { id } => cancel_task(client, id).await,
        TaskCommand::Resume { id } => resume_task(client, id).await,
    }
}

async fn list_tasks(
    client: &mut RadiumClient<tonic::transport::Channel>,
) -> Result<(), Box<dyn std::error::Error>> {
    use tabled::{Table, Tabled};

    #[derive(Tabled)]
    struct TaskRow {
        id: String,
        name: String,
        agent_id: String,
        state: String,
        created_at: String,
    }

    let request = Request::new(radium_core::proto::ListTasksRequest {});
    let response = client.list_tasks(request).await?;
    let tasks = response.into_inner().tasks;

    let rows: Vec<TaskRow> = tasks
        .into_iter()
        .map(|t| TaskRow {
            id: t.id,
            name: t.name,
            agent_id: t.agent_id,
            state: t.state,
            created_at: t.created_at,
        })
        .collect();

    println!("{}", Table::new(rows));
    Ok(())
}

async fn get_task(
    client: &mut RadiumClient<tonic::transport::Channel>,
    id: String,
) -> Result<(), Box<dyn std::error::Error>> {
    use colored::*;

    let request = Request::new(radium_core::proto::GetTaskRequest { task_id: id.clone() });
    let response = client.get_task(request).await?;
    let task = response.into_inner().task;

    if let Some(task) = task {
        println!("{} {}", "Task ID:".bright_cyan(), task.id.bright_white());
        println!("{} {}", "Name:".bright_cyan(), task.name.bright_white());
        println!("{} {}", "Description:".bright_cyan(), task.description.bright_white());
        println!("{} {}", "Agent ID:".bright_cyan(), task.agent_id.bright_white());
        println!("{} {}", "State:".bright_cyan(), task.state.bright_white());
        println!("{} {}", "Input:".bright_cyan(), task.input_json.bright_white());
        if !task.result_json.is_empty() {
            println!("{} {}", "Result:".bright_cyan(), task.result_json.bright_white());
        }
        println!("{} {}", "Created:".bright_cyan(), task.created_at.bright_white());
        println!("{} {}", "Updated:".bright_cyan(), task.updated_at.bright_white());
    } else {
        eprintln!("{} Task not found: {}", "Error:".red().bold(), id);
        return Err("Task not found".into());
    }

    Ok(())
}

async fn create_task(
    client: &mut RadiumClient<tonic::transport::Channel>,
    id: Option<String>,
    name: Option<String>,
    description: Option<String>,
    agent_id: Option<String>,
    input_json: Option<String>,
) -> Result<(), Box<dyn std::error::Error>> {
    use inquire::Text;

    let task_id =
        id.unwrap_or_else(|| Text::new("Task ID:").prompt().expect("Failed to read task ID"));

    let task_name =
        name.unwrap_or_else(|| Text::new("Task name:").prompt().expect("Failed to read task name"));

    let task_description = description.unwrap_or_else(|| {
        Text::new("Task description:").prompt().expect("Failed to read task description")
    });

    let task_agent_id = agent_id
        .unwrap_or_else(|| Text::new("Agent ID:").prompt().expect("Failed to read agent ID"));

    let task_input = input_json.unwrap_or_else(|| {
        Text::new("Input JSON:").with_default("{}").prompt().expect("Failed to read input JSON")
    });

    let task = radium_core::proto::Task {
        id: task_id.clone(),
        name: task_name,
        description: task_description,
        agent_id: task_agent_id,
        input_json: task_input,
        state: "\"pending\"".to_string(),
        result_json: "{}".to_string(),
        created_at: chrono::Utc::now().to_rfc3339(),
        updated_at: chrono::Utc::now().to_rfc3339(),
    };

    let request = Request::new(radium_core::proto::CreateTaskRequest { task: Some(task) });
    let response = client.create_task(request).await?;

    println!("Task created with ID: {}", response.into_inner().task_id);
    Ok(())
}

async fn cancel_task(
    client: &mut RadiumClient<tonic::transport::Channel>,
    id: String,
) -> Result<(), Box<dyn std::error::Error>> {
    use colored::*;

    let request = Request::new(radium_core::proto::CancelTaskRequest { task_id: id.clone() });
    let response = client.cancel_task(request).await?;
    let result = response.into_inner();
    
    if result.success {
        println!("{} {}", "Task cancelled:".green(), id);
        if !result.message.is_empty() {
            println!("{}", result.message);
        }
    } else {
        eprintln!("{} {}", "Error:".red().bold(), result.message);
        return Err(result.message.into());
    }

    Ok(())
}

async fn resume_task(
    client: &mut RadiumClient<tonic::transport::Channel>,
    id: String,
) -> Result<(), Box<dyn std::error::Error>> {
    use colored::*;

    let request = Request::new(radium_core::proto::ResumeTaskRequest { task_id: id.clone() });
    let response = client.resume_task(request).await?;
    let result = response.into_inner();
    
    if result.success {
        println!("{} {}", "Task resumed:".green(), id);
        if !result.message.is_empty() {
            println!("{}", result.message);
        }
    } else {
        eprintln!("{} {}", "Error:".red().bold(), result.message);
        return Err(result.message.into());
    }

    Ok(())
}
