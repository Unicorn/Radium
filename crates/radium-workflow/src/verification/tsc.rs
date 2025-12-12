//! TypeScript compiler verification

use std::path::Path;
use std::process::Command;
use thiserror::Error;

/// TSC error information
#[derive(Debug, Clone)]
pub struct TscError {
    pub file: String,
    pub line: usize,
    pub column: usize,
    pub message: String,
    pub code: String,
}

impl std::fmt::Display for TscError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "{}({}:{}): {} ({})",
            self.file, self.line, self.column, self.message, self.code
        )
    }
}

/// TSC execution result
#[derive(Debug)]
pub struct TscResult {
    pub success: bool,
    pub errors: Vec<TscError>,
    pub stdout: String,
    pub stderr: String,
}

/// Error type for tsc operations
#[derive(Debug, Error)]
pub enum TscRunError {
    #[error("IO error: {0}")]
    IoError(#[from] std::io::Error),
    #[error("tsc not found - ensure TypeScript is installed")]
    TscNotFound,
}

/// Run TypeScript compiler with strict flags
pub async fn run_tsc(project_dir: &Path) -> Result<TscResult, TscRunError> {
    // Try to run npx tsc
    let output = Command::new("npx")
        .args([
            "tsc",
            "--noEmit",
            "--strict",
            "--noImplicitAny",
            "--strictNullChecks",
            "--noUncheckedIndexedAccess",
        ])
        .current_dir(project_dir)
        .output();

    match output {
        Ok(output) => {
            let stdout = String::from_utf8_lossy(&output.stdout).to_string();
            let stderr = String::from_utf8_lossy(&output.stderr).to_string();
            let errors = parse_tsc_output(&stderr);

            Ok(TscResult {
                success: output.status.success() && errors.is_empty(),
                errors,
                stdout,
                stderr,
            })
        },
        Err(e) if e.kind() == std::io::ErrorKind::NotFound => Err(TscRunError::TscNotFound),
        Err(e) => Err(TscRunError::IoError(e)),
    }
}

/// Parse tsc output to extract errors
fn parse_tsc_output(output: &str) -> Vec<TscError> {
    let mut errors = Vec::new();

    // TSC error format: file.ts(line,col): error TS1234: message
    let error_regex =
        regex::Regex::new(r"(?m)^(.+?)\((\d+),(\d+)\):\s*error\s+(TS\d+):\s*(.+)$").unwrap();

    for cap in error_regex.captures_iter(output) {
        let file = cap.get(1).map_or("", |m| m.as_str()).to_string();
        let line: usize = cap.get(2).map_or("0", |m| m.as_str()).parse().unwrap_or(0);
        let column: usize = cap.get(3).map_or("0", |m| m.as_str()).parse().unwrap_or(0);
        let code = cap.get(4).map_or("", |m| m.as_str()).to_string();
        let message = cap.get(5).map_or("", |m| m.as_str()).to_string();

        errors.push(TscError {
            file,
            line,
            column,
            message,
            code,
        });
    }

    errors
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_tsc_output() {
        let output = r#"
src/workflow.ts(10,5): error TS2322: Type 'string' is not assignable to type 'number'.
src/activities.ts(25,10): error TS7006: Parameter 'input' implicitly has an 'any' type.
"#;

        let errors = parse_tsc_output(output);
        assert_eq!(errors.len(), 2);

        assert_eq!(errors[0].file, "src/workflow.ts");
        assert_eq!(errors[0].line, 10);
        assert_eq!(errors[0].column, 5);
        assert_eq!(errors[0].code, "TS2322");
        assert!(errors[0].message.contains("Type 'string'"));

        assert_eq!(errors[1].file, "src/activities.ts");
        assert_eq!(errors[1].code, "TS7006");
        assert!(errors[1].message.contains("any"));
    }

    #[test]
    fn test_empty_output() {
        let output = "";
        let errors = parse_tsc_output(output);
        assert!(errors.is_empty());
    }
}
