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
    /// Service management (create, deploy, interfaces, catalog)
    Service {
        #[command(subcommand)]
        action: commands::services::ServiceAction,
    },
    /// Project management (create, deploy, status)
    Project {
        #[command(subcommand)]
        action: commands::projects::ProjectAction,
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
        Commands::Service { action } => commands::services::run(&cli.profile, action).await,
        Commands::Project { action } => commands::projects::run(&cli.profile, action).await,
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
    use commands::projects::{ProjectAction, ProjectVariableAction};
    use commands::services::{InterfaceAction, ServiceAction, VariableAction};

    // -----------------------------------------------------------------------
    // Existing command tests (Login, Components, Discover, Migrate)
    // -----------------------------------------------------------------------

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
    fn test_parse_with_profile() {
        let cli = Cli::try_parse_from([
            "radium-workflow",
            "--profile",
            "staging",
            "service",
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

    // -----------------------------------------------------------------------
    // Service command tests
    // -----------------------------------------------------------------------

    #[test]
    fn test_parse_service_list() {
        let cli = Cli::try_parse_from(["radium-workflow", "service", "list"]).unwrap();
        if let Commands::Service { action } = &cli.command {
            assert!(matches!(action, ServiceAction::List { project: None }));
        } else {
            panic!("Expected Service command");
        }
    }

    #[test]
    fn test_parse_service_list_with_project() {
        let cli = Cli::try_parse_from([
            "radium-workflow",
            "service",
            "list",
            "--project",
            "proj-123",
        ])
        .unwrap();
        if let Commands::Service {
            action: ServiceAction::List { project },
        } = &cli.command
        {
            assert_eq!(project.as_deref(), Some("proj-123"));
        } else {
            panic!("Expected Service List with project");
        }
    }

    #[test]
    fn test_parse_service_create_with_project() {
        let cli = Cli::try_parse_from([
            "radium-workflow",
            "service",
            "create",
            "my-service.yaml",
            "--project",
            "proj-123",
        ])
        .unwrap();
        if let Commands::Service {
            action: ServiceAction::Create { file, project },
        } = &cli.command
        {
            assert_eq!(file, "my-service.yaml");
            assert_eq!(project, "proj-123");
        } else {
            panic!("Expected Service Create");
        }
    }

    #[test]
    fn test_parse_service_create_requires_project() {
        let result = Cli::try_parse_from([
            "radium-workflow",
            "service",
            "create",
            "my-service.yaml",
        ]);
        assert!(result.is_err());
    }

    #[test]
    fn test_parse_service_deploy() {
        let cli = Cli::try_parse_from([
            "radium-workflow",
            "service",
            "deploy",
            "550e8400-e29b-41d4-a716-446655440000",
        ])
        .unwrap();
        if let Commands::Service {
            action: ServiceAction::Deploy { id },
        } = &cli.command
        {
            assert_eq!(id, "550e8400-e29b-41d4-a716-446655440000");
        } else {
            panic!("Expected Service Deploy");
        }
    }

    #[test]
    fn test_parse_service_interface_list() {
        let cli = Cli::try_parse_from([
            "radium-workflow",
            "service",
            "interface",
            "list",
            "svc-123",
        ])
        .unwrap();
        if let Commands::Service {
            action: ServiceAction::Interface { action },
        } = &cli.command
        {
            assert!(matches!(action, InterfaceAction::List { .. }));
        } else {
            panic!("Expected Service Interface List");
        }
    }

    #[test]
    fn test_parse_service_interface_publish() {
        let cli = Cli::try_parse_from([
            "radium-workflow",
            "service",
            "interface",
            "publish",
            "svc-123",
            "iface-456",
        ])
        .unwrap();
        if let Commands::Service {
            action:
                ServiceAction::Interface {
                    action: InterfaceAction::Publish {
                        service_id,
                        interface_id,
                    },
                },
        } = &cli.command
        {
            assert_eq!(service_id, "svc-123");
            assert_eq!(interface_id, "iface-456");
        } else {
            panic!("Expected Service Interface Publish");
        }
    }

    #[test]
    fn test_parse_service_catalog() {
        let cli =
            Cli::try_parse_from(["radium-workflow", "service", "catalog"]).unwrap();
        if let Commands::Service {
            action: ServiceAction::Catalog { search },
        } = &cli.command
        {
            assert!(search.is_none());
        } else {
            panic!("Expected Service Catalog");
        }
    }

    #[test]
    fn test_parse_service_catalog_with_search() {
        let cli = Cli::try_parse_from([
            "radium-workflow",
            "service",
            "catalog",
            "--search",
            "email",
        ])
        .unwrap();
        if let Commands::Service {
            action: ServiceAction::Catalog { search },
        } = &cli.command
        {
            assert_eq!(search.as_deref(), Some("email"));
        } else {
            panic!("Expected Service Catalog with search");
        }
    }

    #[test]
    fn test_parse_service_import() {
        let cli = Cli::try_parse_from([
            "radium-workflow",
            "service",
            "import",
            "cat-789",
            "--project",
            "proj-123",
        ])
        .unwrap();
        if let Commands::Service {
            action: ServiceAction::Import { catalog_id, project },
        } = &cli.command
        {
            assert_eq!(catalog_id, "cat-789");
            assert_eq!(project, "proj-123");
        } else {
            panic!("Expected Service Import");
        }
    }

    #[test]
    fn test_parse_service_import_requires_project() {
        let result = Cli::try_parse_from([
            "radium-workflow",
            "service",
            "import",
            "cat-789",
        ]);
        assert!(result.is_err());
    }

    #[test]
    fn test_parse_service_validate() {
        let cli = Cli::try_parse_from([
            "radium-workflow",
            "service",
            "validate",
            "my-service.yaml",
        ])
        .unwrap();
        if let Commands::Service {
            action: ServiceAction::Validate { file },
        } = &cli.command
        {
            assert_eq!(file, "my-service.yaml");
        } else {
            panic!("Expected Service Validate");
        }
    }

    #[test]
    fn test_parse_service_publish() {
        let cli = Cli::try_parse_from([
            "radium-workflow",
            "service",
            "publish",
            "svc-123",
        ])
        .unwrap();
        assert!(matches!(
            cli.command,
            Commands::Service {
                action: ServiceAction::Publish { .. }
            }
        ));
    }

    #[test]
    fn test_parse_service_unpublish() {
        let cli = Cli::try_parse_from([
            "radium-workflow",
            "service",
            "unpublish",
            "svc-123",
        ])
        .unwrap();
        assert!(matches!(
            cli.command,
            Commands::Service {
                action: ServiceAction::Unpublish { .. }
            }
        ));
    }

    // -----------------------------------------------------------------------
    // Project command tests
    // -----------------------------------------------------------------------

    #[test]
    fn test_parse_project_list() {
        let cli = Cli::try_parse_from(["radium-workflow", "project", "list"]).unwrap();
        assert!(matches!(
            cli.command,
            Commands::Project {
                action: ProjectAction::List
            }
        ));
    }

    #[test]
    fn test_parse_project_create() {
        let cli = Cli::try_parse_from([
            "radium-workflow",
            "project",
            "create",
            "--name",
            "My Project",
        ])
        .unwrap();
        if let Commands::Project {
            action: ProjectAction::Create { name, description },
        } = &cli.command
        {
            assert_eq!(name, "My Project");
            assert!(description.is_none());
        } else {
            panic!("Expected Project Create");
        }
    }

    #[test]
    fn test_parse_project_create_with_description() {
        let cli = Cli::try_parse_from([
            "radium-workflow",
            "project",
            "create",
            "--name",
            "My Project",
            "--description",
            "A test project",
        ])
        .unwrap();
        if let Commands::Project {
            action: ProjectAction::Create { name, description },
        } = &cli.command
        {
            assert_eq!(name, "My Project");
            assert_eq!(description.as_deref(), Some("A test project"));
        } else {
            panic!("Expected Project Create with description");
        }
    }

    #[test]
    fn test_parse_project_create_requires_name() {
        let result = Cli::try_parse_from(["radium-workflow", "project", "create"]);
        assert!(result.is_err());
    }

    #[test]
    fn test_parse_project_deploy() {
        let cli = Cli::try_parse_from([
            "radium-workflow",
            "project",
            "deploy",
            "proj-123",
        ])
        .unwrap();
        if let Commands::Project {
            action: ProjectAction::Deploy { id },
        } = &cli.command
        {
            assert_eq!(id, "proj-123");
        } else {
            panic!("Expected Project Deploy");
        }
    }

    #[test]
    fn test_parse_project_services() {
        let cli = Cli::try_parse_from([
            "radium-workflow",
            "project",
            "services",
            "proj-123",
        ])
        .unwrap();
        if let Commands::Project {
            action: ProjectAction::Services { id },
        } = &cli.command
        {
            assert_eq!(id, "proj-123");
        } else {
            panic!("Expected Project Services");
        }
    }

    #[test]
    fn test_parse_project_status() {
        let cli = Cli::try_parse_from([
            "radium-workflow",
            "project",
            "status",
            "proj-123",
        ])
        .unwrap();
        if let Commands::Project {
            action: ProjectAction::Status { id },
        } = &cli.command
        {
            assert_eq!(id, "proj-123");
        } else {
            panic!("Expected Project Status");
        }
    }

    #[test]
    fn test_parse_project_show() {
        let cli = Cli::try_parse_from([
            "radium-workflow",
            "project",
            "show",
            "proj-123",
        ])
        .unwrap();
        if let Commands::Project {
            action: ProjectAction::Show { id },
        } = &cli.command
        {
            assert_eq!(id, "proj-123");
        } else {
            panic!("Expected Project Show");
        }
    }

    #[test]
    fn test_parse_project_delete() {
        let cli = Cli::try_parse_from([
            "radium-workflow",
            "project",
            "delete",
            "proj-123",
        ])
        .unwrap();
        if let Commands::Project {
            action: ProjectAction::Delete { id },
        } = &cli.command
        {
            assert_eq!(id, "proj-123");
        } else {
            panic!("Expected Project Delete");
        }
    }

    #[test]
    fn test_parse_project_update() {
        let cli = Cli::try_parse_from([
            "radium-workflow",
            "project",
            "update",
            "proj-123",
            "--name",
            "New Name",
            "--description",
            "New desc",
        ])
        .unwrap();
        if let Commands::Project {
            action: ProjectAction::Update {
                id,
                name,
                description,
            },
        } = &cli.command
        {
            assert_eq!(id, "proj-123");
            assert_eq!(name.as_deref(), Some("New Name"));
            assert_eq!(description.as_deref(), Some("New desc"));
        } else {
            panic!("Expected Project Update");
        }
    }

    // -----------------------------------------------------------------------
    // Service variable command tests
    // -----------------------------------------------------------------------

    #[test]
    fn test_parse_service_variable_list() {
        let cli = Cli::try_parse_from([
            "radium-workflow",
            "service",
            "variable",
            "list",
            "svc-1",
        ])
        .unwrap();
        if let Commands::Service {
            action: ServiceAction::Variable { action: VariableAction::List { service_id } },
        } = &cli.command
        {
            assert_eq!(service_id, "svc-1");
        } else {
            panic!("Expected Service Variable List");
        }
    }

    #[test]
    fn test_parse_service_variable_create() {
        let cli = Cli::try_parse_from([
            "radium-workflow",
            "service",
            "variable",
            "create",
            "svc-1",
            "vars.json",
        ])
        .unwrap();
        if let Commands::Service {
            action:
                ServiceAction::Variable {
                    action: VariableAction::Create { service_id, file },
                },
        } = &cli.command
        {
            assert_eq!(service_id, "svc-1");
            assert_eq!(file, "vars.json");
        } else {
            panic!("Expected Service Variable Create");
        }
    }

    #[test]
    fn test_parse_service_variable_show() {
        let cli = Cli::try_parse_from([
            "radium-workflow",
            "service",
            "variable",
            "show",
            "svc-1",
            "var-1",
        ])
        .unwrap();
        if let Commands::Service {
            action:
                ServiceAction::Variable {
                    action:
                        VariableAction::Show {
                            service_id,
                            variable_id,
                        },
                },
        } = &cli.command
        {
            assert_eq!(service_id, "svc-1");
            assert_eq!(variable_id, "var-1");
        } else {
            panic!("Expected Service Variable Show");
        }
    }

    #[test]
    fn test_parse_service_variable_update() {
        let cli = Cli::try_parse_from([
            "radium-workflow",
            "service",
            "variable",
            "update",
            "svc-1",
            "var-1",
            "vars.json",
        ])
        .unwrap();
        if let Commands::Service {
            action:
                ServiceAction::Variable {
                    action:
                        VariableAction::Update {
                            service_id,
                            variable_id,
                            file,
                        },
                },
        } = &cli.command
        {
            assert_eq!(service_id, "svc-1");
            assert_eq!(variable_id, "var-1");
            assert_eq!(file, "vars.json");
        } else {
            panic!("Expected Service Variable Update");
        }
    }

    #[test]
    fn test_parse_service_variable_delete() {
        let cli = Cli::try_parse_from([
            "radium-workflow",
            "service",
            "variable",
            "delete",
            "svc-1",
            "var-1",
        ])
        .unwrap();
        if let Commands::Service {
            action:
                ServiceAction::Variable {
                    action:
                        VariableAction::Delete {
                            service_id,
                            variable_id,
                        },
                },
        } = &cli.command
        {
            assert_eq!(service_id, "svc-1");
            assert_eq!(variable_id, "var-1");
        } else {
            panic!("Expected Service Variable Delete");
        }
    }

    // -----------------------------------------------------------------------
    // Project variable command tests
    // -----------------------------------------------------------------------

    #[test]
    fn test_parse_project_variable_list() {
        let cli = Cli::try_parse_from([
            "radium-workflow",
            "project",
            "variable",
            "list",
            "proj-1",
        ])
        .unwrap();
        if let Commands::Project {
            action:
                ProjectAction::Variable {
                    action: ProjectVariableAction::List { project_id },
                },
        } = &cli.command
        {
            assert_eq!(project_id, "proj-1");
        } else {
            panic!("Expected Project Variable List");
        }
    }

    #[test]
    fn test_parse_project_variable_create() {
        let cli = Cli::try_parse_from([
            "radium-workflow",
            "project",
            "variable",
            "create",
            "proj-1",
            "vars.json",
        ])
        .unwrap();
        if let Commands::Project {
            action:
                ProjectAction::Variable {
                    action: ProjectVariableAction::Create { project_id, file },
                },
        } = &cli.command
        {
            assert_eq!(project_id, "proj-1");
            assert_eq!(file, "vars.json");
        } else {
            panic!("Expected Project Variable Create");
        }
    }

    #[test]
    fn test_parse_project_variable_show() {
        let cli = Cli::try_parse_from([
            "radium-workflow",
            "project",
            "variable",
            "show",
            "proj-1",
            "var-1",
        ])
        .unwrap();
        if let Commands::Project {
            action:
                ProjectAction::Variable {
                    action:
                        ProjectVariableAction::Show {
                            project_id,
                            variable_id,
                        },
                },
        } = &cli.command
        {
            assert_eq!(project_id, "proj-1");
            assert_eq!(variable_id, "var-1");
        } else {
            panic!("Expected Project Variable Show");
        }
    }

    #[test]
    fn test_parse_project_variable_update() {
        let cli = Cli::try_parse_from([
            "radium-workflow",
            "project",
            "variable",
            "update",
            "proj-1",
            "var-1",
            "vars.json",
        ])
        .unwrap();
        if let Commands::Project {
            action:
                ProjectAction::Variable {
                    action:
                        ProjectVariableAction::Update {
                            project_id,
                            variable_id,
                            file,
                        },
                },
        } = &cli.command
        {
            assert_eq!(project_id, "proj-1");
            assert_eq!(variable_id, "var-1");
            assert_eq!(file, "vars.json");
        } else {
            panic!("Expected Project Variable Update");
        }
    }

    #[test]
    fn test_parse_project_variable_delete() {
        let cli = Cli::try_parse_from([
            "radium-workflow",
            "project",
            "variable",
            "delete",
            "proj-1",
            "var-1",
        ])
        .unwrap();
        if let Commands::Project {
            action:
                ProjectAction::Variable {
                    action:
                        ProjectVariableAction::Delete {
                            project_id,
                            variable_id,
                        },
                },
        } = &cli.command
        {
            assert_eq!(project_id, "proj-1");
            assert_eq!(variable_id, "var-1");
        } else {
            panic!("Expected Project Variable Delete");
        }
    }
}
