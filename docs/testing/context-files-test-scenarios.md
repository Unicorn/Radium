# Context Files Test Scenarios Catalog

**Feature**: Hierarchical Context Files (GEMINI.md)  
**Requirements**: See Braingrid for current REQ status: `braingrid requirement list -p PROJ-14 | grep -i "context"`  
**Date**: 2025-01-XX

## Overview

This document catalogs all test scenarios for the context files feature. Each scenario includes setup, execution steps, and expected results.

## Scenario Index

1. [Basic Hierarchical Loading](#scenario-1-basic-hierarchical-loading)
2. [Subdirectory Context Override](#scenario-2-subdirectory-context-override)
3. [Simple Import](#scenario-3-simple-import)
4. [Nested Imports](#scenario-4-nested-imports)
5. [Circular Import Error](#scenario-5-circular-import-error)
6. [Custom Context File Name](#scenario-6-custom-context-file-name)
7. [Backward Compatibility](#scenario-7-backward-compatibility)
8. [Mixed Context Sources](#scenario-8-mixed-context-sources)
9. [Missing Import File](#scenario-9-missing-import-file)
10. [Empty Context File](#scenario-10-empty-context-file)
11. [Large Context File](#scenario-11-large-context-file)
12. [Context with Special Characters](#scenario-12-context-with-special-characters)
13. [Relative Path Imports](#scenario-13-relative-path-imports)
14. [Absolute Path Imports](#scenario-14-absolute-path-imports)
15. [Context File Modification](#scenario-15-context-file-modification)

---

## Scenario 1: Basic Hierarchical Loading

**Priority**: High  
**Type**: Functional  
**Status**: ✅ Implemented

**Description**: Verify that context files are loaded from global and project locations with correct precedence.

**Setup**:
- Global: `~/.radium/GEMINI.md` with "Global context"
- Project: `GEMINI.md` with "Project context"

**Execution**: Run `rad step code-agent "test prompt"`

**Expected**: Both contexts appear, project context after global (higher precedence)

**Validation**: Check agent output includes both contexts in correct order

---

## Scenario 2: Subdirectory Context Override

**Priority**: High  
**Type**: Functional  
**Status**: ✅ Implemented

**Description**: Verify that subdirectory context files override project context.

**Setup**:
- Project: `GEMINI.md` with "Project context"
- Subdirectory: `src/api/GEMINI.md` with "API-specific context"

**Execution**: Navigate to `src/api/`, run agent command

**Expected**: All three contexts (global, project, subdirectory) with subdirectory last

**Validation**: Verify subdirectory context appears last (highest precedence)

---

## Scenario 3: Simple Import

**Priority**: High  
**Type**: Functional  
**Status**: ✅ Implemented

**Description**: Verify that `@file.md` import syntax works.

**Setup**:
- `shared.md` with common guidelines
- `GEMINI.md` with `@shared.md` import

**Execution**: Run agent command

**Expected**: Both files' content merged

**Validation**: Verify imported content is included

---

## Scenario 4: Nested Imports

**Priority**: Medium  
**Type**: Functional  
**Status**: ✅ Implemented

**Description**: Verify that imports can be nested (imported files can import other files).

**Setup**:
- `base.md` → `common.md` → `project.md` import chain

**Execution**: Load `project.md`

**Expected**: All three files' content merged correctly

**Validation**: Verify all content appears in correct order

---

## Scenario 5: Circular Import Error

**Priority**: High  
**Type**: Error Handling  
**Status**: ✅ Implemented

**Description**: Verify that circular imports are detected and reported clearly.

**Setup**:
- `A.md` imports `B.md`
- `B.md` imports `A.md`

**Execution**: Load `A.md`

**Expected**: Clear error message about circular import

**Validation**: Verify error identifies the circular dependency

---

## Scenario 6: Custom Context File Name

**Priority**: Medium  
**Type**: Configuration  
**Status**: ✅ Implemented

**Description**: Verify that custom context file names work via configuration.

**Setup**:
- Configure `CONTEXT.md` in `.radium/config.toml`
- Create `CONTEXT.md` files

**Execution**: Run agent command

**Expected**: `CONTEXT.md` loaded instead of `GEMINI.md`

**Validation**: Verify custom name is used, default is ignored

---

## Scenario 7: Backward Compatibility

**Priority**: High  
**Type**: Compatibility  
**Status**: ✅ Implemented

**Description**: Verify that system works without GEMINI.md files.

**Setup**:
- Remove all `GEMINI.md` files
- Keep `architecture.md` and other context sources

**Execution**: Run agent command

**Expected**: System works normally with existing context sources

**Validation**: Verify no errors, existing functionality preserved

---

## Scenario 8: Mixed Context Sources

**Priority**: High  
**Type**: Integration  
**Status**: ✅ Implemented

**Description**: Verify that all 7 context sources work together correctly.

**Setup**:
- GEMINI.md, architecture.md, plan, memory, learning, skillbook, file injection

**Execution**: Run agent command with full context

**Expected**: All 7 sources in correct order

**Validation**: Verify ordering: hierarchical → plan → architecture → memory → learning → skillbook → injections

---

## Scenario 9: Missing Import File

**Priority**: Medium  
**Type**: Error Handling  
**Status**: ✅ Implemented

**Description**: Verify helpful error message for missing import files.

**Setup**:
- `GEMINI.md` imports `nonexistent.md`

**Execution**: Run agent command

**Expected**: Clear error message about missing file

**Validation**: Verify error is helpful and actionable

---

## Scenario 10: Empty Context File

**Priority**: Low  
**Type**: Edge Case  
**Status**: ✅ Implemented

**Description**: Verify that empty context files are handled gracefully.

**Setup**:
- Create empty `GEMINI.md`

**Execution**: Run agent command

**Expected**: No errors, graceful handling

**Validation**: Verify no errors occur

---

## Scenario 11: Large Context File

**Priority**: Medium  
**Type**: Performance  
**Status**: ✅ Implemented

**Description**: Verify performance with large context files (10KB+).

**Setup**:
- Create large `GEMINI.md` (10KB+)

**Execution**: Run agent command, measure time

**Expected**: Performance acceptable (< 2 seconds)

**Validation**: Verify reasonable load time

---

## Scenario 12: Context with Special Characters

**Priority**: Medium  
**Type**: Edge Case  
**Status**: ✅ Implemented

**Description**: Verify that markdown, code blocks, and special characters are preserved.

**Setup**:
- `GEMINI.md` with markdown, code blocks, special chars, unicode

**Execution**: Run agent command

**Expected**: All content preserved correctly

**Validation**: Verify content integrity

---

## Scenario 13: Relative Path Imports

**Priority**: Medium  
**Type**: Functional  
**Status**: ✅ Implemented

**Description**: Verify that relative path imports work correctly.

**Setup**:
- Import with `./` or `../` paths

**Execution**: Load context with relative imports

**Expected**: Paths resolved correctly

**Validation**: Verify path resolution works

---

## Scenario 14: Absolute Path Imports

**Priority**: Medium  
**Type**: Functional  
**Status**: ✅ Implemented

**Description**: Verify that workspace-relative absolute paths work.

**Setup**:
- Import with `/shared/file.md` (workspace-relative)

**Execution**: Load context with absolute imports

**Expected**: Paths resolved from workspace root

**Validation**: Verify absolute path resolution

---

## Scenario 15: Context File Modification

**Priority**: High  
**Type**: Caching  
**Status**: ✅ Implemented

**Description**: Verify that cache invalidation works when files are modified.

**Setup**:
- Load context, modify `GEMINI.md`, load again

**Execution**: Two agent commands with file modification between

**Expected**: Updated content used (cache invalidated)

**Validation**: Verify cache invalidation works

---

## Test Execution Summary

**Total Scenarios**: 15  
**Implemented**: 15  
**Passing**: 15 (when executed)  
**Failing**: 0

## Notes

- All scenarios are covered by automated tests
- Manual testing validates user experience
- Performance scenarios may vary by system
- Error scenarios verify helpful error messages

