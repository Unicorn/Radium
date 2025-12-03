//! Orchestrator management commands

use clap::Subcommand;
use radium_core::radium_client::RadiumClient;
use tonic::Request;

/// Orchestrator management subcommands
#[derive(Subcommand, Debug)]
pub enum OrchestratorCommand {
    /// Register an agent
    Register {
        /// Agent ID
        agent_id: String,
        /// Agent type (echo, simple, chat)
        agent_type: String,
        /// Agent description
        #[arg(short, long)]
        description: Option<String>,
    },
    /// Execute an agent
    Execute {
        /// Agent ID
        agent_id: String,
        /// Input for the agent
        input: String,
        /// Model type (mock, gemini, openai)
        #[arg(short, long)]
        model_type: Option<String>,
        /// Model ID
        #[arg(short, long)]
        model_id: Option<String>,
    },
    /// List registered agents
    List,
    /// Start an agent
    Start {
        /// Agent ID
        agent_id: String,
    },
    /// Stop an agent
    Stop {
        /// Agent ID
        agent_id: String,
    },
}

/// Execute orchestrator command
pub async fn execute_orchestrator_command(
    command: OrchestratorCommand,
    client: &mut RadiumClient<tonic::transport::Channel>,
) -> Result<(), Box<dyn std::error::Error>> {
    match command {
        OrchestratorCommand::Register { agent_id, agent_type, description } => {
            register_agent(client, agent_id, agent_type, description).await
        }
        OrchestratorCommand::Execute { agent_id, input, model_type, model_id } => {
            execute_agent(client, agent_id, input, model_type, model_id).await
        }
        OrchestratorCommand::List => list_registered_agents(client).await,
        OrchestratorCommand::Start { agent_id } => start_agent(client, agent_id).await,
        OrchestratorCommand::Stop { agent_id } => stop_agent(client, agent_id).await,
    }
}

async fn register_agent(
    client: &mut RadiumClient<tonic::transport::Channel>,
    agent_id: String,
    agent_type: String,
    description: Option<String>,
) -> Result<(), Box<dyn std::error::Error>> {
    use colored::*;
    use inquire::Text;

    let agent_description = description.unwrap_or_else(|| {
        Text::new("Agent description:").prompt().expect("Failed to read agent description")
    });

    let request = Request::new(radium_core::proto::RegisterAgentRequest {
        agent_id: agent_id.clone(),
        agent_type,
        description: agent_description,
    });
    let response = client.register_agent(request).await?;
    let result = response.into_inner();

    if result.success {
        println!("{} Agent registered: {}", "Success:".green().bold(), agent_id);
    } else {
        eprintln!(
            "{} Failed to register agent: {}",
            "Error:".red().bold(),
            result.error.unwrap_or_else(|| "Unknown error".to_string())
        );
        return Err("Failed to register agent".into());
    }

    Ok(())
}

async fn execute_agent(
    client: &mut RadiumClient<tonic::transport::Channel>,
    agent_id: String,
    input: String,
    model_type: Option<String>,
    model_id: Option<String>,
) -> Result<(), Box<dyn std::error::Error>> {
    use colored::*;

    let request = Request::new(radium_core::proto::ExecuteAgentRequest {
        agent_id: agent_id.clone(),
        input,
        model_type,
        model_id,
    });
    let response = client.execute_agent(request).await?;
    let result = response.into_inner();

    if result.success {
        println!("{} Agent executed: {}", "Success:".green().bold(), agent_id);
        println!("{} {}", "Output:".bright_cyan(), result.output.bright_white());
    } else {
        eprintln!(
            "{} Agent execution failed: {}",
            "Error:".red().bold(),
            result.error.unwrap_or_else(|| "Unknown error".to_string())
        );
        return Err("Agent execution failed".into());
    }

    Ok(())
}

async fn list_registered_agents(
    client: &mut RadiumClient<tonic::transport::Channel>,
) -> Result<(), Box<dyn std::error::Error>> {
    use tabled::{Table, Tabled};

    #[derive(Tabled)]
    struct RegisteredAgentRow {
        id: String,
        description: String,
        state: String,
    }

    let request = Request::new(radium_core::proto::GetRegisteredAgentsRequest {});
    let response = client.get_registered_agents(request).await?;
    let agents = response.into_inner().agents;

    let rows: Vec<RegisteredAgentRow> = agents
        .into_iter()
        .map(|a| RegisteredAgentRow { id: a.id, description: a.description, state: a.state })
        .collect();

    println!("{}", Table::new(rows));
    Ok(())
}

async fn start_agent(
    client: &mut RadiumClient<tonic::transport::Channel>,
    agent_id: String,
) -> Result<(), Box<dyn std::error::Error>> {
    use colored::*;

    let request =
        Request::new(radium_core::proto::StartAgentRequest { agent_id: agent_id.clone() });
    let response = client.start_agent(request).await?;
    let result = response.into_inner();

    if result.success {
        println!("{} Agent started: {}", "Success:".green().bold(), agent_id);
    } else {
        eprintln!(
            "{} Failed to start agent: {}",
            "Error:".red().bold(),
            result.error.unwrap_or_else(|| "Unknown error".to_string())
        );
        return Err("Failed to start agent".into());
    }

    Ok(())
}

async fn stop_agent(
    client: &mut RadiumClient<tonic::transport::Channel>,
    agent_id: String,
) -> Result<(), Box<dyn std::error::Error>> {
    use colored::*;

    let request = Request::new(radium_core::proto::StopAgentRequest { agent_id: agent_id.clone() });
    let response = client.stop_agent(request).await?;
    let result = response.into_inner();

    if result.success {
        println!("{} Agent stopped: {}", "Success:".green().bold(), agent_id);
    } else {
        eprintln!(
            "{} Failed to stop agent: {}",
            "Error:".red().bold(),
            result.error.unwrap_or_else(|| "Unknown error".to_string())
        );
        return Err("Failed to stop agent".into());
    }

    Ok(())
}
