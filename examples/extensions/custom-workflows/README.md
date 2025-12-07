# Custom Workflows Extension

A collection of reusable workflow templates for common development and deployment tasks.

## Installation

```bash
rad extension install ./examples/extensions/custom-workflows
```

## Components

### Workflow Templates

This extension provides workflow templates in the `templates/` directory:

- **code-review-workflow** - Automated code review workflow using specialized review agents
- **deployment-workflow** - Standard deployment workflow with validation and rollback

## Usage

After installation, workflow templates will be discoverable by the workflow system:

```bash
# List available workflow templates
rad workflows list

# Execute a workflow template
rad workflows execute code-review-workflow --language rust --file_path src/
```

## Workflow Templates

### Code Review Workflow

Automated code review using language-specific reviewers.

**Parameters:**
- `language` (required): Programming language (rust, typescript, python)
- `file_path` (required): Path to file or directory to review

**Example:**
```bash
rad workflows execute code-review-workflow \
  --language typescript \
  --file_path src/components/
```

### Deployment Workflow

Standard deployment workflow with validation steps.

**Parameters:**
- `environment` (required, default: staging): Deployment environment
- `version` (required): Version tag to deploy

**Example:**
```bash
rad workflows execute deployment-workflow \
  --environment production \
  --version v1.2.3
```

## Workflow Structure

Workflows are defined as JSON files with:
- `name`: Workflow identifier
- `description`: Human-readable description
- `parameters`: Input parameters with types and validation
- `steps`: Ordered list of workflow steps

## See Also

- [Extension System Guide](../../../docs/extensions/README.md)
- [Workflow Documentation](../../../docs/user-guide/orchestration.md)

