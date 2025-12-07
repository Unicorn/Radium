# Context Files Examples

This directory contains example context files demonstrating different patterns and use cases for GEMINI.md files in Radium.

## Examples

### Basic Project Context

**File**: `basic-project.md`

A simple, self-contained context file with project guidelines and standards. This is a good starting point for new projects.

### Context with Imports

**Files**: `with-imports.md`, `coding-standards.md`, `architecture-notes.md`

Demonstrates how to organize context across multiple files using the `@file.md` import syntax. The main context file imports supporting files for better organization.

### Subdirectory Context

**File**: `subdirectory-example.md`

Shows how to create directory-specific context files that override or extend project-level context. This is useful for modules or features that need specialized instructions.

## Usage

To use these examples:

1. Copy the relevant example file(s) to your project
2. Rename to `GEMINI.md` if using as a project root context file
3. Customize the content to match your project's needs
4. If using imports, ensure imported files are in the correct relative locations

## Templates

See the `templates/` directory for starter templates that can be used when initializing new workspaces with `rad init --with-context`.

