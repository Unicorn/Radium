//! Context file templates for common use cases.
//!
//! Provides pre-defined templates that users can generate to quickly get started
//! with context files. Templates follow best practices and include helpful
//! comments.

/// Template types available for context file generation.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum TemplateType {
    /// Basic project context template.
    Basic,
    /// Coding standards and conventions template.
    CodingStandards,
    /// Architecture documentation template.
    Architecture,
    /// Team-specific conventions template.
    TeamConventions,
}

impl TemplateType {
    /// Gets all available template types.
    pub fn all() -> Vec<Self> {
        vec![
            Self::Basic,
            Self::CodingStandards,
            Self::Architecture,
            Self::TeamConventions,
        ]
    }

    /// Gets the template name as a string.
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::Basic => "basic",
            Self::CodingStandards => "coding-standards",
            Self::Architecture => "architecture",
            Self::TeamConventions => "team-conventions",
        }
    }

    /// Parses a template type from a string.
    pub fn from_str(s: &str) -> Option<Self> {
        match s.to_lowercase().as_str() {
            "basic" => Some(Self::Basic),
            "coding-standards" | "coding_standards" | "codingstandards" => {
                Some(Self::CodingStandards)
            }
            "architecture" => Some(Self::Architecture),
            "team-conventions" | "team_conventions" | "teamconventions" => {
                Some(Self::TeamConventions)
            }
            _ => None,
        }
    }

    /// Gets the description of the template.
    pub fn description(&self) -> &'static str {
        match self {
            Self::Basic => "Simple project context with common sections",
            Self::CodingStandards => "Code style and conventions template",
            Self::Architecture => "Architecture documentation template",
            Self::TeamConventions => "Team-specific guidelines template",
        }
    }
}

/// Generates context file content for the specified template type.
pub fn generate_template(template_type: TemplateType) -> String {
    match template_type {
        TemplateType::Basic => generate_basic_template(),
        TemplateType::CodingStandards => generate_coding_standards_template(),
        TemplateType::Architecture => generate_architecture_template(),
        TemplateType::TeamConventions => generate_team_conventions_template(),
    }
}

/// Generates the basic project context template.
fn generate_basic_template() -> String {
    r#"# Project Context

This file provides persistent instructions to agents working on this project.

## Project Overview

<!-- Describe your project here -->

## Guidelines

<!-- Add project-specific guidelines here -->

## Code Style

<!-- Add code style preferences here -->

## Testing

<!-- Add testing requirements here -->

## Documentation

<!-- Add documentation requirements here -->
"#
    .to_string()
}

/// Generates the coding standards template.
fn generate_coding_standards_template() -> String {
    r#"# Coding Standards

This document defines the coding standards and conventions for this project.

## Language-Specific Standards

<!-- Add language-specific standards here -->
<!-- Example: Rust, TypeScript, Python, etc. -->

## Code Formatting

<!-- Specify formatting requirements -->
<!-- Example: Use `cargo fmt` for Rust, `prettier` for TypeScript -->

## Naming Conventions

<!-- Define naming conventions -->
<!-- Example: Use `snake_case` for functions, `PascalCase` for types -->

## Code Organization

<!-- Define code organization rules -->
<!-- Example: Module structure, file organization, etc. -->

## Best Practices

<!-- List best practices -->
<!-- Example: Error handling, logging, etc. -->

## Anti-Patterns

<!-- List patterns to avoid -->
"#
    .to_string()
}

/// Generates the architecture template.
fn generate_architecture_template() -> String {
    r#"# Architecture Documentation

This document describes the architecture of this project.

## System Overview

<!-- High-level system overview -->

## Components

<!-- List and describe major components -->

## Data Flow

<!-- Describe data flow through the system -->

## Design Decisions

<!-- Document important design decisions -->

## Dependencies

<!-- List external dependencies and their purposes -->

## Future Considerations

<!-- Document planned changes or improvements -->
"#
    .to_string()
}

/// Generates the team conventions template.
fn generate_team_conventions_template() -> String {
    r#"# Team Conventions

This document defines team-specific conventions and guidelines.

## Communication

<!-- Define communication standards -->
<!-- Example: PR review process, meeting schedules, etc. -->

## Workflow

<!-- Define development workflow -->
<!-- Example: Git workflow, branching strategy, etc. -->

## Code Review

<!-- Define code review standards -->
<!-- Example: Required reviewers, review criteria, etc. -->

## Documentation Standards

<!-- Define documentation requirements -->
<!-- Example: When to document, what to document, etc. -->

## Tooling

<!-- Define team tooling preferences -->
<!-- Example: IDE settings, linters, formatters, etc. -->

## Onboarding

<!-- Add information for new team members -->
"#
    .to_string()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_template_type_from_str() {
        assert_eq!(TemplateType::from_str("basic"), Some(TemplateType::Basic));
        assert_eq!(
            TemplateType::from_str("coding-standards"),
            Some(TemplateType::CodingStandards)
        );
        assert_eq!(
            TemplateType::from_str("architecture"),
            Some(TemplateType::Architecture)
        );
        assert_eq!(
            TemplateType::from_str("team-conventions"),
            Some(TemplateType::TeamConventions)
        );
        assert_eq!(TemplateType::from_str("invalid"), None);
    }

    #[test]
    fn test_template_type_as_str() {
        assert_eq!(TemplateType::Basic.as_str(), "basic");
        assert_eq!(TemplateType::CodingStandards.as_str(), "coding-standards");
        assert_eq!(TemplateType::Architecture.as_str(), "architecture");
        assert_eq!(TemplateType::TeamConventions.as_str(), "team-conventions");
    }

    #[test]
    fn test_generate_all_templates() {
        for template_type in TemplateType::all() {
            let content = generate_template(template_type);
            assert!(!content.is_empty());
            assert!(content.contains("#"));
        }
    }

    #[test]
    fn test_basic_template() {
        let content = generate_basic_template();
        assert!(content.contains("Project Context"));
        assert!(content.contains("Guidelines"));
    }

    #[test]
    fn test_coding_standards_template() {
        let content = generate_coding_standards_template();
        assert!(content.contains("Coding Standards"));
        assert!(content.contains("Code Formatting"));
    }

    #[test]
    fn test_architecture_template() {
        let content = generate_architecture_template();
        assert!(content.contains("Architecture"));
        assert!(content.contains("Components"));
    }

    #[test]
    fn test_team_conventions_template() {
        let content = generate_team_conventions_template();
        assert!(content.contains("Team Conventions"));
        assert!(content.contains("Communication"));
    }
}

