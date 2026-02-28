use crate::client::ApiClient;
use crate::config::Config;
use clap::Subcommand;
use std::fs;
use std::path::Path;

#[derive(Subcommand, Clone)]
pub enum ServiceAction {
    /// List services
    List {
        /// Filter by project ID
        #[arg(long)]
        project: Option<String>,
    },
    /// Create a service from a definition file
    Create {
        /// Path to workflow definition file (YAML or JSON)
        file: String,
        /// Project to create the service in
        #[arg(long, required = true)]
        project: String,
    },
    /// Show a specific service
    Show {
        /// Service ID
        id: String,
    },
    /// Update a service from a file
    Update {
        /// Service ID
        id: String,
        /// Path to workflow definition file (YAML or JSON)
        file: String,
    },
    /// Delete a service
    Delete {
        /// Service ID
        id: String,
    },
    /// Validate a service definition file
    Validate {
        /// Path to workflow definition file (YAML or JSON)
        file: String,
    },
    /// Deploy a service
    Deploy {
        /// Service ID
        id: String,
    },
    /// Undeploy a service
    Undeploy {
        /// Service ID
        id: String,
    },
    /// Get service deployment status
    Status {
        /// Service ID
        id: String,
    },
    /// Publish a service to the catalog
    Publish {
        /// Service ID
        id: String,
    },
    /// Unpublish a service from the catalog
    Unpublish {
        /// Service ID
        id: String,
    },
    /// Browse the service catalog
    Catalog {
        /// Search term
        #[arg(long)]
        search: Option<String>,
    },
    /// Import a service from the catalog
    Import {
        /// Catalog service ID to import
        catalog_id: String,
        /// Project to import into
        #[arg(long, required = true)]
        project: String,
    },
    /// Manage service interfaces
    Interface {
        #[command(subcommand)]
        action: InterfaceAction,
    },
}

#[derive(Subcommand, Clone)]
pub enum InterfaceAction {
    /// List interfaces for a service
    List {
        /// Service ID
        service_id: String,
    },
    /// Create an interface from a JSON file
    Create {
        /// Service ID
        service_id: String,
        /// Path to interface definition file (JSON)
        file: String,
    },
    /// Publish an interface
    Publish {
        /// Service ID
        service_id: String,
        /// Interface ID
        interface_id: String,
    },
    /// Unpublish an interface
    Unpublish {
        /// Service ID
        service_id: String,
        /// Interface ID
        interface_id: String,
    },
    /// Delete an interface
    Delete {
        /// Service ID
        service_id: String,
        /// Interface ID
        interface_id: String,
    },
}

/// Detect whether a file is YAML or JSON based on its extension.
fn content_type_for_file(file: &str) -> &'static str {
    let ext = Path::new(file)
        .extension()
        .and_then(|e| e.to_str())
        .unwrap_or("");
    match ext {
        "yaml" | "yml" => "application/x-yaml",
        _ => "application/json",
    }
}

/// Read a file and parse it as a JSON value. Supports both YAML and JSON input.
fn read_workflow_file(file: &str) -> Result<serde_json::Value, Box<dyn std::error::Error>> {
    let content = fs::read_to_string(file)?;
    let ext = Path::new(file)
        .extension()
        .and_then(|e| e.to_str())
        .unwrap_or("");

    let value: serde_json::Value = match ext {
        "yaml" | "yml" => serde_yaml::from_str(&content)?,
        _ => serde_json::from_str(&content)?,
    };

    Ok(value)
}

fn load_client(profile: &str) -> Result<ApiClient, Box<dyn std::error::Error>> {
    let config = Config::load()?;
    let prof = config.get_profile(profile)?;
    Ok(ApiClient::new(prof))
}

/// Dispatch a service action to the appropriate handler.
///
/// # Errors
///
/// Returns an error if the config cannot be loaded or an API call fails.
pub async fn run(
    profile: &str,
    action: &ServiceAction,
) -> Result<String, Box<dyn std::error::Error>> {
    match action {
        ServiceAction::List { project } => list(profile, project.as_deref()).await,
        ServiceAction::Create { file, project } => create(profile, file, project).await,
        ServiceAction::Show { id } => show(profile, id).await,
        ServiceAction::Update { id, file } => update(profile, id, file).await,
        ServiceAction::Delete { id } => delete(profile, id).await,
        ServiceAction::Validate { file } => validate(profile, file).await,
        ServiceAction::Deploy { id } => deploy(profile, id).await,
        ServiceAction::Undeploy { id } => undeploy(profile, id).await,
        ServiceAction::Status { id } => status(profile, id).await,
        ServiceAction::Publish { id } => publish(profile, id).await,
        ServiceAction::Unpublish { id } => unpublish(profile, id).await,
        ServiceAction::Catalog { search } => catalog(profile, search.as_deref()).await,
        ServiceAction::Import {
            catalog_id,
            project,
        } => import(profile, catalog_id, project).await,
        ServiceAction::Interface { action } => run_interface(profile, action).await,
    }
}

/// Dispatch an interface action to the appropriate handler.
async fn run_interface(
    profile: &str,
    action: &InterfaceAction,
) -> Result<String, Box<dyn std::error::Error>> {
    match action {
        InterfaceAction::List { service_id } => interface_list(profile, service_id).await,
        InterfaceAction::Create { service_id, file } => {
            interface_create(profile, service_id, file).await
        }
        InterfaceAction::Publish {
            service_id,
            interface_id,
        } => interface_publish(profile, service_id, interface_id).await,
        InterfaceAction::Unpublish {
            service_id,
            interface_id,
        } => interface_unpublish(profile, service_id, interface_id).await,
        InterfaceAction::Delete {
            service_id,
            interface_id,
        } => interface_delete(profile, service_id, interface_id).await,
    }
}

// ---------------------------------------------------------------------------
// Service handlers
// ---------------------------------------------------------------------------

async fn list(
    profile: &str,
    project: Option<&str>,
) -> Result<String, Box<dyn std::error::Error>> {
    let client = load_client(profile)?;
    let path = match project {
        Some(pid) => format!("/v1/projects/{pid}/services"),
        None => "/v1/services".to_string(),
    };
    let result: serde_json::Value = client.get(&path).await?;
    Ok(serde_json::to_string_pretty(&result)?)
}

async fn create(
    profile: &str,
    file: &str,
    project: &str,
) -> Result<String, Box<dyn std::error::Error>> {
    let client = load_client(profile)?;
    let body = read_workflow_file(file)?;
    let content_type = content_type_for_file(file);
    let path = format!("/v1/services?project_id={project}");
    let result: serde_json::Value = client.post(&path, &body, content_type).await?;
    Ok(serde_json::to_string_pretty(&result)?)
}

async fn show(profile: &str, id: &str) -> Result<String, Box<dyn std::error::Error>> {
    let client = load_client(profile)?;
    let result: serde_json::Value = client.get(&format!("/v1/services/{id}")).await?;
    Ok(serde_json::to_string_pretty(&result)?)
}

async fn update(
    profile: &str,
    id: &str,
    file: &str,
) -> Result<String, Box<dyn std::error::Error>> {
    let client = load_client(profile)?;
    let body = read_workflow_file(file)?;
    let content_type = content_type_for_file(file);
    let result: serde_json::Value = client
        .put(&format!("/v1/services/{id}"), &body, content_type)
        .await?;
    Ok(serde_json::to_string_pretty(&result)?)
}

async fn delete(profile: &str, id: &str) -> Result<String, Box<dyn std::error::Error>> {
    let client = load_client(profile)?;
    client
        .delete_request(&format!("/v1/services/{id}"))
        .await?;
    let result = serde_json::json!({
        "status": "ok",
        "message": format!("Service '{id}' deleted."),
    });
    Ok(serde_json::to_string_pretty(&result)?)
}

async fn validate(profile: &str, file: &str) -> Result<String, Box<dyn std::error::Error>> {
    let client = load_client(profile)?;
    let content = fs::read_to_string(file)?;
    let content_type = content_type_for_file(file);
    let result: serde_json::Value = client
        .post_raw("/v1/services/validate", content, content_type)
        .await?;
    Ok(serde_json::to_string_pretty(&result)?)
}

async fn deploy(profile: &str, id: &str) -> Result<String, Box<dyn std::error::Error>> {
    let client = load_client(profile)?;
    let result: serde_json::Value = client
        .post(
            &format!("/v1/services/{id}/deploy"),
            &serde_json::json!({}),
            "application/json",
        )
        .await?;
    Ok(serde_json::to_string_pretty(&result)?)
}

async fn undeploy(profile: &str, id: &str) -> Result<String, Box<dyn std::error::Error>> {
    let client = load_client(profile)?;
    let result: serde_json::Value = client
        .post(
            &format!("/v1/services/{id}/undeploy"),
            &serde_json::json!({}),
            "application/json",
        )
        .await?;
    Ok(serde_json::to_string_pretty(&result)?)
}

async fn status(profile: &str, id: &str) -> Result<String, Box<dyn std::error::Error>> {
    let client = load_client(profile)?;
    let result: serde_json::Value = client
        .get(&format!("/v1/services/{id}/status"))
        .await?;
    Ok(serde_json::to_string_pretty(&result)?)
}

async fn publish(profile: &str, id: &str) -> Result<String, Box<dyn std::error::Error>> {
    let client = load_client(profile)?;
    let result: serde_json::Value = client
        .post(
            &format!("/v1/services/{id}/publish"),
            &serde_json::json!({}),
            "application/json",
        )
        .await?;
    Ok(serde_json::to_string_pretty(&result)?)
}

async fn unpublish(profile: &str, id: &str) -> Result<String, Box<dyn std::error::Error>> {
    let client = load_client(profile)?;
    let result: serde_json::Value = client
        .post(
            &format!("/v1/services/{id}/unpublish"),
            &serde_json::json!({}),
            "application/json",
        )
        .await?;
    Ok(serde_json::to_string_pretty(&result)?)
}

async fn catalog(
    profile: &str,
    search: Option<&str>,
) -> Result<String, Box<dyn std::error::Error>> {
    let client = load_client(profile)?;
    let path = match search {
        Some(q) => format!("/v1/services/catalog?search={q}"),
        None => "/v1/services/catalog".to_string(),
    };
    let result: serde_json::Value = client.get(&path).await?;
    Ok(serde_json::to_string_pretty(&result)?)
}

async fn import(
    profile: &str,
    catalog_id: &str,
    project: &str,
) -> Result<String, Box<dyn std::error::Error>> {
    let client = load_client(profile)?;
    let body = serde_json::json!({ "project_id": project });
    let result: serde_json::Value = client
        .post(
            &format!("/v1/services/catalog/{catalog_id}/import"),
            &body,
            "application/json",
        )
        .await?;
    Ok(serde_json::to_string_pretty(&result)?)
}

// ---------------------------------------------------------------------------
// Interface handlers
// ---------------------------------------------------------------------------

async fn interface_list(
    profile: &str,
    service_id: &str,
) -> Result<String, Box<dyn std::error::Error>> {
    let client = load_client(profile)?;
    let result: serde_json::Value = client
        .get(&format!("/v1/services/{service_id}/interfaces"))
        .await?;
    Ok(serde_json::to_string_pretty(&result)?)
}

async fn interface_create(
    profile: &str,
    service_id: &str,
    file: &str,
) -> Result<String, Box<dyn std::error::Error>> {
    let client = load_client(profile)?;
    let content = fs::read_to_string(file)?;
    let body: serde_json::Value = serde_json::from_str(&content)?;
    let result: serde_json::Value = client
        .post(
            &format!("/v1/services/{service_id}/interfaces"),
            &body,
            "application/json",
        )
        .await?;
    Ok(serde_json::to_string_pretty(&result)?)
}

async fn interface_publish(
    profile: &str,
    service_id: &str,
    interface_id: &str,
) -> Result<String, Box<dyn std::error::Error>> {
    let client = load_client(profile)?;
    let result: serde_json::Value = client
        .post(
            &format!("/v1/services/{service_id}/interfaces/{interface_id}/publish"),
            &serde_json::json!({}),
            "application/json",
        )
        .await?;
    Ok(serde_json::to_string_pretty(&result)?)
}

async fn interface_unpublish(
    profile: &str,
    service_id: &str,
    interface_id: &str,
) -> Result<String, Box<dyn std::error::Error>> {
    let client = load_client(profile)?;
    let result: serde_json::Value = client
        .post(
            &format!("/v1/services/{service_id}/interfaces/{interface_id}/unpublish"),
            &serde_json::json!({}),
            "application/json",
        )
        .await?;
    Ok(serde_json::to_string_pretty(&result)?)
}

async fn interface_delete(
    profile: &str,
    service_id: &str,
    interface_id: &str,
) -> Result<String, Box<dyn std::error::Error>> {
    let client = load_client(profile)?;
    client
        .delete_request(&format!(
            "/v1/services/{service_id}/interfaces/{interface_id}"
        ))
        .await?;
    let result = serde_json::json!({
        "status": "ok",
        "message": format!("Interface '{interface_id}' deleted from service '{service_id}'."),
    });
    Ok(serde_json::to_string_pretty(&result)?)
}
