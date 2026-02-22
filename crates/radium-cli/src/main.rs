mod client;
mod commands;
mod config;
mod output;

use clap::{Parser, Subcommand};
use commands::components::ComponentAction;
use commands::discover::DiscoverAction;

#[derive(Parser)]
#[command(name = "radium-workflow", about = "Radium Workflow CLI", version)]
struct Cli {
    /// Configuration profile to use
    #[arg(long, default_value = "default")]
    profile: String,

    #[command(subcommand)]
    command: Commands,
}

#[derive(Subcommand)]
enum Commands {
    /// Save API credentials to a profile
    Login {
        /// API base URL
        #[arg(long)]
        url: String,
        /// API key
        #[arg(long)]
        key: String,
    },
    /// List or show workflow components
    Components {
        #[command(subcommand)]
        action: Option<ComponentAction>,
    },
    /// Search and explore the component marketplace
    Discover {
        #[command(subcommand)]
        action: DiscoverAction,
    },
    /// Create a workflow from a file
    Create {
        /// Path to workflow definition file (YAML or JSON)
        file: String,
    },
    /// Validate a workflow file without creating it
    Validate {
        /// Path to workflow definition file (YAML or JSON)
        file: String,
    },
    /// List all workflows
    List,
    /// Show a specific workflow
    Show {
        /// Workflow ID
        id: String,
    },
    /// Update a workflow from a file
    Update {
        /// Workflow ID
        id: String,
        /// Path to workflow definition file (YAML or JSON)
        file: String,
    },
    /// Delete a workflow
    Delete {
        /// Workflow ID
        id: String,
    },
    /// Deploy a workflow
    Deploy {
        /// Workflow ID
        id: String,
    },
    /// Undeploy a workflow
    Undeploy {
        /// Workflow ID
        id: String,
    },
    /// Get workflow deployment status
    Status {
        /// Workflow ID
        id: String,
    },
}

#[tokio::main]
async fn main() {
    let cli = Cli::parse();

    let result = match &cli.command {
        Commands::Login { url, key } => commands::login::run(&cli.profile, url, key),
        Commands::Components { action } => {
            commands::components::run(&cli.profile, action.as_ref()).await
        }
        Commands::Discover { action } => commands::discover::run(&cli.profile, action).await,
        Commands::Create { file } => commands::workflows::create(&cli.profile, file).await,
        Commands::Validate { file } => commands::workflows::validate(&cli.profile, file).await,
        Commands::List => commands::workflows::list(&cli.profile).await,
        Commands::Show { id } => commands::workflows::show(&cli.profile, id).await,
        Commands::Update { id, file } => {
            commands::workflows::update(&cli.profile, id, file).await
        }
        Commands::Delete { id } => commands::workflows::delete(&cli.profile, id).await,
        Commands::Deploy { id } => commands::workflows::deploy(&cli.profile, id).await,
        Commands::Undeploy { id } => commands::workflows::undeploy(&cli.profile, id).await,
        Commands::Status { id } => commands::workflows::status(&cli.profile, id).await,
    };

    match result {
        Ok(output) => {
            println!("{output}");
        }
        Err(e) => {
            output::print_error(&e.to_string());
            let exit_code = if let Some(api_err) = e.downcast_ref::<client::ApiError>() {
                api_err.exit_code()
            } else {
                1
            };
            std::process::exit(exit_code);
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use clap::Parser;

    #[test]
    fn test_parse_login_command() {
        let cli = Cli::try_parse_from([
            "radium-workflow",
            "login",
            "--url",
            "https://radium.example.com",
            "--key",
            "rk_test",
        ])
        .unwrap();
        assert_eq!(cli.profile, "default");
        assert!(matches!(cli.command, Commands::Login { .. }));
    }

    #[test]
    fn test_parse_create_command() {
        let cli =
            Cli::try_parse_from(["radium-workflow", "create", "my-workflow.yaml"]).unwrap();
        assert!(matches!(cli.command, Commands::Create { .. }));
    }

    #[test]
    fn test_parse_validate_command() {
        let cli =
            Cli::try_parse_from(["radium-workflow", "validate", "my-workflow.yaml"]).unwrap();
        assert!(matches!(cli.command, Commands::Validate { .. }));
    }

    #[test]
    fn test_parse_list_command() {
        let cli = Cli::try_parse_from(["radium-workflow", "list"]).unwrap();
        assert!(matches!(cli.command, Commands::List));
    }

    #[test]
    fn test_parse_deploy_command() {
        let cli = Cli::try_parse_from([
            "radium-workflow",
            "deploy",
            "550e8400-e29b-41d4-a716-446655440000",
        ])
        .unwrap();
        assert!(matches!(cli.command, Commands::Deploy { .. }));
    }

    #[test]
    fn test_parse_with_profile() {
        let cli = Cli::try_parse_from([
            "radium-workflow",
            "--profile",
            "staging",
            "list",
        ])
        .unwrap();
        assert_eq!(cli.profile, "staging");
    }

    #[test]
    fn test_no_subcommand_fails() {
        let result = Cli::try_parse_from(["radium-workflow"]);
        assert!(result.is_err());
    }

    #[test]
    fn test_parse_discover_search() {
        let cli =
            Cli::try_parse_from(["radium-workflow", "discover", "search", "email sender"])
                .unwrap();
        assert!(matches!(cli.command, Commands::Discover { .. }));
    }

    #[test]
    fn test_parse_discover_search_with_filters() {
        let cli = Cli::try_parse_from([
            "radium-workflow",
            "discover",
            "search",
            "--type",
            "component,service",
            "--category",
            "communication",
            "email",
        ])
        .unwrap();
        assert!(matches!(cli.command, Commands::Discover { .. }));
    }

    #[test]
    fn test_parse_discover_related() {
        let cli = Cli::try_parse_from([
            "radium-workflow",
            "discover",
            "related",
            "comp-123",
            "--relationship",
            "uses",
        ])
        .unwrap();
        assert!(matches!(cli.command, Commands::Discover { .. }));
    }

    #[test]
    fn test_parse_discover_compare() {
        let cli = Cli::try_parse_from([
            "radium-workflow",
            "discover",
            "compare",
            "comp-1,comp-2,comp-3",
        ])
        .unwrap();
        assert!(matches!(cli.command, Commands::Discover { .. }));
    }

    #[test]
    fn test_parse_discover_deps() {
        let cli =
            Cli::try_parse_from(["radium-workflow", "discover", "deps", "service-123"]).unwrap();
        assert!(matches!(cli.command, Commands::Discover { .. }));
    }
}
