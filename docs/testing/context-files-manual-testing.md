# Context Files Manual Testing Guide

**Feature**: Hierarchical Context Files (GEMINI.md)  
**Requirements**: See Braingrid for current REQ status: `braingrid requirement list -p PROJ-14 | grep -i "context"`  
**Date**: 2025-01-XX

## Overview

This guide provides step-by-step instructions for manually testing the context files feature. Context files allow users to provide persistent instructions to agents through hierarchical GEMINI.md files.

## Prerequisites

- Radium CLI installed and working
- A test workspace initialized (`rad init`)
- Basic familiarity with command line

## Test Environment Setup

1. Create a clean test workspace:
   ```bash
   mkdir -p ~/test-radium-workspace
   cd ~/test-radium-workspace
   rad init
   ```

2. Verify workspace structure:
   ```bash
   ls -la .radium/
   ```

## Test Scenarios

### Scenario 1: Basic Hierarchical Loading

**Goal**: Verify that context files are loaded from global and project locations.

**Steps**:
1. Create global context file:
   ```bash
   mkdir -p ~/.radium
   echo "# Global Context

This is the global context file.
It applies to all projects." > ~/.radium/GEMINI.md
   ```

2. Create project context file:
   ```bash
   echo "# Project Context

This project uses Rust.
Follow these guidelines:
- Use cargo fmt
- Write tests
- Document code" > GEMINI.md
   ```

3. Run an agent command:
   ```bash
   rad step code-agent "Create a hello world function"
   ```

4. **Expected Result**: Agent output should include both global and project context instructions.

**Validation**:
- Check that agent receives both contexts
- Verify project context appears after global context (higher precedence)

---

### Scenario 2: Subdirectory Context Override

**Goal**: Verify that subdirectory context files override project context.

**Steps**:
1. Create project root GEMINI.md:
   ```bash
   echo "# Project Context

General project guidelines." > GEMINI.md
   ```

2. Create subdirectory with context:
   ```bash
   mkdir -p src/api
   echo "# API-Specific Context

This directory contains API code.
Use RESTful conventions." > src/api/GEMINI.md
   ```

3. Navigate to subdirectory:
   ```bash
   cd src/api
   ```

4. Run agent command:
   ```bash
   rad step code-agent "Create an API endpoint"
   ```

5. **Expected Result**: Agent should receive all three contexts (global, project, subdirectory) with subdirectory context last (highest precedence).

**Validation**:
- Verify all three contexts are present
- Verify subdirectory context appears last

---

### Scenario 3: Simple Import

**Goal**: Verify that `@file.md` import syntax works.

**Steps**:
1. Create shared guidelines file:
   ```bash
   echo "# Shared Guidelines

These are shared across the project:
- Use TypeScript
- Follow ESLint rules
- Write JSDoc comments" > shared-guidelines.md
   ```

2. Create GEMINI.md with import:
   ```bash
   echo "# Project Context

@shared-guidelines.md

Additional project-specific rules." > GEMINI.md
   ```

3. Run agent command:
   ```bash
   rad step code-agent "Review this code"
   ```

4. **Expected Result**: Agent should receive content from both GEMINI.md and shared-guidelines.md.

**Validation**:
- Verify imported content is included
- Verify import directive is replaced with content

---

### Scenario 4: Nested Imports

**Goal**: Verify that imports can be nested (imported files can import other files).

**Steps**:
1. Create base.md:
   ```bash
   echo "# Base Guidelines

Foundation rules." > base.md
   ```

2. Create common.md that imports base.md:
   ```bash
   echo "# Common Guidelines

@base.md

Common practices." > common.md
   ```

3. Create GEMINI.md that imports common.md:
   ```bash
   echo "# Project Context

@common.md

Project-specific additions." > GEMINI.md
   ```

4. Run agent command:
   ```bash
   rad step code-agent "Create a new feature"
   ```

5. **Expected Result**: All three files' content should be merged correctly.

**Validation**:
- Verify all three files' content appears
- Verify nested imports are processed correctly

---

### Scenario 5: Circular Import Error

**Goal**: Verify that circular imports are detected and reported clearly.

**Steps**:
1. Create A.md that imports B.md:
   ```bash
   echo "# File A

@B.md

Content from A." > A.md
   ```

2. Create B.md that imports A.md:
   ```bash
   echo "# File B

@A.md

Content from B." > B.md
   ```

3. Create GEMINI.md that imports A.md:
   ```bash
   echo "# Project Context

@A.md" > GEMINI.md
   ```

4. Run agent command:
   ```bash
   rad step code-agent "Test"
   ```

5. **Expected Result**: Clear error message about circular import detected.

**Validation**:
- Verify error message identifies the circular dependency
- Verify error is helpful and actionable

---

### Scenario 6: Custom Context File Name

**Goal**: Verify that custom context file names work via configuration.

**Steps**:
1. Create configuration:
   ```bash
   mkdir -p .radium
   echo '[context]
file_name = "CONTEXT.md"' > .radium/config.toml
   ```

2. Create CONTEXT.md files:
   ```bash
   echo "# Custom Context File

This uses a custom name." > CONTEXT.md
   ```

3. Run agent command:
   ```bash
   rad step code-agent "Test"
   ```

4. **Expected Result**: CONTEXT.md should be loaded instead of GEMINI.md.

**Validation**:
- Verify custom file name is used
- Verify GEMINI.md is ignored if it exists

---

### Scenario 7: Backward Compatibility

**Goal**: Verify that system works without GEMINI.md files.

**Steps**:
1. Remove all GEMINI.md files:
   ```bash
   rm -f GEMINI.md
   rm -f ~/.radium/GEMINI.md
   ```

2. Ensure architecture.md exists:
   ```bash
   echo "# Architecture

System architecture documentation." > .radium/architecture.md
   ```

3. Run agent command:
   ```bash
   rad step code-agent "Review architecture"
   ```

4. **Expected Result**: System should work normally with architecture.md and other context sources.

**Validation**:
- Verify no errors occur
- Verify architecture.md context is still loaded
- Verify other context sources work normally

---

### Scenario 8: Mixed Context Sources

**Goal**: Verify that all context sources work together correctly.

**Steps**:
1. Create GEMINI.md:
   ```bash
   echo "# Project Guidelines

Project-specific rules." > GEMINI.md
   ```

2. Create architecture.md:
   ```bash
   echo "# Architecture

System design." > .radium/architecture.md
   ```

3. Create a plan:
   ```bash
   rad plan create "Test Plan"
   ```

4. Run agent with file injection:
   ```bash
   rad step code-agent[input:README.md] "Review project"
   ```

5. **Expected Result**: All context sources should appear in correct order:
   - Hierarchical context (GEMINI.md) first
   - Plan context
   - Architecture context
   - Memory context (if any)
   - Learning context (if any)
   - File injection (README.md)

**Validation**:
- Verify all 7 context sources are present
- Verify correct ordering
- Verify separators between contexts

---

### Scenario 9: Missing Import File

**Goal**: Verify helpful error message for missing import files.

**Steps**:
1. Create GEMINI.md with missing import:
   ```bash
   echo "# Project Context

@nonexistent-file.md

Other content." > GEMINI.md
   ```

2. Run agent command:
   ```bash
   rad step code-agent "Test"
   ```

3. **Expected Result**: Clear error message about missing import file.

**Validation**:
- Verify error message identifies the missing file
- Verify error is helpful and actionable

---

### Scenario 10: Empty Context File

**Goal**: Verify that empty context files are handled gracefully.

**Steps**:
1. Create empty GEMINI.md:
   ```bash
   touch GEMINI.md
   ```

2. Run agent command:
   ```bash
   rad step code-agent "Test"
   ```

3. **Expected Result**: No errors, empty context handled gracefully.

**Validation**:
- Verify no errors occur
- Verify system continues to work

---

### Scenario 11: Large Context File

**Goal**: Verify performance with large context files.

**Steps**:
1. Create large GEMINI.md (10KB+):
   ```bash
   # Generate large file
   for i in {1..1000}; do
     echo "Guideline $i: Follow best practices for line $i" >> GEMINI.md
   done
   ```

2. Run agent command and measure time:
   ```bash
   time rad step code-agent "Test"
   ```

3. **Expected Result**: Performance should be acceptable (< 2 seconds for context loading).

**Validation**:
- Verify context loads in reasonable time
- Verify no performance degradation

---

### Scenario 12: Context with Special Characters

**Goal**: Verify that markdown, code blocks, and special characters are preserved.

**Steps**:
1. Create GEMINI.md with special content:
   ```bash
   cat > GEMINI.md << 'EOF'
# Project Context

## Code Example

\`\`\`rust
fn main() {
    println!("Hello, world!");
}
\`\`\`

## Special Characters

<>&"'`{}\\[\\]

## Unicode

中文 Español Français 🚀
EOF
   ```

2. Run agent command:
   ```bash
   rad step code-agent "Review code"
   ```

3. **Expected Result**: All content should be preserved correctly.

**Validation**:
- Verify code blocks are preserved
- Verify special characters are preserved
- Verify unicode is preserved

---

### Scenario 13: Relative Path Imports

**Goal**: Verify that relative path imports work correctly.

**Steps**:
1. Create directory structure:
   ```bash
   mkdir -p docs/guides
   ```

2. Create imported file:
   ```bash
   echo "# Guide Content

Guide information." > docs/guides/guide.md
   ```

3. Create main file with relative import:
   ```bash
   echo "# Main Context

@docs/guides/guide.md

Additional content." > GEMINI.md
   ```

4. Run agent command:
   ```bash
   rad step code-agent "Test"
   ```

5. **Expected Result**: Relative import should resolve correctly.

**Validation**:
- Verify relative path is resolved
- Verify imported content is included

---

### Scenario 14: Absolute Path Imports

**Goal**: Verify that workspace-relative absolute paths work.

**Steps**:
1. Create shared file:
   ```bash
   mkdir -p shared
   echo "# Shared Content

Shared guidelines." > shared/rules.md
   ```

2. Create GEMINI.md with absolute path:
   ```bash
   echo "# Project Context

@/shared/rules.md

Project rules." > GEMINI.md
   ```

3. Run agent command:
   ```bash
   rad step code-agent "Test"
   ```

4. **Expected Result**: Absolute path should resolve from workspace root.

**Validation**:
- Verify absolute path is resolved correctly
- Verify imported content is included

---

### Scenario 15: Context File Modification

**Goal**: Verify that cache invalidation works when files are modified.

**Steps**:
1. Create initial GEMINI.md:
   ```bash
   echo "# Original Context

Original content." > GEMINI.md
   ```

2. Run agent command (first load):
   ```bash
   rad step code-agent "Test 1"
   ```

3. Modify GEMINI.md:
   ```bash
   echo "# Updated Context

Updated content with new guidelines." > GEMINI.md
   ```

4. Run agent command again (second load):
   ```bash
   rad step code-agent "Test 2"
   ```

5. **Expected Result**: Second run should use updated content (cache invalidated).

**Validation**:
- Verify updated content is used
- Verify cache invalidation works correctly

---

## Validation Checklist

After executing all scenarios, verify:

- [ ] All scenarios pass
- [ ] Error messages are clear and helpful
- [ ] Performance is acceptable
- [ ] Documentation is accurate
- [ ] Edge cases are handled gracefully
- [ ] Backward compatibility is maintained

## Troubleshooting

### Context Not Loading

**Symptoms**: Context files not appearing in agent output.

**Solutions**:
- Check file locations (global: `~/.radium/GEMINI.md`, project: `GEMINI.md`)
- Verify file names match (default: `GEMINI.md` or custom name)
- Check file permissions (must be readable)
- Verify workspace is initialized correctly

### Imports Not Working

**Symptoms**: Imported content not appearing.

**Solutions**:
- Check import syntax: `@file.md` (must be on its own line)
- Verify file paths are correct (relative or absolute)
- Check for circular imports (review import chain)
- Verify imported files exist and are readable

### Custom Name Not Working

**Symptoms**: Custom context file name not being used.

**Solutions**:
- Check configuration file: `.radium/config.toml`
- Verify configuration syntax is correct
- Check file name doesn't contain path separators
- Verify file name ends with `.md`

### Circular Import Errors

**Symptoms**: Error about circular import detected.

**Solutions**:
- Review import chain to identify cycle
- Remove circular dependency
- Restructure imports to avoid cycles

### Cache Issues

**Symptoms**: Old context content being used after modification.

**Solutions**:
- Wait a moment for filesystem timestamp to update
- Restart Radium process to clear cache
- Verify file modification time changed

## Notes

- Some scenarios may require specific workspace setup
- Global context file location: `~/.radium/GEMINI.md`
- Project context file: `GEMINI.md` in workspace root
- Subdirectory context: `GEMINI.md` in subdirectory
- Import syntax: `@file.md` on its own line
- Cache invalidates automatically when files are modified

