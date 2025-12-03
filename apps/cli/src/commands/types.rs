//! Command type definitions shared between main.rs and tests.

use clap::Subcommand;

#[derive(Subcommand, Debug, Clone)]
pub enum AuthCommand {
    /// Authenticate with AI providers
    Login {
        /// Authenticate with all providers
        #[arg(long)]
        all: bool,

        /// Specific provider to authenticate
        provider: Option<String>,
    },

    /// Log out from AI providers
    Logout {
        /// Log out from all providers
        #[arg(long)]
        all: bool,

        /// Specific provider to log out from
        provider: Option<String>,
    },

    /// Show authentication status
    Status {
        /// Output as JSON
        #[arg(long)]
        json: bool,
    },
}

#[derive(Subcommand, Debug, Clone)]
pub enum AgentsCommand {
    /// List all available agents
    List {
        /// Output as JSON
        #[arg(long)]
        json: bool,

        /// Show detailed information
        #[arg(short, long)]
        verbose: bool,
    },

    /// Search for agents by name or capability
    Search {
        /// Search query (name, description, or capability)
        query: String,

        /// Output as JSON
        #[arg(long)]
        json: bool,
    },

    /// Show detailed information about a specific agent
    Info {
        /// Agent ID
        id: String,

        /// Output as JSON
        #[arg(long)]
        json: bool,
    },

    /// Validate agent configurations
    Validate {
        /// Show detailed validation errors
        #[arg(short, long)]
        verbose: bool,
    },
}

#[derive(Subcommand, Debug, Clone)]
pub enum TemplatesCommand {
    /// List all available workflow templates
    List {
        /// Output as JSON
        #[arg(long)]
        json: bool,

        /// Show detailed information
        #[arg(short, long)]
        verbose: bool,
    },

    /// Show detailed information about a specific template
    Info {
        /// Template name
        name: String,

        /// Output as JSON
        #[arg(long)]
        json: bool,
    },

    /// Validate template configurations
    Validate {
        /// Show detailed validation errors
        #[arg(short, long)]
        verbose: bool,
    },
}

