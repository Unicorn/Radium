---
req_id: REQ-XXX
title: Feature Name
phase: NOW|NEXT|LATER
status: Not Started|In Progress|Completed
priority: Critical|High|Medium|Low
estimated_effort: XX-YY hours
dependencies: [REQ-001, REQ-002]
related_docs:
  - docs/project/03-implementation-plan.md#step-X
  - docs/features/feature-name.md
---

# Feature Name

## Problem Statement

[Why this feature exists, user pain points]

<!-- 
Guidance: Clearly articulate the problem this feature solves. What user needs or pain points does it address?
Include context about why this feature is necessary and what problems it solves.
-->

## Solution Overview

[What the feature does at a high level]

<!--
Guidance: Provide a high-level description of what the feature does. Focus on WHAT, not HOW.
This should give readers a clear understanding of the feature's purpose and scope.
-->

## Functional Requirements

[Detailed WHAT with acceptance criteria]

<!--
Guidance: List all functional requirements with clear acceptance criteria.
Each requirement should be:
- Specific and measurable
- Independent and testable
- Focused on WHAT needs to be built, not HOW
- Include acceptance criteria for each requirement
-->

### FR-1: [Requirement Name]

**Description**: [What this requirement does]

**Acceptance Criteria**:
- [ ] Criterion 1
- [ ] Criterion 2
- [ ] Criterion 3

### FR-2: [Requirement Name]

[Additional requirements...]

## Technical Requirements

[Constraints, integration points, data models]

<!--
Guidance: Document technical constraints, integration points, and data models.
Include:
- Data structures and schemas
- API contracts (high-level, not implementation details)
- Integration points with other systems
- Performance or scalability constraints
- Technology constraints (if any)
-->

### TR-1: [Technical Requirement Name]

**Description**: [Technical constraint or requirement]

**Data Models**:
```rust
// Example data structure
pub struct Example {
    pub field: String,
}
```

**APIs**:
- `function_name(param: Type) -> Result<ReturnType>`

### TR-2: [Additional technical requirements...]

## User Experience

[How users interact with the feature]

<!--
Guidance: Describe how users will interact with this feature.
Include:
- User workflows
- Command-line interfaces (if applicable)
- UI components (if applicable)
- Error handling and user feedback
-->

### UX-1: [User Experience Aspect]

**Description**: [How users interact with this feature]

**Example**:
```bash
rad command --option value
```

## Data Requirements

[Data models, storage, APIs]

<!--
Guidance: Document data storage requirements, data models, and API structures.
Include:
- Database schemas (if applicable)
- File formats (if applicable)
- Data persistence requirements
- Data validation rules
-->

### DR-1: [Data Requirement Name]

**Description**: [Data storage or model requirement]

**Schema**:
```json
{
  "field": "value",
  "type": "object"
}
```

## Dependencies

[Other REQs or systems this depends on]

<!--
Guidance: List all dependencies on other REQs or systems.
Include:
- Required REQs that must be completed first
- System dependencies
- External library or service dependencies
-->

- **REQ-XXX**: [Dependency description]
- **System**: [System dependency]

## Success Criteria

[Measurable outcomes and completion definition]

<!--
Guidance: Define clear, measurable success criteria.
Each criterion should be:
- Specific and testable
- Measurable (with metrics if possible)
- Realistic and achievable
-->

1. [ ] Success criterion 1
2. [ ] Success criterion 2
3. [ ] Success criterion 3

## Out of Scope

[Explicitly deferred items]

<!--
Guidance: Clearly state what is NOT included in this REQ.
This helps prevent scope creep and sets expectations.
-->

- Item 1 (deferred to future REQ)
- Item 2 (not part of this feature)

## References

[Links to original documentation]

<!--
Guidance: Include links to original documentation sources.
This provides traceability and allows readers to dive deeper.
-->

- [Now/Next/Later Roadmap](../project/02-now-next-later.md#step-X)
- [Implementation Plan](../project/03-implementation-plan.md#step-X)
- [Feature Enhancement Doc](../features/feature-name.md)
- [Codebase Reference](../../crates/radium-core/src/module/file.rs)

