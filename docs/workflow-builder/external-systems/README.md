# External Systems Documentation

This directory documents the external systems that the workflow builder integrates with.

## Systems Overview

| System | Purpose | Integration Type |
|--------|---------|------------------|
| Temporal | Workflow orchestration | Code generation |
| Kong | API Gateway | Configuration |
| Supabase | Database & Auth | Data storage |
| Redis | Caching & State | Runtime storage |

## Directory Structure

```
external-systems/
  README.md           # This file
  temporal/           # Temporal integration docs
  kong/               # Kong API Gateway docs
  supabase/           # Supabase database docs
  redis/              # Redis caching docs
```

## Quick Reference

### Temporal
- **What:** Workflow orchestration engine
- **How:** Generated TypeScript uses Temporal SDK
- **Docs:** `temporal/README.md`

### Kong
- **What:** API Gateway for routing and plugins
- **How:** Components generate Kong plugin configs
- **Docs:** `kong/README.md`

### Supabase
- **What:** PostgreSQL database with auth
- **How:** Workflows stored in database tables
- **Docs:** `supabase/README.md`

### Redis
- **What:** In-memory data store
- **How:** State variables can use Redis storage
- **Docs:** `redis/README.md`
