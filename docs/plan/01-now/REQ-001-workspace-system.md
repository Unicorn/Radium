---
req_id: REQ-001
title: Workspace System
phase: NOW
status: Completed
priority: Critical
estimated_effort: 10-14 hours
dependencies: []
related_docs:
  - docs/project/02-now-next-later.md#step-0-workspace-system
  - docs/project/03-implementation-plan.md#step-0-workspace-system
  - docs/legacy/legacy-system-feature-backlog.md#51-workspace-structure
---

# Workspace System

## Problem Statement

Radium requires a standardized workspace structure to organize plans, artifacts, and runtime data. Without a consistent directory structure, it's impossible to:
- Track plans across different stages (backlog, development, review, testing)
- Manage requirement IDs and plan metadata
- Discover and list plans programmatically
- Organize agent outputs, memory, and logs per plan
- Maintain workspace state and configuration

The legacy system used a specific directory structure that all features depended on. Radium needs an equivalent structure that supports plan management, requirement tracking, and workspace operations.

## Solution Overview

Implement a comprehensive workspace system that provides:
- Standardized directory structure with stage-based organization
- Requirement ID system (REQ-XXX format) with auto-incrementing counter
- Plan discovery and listing across all workspace stages
- Plan structure types and validation
- Internal workspace management for artifacts, memory, logs, prompts, and inputs

The workspace system serves as the foundation for all other Radium features, enabling plan tracking, agent execution, and workspace operations.

## Functional Requirements

### FR-1: Workspace Directory Structure

**Description**: Create a standardized directory structure that organizes plans by stage and manages internal workspace data.

**Acceptance Criteria**:
- [x] Root workspace directory (`.radium/`) exists
- [x] Stage directories created: `backlog/`, `development/`, `review/`, `testing/`, `docs/`
- [x] Internal directory structure: `_internals/` with subdirectories:
  - `agents/` - Agent configurations and data
  - `artifacts/` - Generated artifacts
  - `memory/` - Plan-scoped memory storage
  - `logs/` - Agent and system logs
  - `prompts/` - Cached prompt templates
  - `inputs/` - Input files and data
- [x] Plan directories follow `REQ-XXX-slug` naming convention
- [x] Workspace structure validation and initialization

**Implementation**: `crates/radium-core/src/workspace/structure.rs`

### FR-2: Requirement ID System

**Description**: Implement requirement ID management with REQ-XXX format and auto-incrementing counter.

**Acceptance Criteria**:
- [x] RequirementId type with REQ-XXX format (e.g., REQ-001, REQ-002)
- [x] Auto-incrementing counter stored in `.radium/_internals/requirement-counter.json`
- [x] Thread-safe counter management
- [x] ID format validation and parsing
- [x] Manual ID override support
- [x] Duplicate detection

**Implementation**: `crates/radium-core/src/workspace/requirement_id.rs`

### FR-3: Plan Structure Types

**Description**: Define data structures for plans, iterations, and tasks with serialization support.

**Acceptance Criteria**:
- [x] Plan struct with metadata (requirement_id, project_name, folder_name, stage, status, timestamps)
- [x] PlanManifest struct for plan structure (iterations, tasks)
- [x] PlanStatus enum (NotStarted, InProgress, Completed, Blocked)
- [x] Iteration and Task structs
- [x] Serde serialization for plan.json and plan_manifest.json
- [x] Progress tracking (completed_iterations, completed_tasks, total counts)

**Implementation**: `crates/radium-core/src/models/plan.rs`

### FR-4: Plan Discovery

**Description**: Discover and list plans across all workspace stages with filtering and sorting.

**Acceptance Criteria**:
- [x] Scan all workspace stages for plans
- [x] Find plan by REQ-ID (e.g., REQ-001)
- [x] Find plan by folder name (e.g., REQ-001-feature-name)
- [x] List all plans with metadata
- [x] Calculate progress percentages
- [x] Sort by date (created_at, updated_at) or ID
- [x] Filter by stage
- [x] Load plan manifests

**Implementation**: `crates/radium-core/src/workspace/plan_discovery.rs`

## Technical Requirements

### TR-1: Directory Structure

**Description**: Standardized workspace directory layout.

**Structure**:
```
.radium/
├── _internals/
│   ├── agents/
│   ├── artifacts/
│   ├── memory/
│   ├── logs/
│   ├── prompts/
│   ├── inputs/
│   └── requirement-counter.json
├── backlog/          # Generated plans
├── development/      # Active development
├── review/           # Review stage
├── testing/          # Testing stage
└── docs/             # Documentation
```

**Data Models**:
```rust
pub struct WorkspaceStructure {
    root: PathBuf,
}

pub struct Workspace {
    structure: WorkspaceStructure,
    config: WorkspaceConfig,
}
```

### TR-2: Requirement ID Format

**Description**: REQ-XXX format with auto-incrementing.

**Data Models**:
```rust
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct RequirementId {
    number: u32,
}

impl RequirementId {
    pub fn from_str(s: &str) -> Result<Self>;  // Parse "REQ-001"
    pub fn to_string(&self) -> String;         // Format as "REQ-001"
    pub fn number(&self) -> u32;               // Get numeric value
    pub fn next(workspace: &Workspace) -> Result<Self>;  // Auto-increment
}
```

**Storage**: `.radium/_internals/requirement-counter.json`
```json
{
  "counter": 1
}
```

### TR-3: Plan Data Structures

**Description**: Plan metadata and structure types.

**Data Models**:
```rust
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Plan {
    pub requirement_id: RequirementId,
    pub project_name: String,
    pub folder_name: String,
    pub stage: String,
    pub status: PlanStatus,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
    pub total_iterations: u32,
    pub completed_iterations: u32,
    pub total_tasks: u32,
    pub completed_tasks: u32,
    pub metadata: HashMap<String, serde_json::Value>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PlanManifest {
    pub requirement_id: RequirementId,
    pub project_name: String,
    pub iterations: Vec<Iteration>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum PlanStatus {
    NotStarted,
    InProgress,
    Completed,
    Blocked,
}
```

**Storage**: 
- `radium/<stage>/<folder-name>/plan.json` - Plan metadata
- `radium/<stage>/<folder-name>/plan/plan_manifest.json` - Plan structure

### TR-4: Plan Discovery API

**Description**: APIs for discovering and querying plans.

**APIs**:
```rust
pub struct PlanDiscovery {
    workspace: Workspace,
}

impl PlanDiscovery {
    pub fn discover(&self) -> Result<Vec<DiscoveredPlan>>;
    pub fn discover_with_options(&self, options: &PlanDiscoveryOptions) -> Result<Vec<DiscoveredPlan>>;
    pub fn find_by_requirement_id(&self, req_id: RequirementId) -> Result<Option<DiscoveredPlan>>;
    pub fn find_by_folder_name(&self, folder_name: &str) -> Result<Option<DiscoveredPlan>>;
}

pub struct DiscoveredPlan {
    pub plan: Plan,
    pub path: PathBuf,
    pub has_manifest: bool,
}
```

## User Experience

### UX-1: Workspace Initialization

**Description**: Users initialize workspace with `rad init` command.

**Example**:
```bash
$ rad init
Creating workspace structure...
✓ Created .radium directory
✓ Created stage directories
✓ Initialized requirement counter
```

### UX-2: Plan Discovery

**Description**: Users can discover and list plans across stages.

**Example**:
```bash
$ rad status
Workspace: /path/to/project
Plans:
  REQ-001-feature-name (backlog) - 0% complete
  REQ-002-another-feat (development) - 45% complete
```

### UX-3: Requirement ID Generation

**Description**: Requirement IDs auto-increment when creating new plans.

**Example**:
```bash
$ rad plan spec.md
Generating plan...
  Requirement ID: REQ-001
  Folder name: REQ-001-project-name
```

## Data Requirements

### DR-1: Requirement Counter

**Description**: JSON file tracking the next requirement ID.

**Schema**:
```json
{
  "counter": 1
}
```

**Location**: `.radium/_internals/requirement-counter.json`

### DR-2: Plan Metadata

**Description**: JSON file containing plan metadata.

**Schema**: See TR-3 Plan struct

**Location**: `radium/<stage>/<folder-name>/plan.json`

### DR-3: Plan Manifest

**Description**: JSON file containing plan structure (iterations and tasks).

**Schema**: See TR-3 PlanManifest struct

**Location**: `radium/<stage>/<folder-name>/plan/plan_manifest.json`

## Dependencies

None - This is the foundation feature that other features depend on.

## Success Criteria

1. [x] Workspace structure can be created and validated
2. [x] Requirement IDs auto-increment correctly (REQ-001, REQ-002, etc.)
3. [x] Plans can be discovered across all stages
4. [x] Plans can be found by REQ-ID or folder name
5. [x] Plan progress can be calculated accurately
6. [x] All workspace operations have comprehensive test coverage (22+ tests)
7. [x] Workspace structure matches legacy system conventions

**Completion Metrics**:
- **Status**: ✅ Complete
- **Test Coverage**: 22+ passing tests
- **Implementation**: All workspace features fully implemented
- **Files**: 
  - `crates/radium-core/src/workspace/mod.rs`
  - `crates/radium-core/src/workspace/structure.rs`
  - `crates/radium-core/src/workspace/requirement_id.rs`
  - `crates/radium-core/src/workspace/plan_discovery.rs`
  - `crates/radium-core/src/models/plan.rs`

## Out of Scope

- Plan generation logic (covered in REQ-005)
- Plan execution (covered in REQ-005)
- Agent-specific workspace features (covered in REQ-002)
- Memory storage implementation (covered in REQ-006)

## References

- [Now/Next/Later Roadmap](../project/02-now-next-later.md#step-0-workspace-system)
- [Implementation Plan](../project/03-implementation-plan.md#step-0-workspace-system)
- [Feature Backlog](../legacy/legacy-system-feature-backlog.md#51-workspace-structure)
- [Workspace Module](../../crates/radium-core/src/workspace/mod.rs)
- [Requirement ID Implementation](../../crates/radium-core/src/workspace/requirement_id.rs)
- [Plan Discovery Implementation](../../crates/radium-core/src/workspace/plan_discovery.rs)

