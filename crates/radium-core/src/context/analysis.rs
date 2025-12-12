//! Question type detection and analysis planning for intelligent context building.

/// Types of questions that can be asked, each requiring different analysis strategies.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum QuestionType {
    /// General project overview questions (e.g., "Tell me about this project")
    ProjectOverview,
    /// Technology stack questions (e.g., "What is this built on?")
    TechnologyStack,
    /// Architecture questions (e.g., "How does X work?")
    Architecture,
    /// Implementation questions (e.g., "How is Y implemented?")
    Implementation,
    /// Specific file questions (e.g., "What does file.rs do?")
    SpecificFile,
    /// Code analysis questions (e.g., "Analyze this code", "Review this")
    CodeAnalysis,
    /// Feature request or implementation task
    FeatureRequest,
    /// Debugging or troubleshooting question
    Debugging,
    /// Documentation question
    Documentation,
    /// General inquiry that doesn't fit other categories
    General,
}

impl QuestionType {
    /// Detects the question type from user input.
    ///
    /// Uses pattern matching on keywords and question structure to classify
    /// the type of question being asked.
    ///
    /// # Arguments
    /// * `input` - The user's input/question
    ///
    /// # Returns
    /// The detected question type
    pub fn detect(input: &str) -> Self {
        let lower = input.to_lowercase();

        // Project overview patterns
        if Self::matches_patterns(
            &lower,
            &[
                "tell me about",
                "what is this",
                "describe this project",
                "overview",
                "what does this",
                "explain this project",
            ],
        ) {
            return QuestionType::ProjectOverview;
        }

        // Technology stack patterns
        if Self::matches_patterns(
            &lower,
            &[
                "what is this built on",
                "what technologies",
                "what stack",
                "what language",
                "what framework",
                "dependencies",
                "built with",
                "uses",
            ],
        ) {
            return QuestionType::TechnologyStack;
        }

        // Architecture patterns
        if Self::matches_patterns(
            &lower,
            &[
                "how does",
                "how is",
                "architecture",
                "design",
                "structure",
                "how it works",
                "how x works",
            ],
        ) {
            return QuestionType::Architecture;
        }

        // Implementation patterns
        if Self::matches_patterns(
            &lower,
            &[
                "how is",
                "how to implement",
                "implementation",
                "where is",
                "find the",
            ],
        ) {
            return QuestionType::Implementation;
        }

        // Specific file patterns
        if Self::matches_patterns(
            &lower,
            &[
                "what does",
                "file:",
                ".rs",
                ".ts",
                ".py",
                ".js",
                "in file",
            ],
        ) || lower.contains('/') && (lower.contains('.') || lower.contains("file")) {
            return QuestionType::SpecificFile;
        }

        // Code analysis patterns
        if Self::matches_patterns(
            &lower,
            &[
                "analyze",
                "review",
                "check",
                "audit",
                "quality",
                "issues",
                "problems",
                "bugs",
            ],
        ) {
            return QuestionType::CodeAnalysis;
        }

        // Feature request patterns
        if Self::matches_patterns(
            &lower,
            &[
                "implement",
                "add",
                "create",
                "build",
                "feature",
                "new",
            ],
        ) {
            return QuestionType::FeatureRequest;
        }

        // Debugging patterns
        if Self::matches_patterns(
            &lower,
            &[
                "debug",
                "error",
                "bug",
                "fix",
                "broken",
                "not working",
                "issue",
            ],
        ) {
            return QuestionType::Debugging;
        }

        // Documentation patterns
        if Self::matches_patterns(
            &lower,
            &[
                "documentation",
                "docs",
                "readme",
                "guide",
                "how to",
                "tutorial",
            ],
        ) {
            return QuestionType::Documentation;
        }

        QuestionType::General
    }

    /// Checks if input matches any of the given patterns.
    fn matches_patterns(input: &str, patterns: &[&str]) -> bool {
        patterns.iter().any(|pattern| input.contains(pattern))
    }

    /// Returns the recommended files to read for this question type.
    ///
    /// # Returns
    /// Vector of file paths (relative to project root) that should be read
    pub fn recommended_files(&self) -> Vec<&'static str> {
        match self {
            QuestionType::ProjectOverview => vec![
                "README.md",
                "package.json",
                "Cargo.toml",
                "nx.json",
                "docs/development/agent-instructions.md",
                "GEMINI.md",
            ],
            QuestionType::TechnologyStack => vec![
                "package.json",
                "Cargo.toml",
                "requirements.txt",
                "go.mod",
                "pom.xml",
                "build.gradle",
                "README.md",
            ],
            QuestionType::Architecture => vec![
                "README.md",
                "docs/architecture",
                "docs/design",
                "GEMINI.md",
            ],
            QuestionType::Implementation => vec![
                // Implementation-specific files depend on the question
                // This is a base set, should be supplemented with semantic search
            ],
            QuestionType::SpecificFile => vec![
                // File path will be extracted from question
            ],
            QuestionType::CodeAnalysis => vec![
                // Files to analyze will be specified or discovered
            ],
            QuestionType::FeatureRequest => vec![
                "README.md",
                "docs/development/agent-instructions.md",
            ],
            QuestionType::Debugging => vec![
                // Error-specific, will be discovered
            ],
            QuestionType::Documentation => vec![
                "README.md",
                "docs",
            ],
            QuestionType::General => vec![
                "README.md",
                "package.json",
                "Cargo.toml",
            ],
        }
    }

    /// Returns suggested semantic search queries for this question type.
    ///
    /// # Arguments
    /// * `input` - The original user input for context
    ///
    /// # Returns
    /// Vector of suggested search queries
    pub fn suggested_searches(&self, input: &str) -> Vec<String> {
        match self {
            QuestionType::ProjectOverview => vec![
                "What is the main purpose and architecture of this project?".to_string(),
                "How does the project structure work?".to_string(),
            ],
            QuestionType::TechnologyStack => vec![
                "What technologies and frameworks are used in this project?".to_string(),
            ],
            QuestionType::Architecture => {
                // Extract key terms from input for targeted search
                let search_query = if input.len() > 50 {
                    format!("How does {} work?", &input[..50])
                } else {
                    format!("How does {} work?", input)
                };
                vec![search_query]
            }
            QuestionType::Implementation => {
                let search_query = if input.len() > 50 {
                    format!("Where is {} implemented?", &input[..50])
                } else {
                    format!("Where is {} implemented?", input)
                };
                vec![search_query]
            }
            QuestionType::SpecificFile => vec![],
            QuestionType::CodeAnalysis => vec![
                "What are the code quality patterns and issues in this codebase?".to_string(),
            ],
            QuestionType::FeatureRequest => vec![],
            QuestionType::Debugging => vec![],
            QuestionType::Documentation => vec![],
            QuestionType::General => vec![],
        }
    }

    /// Returns guidance for synthesizing information for this question type.
    ///
    /// # Returns
    /// Synthesis guidance string
    pub fn synthesis_guidance(&self) -> &'static str {
        match self {
            QuestionType::ProjectOverview => {
                "Combine information from README, build files, and architecture docs. \
                 Provide comprehensive overview covering purpose, tech stack, architecture, \
                 and key features. Include specific examples with file paths."
            }
            QuestionType::TechnologyStack => {
                "Extract technology information from build files and dependencies. \
                 List languages, frameworks, libraries, and tools. Include version \
                 information where available."
            }
            QuestionType::Architecture => {
                "Explain how the system works by combining architecture docs, code \
                 structure, and implementation details. Use semantic search to find \
                 related components and explain their relationships."
            }
            QuestionType::Implementation => {
                "Find implementation details through semantic search and code reading. \
                 Explain how features are implemented, including code examples and \
                 file references."
            }
            QuestionType::SpecificFile => {
                "Read the specific file and related files. Explain what the file does, \
                 its role in the system, and how it relates to other components."
            }
            QuestionType::CodeAnalysis => {
                "Analyze code quality, patterns, and issues. Provide evidence-based \
                 findings with specific file and line references."
            }
            QuestionType::FeatureRequest => {
                "Understand existing patterns before implementing. Review similar \
                 features and follow project conventions."
            }
            QuestionType::Debugging => {
                "Identify the problem through error analysis, code review, and \
                 related file examination."
            }
            QuestionType::Documentation => {
                "Find and synthesize relevant documentation. Provide clear, organized \
                 information from multiple sources."
            }
            QuestionType::General => {
                "Use semantic search and targeted file reading to answer the question. \
                 Synthesize information from multiple sources."
            }
        }
    }
}

/// Analysis plan for answering a question.
///
/// Provides structured guidance on what files to read, what searches to perform,
/// and how to synthesize the information.
#[derive(Debug, Clone)]
pub struct AnalysisPlan {
    /// The detected question type
    pub question_type: QuestionType,
    /// Files that should be read (relative to project root)
    pub recommended_files: Vec<String>,
    /// Suggested semantic search queries
    pub suggested_searches: Vec<String>,
    /// Guidance for synthesizing information
    pub synthesis_guidance: String,
}

impl AnalysisPlan {
    /// Creates an analysis plan from user input.
    ///
    /// # Arguments
    /// * `input` - The user's input/question
    ///
    /// # Returns
    /// An analysis plan with recommendations
    pub fn from_input(input: &str) -> Self {
        let question_type = QuestionType::detect(input);
        let recommended_files = question_type
            .recommended_files()
            .iter()
            .map(|s| s.to_string())
            .collect();
        let suggested_searches = question_type.suggested_searches(input);
        let synthesis_guidance = question_type.synthesis_guidance().to_string();

        Self {
            question_type,
            recommended_files,
            suggested_searches,
            synthesis_guidance,
        }
    }

    /// Formats the analysis plan as a context string for agent prompts.
    ///
    /// # Returns
    /// Formatted string with analysis guidance
    pub fn to_context_string(&self) -> String {
        let mut context = String::new();
        context.push_str("# Analysis Plan\n\n");
        context.push_str(&format!("**Question Type**: {:?}\n\n", self.question_type));
        
        if !self.recommended_files.is_empty() {
            context.push_str("## Recommended Files to Read\n\n");
            for file in &self.recommended_files {
                context.push_str(&format!("- `{}`\n", file));
            }
            context.push_str("\n");
        }

        if !self.suggested_searches.is_empty() {
            context.push_str("## Suggested Semantic Searches\n\n");
            for search in &self.suggested_searches {
                context.push_str(&format!("- `codebase_search(\"{}\")`\n", search));
            }
            context.push_str("\n");
        }

        context.push_str("## Synthesis Guidance\n\n");
        context.push_str(&self.synthesis_guidance);
        context.push_str("\n\n");

        context
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_project_overview_detection() {
        assert_eq!(
            QuestionType::detect("Tell me about this project"),
            QuestionType::ProjectOverview
        );
        assert_eq!(
            QuestionType::detect("What is this project?"),
            QuestionType::ProjectOverview
        );
    }

    #[test]
    fn test_technology_stack_detection() {
        assert_eq!(
            QuestionType::detect("What is this built on?"),
            QuestionType::TechnologyStack
        );
        assert_eq!(
            QuestionType::detect("What technologies are used?"),
            QuestionType::TechnologyStack
        );
    }

    #[test]
    fn test_analysis_plan_creation() {
        let plan = AnalysisPlan::from_input("Tell me about this project");
        assert_eq!(plan.question_type, QuestionType::ProjectOverview);
        assert!(!plan.recommended_files.is_empty());
        assert!(!plan.suggested_searches.is_empty());
    }

    #[test]
    fn test_analysis_plan_context_string() {
        let plan = AnalysisPlan::from_input("Tell me about this project");
        let context = plan.to_context_string();
        assert!(context.contains("Analysis Plan"));
        assert!(context.contains("README.md"));
        assert!(context.contains("codebase_search"));
    }
}

