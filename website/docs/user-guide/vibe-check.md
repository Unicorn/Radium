---
id: "vibe-check"
title: "Vibe Check (Metacognitive Oversight)"
sidebar_label: "Vibe Check (Metacognitive Oversight)"
---

# Vibe Check (Metacognitive Oversight)

## Overview

Vibe Check is Radium's metacognitive oversight system that provides Chain-Pattern Interrupt (CPI) functionality to prevent reasoning lock-in and improve agent alignment with user intent. Research shows that CPI systems improve agent success rates by +27% and reduce harmful actions by -41%.

## What is Vibe Check?

Vibe Check allows agents to request metacognitive feedback from a second LLM (oversight LLM) to:
- Detect when they're making mistakes
- Recognize when they're overcomplicating solutions
- Identify misalignment with user intent
- Learn from past mistakes
- Apply successful strategies from previous work

## Benefits

- **Improved Success Rates**: +27% improvement in agent task completion
- **Reduced Harmful Actions**: -41% reduction in problematic behaviors
- **Better Alignment**: Agents stay aligned with user intent throughout execution
- **Learning Integration**: Mistakes and successes are captured for future improvement
- **Phase-Aware Feedback**: Oversight adapts to planning, implementation, and review phases

## Usage

### Manual Vibe Check via CLI

You can manually trigger a vibe check using the `rad vibecheck` command:

```bash
# Basic vibe check
rad vibecheck --goal "Build a web application" --plan "Use React and Node.js"

# With phase specification
rad vibecheck --phase planning --goal "Design API" --plan "REST API with Express"

# With progress and task context
rad vibecheck \
  --phase implementation \
  --goal "Build authentication" \
  --plan "JWT-based auth" \
  --progress "50% complete" \
  --task_context "Working on middleware"

# JSON output
rad vibecheck --goal "Test" --plan "Test plan" --json
```

### Automatic Vibe Check in Workflows

Agents can request vibe checks during workflow execution by writing a `behavior.json` file:

```json
{
  "action": "vibecheck",
  "reason": "Uncertain about approach, need oversight"
}
```

The workflow executor will detect this and trigger oversight automatically.

## Command Options

- `--phase <planning|implementation|review>`: Workflow phase (default: implementation)
- `--goal <text>`: Goal or objective being pursued
- `--plan <text>`: Current plan or approach
- `--progress <text>`: Progress made so far
- `--task_context <text>`: Task context or recent actions
- `--json`: Output results as JSON

## Understanding Oversight Feedback

### Risk Score

The risk score (0.0 to 1.0) indicates potential issues:
- **Low (0.0-0.3)**: Green - Approach looks good, continue
- **Medium (0.3-0.7)**: Yellow - Some concerns, consider adjustments
- **High (0.7-1.0)**: Red - Significant issues, major changes needed

### Advice

The oversight LLM provides actionable advice based on:
- Current workflow phase
- Goal and plan alignment
- Past mistakes and successes (from learning system)
- Constitution rules (if set)
- Detected patterns and traits

### Traits

Common traits detected:
- **Complex Solution Bias**: Over-engineering solutions
- **Feature Creep**: Adding unnecessary features
- **Premature Implementation**: Jumping to code too quickly
- **Misalignment**: Wrong direction or misunderstanding
- **Overtooling**: Using too many tools unnecessarily

### Uncertainties

Questions or unclear areas identified by the oversight LLM that should be addressed.

## Examples

### Example 1: Planning Phase Oversight

```bash
$ rad vibecheck --phase planning --goal "Build e-commerce site" --plan "Use microservices architecture"

Oversight Feedback

  • Risk Score: 0.65 (Medium)

  Advice:
  The microservices approach may be over-engineered for an initial e-commerce site.
  Consider starting with a monolithic architecture and refactoring to microservices
  only if scale demands it. Focus on core features first.

  • Traits:
    - Complex Solution Bias
    - Premature Implementation

  • Uncertainties:
    - Expected traffic volume unclear
    - Team size and microservices expertise unknown
```

### Example 2: Implementation Phase Oversight

```bash
$ rad vibecheck --phase implementation --goal "Add authentication" --plan "JWT tokens" --progress "50% complete"

Oversight Feedback

  • Risk Score: 0.35 (Low)

  Advice:
  JWT approach is appropriate for this use case. Ensure proper token expiration
  and refresh token handling. Consider adding rate limiting for login endpoints.

  • Helpful Patterns:
    + Using industry-standard JWT tokens
    + Proper error handling in place
```

## Integration with Learning System

Vibe Check automatically integrates with the learning system:
- Mistakes detected during oversight are logged
- Successful patterns are extracted and added to the skillbook
- Learning context is injected into future oversight requests
- Skills are tagged as helpful/harmful based on outcomes

## Best Practices

1. **Request vibe checks early**: Don't wait until you're stuck - get feedback during planning
2. **Be specific**: Provide clear goals, plans, and context for better feedback
3. **Review traits**: Pay attention to detected traits - they indicate recurring patterns
4. **Address uncertainties**: Use uncertainties as a checklist of things to clarify
5. **Learn from feedback**: Mistakes captured become part of the learning system

## Troubleshooting

### Vibe check not triggering in workflow

- Ensure `behavior.json` exists in `.radium/memory/behavior.json`
- Verify the action is set to `"vibecheck"` (lowercase)
- Check that the workflow executor has access to the workspace

### Oversight feedback seems generic

- Provide more context via `--goal`, `--plan`, and `--task_context`
- Ensure learning system has data (run `rad learning list` to check)
- Try different phases to get phase-specific feedback

### Risk score always low/high

- Risk scores are estimated from advice content keywords
- Very generic advice may result in default scores
- Provide specific context for more accurate risk assessment

## References

- [Learning System Documentation](./learning-system.md)
- [Constitution Rules Documentation](./constitution-rules.md)
- [Vibe Check Implementation](../../crates/radium-core/src/workflow/behaviors/vibe_check.rs)
- [Oversight Service Implementation](../../crates/radium-core/src/oversight/)

