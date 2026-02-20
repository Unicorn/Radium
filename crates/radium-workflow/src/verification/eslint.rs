//! ESLint verification

use std::path::Path;
use std::process::Command;
use thiserror::Error;

/// ESLint error information
#[derive(Debug, Clone)]
pub struct EslintError {
    pub file: String,
    pub line: usize,
    pub column: usize,
    pub message: String,
    pub rule: String,
    pub severity: EslintSeverity,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum EslintSeverity {
    Warning,
    Error,
}

impl std::fmt::Display for EslintError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "{}:{}:{}: {} [{}]",
            self.file, self.line, self.column, self.message, self.rule
        )
    }
}

/// ESLint execution result
#[derive(Debug)]
pub struct EslintResult {
    pub success: bool,
    pub errors: Vec<EslintError>,
    pub warning_count: usize,
    pub error_count: usize,
}

/// Error type for eslint operations
#[derive(Debug, Error)]
pub enum EslintRunError {
    #[error("IO error: {0}")]
    IoError(#[from] std::io::Error),
    #[error("ESLint not found - ensure eslint is installed")]
    EslintNotFound,
}

/// Run ESLint with no-explicit-any rule
pub async fn run_eslint(project_dir: &Path) -> Result<EslintResult, EslintRunError> {
    // Try to run npx eslint
    let output = Command::new("npx")
        .args([
            "eslint",
            "--format",
            "json",
            "--rule",
            "@typescript-eslint/no-explicit-any: error",
            "src/**/*.ts",
        ])
        .current_dir(project_dir)
        .output();

    match output {
        Ok(output) => {
            let stdout = String::from_utf8_lossy(&output.stdout).to_string();
            let (errors, warning_count, error_count) = parse_eslint_output(&stdout);

            Ok(EslintResult {
                success: error_count == 0,
                errors,
                warning_count,
                error_count,
            })
        },
        Err(e) if e.kind() == std::io::ErrorKind::NotFound => Err(EslintRunError::EslintNotFound),
        Err(e) => Err(EslintRunError::IoError(e)),
    }
}

/// Parse ESLint JSON output
fn parse_eslint_output(output: &str) -> (Vec<EslintError>, usize, usize) {
    let mut errors = Vec::new();
    let mut warning_count = 0;
    let mut error_count = 0;

    // Try to parse as JSON
    if let Ok(json) = serde_json::from_str::<serde_json::Value>(output) {
        if let Some(files) = json.as_array() {
            for file in files {
                let file_path = file["filePath"].as_str().unwrap_or("");

                if let Some(messages) = file["messages"].as_array() {
                    for msg in messages {
                        let severity = match msg["severity"].as_u64() {
                            Some(2) => {
                                error_count += 1;
                                EslintSeverity::Error
                            },
                            _ => {
                                warning_count += 1;
                                EslintSeverity::Warning
                            },
                        };

                        errors.push(EslintError {
                            file: file_path.to_string(),
                            line: msg["line"].as_u64().unwrap_or(0) as usize,
                            column: msg["column"].as_u64().unwrap_or(0) as usize,
                            message: msg["message"].as_str().unwrap_or("").to_string(),
                            rule: msg["ruleId"].as_str().unwrap_or("").to_string(),
                            severity,
                        });
                    }
                }
            }
        }
    }

    (errors, warning_count, error_count)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_eslint_json() {
        let output = r#"[
            {
                "filePath": "/project/src/workflow.ts",
                "messages": [
                    {
                        "ruleId": "@typescript-eslint/no-explicit-any",
                        "severity": 2,
                        "message": "Unexpected any. Specify a different type.",
                        "line": 15,
                        "column": 10
                    }
                ],
                "errorCount": 1,
                "warningCount": 0
            }
        ]"#;

        let (errors, warning_count, error_count) = parse_eslint_output(output);
        assert_eq!(errors.len(), 1);
        assert_eq!(error_count, 1);
        assert_eq!(warning_count, 0);

        assert_eq!(errors[0].line, 15);
        assert_eq!(errors[0].rule, "@typescript-eslint/no-explicit-any");
        assert_eq!(errors[0].severity, EslintSeverity::Error);
    }

    #[test]
    fn test_empty_output() {
        let output = "[]";
        let (errors, warning_count, error_count) = parse_eslint_output(output);
        assert!(errors.is_empty());
        assert_eq!(warning_count, 0);
        assert_eq!(error_count, 0);
    }
}
