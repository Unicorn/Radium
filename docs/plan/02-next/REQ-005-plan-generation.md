---
req_id: REQ-005
title: Plan Generation & Execution
phase: NEXT
status: Completed
priority: High
estimated_effort: 15-20 hours
dependencies: [REQ-001, REQ-002, REQ-003]
related_docs:
  - docs/project/02-now-next-later.md#step-4-plan-generation--execution
  - docs/project/03-implementation-plan.md#step-4-plan-generation--execution
  - docs/legacy/legacy-system-feature-backlog.md#rad-plan-spec-path
---

# Plan Generation & Execution

## Problem Statement

Users need a way to generate structured, executable plans from high-level specifications. Without plan generation, users cannot:
- Convert feature specifications into actionable task breakdowns
- Organize work into logical iterations
- Track dependencies between tasks
- Execute plans systematically with progress tracking
- Resume execution from checkpoints
- Coordinate multiple agents across plan tasks

The legacy system provided `rad plan` and `rad craft` commands that generated plans from markdown specifications and executed them iteration-by-iteration. Radium needs an equivalent system with AI-powered plan generation and robust execution capabilities.

## Solution Overview

Implement a comprehensive plan generation and execution system that provides:
- AI-powered plan generation from markdown specifications
- Automatic iteration and task structure creation
- Dependency tracking and validation
- Plan manifest generation with metadata
- Plan execution with state persistence
- Progress tracking and checkpoint support
- Integration with agent system for task execution

The plan generation system enables users to break down complex features into manageable, executable plans with clear dependencies and acceptance criteria.

## Functional Requirements

### FR-1: Plan Generation (`rad plan`)

**Description**: Generate structured plans from markdown specifications using AI.

**Acceptance Criteria**:
- [x] Specification file parsing (file path or direct input)
- [x] AI-powered plan generation using LLM abstraction
- [x] Automatic RequirementId generation and validation
- [x] Plan directory structure creation
- [x] Iteration structure creation (3-5 iterations)
- [x] Task extraction with dependencies
- [x] Plan manifest generation with iterations/tasks
- [x] Markdown file generation (4 files):
  - `01_Plan_Overview_and_Setup.md`
  - `02_Iteration_I*.md` (one per iteration)
  - `03_Verification_and_Glossary.md`
  - `coordinator-prompt.md`
- [x] plan.json and plan_manifest.json output
- [x] Tech stack detection from project files

**Implementation**: 
- `apps/cli/src/commands/plan.rs` (~259 lines)
- `crates/radium-core/src/planning/generator.rs`
- `crates/radium-core/src/planning/parser.rs`
- `crates/radium-core/src/planning/markdown.rs`

### FR-2: Plan Execution (`rad craft`)

**Description**: Execute plans iteration-by-iteration with state persistence.

**Acceptance Criteria**:
- [x] Plan discovery by REQ-ID or folder name
- [x] Plan selection menu (if no identifier provided)
- [x] Iteration-by-iteration execution
- [x] Task-by-task execution with state persistence
- [x] Resume from checkpoint (full implementation)
- [x] Agent discovery and execution
- [x] Model execution with mock fallback
- [x] Dry-run mode
- [x] Dependency validation
- [x] Progress tracking with percentage display
- [x] Checkpoint persistence after each task
- [x] JSON output for CI/CD integration

**Implementation**: 
- `apps/cli/src/commands/craft.rs` (~305 lines)
- `crates/radium-core/src/planning/executor.rs` (~410 lines)

### FR-3: Plan Structure and Metadata

**Description**: Structured plan data with iterations, tasks, and dependencies.

**Acceptance Criteria**:
- [x] Plan struct with metadata (requirement_id, project_name, folder_name, stage, status, timestamps)
- [x] PlanManifest struct for plan structure (iterations, tasks)
- [x] Iteration struct with goal, description, tasks
- [x] Task struct with title, description, agent_id, dependencies, acceptance_criteria
- [x] Dependency tracking and validation
- [x] Progress calculation (completed_iterations, completed_tasks)
- [x] Plan status tracking (NotStarted, InProgress, Completed, Blocked)

**Implementation**: `crates/radium-core/src/models/plan.rs`

## Technical Requirements

### TR-1: Plan Generation API

**Description**: APIs for generating plans from specifications.

**APIs**:
```rust
pub struct PlanGenerator {
    config: PlanGeneratorConfig,
}

impl PlanGenerator {
    pub fn generate(&self, spec: &str) -> Result<ParsedPlan>;
    pub fn generate_with_model(&self, spec: &str, model: &dyn Model) -> Result<ParsedPlan>;
}

pub struct PlanParser;

impl PlanParser {
    pub fn parse(response: &str) -> Result<ParsedPlan>;
}
```

**Data Models**:
```rust
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ParsedPlan {
    pub project_name: String,
    pub description: Option<String>,
    pub tech_stack: Vec<String>,
    pub iterations: Vec<ParsedIteration>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ParsedIteration {
    pub number: u32,
    pub name: String,
    pub description: Option<String>,
    pub goal: Option<String>,
    pub tasks: Vec<ParsedTask>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ParsedTask {
    pub number: u32,
    pub title: String,
    pub description: Option<String>,
    pub agent_id: Option<String>,
    pub dependencies: Vec<String>,
    pub acceptance_criteria: Vec<String>,
}
```

### TR-2: Plan Execution API

**Description**: APIs for executing plans with state persistence.

**APIs**:
```rust
pub struct PlanExecutor {
    config: ExecutionConfig,
}

impl PlanExecutor {
    pub fn execute(&self, manifest: &PlanManifest) -> Result<Vec<TaskResult>>;
    pub fn execute_iteration(&self, manifest: &PlanManifest, iteration_num: u32) -> Result<Vec<TaskResult>>;
    pub fn execute_task(&self, manifest: &PlanManifest, task_id: &str) -> Result<TaskResult>;
    pub fn resume_from_checkpoint(&self, manifest_path: &Path) -> Result<Vec<TaskResult>>;
}

pub struct ExecutionConfig {
    pub resume: bool,
    pub skip_completed: bool,
    pub check_dependencies: bool,
    pub state_path: PathBuf,
}
```

### TR-3: Plan File Structure

**Description**: Directory structure and file formats for generated plans.

**Structure**:
```
radium/backlog/REQ-XXX-feature-name/
├── plan.json                    # Plan metadata
├── specifications.md            # Original specification
├── coordinator-prompt.md        # Coordinator agent prompt
└── plan/
    ├── plan_manifest.json       # Plan structure (iterations, tasks)
    ├── 01_Plan_Overview_and_Setup.md
    ├── 02_Iteration_I1.md
    ├── 02_Iteration_I2.md
    └── 03_Verification_and_Glossary.md
```

## User Experience

### UX-1: Plan Generation

**Description**: Users generate plans from specification files.

**Example**:
```bash
$ rad plan spec.md
Generating plan...
  Requirement ID: REQ-001
  Folder name: REQ-001-feature-name
  ✓ Generated 3 iterations
  ✓ Generated 12 tasks
Plan created: radium/backlog/REQ-001-feature-name
```

### UX-2: Plan Execution

**Description**: Users execute plans with progress tracking.

**Example**:
```bash
$ rad craft REQ-001
Executing plan: REQ-001-feature-name
Iteration 1 (3/12 tasks complete, 25%)
  ✓ Task 1.1: Setup workspace
  ✓ Task 1.2: Create data models
  → Task 1.3: Implement API endpoints
```

### UX-3: Resume from Checkpoint

**Description**: Users resume execution from last checkpoint.

**Example**:
```bash
$ rad craft REQ-001 --resume
Resuming from checkpoint...
  Last checkpoint: Task 1.3
  Continuing execution...
```

## Data Requirements

### DR-1: Plan Metadata

**Description**: JSON file containing plan metadata.

**Location**: `radium/<stage>/<folder-name>/plan.json`

**Schema**: See REQ-001 Plan struct

### DR-2: Plan Manifest

**Description**: JSON file containing plan structure (iterations and tasks).

**Location**: `radium/<stage>/<folder-name>/plan/plan_manifest.json`

**Schema**: See REQ-001 PlanManifest struct

### DR-3: Plan Markdown Files

**Description**: Markdown files documenting plan structure.

**Location**: `radium/<stage>/<folder-name>/plan/*.md`

**Format**: Markdown with plan overview, iteration details, and verification docs

## Dependencies

- **REQ-001**: Workspace System - Required for workspace structure and plan discovery
- **REQ-002**: Agent Configuration - Required for agent discovery and execution
- **REQ-003**: Core CLI Commands - Required for `rad plan` and `rad craft` commands

## Success Criteria

1. [x] Plans can be generated from markdown specifications
2. [x] AI-powered plan generation creates structured iterations and tasks
3. [x] Plans can be executed iteration-by-iteration
4. [x] Task dependencies are validated before execution
5. [x] Progress tracking accurately reflects completion status
6. [x] Execution can resume from checkpoints
7. [x] Plan manifest persists state correctly
8. [x] All plan operations have comprehensive test coverage (10+ tests)

**Completion Metrics**:
- **Status**: ✅ Complete
- **Lines of Code**: ~1,110 lines (planning module) + ~564 lines (CLI commands)
- **Test Coverage**: 10+ tests for planning module, 5+ tests for executor
- **Implementation**: Full plan generation and execution system
- **Files**: 
  - `crates/radium-core/src/planning/` (generator, parser, executor, markdown)
  - `apps/cli/src/commands/plan.rs`
  - `apps/cli/src/commands/craft.rs`

## Out of Scope

- Advanced plan optimization (future enhancement)
- Multi-plan coordination (future enhancement)
- Plan templates (covered in workflow templates)
- Plan versioning (future enhancement)

## References

- [Now/Next/Later Roadmap](../project/02-now-next-later.md#step-4-plan-generation--execution)
- [Implementation Plan](../project/03-implementation-plan.md#step-4-plan-generation--execution)
- [Feature Backlog](../legacy/legacy-system-feature-backlog.md#rad-plan-spec-path)
- [Planning Module Implementation](../../crates/radium-core/src/planning/)
- [Plan Command Implementation](../../apps/cli/src/commands/plan.rs)
- [Craft Command Implementation](../../apps/cli/src/commands/craft.rs)

