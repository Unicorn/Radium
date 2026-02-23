use crate::client::ApiClient;
use crate::config::Config;
use clap::Subcommand;

#[derive(Subcommand, Clone)]
pub enum ComponentAction {
    /// Show details for a specific component type
    Show {
        /// Component type name
        component_type: String,
    },
    /// Show version info for a component type
    Versions {
        /// Component type name
        component_type: String,
    },
}

/// List all components or show a specific component type.
///
/// # Errors
///
/// Returns an error if the config cannot be loaded, or the API call fails.
pub async fn run(
    profile: &str,
    action: Option<&ComponentAction>,
) -> Result<String, Box<dyn std::error::Error>> {
    let config = Config::load()?;
    let prof = config.get_profile(profile)?;
    let client = ApiClient::new(prof);

    match action {
        None => {
            let result: serde_json::Value = client.get("/v1/components").await?;
            Ok(serde_json::to_string_pretty(&result)?)
        }
        Some(ComponentAction::Show { component_type }) => {
            let result: serde_json::Value =
                client.get(&format!("/v1/components/{component_type}")).await?;
            Ok(serde_json::to_string_pretty(&result)?)
        }
        Some(ComponentAction::Versions { component_type }) => {
            let result: serde_json::Value =
                client.get(&format!("/v1/components/{component_type}")).await?;
            // Extract and display version information from the component record
            let version = result
                .get("version")
                .and_then(|v| v.as_str())
                .unwrap_or("unknown");
            let name = result
                .get("name")
                .or_else(|| result.get("component_type"))
                .and_then(|v| v.as_str())
                .unwrap_or(component_type);
            Ok(format!(
                "Component: {name}\nCurrent version: {version}\n"
            ))
        }
    }
}
