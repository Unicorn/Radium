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

        /// Filter by performance profile (speed, balanced, thinking, expert)
        #[arg(long)]
        profile: Option<String>,
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

        /// Filter by tags (comma-separated)
        #[arg(long)]
        tags: Option<String>,

        /// Sort results by field (name, category, engine) or multiple fields (category,name)
        #[arg(long)]
        sort: Option<String>,

        /// Use fuzzy search instead of exact/contains
        #[arg(long)]
        fuzzy: bool,

        /// Use OR logic for filters (default: AND)
        #[arg(long)]
        or: bool,
    },

    /// Show detailed information about a specific agent
    Info {
        /// Agent ID
        id: String,

        /// Output as JSON
        #[arg(long)]
        json: bool,
    },

    /// Show persona configuration for an agent
    Persona {
        /// Agent ID (optional if using --list or --validate)
        id: Option<String>,

        /// List all agents with persona configs
        #[arg(long)]
        list: bool,

        /// Validate persona config for agent
        #[arg(long)]
        validate: bool,

        /// Output as JSON
        #[arg(long)]
        json: bool,
    },

    /// Show cost estimate for running an agent
    Cost {
        /// Agent ID
        id: String,

        /// Expected input tokens (default: uses agent's estimated_tokens if available)
        #[arg(long)]
        input_tokens: Option<u64>,

        /// Expected output tokens (default: uses agent's estimated_tokens if available)
        #[arg(long)]
        output_tokens: Option<u64>,

        /// Output as JSON
        #[arg(long)]
        json: bool,
    },

    /// Validate agent configurations
    Validate {
        /// Show detailed validation errors
        #[arg(short, long)]
        verbose: bool,
        /// Output results as JSON
        #[arg(short, long)]
        json: bool,
        /// Use strict validation (treat warnings as errors)
        #[arg(long)]
        strict: bool,
    },

    /// Lint agent prompt templates
    Lint {
        /// Agent ID to lint (if not specified, lints all agents)
        id: Option<String>,
        /// Output results as JSON
        #[arg(short, long)]
        json: bool,
        /// Use strict linting (treat warnings as errors)
        #[arg(long)]
        strict: bool,
    },

    /// Show agent usage statistics
    Stats {
        /// Output as JSON
        #[arg(long)]
        json: bool,
    },

    /// List most popular agents
    Popular {
        /// Number of agents to show (default: 10)
        #[arg(short, long, default_value = "10")]
        limit: usize,
        /// Output as JSON
        #[arg(long)]
        json: bool,
    },

    /// Show agent performance metrics
    Performance {
        /// Number of agents to show (default: 10)
        #[arg(short, long, default_value = "10")]
        limit: usize,
        /// Output as JSON
        #[arg(long)]
        json: bool,
    },

    /// Show agent analytics with filtering options
    Analytics {
        /// Agent ID (optional, shows all if not specified)
        agent_id: Option<String>,
        /// Show all agents
        #[arg(long)]
        all: bool,
        /// Filter by category
        #[arg(long)]
        category: Option<String>,
        /// Filter from date (YYYY-MM-DD)
        #[arg(long)]
        from: Option<String>,
        /// Filter to date (YYYY-MM-DD)
        #[arg(long)]
        to: Option<String>,
        /// Output as JSON
        #[arg(long)]
        json: bool,
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

        /// Include persona configuration template
        #[arg(long)]
        with_persona: bool,
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
    /// Search only marketplace:
    ///   $ rad extension search github --marketplace-only
    ///
    /// Search only local extensions:
    ///   $ rad extension search github --local-only
    ///
    /// See `docs/extensions/README.md` for more information.
    Search {
        /// Search query (searches name and description)
        query: String,

        /// Output as JSON
        #[arg(long)]
        json: bool,

        /// Search only marketplace extensions
        #[arg(long)]
        marketplace_only: bool,

        /// Search only locally installed extensions
        #[arg(long)]
        local_only: bool,
    },

    /// Browse popular extensions from the marketplace
    ///
    /// # Examples
    ///
    /// Browse popular extensions:
    ///   $ rad extension browse
    ///
    /// Browse and output as JSON:
    ///   $ rad extension browse --json
    ///
    /// See `docs/extensions/README.md` for more information.
    Browse {
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

    /// Sign an extension with a private key
    ///
    /// # Examples
    ///
    /// Sign an extension:
    ///   $ rad extension sign ./my-extension --key-file ./private.key
    ///
    /// Generate a new keypair and sign:
    ///   $ rad extension sign ./my-extension --generate-key
    Sign {
        /// Path to extension directory
        path: String,

        /// Path to private key file
        #[arg(long)]
        key_file: Option<String>,

        /// Generate a new keypair and save it
        #[arg(long)]
        generate_key: bool,
    },

    /// Verify an extension signature
    ///
    /// # Examples
    ///
    /// Verify an installed extension:
    ///   $ rad extension verify my-extension
    ///
    /// Verify with a specific public key:
    ///   $ rad extension verify my-extension --key-file ./public.key
    Verify {
        /// Extension name or path
        name_or_path: String,

        /// Path to public key file (optional, uses trusted keys if not provided)
        #[arg(long)]
        key_file: Option<String>,
    },

    /// Manage trusted public keys
    ///
    /// # Examples
    ///
    /// Add a trusted key:
    ///   $ rad extension trust-key add publisher-name --key-file ./publisher.pub
    ///
    /// List trusted keys:
    ///   $ rad extension trust-key list
    ///
    /// Remove a trusted key:
    ///   $ rad extension trust-key remove publisher-name
    TrustKey {
        /// Action: add, list, or remove
        action: String,

        /// Key name (for add/remove)
        name: Option<String>,

        /// Path to public key file (for add)
        #[arg(long)]
        key_file: Option<String>,
    },

    /// Publish an extension to the marketplace
    ///
    /// # Examples
    ///
    /// Publish an extension:
    ///   $ rad extension publish ./my-extension --api-key YOUR_API_KEY
    ///
    /// Publish and sign automatically:
    ///   $ rad extension publish ./my-extension --api-key YOUR_API_KEY --sign-with-key ./private.key
    Publish {
        /// Path to extension directory
        path: String,

        /// Marketplace API key
        #[arg(long)]
        api_key: Option<String>,

        /// Path to private key for signing (optional, extension must be signed if not provided)
        #[arg(long)]
        sign_with_key: Option<String>,
    },

    /// Check for available extension updates
    ///
    /// # Examples
    ///
    /// Check for updates:
    ///   $ rad extension check-updates
    ///
    /// Check and output as JSON:
    ///   $ rad extension check-updates --json
    CheckUpdates {
        /// Output as JSON
        #[arg(long)]
        json: bool,
    },

    /// Update an extension to the latest version
    ///
    /// # Examples
    ///
    /// Update a specific extension:
    ///   $ rad extension update my-extension
    ///
    /// Update all extensions:
    ///   $ rad extension update --all
    ///
    /// Preview updates without applying:
    ///   $ rad extension update --all --dry-run
    Update {
        /// Extension name (or --all for all extensions)
        name: Option<String>,

        /// Update all extensions
        #[arg(long)]
        all: bool,

        /// Preview updates without applying
        #[arg(long)]
        dry_run: bool,
    },

    /// Manage extension analytics
    ///
    /// # Examples
    ///
    /// Show analytics status:
    ///   $ rad extension analytics status
    ///
    /// Opt in to analytics:
    ///   $ rad extension analytics opt-in
    ///
    /// Opt out of analytics:
    ///   $ rad extension analytics opt-out
    ///
    /// View analytics data:
    ///   $ rad extension analytics view
    ///
    /// View analytics for specific extension:
    ///   $ rad extension analytics view my-extension
    ///
    /// Clear analytics data:
    ///   $ rad extension analytics clear
    Analytics {
        /// Action: status, opt-in, opt-out, view, clear
        action: String,

        /// Extension name (for view action)
        name: Option<String>,

        /// Output as JSON
        #[arg(long)]
        json: bool,
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

#[derive(Subcommand, Debug, Clone)]
pub enum EnginesCommand {
    /// List all available engines
    List {
        /// Output as JSON
        #[arg(long)]
        json: bool,
    },

    /// Show detailed information about a specific engine
    Show {
        /// Engine ID
        engine_id: String,

        /// Output as JSON
        #[arg(long)]
        json: bool,
    },

    /// Show authentication status for all engines
    Status {
        /// Output as JSON
        #[arg(long)]
        json: bool,
    },

    /// Set the default engine
    SetDefault {
        /// Engine ID
        engine_id: String,
    },

    /// Check health of all engines
    Health {
        /// Output as JSON
        #[arg(long)]
        json: bool,

        /// Timeout in seconds for each health check
        #[arg(long, default_value = "5")]
        timeout: u64,
    },

    /// Manage engine configuration
    #[command(subcommand)]
    Config(EngineConfigCommand),
}

#[derive(Subcommand, Debug, Clone)]
pub enum EngineConfigCommand {
    /// Show current engine configuration
    Show {
        /// Output as JSON
        #[arg(long)]
        json: bool,
    },

    /// Set a configuration value
    Set {
        /// Configuration key in format: <engine>.<key> (e.g., gemini.temperature)
        key: String,

        /// Configuration value
        value: String,
    },

    /// Reset configuration to defaults
    Reset {
        /// Reset specific engine (if omitted, resets all)
        engine: Option<String>,
    },
}
