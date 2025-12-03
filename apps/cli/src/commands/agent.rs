//! Agent management commands

use clap::Subcommand;
use radium_core::{proto::Agent, radium_client::RadiumClient};
use tonic::Request;

/// Agent management subcommands
#[derive(Subcommand, Debug)]
pub enum AgentCommand {
    /// Create a new agent
    Create {
        /// Agent ID
        #[arg(short, long)]
        id: Option<String>,
        /// Agent name
        #[arg(short, long)]
        name: Option<String>,
        /// Agent description
        #[arg(short, long)]
        description: Option<String>,
    },
    /// List all agents
    List,
    /// Get agent details
    Get {
        /// Agent ID
        id: String,
    },
    /// Update an agent
    Update {
        /// Agent ID
        id: String,
        /// Agent name
        #[arg(short, long)]
        name: Option<String>,
        /// Agent description
        #[arg(short, long)]
        description: Option<String>,
    },
    /// Delete an agent
    Delete {
        /// Agent ID
        id: String,
    },
}

/// Execute agent command
pub async fn execute_agent_command(
    command: AgentCommand,
    client: &mut RadiumClient<tonic::transport::Channel>,
) -> Result<(), Box<dyn std::error::Error>> {
    match command {
        AgentCommand::Create { id, name, description } => {
            create_agent(client, id, name, description).await
        }
        AgentCommand::List => list_agents(client).await,
        AgentCommand::Get { id } => get_agent(client, id).await,
        AgentCommand::Update { id, name, description } => {
            update_agent(client, id, name, description).await
        }
        AgentCommand::Delete { id } => delete_agent(client, id).await,
    }
}

async fn create_agent(
    client: &mut RadiumClient<tonic::transport::Channel>,
    id: Option<String>,
    name: Option<String>,
    description: Option<String>,
) -> Result<(), Box<dyn std::error::Error>> {
    use inquire::Text;

    let agent_id =
        id.unwrap_or_else(|| Text::new("Agent ID:").prompt().expect("Failed to read agent ID"));

    let agent_name = name
        .unwrap_or_else(|| Text::new("Agent name:").prompt().expect("Failed to read agent name"));

    let agent_description = description.unwrap_or_else(|| {
        Text::new("Agent description:").prompt().expect("Failed to read agent description")
    });

    let agent = Agent {
        id: agent_id.clone(),
        name: agent_name,
        description: agent_description,
        config_json: "{}".to_string(),
        state: "\"idle\"".to_string(),
        created_at: chrono::Utc::now().to_rfc3339(),
        updated_at: chrono::Utc::now().to_rfc3339(),
    };

    let request = Request::new(radium_core::proto::CreateAgentRequest { agent: Some(agent) });
    let response = client.create_agent(request).await?;

    println!("Agent created with ID: {}", response.into_inner().agent_id);
    Ok(())
}

async fn list_agents(
    client: &mut RadiumClient<tonic::transport::Channel>,
) -> Result<(), Box<dyn std::error::Error>> {
    use tabled::{Table, Tabled};

    let request = Request::new(radium_core::proto::ListAgentsRequest {});
    let response = client.list_agents(request).await?;
    let agents = response.into_inner().agents;

    #[derive(Tabled)]
    struct AgentRow {
        id: String,
        name: String,
        description: String,
        state: String,
        created_at: String,
    }

    let rows: Vec<AgentRow> = agents
        .into_iter()
        .map(|a| AgentRow {
            id: a.id,
            name: a.name,
            description: a.description,
            state: a.state,
            created_at: a.created_at,
        })
        .collect();

    println!("{}", Table::new(rows));
    Ok(())
}

async fn get_agent(
    client: &mut RadiumClient<tonic::transport::Channel>,
    id: String,
) -> Result<(), Box<dyn std::error::Error>> {
    use colored::*;

    let request = Request::new(radium_core::proto::GetAgentRequest { agent_id: id.clone() });
    let response = client.get_agent(request).await?;
    let agent = response.into_inner().agent;

    if let Some(agent) = agent {
        println!("{} {}", "Agent ID:".bright_cyan(), agent.id.bright_white());
        println!("{} {}", "Name:".bright_cyan(), agent.name.bright_white());
        println!("{} {}", "Description:".bright_cyan(), agent.description.bright_white());
        println!("{} {}", "State:".bright_cyan(), agent.state.bright_white());
        println!("{} {}", "Config:".bright_cyan(), agent.config_json.bright_white());
        println!("{} {}", "Created:".bright_cyan(), agent.created_at.bright_white());
        println!("{} {}", "Updated:".bright_cyan(), agent.updated_at.bright_white());
    } else {
        eprintln!("{} Agent not found: {}", "Error:".red().bold(), id);
        return Err("Agent not found".into());
    }

    Ok(())
}

async fn update_agent(
    client: &mut RadiumClient<tonic::transport::Channel>,
    id: String,
    name: Option<String>,
    description: Option<String>,
) -> Result<(), Box<dyn std::error::Error>> {
    use inquire::Text;

    // First get the current agent
    let get_request = Request::new(radium_core::proto::GetAgentRequest { agent_id: id.clone() });
    let get_response = client.get_agent(get_request).await?;
    let mut agent = get_response.into_inner().agent.ok_or_else(|| "Agent not found".to_string())?;

    // Update fields if provided
    if let Some(new_name) = name {
        agent.name = new_name;
    } else {
        let new_name = Text::new("Agent name:")
            .with_default(&agent.name)
            .prompt()
            .expect("Failed to read agent name");
        agent.name = new_name;
    }

    if let Some(new_description) = description {
        agent.description = new_description;
    } else {
        let new_description = Text::new("Agent description:")
            .with_default(&agent.description)
            .prompt()
            .expect("Failed to read agent description");
        agent.description = new_description;
    }

    agent.updated_at = chrono::Utc::now().to_rfc3339();

    let request = Request::new(radium_core::proto::UpdateAgentRequest { agent: Some(agent) });
    let response = client.update_agent(request).await?;

    println!("Agent updated: {}", response.into_inner().agent_id);
    Ok(())
}

async fn delete_agent(
    client: &mut RadiumClient<tonic::transport::Channel>,
    id: String,
) -> Result<(), Box<dyn std::error::Error>> {
    use colored::*;
    use inquire::Confirm;

    let confirmed = Confirm::new(&format!("Are you sure you want to delete agent '{}'?", id))
        .with_default(false)
        .prompt()
        .expect("Failed to read confirmation");

    if !confirmed {
        println!("{}", "Deletion cancelled.".yellow());
        return Ok(());
    }

    let request = Request::new(radium_core::proto::DeleteAgentRequest { agent_id: id.clone() });
    let response = client.delete_agent(request).await?;

    if response.into_inner().success {
        println!("{} Agent deleted: {}", "Success:".green().bold(), id);
    } else {
        eprintln!("{} Failed to delete agent: {}", "Error:".red().bold(), id);
        return Err("Failed to delete agent".into());
    }

    Ok(())
}
