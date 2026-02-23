use std::fs;
use std::path::Path;

/// Deprecated component type names and their canonical replacements.
const RENAMES: &[(&str, &str)] = &[
    ("activity", "action"),
    ("child_workflow", "child_service"),
    ("signal", "message"),
];

/// Result of migrating a single file.
pub struct MigrateResult {
    pub path: String,
    pub changes: Vec<String>,
    pub already_current: bool,
}

/// Migrate workflow files to use canonical component names.
///
/// Reads each file, replaces deprecated `type:` values with canonical names,
/// and writes the result back (in-place) or to a new path.
///
/// Operates on raw text to preserve YAML formatting, comments, and ordering.
pub fn run(
    files: &[String],
    dry_run: bool,
    output_dir: Option<&str>,
) -> Result<String, Box<dyn std::error::Error>> {
    if files.is_empty() {
        return Err("No files specified. Usage: radium-workflow migrate <FILE>...".into());
    }

    let mut results: Vec<MigrateResult> = Vec::new();

    for file in files {
        let result = migrate_file(file, dry_run, output_dir)?;
        results.push(result);
    }

    Ok(format_results(&results, dry_run))
}

fn migrate_file(
    file: &str,
    dry_run: bool,
    output_dir: Option<&str>,
) -> Result<MigrateResult, Box<dyn std::error::Error>> {
    let path = Path::new(file);
    if !path.exists() {
        return Err(format!("File not found: {file}").into());
    }

    let content = fs::read_to_string(path)?;

    // Validate it's parseable YAML before modifying
    let _: serde_yaml::Value = serde_yaml::from_str(&content)
        .map_err(|e| format!("Invalid YAML in {file}: {e}"))?;

    let (new_content, changes) = rename_component_types(&content);

    let already_current = changes.is_empty();

    if !dry_run && !already_current {
        let output_path = match output_dir {
            Some(dir) => {
                let dir_path = Path::new(dir);
                fs::create_dir_all(dir_path)?;
                let filename = path.file_name().ok_or("Invalid filename")?;
                dir_path.join(filename)
            }
            None => path.to_path_buf(),
        };
        fs::write(&output_path, &new_content)?;
    }

    Ok(MigrateResult {
        path: file.to_string(),
        changes,
        already_current,
    })
}

/// Replace deprecated component type values in YAML content.
///
/// Targets lines matching the pattern `type: <old_name>` in component
/// definitions. Uses line-by-line replacement to preserve formatting.
fn rename_component_types(content: &str) -> (String, Vec<String>) {
    let mut changes: Vec<String> = Vec::new();
    let mut result = String::with_capacity(content.len());

    for line in content.lines() {
        let trimmed = line.trim();
        let mut replaced = false;

        // Match `type: old_name` pattern (with optional quotes and trailing content)
        for &(old, new) in RENAMES {
            // Unquoted: `type: activity` or `type: activity  # comment`
            let pattern_unquoted = format!("type: {old}");
            // Single-quoted: `type: 'activity'`
            let pattern_single = format!("type: '{old}'");
            // Double-quoted: `type: "activity"`
            let pattern_double = format!("type: \"{old}\"");

            let matches = trimmed == pattern_unquoted
                || trimmed.starts_with(&format!("{pattern_unquoted} "))
                || trimmed.starts_with(&format!("{pattern_unquoted}\t"))
                || trimmed.starts_with(&format!("{pattern_unquoted}#"))
                || trimmed == pattern_single
                || trimmed.starts_with(&format!("{pattern_single} "))
                || trimmed == pattern_double
                || trimmed.starts_with(&format!("{pattern_double} "));

            if matches {
                let new_line = line.replacen(old, new, 1);
                result.push_str(&new_line);
                changes.push(format!("{old} -> {new}"));
                replaced = true;
                break;
            }
        }

        if !replaced {
            result.push_str(line);
        }
        result.push('\n');
    }

    // Preserve original trailing newline behavior
    if !content.ends_with('\n') && result.ends_with('\n') {
        result.pop();
    }

    (result, changes)
}

fn format_results(results: &[MigrateResult], dry_run: bool) -> String {
    let mut output = String::new();

    let prefix = if dry_run { "[DRY RUN] " } else { "" };

    let mut total_changes = 0;
    let mut files_changed = 0;
    let mut files_current = 0;

    for result in results {
        if result.already_current {
            files_current += 1;
            output.push_str(&format!("  {} - already current\n", result.path));
        } else {
            files_changed += 1;
            total_changes += result.changes.len();
            output.push_str(&format!("  {} - {} rename(s):\n", result.path, result.changes.len()));
            for change in &result.changes {
                output.push_str(&format!("    - {change}\n"));
            }
        }
    }

    let summary = format!(
        "\n{prefix}Migration complete: {total_changes} rename(s) across {files_changed} file(s), {files_current} file(s) already current.\n"
    );
    output.push_str(&summary);

    output
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::TempDir;

    #[test]
    fn test_rename_activity_to_action() {
        let yaml = r#"
name: Test Workflow
components:
  - id: start
    type: trigger
  - id: do_thing
    type: activity
    config:
      name: processData
  - id: end
    type: stop
connections:
  - from: start
    to: do_thing
  - from: do_thing
    to: end
"#;
        let (result, changes) = rename_component_types(yaml);
        assert_eq!(changes.len(), 1);
        assert_eq!(changes[0], "activity -> action");
        assert!(result.contains("type: action"));
        assert!(!result.contains("type: activity"));
        // Other types unchanged
        assert!(result.contains("type: trigger"));
        assert!(result.contains("type: stop"));
    }

    #[test]
    fn test_rename_child_workflow_to_child_service() {
        let yaml = "  - id: sub\n    type: child_workflow\n";
        let (result, changes) = rename_component_types(yaml);
        assert_eq!(changes.len(), 1);
        assert_eq!(changes[0], "child_workflow -> child_service");
        assert!(result.contains("type: child_service"));
    }

    #[test]
    fn test_rename_signal_to_message() {
        let yaml = "  - id: notify\n    type: signal\n";
        let (result, changes) = rename_component_types(yaml);
        assert_eq!(changes.len(), 1);
        assert_eq!(changes[0], "signal -> message");
        assert!(result.contains("type: message"));
    }

    #[test]
    fn test_rename_quoted_values() {
        let yaml_single = "    type: 'activity'\n";
        let (result, changes) = rename_component_types(yaml_single);
        assert_eq!(changes.len(), 1);
        assert!(result.contains("type: 'action'"));

        let yaml_double = "    type: \"child_workflow\"\n";
        let (result, changes) = rename_component_types(yaml_double);
        assert_eq!(changes.len(), 1);
        assert!(result.contains("type: \"child_service\""));
    }

    #[test]
    fn test_multiple_renames_in_one_file() {
        let yaml = r#"
name: Multi Rename
components:
  - id: start
    type: trigger
  - id: act
    type: activity
  - id: sig
    type: signal
  - id: child
    type: child_workflow
  - id: end
    type: stop
"#;
        let (result, changes) = rename_component_types(yaml);
        assert_eq!(changes.len(), 3);
        assert!(result.contains("type: action"));
        assert!(result.contains("type: message"));
        assert!(result.contains("type: child_service"));
        // Untouched types
        assert!(result.contains("type: trigger"));
        assert!(result.contains("type: stop"));
    }

    #[test]
    fn test_no_changes_when_already_current() {
        let yaml = r#"
name: Current Workflow
components:
  - id: start
    type: trigger
  - id: act
    type: action
  - id: msg
    type: message
  - id: end
    type: stop
"#;
        let (_result, changes) = rename_component_types(yaml);
        assert!(changes.is_empty());
    }

    #[test]
    fn test_preserves_formatting_and_comments() {
        let yaml = "# My workflow\nname: Test\ncomponents:\n  - id: act\n    type: activity  # deprecated name\n  - id: end\n    type: stop\n";
        let (result, changes) = rename_component_types(yaml);
        assert_eq!(changes.len(), 1);
        // Comment is preserved (it's on the same line, which gets the replacement)
        assert!(result.contains("# My workflow"));
        assert!(result.contains("type: stop"));
    }

    #[test]
    fn test_does_not_rename_non_type_fields() {
        let yaml = r#"
name: Test
components:
  - id: act
    type: action
    config:
      activity_name: myActivity
      signal_name: mySignal
"#;
        let (result, changes) = rename_component_types(yaml);
        assert!(changes.is_empty());
        // Config values with "activity" or "signal" in them are NOT renamed
        assert!(result.contains("activity_name: myActivity"));
        assert!(result.contains("signal_name: mySignal"));
    }

    #[test]
    fn test_run_dry_run() {
        let dir = TempDir::new().unwrap();
        let file_path = dir.path().join("test.yaml");
        let yaml = "name: Test\ncomponents:\n  - id: act\n    type: activity\n";
        fs::write(&file_path, yaml).unwrap();

        let files = vec![file_path.to_str().unwrap().to_string()];
        let output = run(&files, true, None).unwrap();

        assert!(output.contains("[DRY RUN]"));
        assert!(output.contains("activity -> action"));

        // File should NOT be modified
        let after = fs::read_to_string(&file_path).unwrap();
        assert!(after.contains("type: activity"));
    }

    #[test]
    fn test_run_in_place() {
        let dir = TempDir::new().unwrap();
        let file_path = dir.path().join("test.yaml");
        let yaml = "name: Test\ncomponents:\n  - id: act\n    type: activity\n";
        fs::write(&file_path, yaml).unwrap();

        let files = vec![file_path.to_str().unwrap().to_string()];
        let output = run(&files, false, None).unwrap();

        assert!(output.contains("1 rename(s) across 1 file(s)"));

        // File should be modified
        let after = fs::read_to_string(&file_path).unwrap();
        assert!(after.contains("type: action"));
        assert!(!after.contains("type: activity"));
    }

    #[test]
    fn test_run_with_output_dir() {
        let dir = TempDir::new().unwrap();
        let file_path = dir.path().join("test.yaml");
        let output_dir = dir.path().join("migrated");
        let yaml = "name: Test\ncomponents:\n  - id: sig\n    type: signal\n";
        fs::write(&file_path, yaml).unwrap();

        let files = vec![file_path.to_str().unwrap().to_string()];
        let output = run(&files, false, Some(output_dir.to_str().unwrap())).unwrap();

        assert!(output.contains("signal -> message"));

        // Original file unchanged
        let original = fs::read_to_string(&file_path).unwrap();
        assert!(original.contains("type: signal"));

        // Output file has new names
        let migrated = fs::read_to_string(output_dir.join("test.yaml")).unwrap();
        assert!(migrated.contains("type: message"));
    }

    #[test]
    fn test_run_file_not_found() {
        let files = vec!["/nonexistent/file.yaml".to_string()];
        let result = run(&files, false, None);
        assert!(result.is_err());
        assert!(result.unwrap_err().to_string().contains("File not found"));
    }

    #[test]
    fn test_run_invalid_yaml() {
        let dir = TempDir::new().unwrap();
        let file_path = dir.path().join("bad.yaml");
        fs::write(&file_path, "{{invalid yaml::").unwrap();

        let files = vec![file_path.to_str().unwrap().to_string()];
        let result = run(&files, false, None);
        assert!(result.is_err());
        assert!(result.unwrap_err().to_string().contains("Invalid YAML"));
    }

    #[test]
    fn test_run_no_files() {
        let files: Vec<String> = vec![];
        let result = run(&files, false, None);
        assert!(result.is_err());
        assert!(result.unwrap_err().to_string().contains("No files specified"));
    }
}
