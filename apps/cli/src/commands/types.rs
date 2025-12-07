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

        /// Filter by category (partial match)
        #[arg(long)]
        category: Option<String>,

        /// Filter by engine (exact match)
        #[arg(long)]
        engine: Option<String>,

        /// Filter by model (partial match)
        #[arg(long)]
        model: Option<String>,

        /// Sort results by field (name, category, engine)
        #[arg(long)]
        sort: Option<String>,
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
        id: Option<String>,

        /// Agent name (e.g., "My Agent")
        name: Option<String>,

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

        /// Template to use (basic, advanced, workflow, or path to custom template)
        #[arg(short, long)]
        template: Option<String>,

        /// Interactive mode - prompt for all fields
        #[arg(short, long)]
        interactive: bool,
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
    /// Install an extension from a local path, archive, or URL
    ///
    /// # Examples
    ///
    /// Install from a local directory:
    ///   $ rad extension install ./my-extension
    ///
    /// Install from a local archive:
    ///   $ rad extension install ./my-extension.tar.gz
    ///
    /// Install from a URL:
    ///   $ rad extension install https://example.com/extensions/my-extension.tar.gz
    ///
    /// Install and overwrite existing extension:
    ///   $ rad extension install ./my-extension --overwrite
    ///
    /// Install with automatic dependency resolution:
    ///   $ rad extension install ./my-extension --install-deps
    ///
    /// See `docs/extensions/README.md` for more information.
    Install {
        /// Path to extension package directory, archive file, or URL
        source: String,

        /// Overwrite existing installation
        #[arg(long)]
        overwrite: bool,

        /// Install dependencies automatically
        #[arg(long)]
        install_deps: bool,
    },

    /// Uninstall an extension
    ///
    /// # Examples
    ///
    /// Uninstall an extension:
    ///   $ rad extension uninstall my-extension
    ///
    /// See `docs/extensions/README.md` for more information.
    Uninstall {
        /// Extension name
        name: String,
    },

    /// List all installed extensions
    ///
    /// # Examples
    ///
    /// List all extensions in table format:
    ///   $ rad extension list
    ///
    /// List with detailed information:
    ///   $ rad extension list --verbose
    ///
    /// List in JSON format:
    ///   $ rad extension list --json
    ///
    /// See `docs/extensions/README.md` for more information.
    List {
        /// Output as JSON
        #[arg(long)]
        json: bool,

        /// Show detailed information
        #[arg(short, long)]
        verbose: bool,
    },

    /// Show detailed information about a specific extension
    ///
    /// # Examples
    ///
    /// Show extension information:
    ///   $ rad extension info my-extension
    ///
    /// Show extension information in JSON format:
    ///   $ rad extension info my-extension --json
    ///
    /// See `docs/extensions/README.md` for more information.
    Info {
        /// Extension name
        name: String,

        /// Output as JSON
        #[arg(long)]
        json: bool,
    },

    /// Search for extensions by name or description
    ///
    /// # Examples
    ///
    /// Search for extensions:
    ///   $ rad extension search code-review
    ///
    /// Search and output as JSON:
    ///   $ rad extension search mcp --json
    ///
    /// See `docs/extensions/README.md` for more information.
    Search {
        /// Search query (searches name and description)
        query: String,

        /// Output as JSON
        #[arg(long)]
        json: bool,
    },

    /// Create a new extension template
    ///
    /// Generates a new extension directory with the proper structure and manifest file.
    ///
    /// # Examples
    ///
    /// Create a basic extension:
    ///   $ rad extension create my-extension
    ///
    /// Create with author and description:
    ///   $ rad extension create my-extension --author "Your Name" --description "My custom extension"
    ///
    /// See `docs/extensions/creating-extensions.md` for more information.
    Create {
        /// Extension name (must be alphanumeric with dashes/underscores, start with letter)
        name: String,

        /// Extension author
        #[arg(short, long)]
        author: Option<String>,

        /// Extension description
        #[arg(short, long)]
        description: Option<String>,
    },
}

#[derive(Subcommand, Debug, Clone)]
pub enum CustomCommand {
    /// List all available custom commands
    List {
        /// Filter by namespace
        #[arg(long)]
        namespace: Option<String>,

        /// Show detailed information
        #[arg(short, long)]
        verbose: bool,
    },

    /// Execute a custom command
    Execute {
        /// Command name (with optional namespace prefix)
        name: String,

        /// Arguments to pass to the command
        #[arg(trailing_var_arg = true, allow_hyphen_values = true)]
        args: Vec<String>,
    },

    /// Create a new custom command
    Create {
        /// Command name
        name: String,

        /// Command description
        #[arg(short, long)]
        description: Option<String>,

        /// Command template (if not provided, will prompt)
        #[arg(short, long)]
        template: Option<String>,

        /// Create in user directory instead of project
        #[arg(long)]
        user: bool,

        /// Namespace (creates subdirectory)
        #[arg(long)]
        namespace: Option<String>,
    },

    /// Validate custom command files
    Validate {
        /// Specific command name to validate (validates all if omitted)
        name: Option<String>,

        /// Show detailed validation information
        #[arg(short, long)]
        verbose: bool,
    },
}
