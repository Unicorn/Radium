//! Validation result display formatting.

use colored::Colorize;

/// Formats validation results as a checklist for display.
///
/// # Arguments
///
/// * `results` - Vector of validation results
///
/// # Returns
///
/// Formatted checklist string with [✓] for accessible sources and [x] for inaccessible ones
pub fn format_validation_results(results: &[radium_core::context::SourceValidationResult]) -> String {
    let mut output = String::new();
    
    for result in results {
        let scheme = extract_scheme(&result.source);
        if result.accessible {
            output.push_str(&format!("  {} {}: {}\n", 
                "✓".green(), 
                scheme.cyan(),
                result.source
            ));
        } else {
            output.push_str(&format!("  {} {}: {} ({})\n",
                "✗".red(),
                scheme.cyan(),
                result.source,
                result.error_message.red()
            ));
            
            // Add helpful suggestions
            let suggestion = get_error_suggestion(&result.error_message);
            if !suggestion.is_empty() {
                output.push_str(&format!("      {}\n", suggestion.yellow()));
            }
        }
    }
    
    output
}

/// Extracts the scheme from a URI or path.
fn extract_scheme(uri: &str) -> &str {
    if let Some(pos) = uri.find("://") {
        &uri[..pos]
    } else {
        "file"
    }
}

/// Gets a helpful suggestion based on the error message.
fn get_error_suggestion(error_msg: &str) -> String {
    if error_msg.contains("Authentication") || error_msg.contains("Unauthorized") {
        "Try running: rad auth login".to_string()
    } else if error_msg.contains("not found") || error_msg.contains("Not found") {
        "Check the file path".to_string()
    } else {
        String::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use radium_core::context::SourceValidationResult;

    #[test]
    fn test_extract_scheme() {
        assert_eq!(extract_scheme("file:///path/to/file"), "file");
        assert_eq!(extract_scheme("http://example.com"), "http");
        assert_eq!(extract_scheme("jira://PROJ-123"), "jira");
        assert_eq!(extract_scheme("relative/path"), "file");
    }

    #[test]
    fn test_get_error_suggestion() {
        assert!(get_error_suggestion("Authentication required").contains("rad auth login"));
        assert!(get_error_suggestion("Unauthorized").contains("rad auth login"));
        assert!(get_error_suggestion("File not found").contains("Check the file path"));
        assert!(get_error_suggestion("Network error").is_empty());
    }
}
