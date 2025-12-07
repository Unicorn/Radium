//! Command type definitions shared between main.rs and tests.

use clap::{Args, Subcommand};

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

    /// Create a new agent template
    Create {
        /// Agent ID (e.g., "my-agent")
        id: String,

        /// Agent name (e.g., "My Agent")
        name: String,

        /// Agent description
        #[arg(short, long)]
        description: Option<String>,

        /// Agent category (e.g., "custom")
        #[arg(short, long)]
        category: Option<String>,

        /// Default engine (e.g., "gemini", "openai", "claude")
        #[arg(short, long)]
        engine: Option<String>,

        /// Default model (e.g., "gemini-2.0-flash-exp")
        #[arg(short, long)]
        model: Option<String>,

        /// Reasoning effort level (low, medium, high)
        #[arg(short, long)]
        reasoning: Option<String>,

        /// Output directory (default: ./agents/)
        #[arg(short, long)]
        output: Option<String>,
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

#[derive(Subcommand, Debug, Clone)]
pub enum HooksCommand {
    /// List all registered hooks
    List {
        /// Filter by hook type
        #[arg(long)]
        r#type: Option<String>,

        /// Output as JSON
        #[arg(long)]
        json: bool,

        /// Show detailed information
        #[arg(short, long)]
        verbose: bool,
    },

    /// Show detailed information about a specific hook
    Info {
        /// Hook name
        name: String,

        /// Output as JSON
        #[arg(long)]
        json: bool,
    },

    /// Enable a hook
    Enable {
        /// Hook name
        name: String,
    },

    /// Disable a hook
    Disable {
        /// Hook name
        name: String,
    },
}

#[derive(Subcommand, Debug, Clone)]
pub enum ExtensionCommand {
    /// Install an extension from a local path or URL
    Install {
        /// Path to extension package directory or URL
        source: String,

        /// Overwrite existing installation
        #[arg(long)]
        overwrite: bool,

        /// Install dependencies automatically
        #[arg(long)]
        install_deps: bool,
    },

    /// Uninstall an extension
    Uninstall {
        /// Extension name
        name: String,
    },

    /// List all installed extensions
    List {
        /// Output as JSON
        #[arg(long)]
        json: bool,

        /// Show detailed information
        #[arg(short, long)]
        verbose: bool,
    },

    /// Show detailed information about a specific extension
    Info {
        /// Extension name
        name: String,

        /// Output as JSON
        #[arg(long)]
        json: bool,
    },

    /// Search for extensions by name or description
    Search {
        /// Search query
        query: String,

        /// Output as JSON
        #[arg(long)]
        json: bool,
    },
}
