# Radium Implementation Plans

This directory contains detailed implementation plans for major features and initiatives in the Radium project. Each plan is designed to be imported into Braingrid for project management and tracking.

## Purpose

Implementation plans serve as:
- **Technical Design Documents** - Detailed architecture and implementation approach
- **Project Roadmaps** - Phased breakdown of work with clear milestones
- **Task Tracking** - Granular tasks that map to Braingrid items
- **Reference Documentation** - Historical record of design decisions

## Plan Structure

Each plan follows this structure:

### Header
- **Status**: Planning, In Progress, Complete, Cancelled
- **Priority**: High, Medium, Low
- **Estimated Effort**: Time estimate
- **Owner**: Assigned team member
- **Created**: Creation date

### Sections
1. **Overview** - High-level summary of the feature
2. **Goals** - Primary goals and success criteria
3. **Architecture** - Technical design and components
4. **Implementation Phases** - Phased breakdown with tasks
5. **Configuration** - Example configuration
6. **Technical Decisions** - Key design choices
7. **Dependencies** - Required libraries/APIs
8. **Performance Considerations** - Optimization strategies
9. **Risk Mitigation** - Risks and mitigations
10. **Success Metrics** - How we measure success
11. **Future Enhancements** - Post-MVP ideas
12. **References** - Links to docs and resources

## Current Plans

> **All implementation plans are now tracked in BrainGrid (PROJ-14).**  
> Query for plans: `braingrid requirement list -p PROJ-14`  
> See [BRAINGRID_WORKFLOW.md](../project/BRAINGRID_WORKFLOW.md) for details.

### Active Plans
All active plans are in BrainGrid. Query for current status:
```bash
braingrid requirement list -p PROJ-14 --status IN_PROGRESS
```

### Completed Plans
All completed plans are in BrainGrid. Query for completed work:
```bash
braingrid requirement list -p PROJ-14 --status COMPLETED
```

**Example**: Model-Agnostic Orchestration is tracked as REQ-160 in BrainGrid:
```bash
braingrid requirement show REQ-160 -p PROJ-14
```

## Workflow

### 1. Create Plan
- Draft plan using the template structure
- Get review and feedback from team
- Update based on feedback

### 2. Import to Braingrid
- Import plan tasks into Braingrid
- Assign owners and due dates
- Set up dependencies between tasks

### 3. Implementation
- Work through phases sequentially
- Update task status in Braingrid
- Document decisions and changes

### 4. Review & Close
- Verify success criteria met
- Update plan status to Complete
- Document lessons learned

## Task Naming Convention

Tasks use the following format:
```
[PREFIX]-[NUMBER]: [Description]
```

**Examples:**
- `ORCH-001`: Create OrchestrationProvider trait
- `TUI-042`: Add streaming response UI
- `AUTH-015`: Implement OAuth flow

**Prefixes:**
- `ORCH`: Orchestration
- `TUI`: Terminal UI
- `CLI`: Command Line Interface
- `CORE`: Core infrastructure
- `AUTH`: Authentication
- `MODEL`: Model abstraction
- `AGENT`: Agent system
- `WEB`: Web interface
- `DESK`: Desktop app

## Best Practices

### Writing Plans
- ✅ Be specific about file paths and components
- ✅ Include code examples for clarity
- ✅ Define clear success criteria
- ✅ Estimate effort realistically
- ✅ Identify dependencies early
- ❌ Don't make plans too granular (combine small tasks)
- ❌ Don't skip the "why" (include rationale for decisions)

### Managing Plans
- Review plans before starting implementation
- Update plans as decisions change
- Archive completed plans (don't delete)
- Reference plans in commit messages

## Templates

### Quick Task Template
```markdown
- [ ] **[PREFIX]-[NUM]**: [Task Description]
  - [Implementation details]
  - [Expected outcome]
  - **Files**: `path/to/file.rs`
  - **Dependencies**: [Other tasks]
```

### Plan Section Template
```markdown
### Phase N: [Phase Name] (Days X-Y)
**Goal**: [What this phase accomplishes]

#### Tasks
- [ ] **TASK-001**: [Task description]
  - [Details]
  - **Files**: [Files to modify/create]
  - **Dependencies**: [Prerequisites]
```

## Integration with Braingrid

**All plans are now managed in BrainGrid (PROJ-14).** This directory serves as a reference for:
- Historical plan documents (if needed for reference)
- Plan templates and structure guidelines
- Best practices for creating plans in BrainGrid

When creating new plans:
1. Create the requirement in BrainGrid using `braingrid specify -p PROJ-14 --prompt "Feature description"`
2. BrainGrid will automatically break it down into tasks
3. Review and refine the task breakdown in BrainGrid
4. Reference the BrainGrid REQ in commit messages: `[REQ-XXX]`

For details, see [BRAINGRID_WORKFLOW.md](../project/BRAINGRID_WORKFLOW.md).

## Questions?

For questions about:
- **Plan structure**: See this README
- **Implementation details**: See the specific plan
- **Project status**: Check Braingrid
- **Technical decisions**: See plan's "Technical Decisions" section
