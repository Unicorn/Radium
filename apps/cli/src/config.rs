//! CLI configuration loading and merging.

use radium_core::cli_config::CliConfig;

/// Load and merge CLI configuration.
///
/// Configuration precedence:
/// 1. CLI arguments (handled by clap)
/// 2. Environment variables
/// 3. Local config file (./.radiumrc)
/// 4. Global config file (~/.radium/config.toml)
/// 5. Defaults
pub fn load_config() -> CliConfig {
    CliConfig::discover_and_load()
}

/// Apply configuration to environment if not already set.
///
/// # Safety
///
/// This function modifies environment variables. It should only be called
/// from single-threaded code before spawning threads.
pub unsafe fn apply_config_to_env(config: &CliConfig) {
    config.apply_to_env();
}

