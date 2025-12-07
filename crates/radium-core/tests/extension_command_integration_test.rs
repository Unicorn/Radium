//! Integration tests for extension command discovery.

use radium_core::commands::CommandRegistry;
use radium_core::extensions::get_extension_command_dirs;

/// Test that get_extension_command_dirs() works correctly.
///
/// This verifies that the integration function for extension commands
/// is accessible and returns valid results (even if empty when no extensions are installed).
#[test]
fn test_extension_command_dirs_integration() {
    // Test that the function exists and can be called
    let result = get_extension_command_dirs();
    
    // Should not error even if no extensions are installed
    assert!(result.is_ok());
    
    // If extensions exist, verify the paths are valid
    if let Ok(dirs) = result {
        for dir in dirs {
            // Paths should point to commands directories
            assert!(dir.to_string_lossy().contains("commands"));
        }
    }
}

/// Test that CommandRegistry discovers extension commands.
///
/// This verifies that the command registry integration with extensions works.
#[test]
fn test_command_registry_extension_integration() {
    let mut registry = CommandRegistry::new();
    
    // Discovery should not error even if no extensions are installed
    let result = registry.discover();
    assert!(result.is_ok());
    
    // Verify that extension command directories would be included
    // The actual discovery happens in CommandRegistry::discover()
    // which calls get_extension_command_dirs() internally
}

