use crate::client::ApiClient;
use crate::config::Config;
use clap::Subcommand;
use std::fs;

#[derive(Subcommand, Clone)]
pub enum ProjectAction {
    /// List projects
    List,
    /// Create a new project
    Create {
        /// Project name
        #[arg(long, required = true)]
        name: String,
        /// Project description
        #[arg(long)]
        description: Option<String>,
    },
    /// Show a specific project
    Show {
        /// Project ID
        id: String,
    },
    /// Update a project
    Update {
        /// Project ID
        id: String,
        /// New project name
        #[arg(long)]
        name: Option<String>,
        /// New project description
        #[arg(long)]
        description: Option<String>,
    },
    /// Delete a project
    Delete {
        /// Project ID
        id: String,
    },
    /// Deploy all services in a project
    Deploy {
        /// Project ID
        id: String,
    },
    /// Get project deployment status
    Status {
        /// Project ID
        id: String,
    },
    /// List services in a project
    Services {
        /// Project ID
        id: String,
    },
    /// Manage project state variables
    Variable {
        #[command(subcommand)]
        action: ProjectVariableAction,
    },
}

#[derive(Subcommand, Clone)]
pub enum ProjectVariableAction {
    /// List shared state variables for a project
    List {
        /// Project ID
        project_id: String,
    },
    /// Create a shared state variable from a JSON file
    Create {
        /// Project ID
        project_id: String,
        /// Path to variable definition file (JSON)
        file: String,
    },
    /// Show a specific shared state variable
    Show {
        /// Project ID
        project_id: String,
        /// Variable ID
        variable_id: String,
    },
    /// Update a shared state variable from a JSON file
    Update {
        /// Project ID
        project_id: String,
        /// Variable ID
        variable_id: String,
        /// Path to variable definition file (JSON)
        file: String,
    },
    /// Delete a shared state variable
    Delete {
        /// Project ID
        project_id: String,
        /// Variable ID
        variable_id: String,
    },
}

fn load_client(profile: &str) -> Result<ApiClient, Box<dyn std::error::Error>> {
    let config = Config::load()?;
    let prof = config.get_profile(profile)?;
    Ok(ApiClient::new(prof))
}

/// Dispatch a project action to the appropriate handler.
///
/// # Errors
///
/// Returns an error if the config cannot be loaded or an API call fails.
pub async fn run(
    profile: &str,
    action: &ProjectAction,
) -> Result<String, Box<dyn std::error::Error>> {
    match action {
        ProjectAction::List => list(profile).await,
        ProjectAction::Create { name, description } => {
            create(profile, name, description.as_deref()).await
        }
        ProjectAction::Show { id } => show(profile, id).await,
        ProjectAction::Update {
            id,
            name,
            description,
        } => update(profile, id, name.as_deref(), description.as_deref()).await,
        ProjectAction::Delete { id } => delete(profile, id).await,
        ProjectAction::Deploy { id } => deploy(profile, id).await,
        ProjectAction::Status { id } => status(profile, id).await,
        ProjectAction::Services { id } => services(profile, id).await,
        ProjectAction::Variable { action } => run_variable(profile, action).await,
    }
}

async fn list(profile: &str) -> Result<String, Box<dyn std::error::Error>> {
    let client = load_client(profile)?;
    let result: serde_json::Value = client.get("/v1/projects").await?;
    Ok(serde_json::to_string_pretty(&result)?)
}

async fn create(
    profile: &str,
    name: &str,
    description: Option<&str>,
) -> Result<String, Box<dyn std::error::Error>> {
    let client = load_client(profile)?;
    let body = serde_json::json!({
        "name": name,
        "description": description.unwrap_or(""),
    });
    let result: serde_json::Value = client
        .post("/v1/projects", &body, "application/json")
        .await?;
    Ok(serde_json::to_string_pretty(&result)?)
}

async fn show(profile: &str, id: &str) -> Result<String, Box<dyn std::error::Error>> {
    let client = load_client(profile)?;
    let result: serde_json::Value = client.get(&format!("/v1/projects/{id}")).await?;
    Ok(serde_json::to_string_pretty(&result)?)
}

async fn update(
    profile: &str,
    id: &str,
    name: Option<&str>,
    description: Option<&str>,
) -> Result<String, Box<dyn std::error::Error>> {
    let client = load_client(profile)?;
    let mut body = serde_json::Map::new();
    if let Some(n) = name {
        body.insert("name".to_string(), serde_json::json!(n));
    }
    if let Some(d) = description {
        body.insert("description".to_string(), serde_json::json!(d));
    }
    let result: serde_json::Value = client
        .put(
            &format!("/v1/projects/{id}"),
            &serde_json::Value::Object(body),
            "application/json",
        )
        .await?;
    Ok(serde_json::to_string_pretty(&result)?)
}

async fn delete(profile: &str, id: &str) -> Result<String, Box<dyn std::error::Error>> {
    let client = load_client(profile)?;
    client
        .delete_request(&format!("/v1/projects/{id}"))
        .await?;
    let result = serde_json::json!({
        "status": "ok",
        "message": format!("Project '{id}' deleted."),
    });
    Ok(serde_json::to_string_pretty(&result)?)
}

async fn deploy(profile: &str, id: &str) -> Result<String, Box<dyn std::error::Error>> {
    let client = load_client(profile)?;
    let result: serde_json::Value = client
        .post(
            &format!("/v1/projects/{id}/deploy"),
            &serde_json::json!({}),
            "application/json",
        )
        .await?;
    Ok(serde_json::to_string_pretty(&result)?)
}

async fn status(profile: &str, id: &str) -> Result<String, Box<dyn std::error::Error>> {
    let client = load_client(profile)?;
    let result: serde_json::Value = client
        .get(&format!("/v1/projects/{id}/status"))
        .await?;
    Ok(serde_json::to_string_pretty(&result)?)
}

async fn services(profile: &str, id: &str) -> Result<String, Box<dyn std::error::Error>> {
    let client = load_client(profile)?;
    let result: serde_json::Value = client
        .get(&format!("/v1/projects/{id}/services"))
        .await?;
    Ok(serde_json::to_string_pretty(&result)?)
}

// ---------------------------------------------------------------------------
// Variable handlers
// ---------------------------------------------------------------------------

/// Dispatch a project variable action to the appropriate handler.
async fn run_variable(
    profile: &str,
    action: &ProjectVariableAction,
) -> Result<String, Box<dyn std::error::Error>> {
    match action {
        ProjectVariableAction::List { project_id } => variable_list(profile, project_id).await,
        ProjectVariableAction::Create { project_id, file } => {
            variable_create(profile, project_id, file).await
        }
        ProjectVariableAction::Show {
            project_id,
            variable_id,
        } => variable_show(profile, project_id, variable_id).await,
        ProjectVariableAction::Update {
            project_id,
            variable_id,
            file,
        } => variable_update(profile, project_id, variable_id, file).await,
        ProjectVariableAction::Delete {
            project_id,
            variable_id,
        } => variable_delete(profile, project_id, variable_id).await,
    }
}

async fn variable_list(
    profile: &str,
    project_id: &str,
) -> Result<String, Box<dyn std::error::Error>> {
    let client = load_client(profile)?;
    let result: serde_json::Value = client
        .get(&format!("/v1/projects/{project_id}/variables"))
        .await?;
    Ok(serde_json::to_string_pretty(&result)?)
}

async fn variable_create(
    profile: &str,
    project_id: &str,
    file: &str,
) -> Result<String, Box<dyn std::error::Error>> {
    let client = load_client(profile)?;
    let content = fs::read_to_string(file)?;
    let body: serde_json::Value = serde_json::from_str(&content)?;
    let result: serde_json::Value = client
        .post(
            &format!("/v1/projects/{project_id}/variables"),
            &body,
            "application/json",
        )
        .await?;
    Ok(serde_json::to_string_pretty(&result)?)
}

async fn variable_show(
    profile: &str,
    project_id: &str,
    variable_id: &str,
) -> Result<String, Box<dyn std::error::Error>> {
    let client = load_client(profile)?;
    let result: serde_json::Value = client
        .get(&format!(
            "/v1/projects/{project_id}/variables/{variable_id}"
        ))
        .await?;
    Ok(serde_json::to_string_pretty(&result)?)
}

async fn variable_update(
    profile: &str,
    project_id: &str,
    variable_id: &str,
    file: &str,
) -> Result<String, Box<dyn std::error::Error>> {
    let client = load_client(profile)?;
    let content = fs::read_to_string(file)?;
    let body: serde_json::Value = serde_json::from_str(&content)?;
    let result: serde_json::Value = client
        .put(
            &format!("/v1/projects/{project_id}/variables/{variable_id}"),
            &body,
            "application/json",
        )
        .await?;
    Ok(serde_json::to_string_pretty(&result)?)
}

async fn variable_delete(
    profile: &str,
    project_id: &str,
    variable_id: &str,
) -> Result<String, Box<dyn std::error::Error>> {
    let client = load_client(profile)?;
    client
        .delete_request(&format!(
            "/v1/projects/{project_id}/variables/{variable_id}"
        ))
        .await?;
    let result = serde_json::json!({
        "status": "ok",
        "message": format!("Variable '{variable_id}' deleted from project '{project_id}'."),
    });
    Ok(serde_json::to_string_pretty(&result)?)
}
