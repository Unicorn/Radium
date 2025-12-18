# End-to-End Test Suite Documentation

This directory contains comprehensive end-to-end (E2E) tests for the Workflow Builder application, including the complete workflow execution lifecycle.

## Test Coverage

### Workflow Execution Tests (`workflow-execution.spec.ts`)

Complete integration tests covering the entire workflow lifecycle:

1. **Create Workflow** - Create a new workflow through the UI
2. **Build Visually** - Add nodes and configure workflow in the canvas
3. **Compile** - Compile workflow to TypeScript code
4. **Execute** - Run workflow through Temporal worker
5. **Monitor** - Track execution status and results
6. **View History** - Access execution history and statistics

### Test Suites

#### 1. Complete E2E Flow
- ✅ Create → Build → Compile → Execute → Monitor
- ✅ Workflow execution failure handling
- ✅ Validation before execution
- ✅ Compilation progress and error handling
- ✅ Retry failed executions
- ✅ Display execution statistics

#### 2. Workflow Builder UI
- ✅ Undo/redo operations
- ✅ Keyboard shortcuts (Delete, Cmd+Z, Cmd+Shift+Z)
- ✅ Auto-save functionality

#### 3. Workflow Compilation
- ✅ TypeScript code generation
- ✅ Proper imports and exports
- ✅ All required code files (workflow, activities, worker)

## Prerequisites

Before running E2E tests, ensure the following services are running:

### 1. Temporal Server

```bash
# From project root
docker-compose up temporal
```

Or if using Temporal Dev Server:
```bash
temporal server start-dev
```

### 2. Database

PostgreSQL database must be running with migrations applied:

```bash
# From workflow-builder directory
yarn db:push
```

Seed test user:
```bash
yarn seed:auth
```

### 3. Next.js Development Server

```bash
# From workflow-builder directory
yarn dev

# Should be running on http://localhost:3010
```

### 4. Temporal Worker (Optional but Recommended)

For full execution testing, run the workflow worker:

```bash
# From workflow-builder directory
yarn worker:dev
```

## Running Tests

### Run All E2E Tests

```bash
# From workflow-builder directory
yarn test:e2e
```

### Run Specific Test File

```bash
yarn playwright test workflow-execution.spec.ts
```

### Run with UI Mode (Interactive)

```bash
yarn playwright test --ui
```

### Run in Headed Mode (See Browser)

```bash
yarn playwright test --headed
```

### Run Specific Test Suite

```bash
yarn playwright test -g "complete workflow lifecycle"
```

### Debug Mode

```bash
yarn playwright test --debug
```

## Test Structure

### Test Helpers (`helpers/`)

- **`auth.ts`** - Authentication utilities (sign in, sign out, session management)
- **`workflow.ts`** - Workflow-specific helpers (create, build, compile, execute)
- **`console-errors.ts`** - Console error capture and reporting

### Key Helper Functions

#### Workflow Creation
```typescript
const workflowId = await createWorkflow(page, {
  name: 'My Workflow',
  description: 'Test workflow',
  useDefaultProject: true,
});
```

#### Workflow Building
```typescript
await openWorkflowBuilder(page, workflowId);

await addActivityNode(page, {
  name: 'sampleActivity',
  timeout: '5 minutes',
  retryPolicy: 'exponential',
});

await saveWorkflow(page);
```

#### Workflow Compilation
```typescript
await compileWorkflow(page);

const workflowCode = await getGeneratedCode(page, 'workflow');
expect(workflowCode).toContain('proxyActivities');

await closeCodeViewer(page);
```

#### Workflow Execution
```typescript
const executionId = await executeWorkflow(page, {
  message: 'Hello from test',
});

const status = await waitForExecutionComplete(page, 30000);
expect(status).toBe('completed');

const result = await getExecutionResult(page);
console.log('Execution result:', result);
```

## Data Test IDs

The following `data-testid` attributes are available for reliable test selectors:

### Toolbar Components
- `save-workflow-button` - Save workflow button
- `compile-workflow-button` - Compile workflow button
- `deploy-workflow-button` - Deploy workflow button
- `undo-button` - Undo button
- `redo-button` - Redo button

### Code Viewer Modal
- `code-viewer-modal` - Main modal container
- `code-tab-workflow` - Workflow code tab
- `code-tab-activities` - Activities code tab
- `code-tab-worker` - Worker code tab
- `code-viewer-content` - Code display area
- `code-content` - Actual code content
- `copy-code-button` - Copy code button
- `close-code-viewer` - Close modal button

### Execution Panel
- `execution-panel` - Main execution panel
- `execution-status` - Status badge (Building, Running, Completed, Failed)
- `execution-duration` - Execution duration display
- `execution-error` - Error message display
- `execution-output` - Result/output display
- `build-run-workflow-button` - Build & Run button
- `retry-execution-button` - Retry/Run Again button

## Configuration

### Playwright Configuration (`playwright.config.ts`)

Key settings:
- **Base URL**: `http://localhost:3010`
- **Workers**: 1 (sequential execution)
- **Timeout**: 30s per test
- **Headless**: true (can override with `--headed`)
- **Storage State**: Authentication persisted in `playwright/.auth/user.json`

### Environment Variables

Create `.env.local` in workflow-builder directory:

```env
BASE_URL=http://localhost:3010
DATABASE_URL=postgresql://user:password@localhost:5432/workflow_builder
TEMPORAL_ADDRESS=localhost:7233
```

## Test Authentication

Tests use authenticated state by default:

1. **Setup Phase** (`auth.setup.ts`) - Runs once before all tests
   - Authenticates test user
   - Saves session to `playwright/.auth/user.json`

2. **Test Phase** - All tests reuse authenticated session
   - No need to sign in for each test
   - Faster test execution

### Test User Credentials

Default test user (created by `seed:auth`):
- Email: `test@example.com`
- Password: `TestPassword123!`

## Debugging Tests

### View Test Reports

After running tests:
```bash
yarn playwright show-report
```

### Screenshots and Videos

Failed tests automatically capture:
- Screenshots (`test-results/`)
- Videos (if configured)
- Traces (`test-results/`)

### View Trace

```bash
yarn playwright show-trace test-results/path-to-trace.zip
```

## Common Issues

### 1. Temporal Server Not Running

**Error**: "Connection refused to localhost:7233"

**Solution**:
```bash
docker-compose up temporal
```

### 2. Database Not Seeded

**Error**: "Invalid credentials" or "User not found"

**Solution**:
```bash
yarn seed:auth
```

### 3. Dev Server Not Running

**Error**: "net::ERR_CONNECTION_REFUSED"

**Solution**:
```bash
yarn dev
```

### 4. Port Already in Use

**Error**: "Port 3010 already in use"

**Solution**:
```bash
# Kill process on port 3010
lsof -ti:3010 | xargs kill -9

# Or change BASE_URL in .env.local
```

### 5. Authentication Failures

**Error**: "Authentication failed" or "Session expired"

**Solution**:
```bash
# Delete auth storage and re-run setup
rm -rf playwright/.auth
yarn playwright test auth.setup.ts
```

## Best Practices

### 1. Use Test Steps

Break complex tests into logical steps:
```typescript
test('complex workflow', async ({ page }) => {
  await test.step('Create workflow', async () => {
    // ...
  });

  await test.step('Build workflow', async () => {
    // ...
  });
});
```

### 2. Wait Strategies

Prefer explicit waits over timeouts:
```typescript
// Good
await expect(element).toBeVisible({ timeout: 5000 });

// Avoid
await page.waitForTimeout(5000);
```

### 3. Cleanup

Clean up test data when possible:
```typescript
test.afterEach(async ({ page }) => {
  // Delete test workflow if needed
  await deleteWorkflow(page, workflowId);
});
```

### 4. Unique Test Data

Use timestamps to avoid conflicts:
```typescript
const workflowName = `Test Workflow ${Date.now()}`;
```

## CI/CD Integration

### GitHub Actions Example

```yaml
name: E2E Tests

on: [push, pull_request]

jobs:
  test:
    runs-on: ubuntu-latest
    steps:
      - uses: actions/checkout@v3

      - name: Setup Node
        uses: actions/setup-node@v3
        with:
          node-version: '18'

      - name: Install dependencies
        run: yarn install

      - name: Start Temporal
        run: docker-compose up -d temporal

      - name: Setup database
        run: |
          yarn db:push
          yarn seed:auth

      - name: Run E2E tests
        run: yarn test:e2e

      - name: Upload test results
        if: always()
        uses: actions/upload-artifact@v3
        with:
          name: playwright-report
          path: playwright-report/
```

## Performance Optimization

### 1. Parallel Execution

Currently disabled for stability. To enable:
```typescript
// playwright.config.ts
workers: process.env.CI ? 2 : 4,
fullyParallel: true,
```

### 2. Test Isolation

Each test creates its own workflow to avoid conflicts.

### 3. Reuse Authentication

Authentication state is cached and reused across tests.

## Test Metrics

Target metrics:
- **Total tests**: 15+
- **Test execution time**: < 5 minutes
- **Success rate**: > 95%
- **Code coverage**: > 80% of workflow execution paths

## Contributing

When adding new E2E tests:

1. **Follow naming conventions**: `feature-name.spec.ts`
2. **Add data-testid attributes**: For new UI components
3. **Create helper functions**: For reusable operations
4. **Document complex tests**: Add comments explaining test logic
5. **Update this README**: Add new test coverage details

## Support

For issues or questions:
- Check existing test patterns in `workflow-execution.spec.ts`
- Review Playwright documentation: https://playwright.dev
- Consult test helpers in `helpers/` directory
