# Agent Routing Accuracy Test Plan

This document outlines the test cases for validating agent routing accuracy (≥90% target).

## Test Dataset (20+ Common Development Tasks)

### Code Refactoring Tasks
1. "Refactor the authentication module"
   - Expected: senior-developer, architect, or code-reviewer
2. "Improve error handling in the API"
   - Expected: senior-developer or backend-developer
3. "Clean up unused imports and dead code"
   - Expected: code-reviewer or senior-developer
4. "Optimize database queries"
   - Expected: backend-developer or database-specialist

### New Feature Implementation
5. "Create a new feature for task templates"
   - Expected: product-manager → architect → senior-developer (multi-agent)
6. "Add user authentication"
   - Expected: senior-developer or backend-developer
7. "Implement file upload functionality"
   - Expected: senior-developer or full-stack-developer
8. "Add dark mode to the UI"
   - Expected: frontend-developer or ui-designer

### Bug Fixing
9. "Fix the login bug"
   - Expected: senior-developer or bug-fixer
10. "Debug the API timeout issue"
    - Expected: backend-developer or senior-developer
11. "Fix memory leak in the application"
    - Expected: senior-developer or performance-specialist
12. "Resolve the database connection error"
    - Expected: backend-developer or database-specialist

### Documentation Tasks
13. "Write API documentation"
    - Expected: technical-writer or doc-writer
14. "Create user guide"
    - Expected: technical-writer or doc-writer
15. "Document the architecture"
    - Expected: architect or technical-writer

### Architecture Design
16. "Design the microservices architecture"
    - Expected: architect
17. "Plan the database schema"
    - Expected: architect or database-specialist
18. "Design the API structure"
    - Expected: architect or backend-developer

### Testing and QA
19. "Write unit tests for the authentication module"
    - Expected: tester or test-engineer
20. "Create integration tests"
    - Expected: tester or test-engineer
21. "Perform security audit"
    - Expected: security-specialist or security-auditor
22. "Test the payment flow"
    - Expected: tester or qa-engineer

### Multi-Agent Workflow Tasks
23. "Create a new feature with tests and documentation"
    - Expected: product-manager → architect → senior-developer → tester → doc-writer (multi-agent)
24. "Refactor the system and add comprehensive tests"
    - Expected: architect → senior-developer → tester (multi-agent)
25. "Design and implement a new API endpoint with tests"
    - Expected: architect → backend-developer → tester (multi-agent)

## Validation Criteria

For each test case:
1. Submit task to orchestrator
2. Record which agent(s) were selected
3. Compare with expected agent(s)
4. Mark as correct if:
   - Single-agent task: exact match OR reasonable alternative (e.g., senior-developer vs backend-developer for backend task)
   - Multi-agent task: all expected agents selected OR reasonable alternatives

## Routing Accuracy Calculation

```
Accuracy = (Correct Selections / Total Tasks) × 100
Target: ≥ 90%
```

## Notes

- Routing decisions depend on agent descriptions in agent config files
- Different providers (Gemini, Claude, OpenAI) may route differently
- Multi-agent workflows should be identified correctly
- Edge cases (ambiguous tasks) should result in reasonable selections

