//! Discovery commands — search, related, compare, deps

use crate::client::ApiClient;
use crate::config::Config;
use clap::Subcommand;

#[derive(Subcommand, Clone)]
pub enum DiscoverAction {
    /// Search for components, services, or projects
    Search {
        /// Search query (semantic)
        query: Option<String>,
        /// Filter by type: component, service, project
        #[arg(long, value_delimiter = ',')]
        r#type: Option<Vec<String>>,
        /// Filter by category
        #[arg(long)]
        category: Option<String>,
        /// Limit results
        #[arg(long, default_value = "10")]
        limit: i64,
    },
    /// Show items related to a given item
    Related {
        /// Item ID
        id: String,
        /// Relationship type (uses, depends_on, similar_schema, co_used_with)
        #[arg(long)]
        relationship: Option<String>,
        /// Traversal depth
        #[arg(long, default_value = "1")]
        depth: i64,
    },
    /// Compare multiple items side by side
    Compare {
        /// Item IDs (comma-separated)
        #[arg(value_delimiter = ',')]
        ids: Vec<String>,
    },
    /// Show dependency tree of an item
    Deps {
        /// Item ID
        id: String,
    },
}

/// Run discovery commands — search, related, compare, deps.
///
/// # Errors
///
/// Returns an error if the config cannot be loaded, or the API call fails.
pub async fn run(
    profile: &str,
    action: &DiscoverAction,
) -> Result<String, Box<dyn std::error::Error>> {
    let config = Config::load()?;
    let prof = config.get_profile(profile)?;
    let client = ApiClient::new(prof);

    match action {
        DiscoverAction::Search {
            query,
            r#type,
            category,
            limit,
        } => {
            search(
                &client,
                query.as_deref(),
                r#type.as_deref(),
                category.as_deref(),
                *limit,
            )
            .await
        }
        DiscoverAction::Related {
            id,
            relationship,
            depth,
        } => related(&client, id, relationship.as_deref(), *depth).await,
        DiscoverAction::Compare { ids } => compare(&client, ids).await,
        DiscoverAction::Deps { id } => deps(&client, id).await,
    }
}

async fn search(
    client: &ApiClient,
    query: Option<&str>,
    kind: Option<&[String]>,
    category: Option<&str>,
    limit: i64,
) -> Result<String, Box<dyn std::error::Error>> {
    let mut body = serde_json::json!({ "limit": limit });

    if let Some(q) = query {
        body["query"] = serde_json::Value::String(q.to_string());
    }

    let mut filters = serde_json::Map::new();
    if let Some(kinds) = kind {
        filters.insert("type".to_string(), serde_json::json!(kinds));
    }
    if let Some(cat) = category {
        filters.insert("category".to_string(), serde_json::json!(cat));
    }
    if !filters.is_empty() {
        body["filters"] = serde_json::Value::Object(filters);
    }

    let result: serde_json::Value = client
        .post("/v1/discover/search", &body, "application/json")
        .await?;

    Ok(serde_json::to_string_pretty(&result)?)
}

async fn related(
    client: &ApiClient,
    id: &str,
    relationship: Option<&str>,
    depth: i64,
) -> Result<String, Box<dyn std::error::Error>> {
    let path = match relationship {
        Some(rel) => format!("/v1/discover/{id}/related?depth={depth}&relationship={rel}"),
        None => format!("/v1/discover/{id}/related?depth={depth}"),
    };

    let result: serde_json::Value = client.get(&path).await?;
    Ok(serde_json::to_string_pretty(&result)?)
}

async fn compare(
    client: &ApiClient,
    ids: &[String],
) -> Result<String, Box<dyn std::error::Error>> {
    let ids_str = ids.join(",");
    let path = format!("/v1/discover/compare?ids={ids_str}");

    let result: serde_json::Value = client.get(&path).await?;
    Ok(serde_json::to_string_pretty(&result)?)
}

async fn deps(
    client: &ApiClient,
    id: &str,
) -> Result<String, Box<dyn std::error::Error>> {
    let result: serde_json::Value = client
        .get(&format!("/v1/discover/{id}/dependencies"))
        .await?;
    Ok(serde_json::to_string_pretty(&result)?)
}
