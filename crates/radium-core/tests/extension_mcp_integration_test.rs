//! Integration tests for extension MCP config discovery.

use radium_core::extensions::get_extension_mcp_configs;

/// Test that get_extension_mcp_configs() works correctly.
///
/// This verifies that the integration function for extension MCP configs
/// is accessible and returns valid results (even if empty when no extensions are installed).
#[test]
fn test_extension_mcp_configs_discoverable() {
    // Test that the function exists and can be called
    let result = get_extension_mcp_configs();
    
    // Should not error even if no extensions are installed
    assert!(result.is_ok());
    
    // If extensions exist, verify the paths are valid
    if let Ok(configs) = result {
        for config_path in configs {
            // Paths should point to MCP config files
            assert!(config_path.to_string_lossy().contains("mcp"));
        }
    }
}

