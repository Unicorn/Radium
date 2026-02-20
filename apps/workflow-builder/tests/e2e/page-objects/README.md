# Page Object Models

This directory contains Page Object Model (POM) implementations for E2E tests.

## Overview

Page Objects provide a clean abstraction layer between test code and the UI implementation. They encapsulate:
- Element selectors (locators)
- User interactions (clicks, typing, etc.)
- Assertions and verifications
- Common workflows

## Benefits

1. **Maintainability**: When UI changes, update only the page object
2. **Reusability**: Common actions shared across multiple tests
3. **Readability**: Tests read like user stories
4. **Type Safety**: TypeScript provides autocompletion and type checking

## Available Page Objects

### WorkflowBuilderPage

Handles all interactions with the workflow builder interface.

**Key Methods**:
- `createWorkflow(config)` - Create new workflow via form
- `openBuilder(workflowId)` - Navigate to workflow builder
- `addActivityNode(config)` - Add and configure activity node
- `deleteNode(nodeId)` - Remove node from canvas
- `connectNodes(source, target)` - Connect two nodes
- `save()` - Save workflow
- `compile()` - Compile workflow and open code viewer
- `getGeneratedCode(tab)` - Extract generated code
- `buildAndRun(input)` - Execute workflow
- `waitForExecution(timeout)` - Wait for execution completion

**Usage Example**:
```typescript
import { WorkflowBuilderPage } from '../page-objects/WorkflowBuilderPage';

test('create workflow', async ({ page }) => {
  const builder = new WorkflowBuilderPage(page);

  // Create workflow
  const workflowId = await builder.createWorkflow({
    name: 'My Workflow',
    useDefaultProject: true,
  });

  // Open builder
  await builder.openBuilder(workflowId);

  // Add activity
  await builder.addActivityNode({
    name: 'myActivity',
    timeout: '5 minutes',
  });

  // Compile and execute
  await builder.save();
  await builder.compile();
  await builder.buildAndRun({ input: 'data' });

  // Wait for completion
  const status = await builder.waitForExecution(30000);
  expect(status).toBe('completed');
});
```

### ExecutionMonitorPage

Handles execution history and statistics monitoring.

**Key Methods**:
- `navigateToExecutionHistory(workflowId)` - Open execution history tab
- `navigateToStatistics(workflowId)` - Open statistics tab
- `getExecutionList()` - Retrieve execution summary list
- `viewExecutionDetails(executionId)` - Open execution detail view
- `getExecutionStatus()` - Get status from detail view
- `getExecutionOutput()` - Get execution output/result
- `getStatistics()` - Retrieve workflow statistics
- `backToHistory()` - Navigate back from detail view

**Usage Example**:
```typescript
import { ExecutionMonitorPage } from '../page-objects/ExecutionMonitorPage';

test('view execution history', async ({ page }) => {
  const monitor = new ExecutionMonitorPage(page);

  // Navigate to history
  await monitor.navigateToExecutionHistory(workflowId);

  // Get executions
  const executions = await monitor.getExecutionList();
  expect(executions.length).toBeGreaterThan(0);

  // View details
  await monitor.viewExecutionDetails(executions[0].id);

  const status = await monitor.getExecutionStatus();
  const output = await monitor.getExecutionOutput();

  expect(status).toBe('completed');
  expect(output).toBeTruthy();
});
```

## Best Practices

### 1. Keep Page Objects UI-Focused

Page objects should only interact with the UI, not contain test assertions or business logic.

**Good**:
```typescript
async addActivityNode(config: ActivityNodeConfig): Promise<string> {
  await this.componentPalette.click();
  // ... add node logic
  return nodeId;
}
```

**Avoid**:
```typescript
async addActivityNode(config: ActivityNodeConfig): Promise<void> {
  await this.componentPalette.click();
  // ... add node logic
  expect(nodeCount).toBe(1); // Don't put assertions in page objects
}
```

### 2. Return Meaningful Data

Methods should return data needed for test assertions:

```typescript
// Good - returns status for test to assert
async waitForExecution(): Promise<'completed' | 'failed'> {
  // wait logic
  return status;
}

// Less useful - no return value
async waitForExecution(): Promise<void> {
  // wait logic
}
```

### 3. Use Descriptive Method Names

Method names should clearly describe the user action:

```typescript
// Good
await builder.createWorkflow({ name: 'Test' });
await builder.addActivityNode({ name: 'activity' });

// Avoid
await builder.create({ name: 'Test' });
await builder.add({ name: 'activity' });
```

### 4. Handle Optional Elements

Use conditional checks for optional UI elements:

```typescript
const descriptionField = this.page.getByLabel(/description/i);
if (await descriptionField.count() > 0) {
  await descriptionField.fill(config.description);
}
```

### 5. Provide Flexible Locators

Use `.or()` to handle multiple possible selectors:

```typescript
this.executionStatus = page
  .locator('[data-testid="execution-status"]')
  .or(page.locator('text=/completed|failed/i'));
```

### 6. Use TypeScript Interfaces

Define clear interfaces for method parameters:

```typescript
export interface ActivityNodeConfig {
  name: string;
  timeout?: string;
  retryPolicy?: {
    maximumAttempts?: number;
    initialInterval?: string;
    backoffCoefficient?: number;
  };
}
```

## Locator Strategy

### Priority Order

1. **Test ID** (highest priority)
   ```typescript
   page.locator('[data-testid="execution-panel"]')
   ```

2. **Role with accessible name**
   ```typescript
   page.getByRole('button', { name: /save/i })
   ```

3. **Label**
   ```typescript
   page.getByLabel(/workflow name/i)
   ```

4. **Text content**
   ```typescript
   page.getByText(/generated typescript code/i)
   ```

5. **CSS selectors** (lowest priority)
   ```typescript
   page.locator('.react-flow')
   ```

### Avoid

- XPath selectors (brittle)
- Complex CSS chains (fragile)
- Index-based selection without data-testid

## Error Handling

### Graceful Fallbacks

```typescript
async getExecutionResult(): Promise<any> {
  // Check if element exists
  if (await this.outputLocator.count() === 0) {
    return null;
  }

  const text = await this.outputLocator.textContent();

  // Try to parse JSON, fallback to raw text
  try {
    return JSON.parse(text);
  } catch {
    return text;
  }
}
```

### Timeout Configuration

Provide timeout parameters for long-running operations:

```typescript
async waitForExecution(timeout: number = 30000): Promise<Status> {
  await expect(this.statusLocator).toHaveText(/completed|failed/i, { timeout });
  // ...
}
```

## Adding New Page Objects

When adding a new page object:

1. Create file: `NewFeaturePage.ts`
2. Export class extending base pattern
3. Define locators in constructor
4. Implement user action methods
5. Document with JSDoc comments
6. Add usage example to this README

**Template**:

```typescript
/**
 * NewFeature Page Object Model
 *
 * Brief description of what this page does.
 */

import type { Page, Locator } from '@playwright/test';
import { expect } from '@playwright/test';

export interface NewFeatureConfig {
  // Configuration options
}

export class NewFeaturePage {
  readonly page: Page;
  readonly baseUrl: string;

  // Locators
  readonly mainElement: Locator;

  constructor(page: Page, baseUrl: string = 'http://localhost:3010') {
    this.page = page;
    this.baseUrl = baseUrl;

    this.mainElement = page.locator('[data-testid="main"]');
  }

  /**
   * Navigate to feature page
   */
  async navigateTo(): Promise<void> {
    await this.page.goto(`${this.baseUrl}/feature`);
    await this.page.waitForLoadState('networkidle');
  }

  /**
   * Perform key action
   */
  async performAction(config: NewFeatureConfig): Promise<string> {
    // Implementation
    return 'result';
  }
}
```

## Testing Page Objects

Page objects themselves don't need unit tests, but should be validated through E2E tests that use them.

If a page object method is complex, consider:
1. Breaking it into smaller private methods
2. Adding JSDoc with examples
3. Ensuring it's used in at least one E2E test

## Updating Page Objects

When UI changes require page object updates:

1. **Update locators** in constructor
2. **Run affected tests** to verify changes
3. **Update documentation** if method signatures change
4. **Commit page object changes** separately from test changes

## Resources

- [Playwright Page Object Pattern](https://playwright.dev/docs/pom)
- [Playwright Locators](https://playwright.dev/docs/locators)
- [Playwright Best Practices](https://playwright.dev/docs/best-practices)
- [E2E Testing Documentation](../../docs/testing/e2e-workflow-tests.md)

## Questions?

For questions or issues:
1. Check [E2E Testing Documentation](../../docs/testing/e2e-workflow-tests.md)
2. Review existing page object implementations
3. Consult team members familiar with Playwright
