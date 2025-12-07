//! Integration tests for extension prompt discovery with agent system.

use radium_core::extensions::get_extension_prompt_dirs;

/// Test that get_extension_prompt_dirs() works correctly.
///
/// This verifies that the integration function for extension prompts
/// is accessible and returns valid results (even if empty when no extensions are installed).
#[test]
fn test_extension_prompts_discoverable() {
    // Test that the function exists and can be called
    let result = get_extension_prompt_dirs();
    
    // Should not error even if no extensions are installed
    assert!(result.is_ok());
    
    // If extensions exist, verify the paths are valid
    if let Ok(dirs) = result {
        for dir in dirs {
            // Paths should point to prompts directories
            assert!(dir.to_string_lossy().contains("prompts"));
        }
    }
}

#[test]
fn test_extension_prompt_dirs_integration() {
    // Test that get_extension_prompt_dirs returns valid paths
    let dirs = get_extension_prompt_dirs();
    
    // Should not error even if no extensions are installed
    assert!(dirs.is_ok());
    
    // If extensions exist, verify paths are valid
    if let Ok(dirs) = dirs {
        for dir in dirs {
            // Paths should be absolute or relative to extensions directory
            assert!(dir.to_string_lossy().contains("prompts"));
        }
    }
}

