# Documentation Migration Mapping

This document maps existing `/docs/` categories to the new Docusaurus structure.

## Category Mapping

| Source Category | Target Section | Notes |
|----------------|----------------|-------|
| `adr/` | `developer-guide/adr/` | Architecture Decision Records |
| `api/` | `api/` | API documentation (keep as-is) |
| `architecture/` | `developer-guide/architecture/` | Architecture docs |
| `cli/` | `cli/` | CLI reference (keep structure) |
| `design/` | `developer-guide/design/` | Design documents |
| `developer-guide/` | `developer-guide/` | Already matches! |
| `development/` | `developer-guide/development/` | Development process |
| `editor-integration/` | `features/editor-integration/` | Editor integration feature |
| `examples/` | `examples/` | Already matches! |
| `extensions/` | `extensions/` | Already matches! |
| `features/` | `features/` | Already matches! |
| `guides/` | `user-guide/` | User-facing guides |
| `hooks/` | `hooks/` | Already matches! |
| `mcp/` | `mcp/` | Already matches! |
| `monitoring/` | `features/monitoring/` | Monitoring & telemetry |
| `planning/` | `features/planning/` | Planning features |
| `requirements/` | **SKIP** | Internal project tracking |
| `security/` | `features/security/` | Security features |
| `self-hosted-models/` | `self-hosted/` | Self-hosted models |
| `temp/` | **SKIP** | Temporary files |
| `testing/` | `developer-guide/testing/` | Testing documentation |
| `user-guide/` | `user-guide/` | Already matches! |
| `yolo-mode/` | `features/yolo-mode/` | YOLO mode feature |
| Root files | `getting-started/` or appropriate | Standalone docs at root |

## Migration Statistics

- **Total categories**: 23
- **Direct matches**: 8 (api, cli, developer-guide, examples, extensions, features, hooks, mcp, user-guide)
- **Need reorganization**: 13
- **Skip**: 2 (requirements, temp)

## Files to Migrate

Total: **160 markdown files**

### Priority 1: Core Documentation (Already Well-Organized)
- `user-guide/` - User-facing documentation
- `features/` - Feature documentation
- `cli/` - CLI reference
- `examples/` - Examples
- `developer-guide/` - Developer docs

### Priority 2: Reorganize Into Sections
- `guides/` → `user-guide/`
- `architecture/` → `developer-guide/architecture/`
- `testing/` → `developer-guide/testing/`
- `development/` → `developer-guide/development/`
- `design/` → `developer-guide/design/`
- `adr/` → `developer-guide/adr/`

### Priority 3: Features Subcategories
- `monitoring/` → `features/monitoring/`
- `planning/` → `features/planning/`
- `security/` → `features/security/`
- `editor-integration/` → `features/editor-integration/`
- `yolo-mode/` → `features/yolo-mode/`

### Priority 4: Rename to Match Convention
- `self-hosted-models/` → `self-hosted/`
- `mcp/` → `mcp/` (keep as-is)
- `api/` → `api/` (keep as-is)

## Migration Approach

1. **Copy well-organized categories as-is**: user-guide, features, cli, examples, developer-guide, extensions, hooks
2. **Reorganize into target directories**: Merge guides into user-guide, architecture/testing/development/design/adr into developer-guide
3. **Create feature subdirectories**: Move monitoring, planning, security, editor-integration, yolo-mode under features/
4. **Rename**: self-hosted-models → self-hosted
5. **Skip**: requirements, temp

## Frontmatter Template

```yaml
---
id: <filename-without-extension>
title: <Human-Readable Title>
sidebar_label: <Short Label>
sidebar_position: <number>
---
```

## Link Rewriting Rules

- Internal links: `/docs/...` → relative links
- Markdown links: `[text](../other.md)` → `[text](../other.md)` (verify paths)
- GitHub links: Keep absolute for external references
