use crate::client::ApiClient;
use crate::config::Config;
use std::fs;
use std::path::Path;

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

/// Create a workflow from a file (YAML or JSON).
///
/// # Errors
///
/// Returns an error if the file cannot be read, parsed, or the API call fails.
pub async fn create(
    profile: &str,
    file: &str,
) -> Result<String, Box<dyn std::error::Error>> {
    let client = load_client(profile)?;
    let body = read_workflow_file(file)?;
    let content_type = content_type_for_file(file);
    let result: serde_json::Value = client.post("/v1/workflows", &body, content_type).await?;
    Ok(serde_json::to_string_pretty(&result)?)
}

/// Validate a workflow file by sending raw content to the API for server-side validation.
///
/// # Errors
///
/// Returns an error if the file cannot be read or the API call fails.
pub async fn validate(
    profile: &str,
    file: &str,
) -> Result<String, Box<dyn std::error::Error>> {
    let client = load_client(profile)?;
    let content = fs::read_to_string(file)?;
    let content_type = content_type_for_file(file);
    let result: serde_json::Value = client
        .post_raw("/v1/workflows/validate", content, content_type)
        .await?;
    Ok(serde_json::to_string_pretty(&result)?)
}

/// List all workflows.
///
/// # Errors
///
/// Returns an error if the config cannot be loaded or the API call fails.
pub async fn list(profile: &str) -> Result<String, Box<dyn std::error::Error>> {
    let client = load_client(profile)?;
    let result: serde_json::Value = client.get("/v1/workflows").await?;
    Ok(serde_json::to_string_pretty(&result)?)
}

/// Show a specific workflow by ID.
///
/// # Errors
///
/// Returns an error if the config cannot be loaded or the API call fails.
pub async fn show(
    profile: &str,
    id: &str,
) -> Result<String, Box<dyn std::error::Error>> {
    let client = load_client(profile)?;
    let result: serde_json::Value = client.get(&format!("/v1/workflows/{id}")).await?;
    Ok(serde_json::to_string_pretty(&result)?)
}

/// Update a workflow by ID from a file.
///
/// # Errors
///
/// Returns an error if the file cannot be read, parsed, or the API call fails.
pub async fn update(
    profile: &str,
    id: &str,
    file: &str,
) -> Result<String, Box<dyn std::error::Error>> {
    let client = load_client(profile)?;
    let body = read_workflow_file(file)?;
    let content_type = content_type_for_file(file);
    let result: serde_json::Value = client
        .put(&format!("/v1/workflows/{id}"), &body, content_type)
        .await?;
    Ok(serde_json::to_string_pretty(&result)?)
}

/// Delete a workflow by ID.
///
/// # Errors
///
/// Returns an error if the config cannot be loaded or the API call fails.
pub async fn delete(
    profile: &str,
    id: &str,
) -> Result<String, Box<dyn std::error::Error>> {
    let client = load_client(profile)?;
    client.delete_request(&format!("/v1/workflows/{id}")).await?;
    let result = serde_json::json!({
        "status": "ok",
        "message": format!("Workflow '{id}' deleted."),
    });
    Ok(serde_json::to_string_pretty(&result)?)
}

/// Deploy a workflow by ID.
///
/// # Errors
///
/// Returns an error if the config cannot be loaded or the API call fails.
pub async fn deploy(
    profile: &str,
    id: &str,
) -> Result<String, Box<dyn std::error::Error>> {
    let client = load_client(profile)?;
    let result: serde_json::Value = client
        .post(
            &format!("/v1/workflows/{id}/deploy"),
            &serde_json::json!({}),
            "application/json",
        )
        .await?;
    Ok(serde_json::to_string_pretty(&result)?)
}

/// Undeploy a workflow by ID.
///
/// # Errors
///
/// Returns an error if the config cannot be loaded or the API call fails.
pub async fn undeploy(
    profile: &str,
    id: &str,
) -> Result<String, Box<dyn std::error::Error>> {
    let client = load_client(profile)?;
    let result: serde_json::Value = client
        .post(
            &format!("/v1/workflows/{id}/undeploy"),
            &serde_json::json!({}),
            "application/json",
        )
        .await?;
    Ok(serde_json::to_string_pretty(&result)?)
}

/// Get the status of a workflow by ID.
///
/// # Errors
///
/// Returns an error if the config cannot be loaded or the API call fails.
pub async fn status(
    profile: &str,
    id: &str,
) -> Result<String, Box<dyn std::error::Error>> {
    let client = load_client(profile)?;
    let result: serde_json::Value =
        client.get(&format!("/v1/workflows/{id}/status")).await?;
    Ok(serde_json::to_string_pretty(&result)?)
}
