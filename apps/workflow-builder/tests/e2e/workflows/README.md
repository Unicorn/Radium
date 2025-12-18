# End-to-End Workflow Tests

This directory contains comprehensive E2E tests for the complete workflow lifecycle.

## Test Suite: `end-to-end.spec.ts`

### Coverage Summary

The test suite covers all acceptance criteria from **M1-T080** with 10 comprehensive tests:

#### Core E2E Tests (7 Required Tests)

1. **Single Activity Workflow** ✓
   - Create workflow with 1 activity
   - Deploy and execute
   - Verify successful completion

2. **Multi-Activity Workflow** ✓
   - Create workflow with 5 activities
   - Deploy and execute
   - Verify all activities run

3. **Timeout Configuration** ✓
   - Create workflow with timeout settings
   - Verify timeout in generated code
   - Execute with timeout applied

4. **Retry Policy** ✓
   - Create workflow with retry configuration
   - Verify retry policy in generated code
   - Execute with retry behavior

5. **Code Viewing & Results** ✓
   - View generated code before execution
   - Execute workflow
   - View execution results in history

6. **Invalid Workflow Validation** ✓
   - Attempt to deploy invalid workflow
   - Verify validation error displayed
   - Ensure deployment is blocked

7. **Node Deletion** ✓
   - Create workflow with multiple nodes
   - Delete node from canvas
   - Deploy and execute modified workflow

#### Additional Tests (3 Bonus Tests)

8. **Keyboard Shortcuts** ✓
   - Delete node using keyboard shortcut

9. **Auto-Save** ✓
   - Verify workflow changes auto-save
   - Persist across page reloads

10. **Execution Monitoring** ✓
    - View workflow statistics
    - Track execution metrics

## Quick Start

### Prerequisites

Ensure all services are running:

```bash
# 1. Start Temporal server
docker-compose up -d temporal

# 2. Start worker service
cd packages/workflow-worker-service
npm run dev

# 3. Start dev server
cd packages/workflow-builder
npm run dev
```

### Run Tests

```bash
# Run all E2E workflow tests
npx playwright test tests/e2e/workflows/end-to-end.spec.ts

# Run specific test
npx playwright test --grep "should create workflow with 1 activity"

# Run in headed mode (watch browser)
npx playwright test tests/e2e/workflows/end-to-end.spec.ts --headed

# Debug mode (step through tests)
npx playwright test tests/e2e/workflows/end-to-end.spec.ts --debug
```

### View Results

```bash
# Open HTML report
npx playwright show-report

# View trace for failed test
npx playwright show-trace test-results/[test-name]/trace.zip
```

## Test Structure

### Using Page Objects

All tests use the page object pattern for maintainability:

```typescript
import { WorkflowBuilderPage } from '../page-objects/WorkflowBuilderPage';
import { ExecutionMonitorPage } from '../page-objects/ExecutionMonitorPage';

test('example test', async ({ page }) => {
  const builder = new WorkflowBuilderPage(page);
  const monitor = new ExecutionMonitorPage(page);

  // Create workflow
  const workflowId = await builder.createWorkflow({
    name: 'Test Workflow',
    useDefaultProject: true,
  });

  // Build and execute
  await builder.openBuilder(workflowId);
  await builder.addActivityNode({ name: 'activity1' });
  await builder.save();
  await builder.buildAndRun({});

  // Monitor results
  await monitor.navigateToExecutionHistory(workflowId);
  const executions = await monitor.getExecutionList();
});
```

## Test Execution Flow

Each test follows this pattern:

1. **Setup** (beforeEach)
   - Initialize page objects
   - Setup error capture
   - Authenticated state loaded from `auth.setup.ts`

2. **Test Steps**
   - Create workflow through UI
   - Build workflow visually with nodes
   - Configure node properties
   - Compile workflow
   - Execute workflow
   - Verify results

3. **Cleanup** (afterEach)
   - Track created workflow IDs
   - Automatic cleanup on test completion

## Test Data

Tests use timestamped names to avoid conflicts:

```typescript
const workflowName = `Test Workflow ${Date.now()}`;
```

This ensures:
- No naming collisions between test runs
- Tests can run in parallel (with workers: 1 for stability)
- Easy identification of test-created workflows

## Debugging

### View Browser During Test

```bash
npx playwright test --headed --grep "test name"
```

### Slow Down Execution

```typescript
test.use({ launchOptions: { slowMo: 1000 } });
```

### Add Debug Breakpoint

```typescript
await page.pause(); // Pauses test execution
```

### Check Console Errors

Tests automatically capture console errors:

```typescript
setupConsoleErrorCapture(page, false); // Logs but doesn't fail
```

## Common Issues

### Test Timeout

If test exceeds 60 seconds:

```typescript
test('long test', async ({ page }) => {
  test.setTimeout(120000); // Increase to 2 minutes
  // test code
});
```

### Element Not Found

1. Run in headed mode to inspect UI
2. Check if element locator changed
3. Update page object locators
4. Add explicit waits:

```typescript
await expect(element).toBeVisible({ timeout: 10000 });
```

### Workflow Execution Fails

1. Check Temporal server is running
2. Verify worker service is active
3. Check worker logs for errors
4. Increase execution timeout:

```typescript
const status = await builder.waitForExecution(60000); // 1 minute
```

### Authentication Errors

Re-run auth setup:

```bash
npx playwright test auth.setup.ts
```

Verify auth file exists:

```bash
ls -la playwright/.auth/user.json
```

## Performance

Expected execution times:

| Test | Duration |
|------|----------|
| Single activity | ~5-10s |
| 5 activities | ~10-15s |
| Timeout config | ~5-10s |
| Retry policy | ~5-15s |
| Code viewing | ~8-12s |
| Invalid workflow | ~3-5s |
| Node deletion | ~10-15s |
| Keyboard shortcuts | ~5-8s |
| Auto-save | ~8-10s |
| Statistics | ~10-15s |

**Total Suite**: ~60-90 seconds

## Continuous Integration

Tests run in CI with:

- Sequential execution (workers: 1)
- 2 retries on failure
- Screenshot capture on failure
- Trace recording on retry
- HTML report generation

See `.github/workflows/e2e-tests.yml` for full CI configuration.

## Test Maintenance

### When UI Changes

1. Update page object locators
2. Run affected tests
3. Fix any failures
4. Commit page object and test changes together

### Adding New Tests

1. Follow existing test structure
2. Use page objects (don't use raw locators)
3. Add test.step() for readability
4. Include proper assertions
5. Document what the test validates

### Best Practices

- ✓ Use page objects
- ✓ Use test.step() for clarity
- ✓ Use explicit waits (not setTimeout)
- ✓ Make tests independent
- ✓ Use descriptive test names
- ✗ Don't use arbitrary timeouts
- ✗ Don't rely on test execution order
- ✗ Don't use raw locators in tests

## Resources

- [Main E2E Testing Documentation](../../../docs/testing/e2e-workflow-tests.md)
- [Page Objects README](../page-objects/README.md)
- [Playwright Documentation](https://playwright.dev)
- [MILESTONE-1-TASKS.md](../../../../../MILESTONE-1-TASKS.md)

## Support

For issues or questions:

1. Check this README
2. Review main E2E documentation
3. Inspect test execution in headed mode
4. Review Playwright traces
5. Consult with team

## Test Status

Current status: ✅ **All 7 required tests implemented + 3 bonus tests**

- TypeScript compilation: ✓ Passing
- Test structure: ✓ Valid
- Page objects: ✓ Complete
- Documentation: ✓ Comprehensive

Ready for execution once services are running!
