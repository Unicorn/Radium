//! Component migration framework
//!
//! This module provides infrastructure for systematic migration of TypeScript
//! components to Rust schemas with generated TypeScript output.
//!
//! # Architecture
//!
//! The migration framework consists of:
//! - `ComponentMigration` trait - defines the migration interface
//! - `MigrationRecord` - detailed YAML records for each migrated component
//! - `MigrationRunner` - batch migration execution
//! - `Verification` - comparison testing against original behavior

mod framework;
mod record;
mod runner;
mod verification;

pub use framework::{
    ComponentAnalysis, ComponentMigration, ExternalCall, FieldAnalysis, MigrationError,
    SchemaAnalysis, ValidationRule,
};
pub use record::{
    Alternative, ChallengeRecord, ComponentInfo, ConnectionRules, DependencyInfo, Difficulty,
    DiscoveryInfo, FieldDefinition, FutureImprovement, LessonsLearned, MigrationMetadata,
    MigrationRecord, RelatedComponent, RustSchemaRecord, SchemaDecision, SchemaDefinition,
    TestCategory, TestCaseRecord, TypeScriptTemplateRecord, ValidationRuleRecord,
};
pub use runner::{MigrationRunner, MigrationStatus};
pub use verification::{BehaviorDifference, DifferenceSeverity, TestResult, VerificationResult};
