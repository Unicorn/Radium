use crate::client::ApiClient;
use crate::config::Config;
use clap::Subcommand;

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
