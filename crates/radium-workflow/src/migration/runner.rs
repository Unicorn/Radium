//! Migration runner for batch component migrations
//!
//! Provides utilities for running migrations across multiple components.

use std::collections::HashMap;
use std::path::{Path, PathBuf};

use super::framework::{ComponentMigration, MigrationError};
use super::record::MigrationRecord;

/// Status of a component migration
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum MigrationStatus {
    /// Not started
    Pending,
    /// Currently in progress
    InProgress,
    /// Successfully completed
    Completed,
    /// Failed with error
    Failed(String),
    /// Skipped (e.g., already migrated)
    Skipped,
}

impl MigrationStatus {
    /// Check if migration is complete (success or failure)
    pub fn is_complete(&self) -> bool {
        matches!(
            self,
            MigrationStatus::Completed | MigrationStatus::Failed(_) | MigrationStatus::Skipped
        )
    }

    /// Check if migration succeeded
    pub fn is_success(&self) -> bool {
        matches!(self, MigrationStatus::Completed | MigrationStatus::Skipped)
    }
}

/// Migration runner for batch processing
pub struct MigrationRunner {
    /// Components to migrate
    components: Vec<Box<dyn ComponentMigration>>,

    /// Status of each component
    status: HashMap<String, MigrationStatus>,

    /// Records directory
    records_dir: PathBuf,

    /// Output directory for generated code
    output_dir: PathBuf,

    /// Whether to continue on error
    continue_on_error: bool,
}

impl MigrationRunner {
    /// Create a new migration runner
    pub fn new(records_dir: impl Into<PathBuf>, output_dir: impl Into<PathBuf>) -> Self {
        Self {
            components: Vec::new(),
            status: HashMap::new(),
            records_dir: records_dir.into(),
            output_dir: output_dir.into(),
            continue_on_error: true,
        }
    }

    /// Add a component to migrate
    pub fn add_component(&mut self, component: Box<dyn ComponentMigration>) {
        let name = component.component_type().to_string();
        self.status.insert(name, MigrationStatus::Pending);
        self.components.push(component);
    }

    /// Set whether to continue on error
    pub fn continue_on_error(mut self, continue_on_error: bool) -> Self {
        self.continue_on_error = continue_on_error;
        self
    }

    /// Run all migrations
    pub fn run_all(&mut self) -> Result<MigrationSummary, MigrationError> {
        let mut summary = MigrationSummary::new();

        for component in &self.components {
            let name = component.component_type().to_string();
            self.status.insert(name.clone(), MigrationStatus::InProgress);

            match self.migrate_component(component.as_ref()) {
                Ok(record) => {
                    self.status.insert(name.clone(), MigrationStatus::Completed);
                    summary.add_success(&name, record);
                }
                Err(e) => {
                    let error_msg = e.to_string();
                    self.status
                        .insert(name.clone(), MigrationStatus::Failed(error_msg.clone()));
                    summary.add_failure(&name, error_msg.clone());

                    if !self.continue_on_error {
                        return Err(e);
                    }
                }
            }
        }

        Ok(summary)
    }

    /// Migrate a single component
    fn migrate_component(
        &self,
        component: &dyn ComponentMigration,
    ) -> Result<MigrationRecord, MigrationError> {
        // Analyze
        let _analysis = component.analyze()?;

        // Generate TypeScript
        let typescript = component.generate_typescript()?;

        // Save generated TypeScript
        let output_path = self
            .output_dir
            .join(format!("{}.ts", component.component_type()));
        std::fs::write(&output_path, typescript)?;

        // Verify
        let verification = component.verify()?;
        if !verification.passed {
            return Err(MigrationError::VerificationFailed(format!(
                "Component {} verification failed: {:?}",
                component.component_type(),
                verification.errors
            )));
        }

        // Create and save record
        let record = component.create_record();
        let record_path = self
            .records_dir
            .join(format!("{}.yaml", component.component_type()));
        record.save(&record_path)?;

        Ok(record)
    }

    /// Get status of a component
    pub fn get_status(&self, component: &str) -> Option<&MigrationStatus> {
        self.status.get(component)
    }

    /// Get all statuses
    pub fn all_status(&self) -> &HashMap<String, MigrationStatus> {
        &self.status
    }

    /// Check if all migrations completed successfully
    pub fn all_succeeded(&self) -> bool {
        self.status.values().all(|s| s.is_success())
    }
}

/// Summary of migration run
#[derive(Debug, Clone)]
pub struct MigrationSummary {
    /// Successfully migrated components
    pub succeeded: Vec<String>,

    /// Failed components with error messages
    pub failed: Vec<(String, String)>,

    /// Migration records for successful migrations
    pub records: HashMap<String, MigrationRecord>,

    /// Total time in milliseconds
    pub duration_ms: u64,
}

impl MigrationSummary {
    /// Create a new summary
    pub fn new() -> Self {
        Self {
            succeeded: Vec::new(),
            failed: Vec::new(),
            records: HashMap::new(),
            duration_ms: 0,
        }
    }

    /// Add a successful migration
    pub fn add_success(&mut self, name: &str, record: MigrationRecord) {
        self.succeeded.push(name.to_string());
        self.records.insert(name.to_string(), record);
    }

    /// Add a failed migration
    pub fn add_failure(&mut self, name: &str, error: String) {
        self.failed.push((name.to_string(), error));
    }

    /// Get total count
    pub fn total(&self) -> usize {
        self.succeeded.len() + self.failed.len()
    }

    /// Get success rate as percentage
    pub fn success_rate(&self) -> f64 {
        if self.total() == 0 {
            return 100.0;
        }
        (self.succeeded.len() as f64 / self.total() as f64) * 100.0
    }

    /// Check if all succeeded
    pub fn all_succeeded(&self) -> bool {
        self.failed.is_empty()
    }
}

impl Default for MigrationSummary {
    fn default() -> Self {
        Self::new()
    }
}

/// Utility to verify generated TypeScript compiles
pub fn verify_typescript_compiles(code: &str, output_dir: &Path) -> Result<bool, MigrationError> {
    use std::process::Command;

    // Write to temp file
    let temp_file = output_dir.join("_temp_verify.ts");
    std::fs::write(&temp_file, code)?;

    // Run tsc
    let output = Command::new("npx")
        .args(["tsc", "--noEmit", "--strict"])
        .arg(&temp_file)
        .output()
        .map_err(|e| MigrationError::VerificationFailed(format!("Failed to run tsc: {}", e)))?;

    // Clean up
    let _ = std::fs::remove_file(&temp_file);

    Ok(output.status.success())
}

/// Utility to run ESLint on generated TypeScript
pub fn verify_eslint_passes(code: &str, output_dir: &Path) -> Result<bool, MigrationError> {
    use std::process::Command;

    // Write to temp file
    let temp_file = output_dir.join("_temp_verify.ts");
    std::fs::write(&temp_file, code)?;

    // Run ESLint
    let output = Command::new("npx")
        .args(["eslint", "--no-eslintrc", "--parser", "@typescript-eslint/parser"])
        .arg(&temp_file)
        .output()
        .map_err(|e| MigrationError::VerificationFailed(format!("Failed to run eslint: {}", e)))?;

    // Clean up
    let _ = std::fs::remove_file(&temp_file);

    Ok(output.status.success())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_migration_status() {
        assert!(!MigrationStatus::Pending.is_complete());
        assert!(!MigrationStatus::InProgress.is_complete());
        assert!(MigrationStatus::Completed.is_complete());
        assert!(MigrationStatus::Failed("error".to_string()).is_complete());
        assert!(MigrationStatus::Skipped.is_complete());

        assert!(MigrationStatus::Completed.is_success());
        assert!(MigrationStatus::Skipped.is_success());
        assert!(!MigrationStatus::Failed("error".to_string()).is_success());
    }

    #[test]
    fn test_migration_summary() {
        let mut summary = MigrationSummary::new();

        summary.add_success("trigger", MigrationRecord::new("trigger", "control-flow"));
        summary.add_success("start", MigrationRecord::new("start", "control-flow"));
        summary.add_failure("loop", "Complex migration failed".to_string());

        assert_eq!(summary.total(), 3);
        assert_eq!(summary.succeeded.len(), 2);
        assert_eq!(summary.failed.len(), 1);
        assert!(!summary.all_succeeded());

        // ~66.67% success rate
        assert!(summary.success_rate() > 66.0);
        assert!(summary.success_rate() < 67.0);
    }

    #[test]
    fn test_empty_summary() {
        let summary = MigrationSummary::new();
        assert_eq!(summary.total(), 0);
        assert_eq!(summary.success_rate(), 100.0);
        assert!(summary.all_succeeded());
    }
}
