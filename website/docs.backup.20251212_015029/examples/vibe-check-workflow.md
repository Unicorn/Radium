# Vibe Check Workflow Example

## Complete Workflow with Vibe Check

This example demonstrates a complete workflow that uses vibe check for metacognitive oversight at key checkpoints.

## Workflow Structure

```yaml
# workflow.yaml
name: Feature Development with Oversight
description: Build a feature with vibe check at each phase

steps:
  - id: plan
    name: Planning Phase
    agent: planner
    checkpoint: vibecheck
    
  - id: implement
    name: Implementation Phase
    agent: developer
    checkpoint: vibecheck
    
  - id: review
    name: Review Phase
    agent: reviewer
    checkpoint: vibecheck
```

## Step 1: Planning Phase

### Agent Writes behavior.json

```json
{
  "action": "vibecheck",
  "reason": "Need to verify the plan aligns with requirements before proceeding"
}
```

### Oversight Feedback

```
Oversight Feedback

  • Risk Score: 0.55 (Medium)

  Advice:
  The plan looks comprehensive but may be over-engineered for the initial requirements.
  Consider starting with a simpler approach and iterating. Focus on core features first.

  • Traits:
    - Complex Solution Bias
    - Feature Creep

  • Uncertainties:
    - Performance requirements unclear
    - Scalability needs unknown
```

### Learning Capture

The oversight feedback triggers learning capture:
- Mistake logged: "Complex Solution Bias - Over-engineered initial plan"
- Skill added: "Start with simpler approach, iterate as needed"
- Pattern extracted: "Avoid over-engineering in planning phase"

## Step 2: Implementation Phase

### Agent Writes behavior.json

```json
{
  "action": "vibecheck",
  "reason": "Uncertain if current implementation approach is optimal"
}
```

### Oversight Feedback (with Learning Context)

```
Oversight Feedback

  • Risk Score: 0.40 (Low)

  Advice:
  The implementation is progressing well. Based on past learning, remember to keep
  the solution simple and avoid unnecessary abstractions. The current approach aligns
  with the simplified plan.

  • Helpful Patterns:
    + Following the simplified plan from planning phase
    + Using straightforward error handling

  • Uncertainties:
    - Edge case handling needs verification
```

### Learning Updates

- Skill tagged as helpful: "Start with simpler approach"
- New skill added: "Use straightforward error handling patterns"
- Learning context updated with successful patterns

## Step 3: Review Phase

### Agent Writes behavior.json

```json
{
  "action": "vibecheck",
  "reason": "Final review before completion"
}
```

### Oversight Feedback

```
Oversight Feedback

  • Risk Score: 0.30 (Low)

  Advice:
  The review shows good progress. The simplified approach worked well. Ensure all
  edge cases from the implementation phase are covered. Consider adding integration
  tests for the error handling patterns.

  • Helpful Patterns:
    + Simplified approach maintained throughout
    + Consistent error handling

  • Uncertainties:
    - Integration test coverage needs verification
```

## Complete Learning Loop

After the workflow completes:

### Learning Store Contents

```bash
$ rad learning list

Learning Entries

  • [Mistake] Complex Solution Bias
    Over-engineered initial plan
    → Solution: Start with simpler approach, iterate as needed
    Timestamp: 2025-12-07 10:00:00

  • [Success] General
    Simplified approach worked well
    Timestamp: 2025-12-07 10:30:00
```

### Skillbook Contents

```bash
$ rad learning show-skillbook

Skillbook

  task_guidance
    • [skill-00001]
      Start with simpler approach, iterate as needed
      Stats: Helpful: 2 | Harmful: 0 | Neutral: 0

  code_patterns
    • [skill-00002]
      Use straightforward error handling patterns
      Stats: Helpful: 1 | Harmful: 0 | Neutral: 0
```

## Benefits Demonstrated

1. **Early Detection**: Planning phase oversight caught over-engineering before implementation
2. **Context Awareness**: Implementation phase oversight used learning from planning phase
3. **Pattern Learning**: Successful patterns were extracted and added to skillbook
4. **Continuous Improvement**: Each phase built on learnings from previous phases

## Next Workflow

In the next similar workflow, the learning system will:
- Inject the "Start with simpler approach" skill into oversight context
- Reference the "Complex Solution Bias" mistake to avoid repetition
- Apply learned patterns automatically

This creates a continuous learning loop that improves over time.

## References

- [Vibe Check Documentation](../user-guide/vibe-check.md)
- [Learning System Documentation](../user-guide/learning-system.md)
- [Constitution Rules Documentation](../user-guide/constitution-rules.md)

