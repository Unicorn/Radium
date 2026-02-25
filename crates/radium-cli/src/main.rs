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
    /// Migrate workflow files to use canonical component names
    Migrate {
        /// Workflow YAML files to migrate
        #[arg(required = true)]
        files: Vec<String>,
        /// Preview changes without modifying files
        #[arg(long)]
        dry_run: bool,
        /// Write migrated files to this directory instead of in-place
        #[arg(long, value_name = "DIR")]
        output_dir: Option<String>,
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
        Commands::Migrate {
            files,
            dry_run,
            output_dir,
        } => commands::migrate::run(files, *dry_run, output_dir.as_deref()),
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
    fn test_parse_undeploy_command() {
        let cli = Cli::try_parse_from([
            "radium-workflow",
            "undeploy",
            "550e8400-e29b-41d4-a716-446655440000",
        ])
        .unwrap();
        assert!(matches!(cli.command, Commands::Undeploy { .. }));
    }

    #[test]
    fn test_parse_status_command() {
        let cli = Cli::try_parse_from([
            "radium-workflow",
            "status",
            "550e8400-e29b-41d4-a716-446655440000",
        ])
        .unwrap();
        assert!(matches!(cli.command, Commands::Status { .. }));
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

    #[test]
    fn test_parse_migrate_command() {
        let cli = Cli::try_parse_from([
            "radium-workflow",
            "migrate",
            "workflow.yaml",
        ])
        .unwrap();
        assert!(matches!(cli.command, Commands::Migrate { .. }));
    }

    #[test]
    fn test_parse_migrate_with_dry_run() {
        let cli = Cli::try_parse_from([
            "radium-workflow",
            "migrate",
            "--dry-run",
            "a.yaml",
            "b.yaml",
        ])
        .unwrap();
        if let Commands::Migrate { files, dry_run, .. } = &cli.command {
            assert!(dry_run);
            assert_eq!(files.len(), 2);
        } else {
            panic!("Expected Migrate command");
        }
    }

    #[test]
    fn test_parse_migrate_with_output_dir() {
        let cli = Cli::try_parse_from([
            "radium-workflow",
            "migrate",
            "--output-dir",
            "/tmp/migrated",
            "workflow.yaml",
        ])
        .unwrap();
        if let Commands::Migrate { output_dir, .. } = &cli.command {
            assert_eq!(output_dir.as_deref(), Some("/tmp/migrated"));
        } else {
            panic!("Expected Migrate command");
        }
    }

    #[test]
    fn test_migrate_requires_files() {
        let result = Cli::try_parse_from(["radium-workflow", "migrate"]);
        assert!(result.is_err());
    }

    #[test]
    fn test_parse_components_versions() {
        let cli = Cli::try_parse_from([
            "radium-workflow",
            "components",
            "versions",
            "http_request",
        ])
        .unwrap();
        if let Commands::Components { action: Some(ComponentAction::Versions { component_type }) } =
            &cli.command
        {
            assert_eq!(component_type, "http_request");
        } else {
            panic!("Expected Components {{ action: Some(Versions {{ .. }}) }}");
        }
    }

    #[test]
    fn test_parse_components_show_still_works() {
        let cli = Cli::try_parse_from([
            "radium-workflow",
            "components",
            "show",
            "activity",
        ])
        .unwrap();
        assert!(matches!(
            cli.command,
            Commands::Components { action: Some(ComponentAction::Show { .. }) }
        ));
    }
}
