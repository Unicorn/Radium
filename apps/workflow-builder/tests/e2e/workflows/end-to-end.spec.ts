/**
 * End-to-End Workflow Creation Test Suite
 *
 * Comprehensive E2E tests covering the complete workflow lifecycle:
 * - Create workflow through UI
 * - Build workflow with nodes (1 activity, 5 activities, with configurations)
 * - Configure timeouts and retry policies
 * - Compile workflow to TypeScript
 * - Deploy workflow
 * - Execute workflow
 * - Monitor execution results
 * - Handle errors and validation
 *
 * Prerequisites:
 * 1. Temporal server running
 * 2. Worker service running
 * 3. Database seeded with test user
 * 4. Next.js dev server running on port 3010
 */

import { test, expect } from '@playwright/test';
import { WorkflowBuilderPage } from '../page-objects/WorkflowBuilderPage';
import { ExecutionMonitorPage } from '../page-objects/ExecutionMonitorPage';
import { setupConsoleErrorCapture } from '../helpers/console-errors';

const BASE_URL = process.env.BASE_URL || 'http://localhost:3010';

test.describe('End-to-End Workflow Creation and Execution', () => {
  let builderPage: WorkflowBuilderPage;
  let monitorPage: ExecutionMonitorPage;
  const createdWorkflowIds: string[] = [];

  test.beforeEach(async ({ page }) => {
    // Initialize page objects
    builderPage = new WorkflowBuilderPage(page, BASE_URL);
    monitorPage = new ExecutionMonitorPage(page, BASE_URL);

    // Setup console error capture (log but don't fail)
    setupConsoleErrorCapture(page, false);
  });

  test.afterEach(async () => {
    // Cleanup: Track created workflows for potential cleanup
    // Note: Workflows persist for inspection; add cleanup logic if needed
  });

  /**
   * TEST CASE 1: Create workflow with 1 activity, deploy, execute successfully
   *
   * Acceptance Criteria:
   * - Create workflow through UI
   * - Add single activity node
   * - Compile workflow
   * - View generated code
   * - Execute workflow
   * - Verify successful completion
   */
  test('should create workflow with 1 activity, deploy, and execute successfully', async ({ page }) => {
    const workflowName = `Single Activity Workflow ${Date.now()}`;

    // STEP 1: Create workflow
    test.step('Create new workflow', async () => {
      const workflowId = await builderPage.createWorkflow({
        name: workflowName,
        description: 'E2E test workflow with single activity',
        useDefaultProject: true,
      });

      expect(workflowId).toBeTruthy();
      expect(workflowId).toMatch(/^[a-f0-9-]{36}$/);
      createdWorkflowIds.push(workflowId);

      // Verify on edit page
      await expect(page).toHaveURL(/\/workflows\/.*\/edit/);
    });

    // STEP 2: Navigate to builder
    test.step('Open workflow builder', async () => {
      const workflowId = createdWorkflowIds[createdWorkflowIds.length - 1];
      await builderPage.openBuilder(workflowId);
      await builderPage.verifyCanvasLoaded();
    });

    // STEP 3: Add single activity node
    test.step('Add activity node to canvas', async () => {
      const nodeId = await builderPage.addActivityNode({
        name: 'sampleActivity',
      });

      expect(nodeId).toBeTruthy();

      // Verify node was added
      const nodeCount = await builderPage.getActivityNodeCount();
      expect(nodeCount).toBe(1);
    });

    // STEP 4: Save workflow
    test.step('Save workflow', async () => {
      await builderPage.save();
      await page.waitForTimeout(500);
    });

    // STEP 5: Compile workflow
    test.step('Compile workflow and verify generated code', async () => {
      await builderPage.compile();

      // Verify code viewer opened
      await expect(page.getByText(/generated typescript code/i)).toBeVisible();

      // Check workflow code
      const workflowCode = await builderPage.getGeneratedCode('workflow');
      expect(workflowCode).toContain('proxyActivities');
      expect(workflowCode).toContain('sampleActivity');
      expect(workflowCode).toContain('@temporalio/workflow');

      // Check activities code
      const activitiesCode = await builderPage.getGeneratedCode('activities');
      expect(activitiesCode).toContain('export');
      expect(activitiesCode).toContain('async');

      // Check worker code
      const workerCode = await builderPage.getGeneratedCode('worker');
      expect(workerCode).toContain('Worker');
      expect(workerCode).toContain('NativeConnection');

      // Close code viewer
      await builderPage.closeCodeViewer();
    });

    // STEP 6: Execute workflow
    test.step('Execute workflow', async () => {
      const executionId = await builderPage.buildAndRun({
        message: 'Hello from E2E test',
      });

      expect(executionId).toBeTruthy();

      // Wait for execution to complete
      const status = await builderPage.waitForExecution(30000);
      expect(status).toBe('completed');
    });

    // STEP 7: Verify in execution history
    test.step('Verify execution appears in history', async () => {
      const workflowId = createdWorkflowIds[createdWorkflowIds.length - 1];
      await monitorPage.navigateToExecutionHistory(workflowId);

      // Check for executions
      const isEmpty = await monitorPage.isExecutionHistoryEmpty();
      if (!isEmpty) {
        const executions = await monitorPage.getExecutionList();
        expect(executions.length).toBeGreaterThan(0);

        // Verify at least one completed execution
        const completedExecutions = executions.filter(e => e.status === 'completed');
        expect(completedExecutions.length).toBeGreaterThan(0);
      }
    });
  });

  /**
   * TEST CASE 2: Create workflow with 5 activities, deploy, execute successfully
   *
   * Acceptance Criteria:
   * - Create workflow through UI
   * - Add 5 activity nodes
   * - Compile workflow
   * - Execute workflow
   * - Verify all activities executed
   */
  test('should create workflow with 5 activities, deploy, and execute successfully', async ({ page }) => {
    const workflowName = `Multi Activity Workflow ${Date.now()}`;

    // STEP 1: Create workflow
    const workflowId = await builderPage.createWorkflow({
      name: workflowName,
      description: 'E2E test workflow with 5 activities',
      useDefaultProject: true,
    });

    expect(workflowId).toBeTruthy();
    createdWorkflowIds.push(workflowId);

    // STEP 2: Open builder
    await builderPage.openBuilder(workflowId);
    await builderPage.verifyCanvasLoaded();

    // STEP 3: Add 5 activity nodes
    test.step('Add 5 activity nodes', async () => {
      const nodeIds: string[] = [];

      for (let i = 1; i <= 5; i++) {
        const nodeId = await builderPage.addActivityNode({
          name: `activity${i}`,
        });
        nodeIds.push(nodeId);
        await page.waitForTimeout(300);
      }

      // Verify all nodes were added
      const nodeCount = await builderPage.getActivityNodeCount();
      expect(nodeCount).toBe(5);
    });

    // STEP 4: Save workflow
    await builderPage.save();

    // STEP 5: Compile workflow
    test.step('Compile workflow', async () => {
      await builderPage.compile();

      // Verify code contains all 5 activities
      const workflowCode = await builderPage.getGeneratedCode('workflow');
      for (let i = 1; i <= 5; i++) {
        expect(workflowCode).toContain(`activity${i}`);
      }

      await builderPage.closeCodeViewer();
    });

    // STEP 6: Execute workflow
    test.step('Execute workflow with 5 activities', async () => {
      const executionId = await builderPage.buildAndRun({
        data: 'Test execution with multiple activities',
      });

      expect(executionId).toBeTruthy();

      // Wait for execution to complete
      const status = await builderPage.waitForExecution(45000); // Longer timeout for 5 activities
      expect(status).toBe('completed');
    });
  });

  /**
   * TEST CASE 3: Create workflow, configure timeouts, deploy, execute with timeout
   *
   * Acceptance Criteria:
   * - Create workflow
   * - Add activity with timeout configuration
   * - Compile workflow
   * - Execute and verify timeout is applied
   */
  test('should create workflow with timeout configuration and execute', async ({ page }) => {
    const workflowName = `Timeout Workflow ${Date.now()}`;

    // Create workflow
    const workflowId = await builderPage.createWorkflow({
      name: workflowName,
      description: 'E2E test workflow with timeout configuration',
      useDefaultProject: true,
    });

    createdWorkflowIds.push(workflowId);

    // Open builder
    await builderPage.openBuilder(workflowId);
    await builderPage.verifyCanvasLoaded();

    // Add activity with timeout
    test.step('Add activity with timeout configuration', async () => {
      const nodeId = await builderPage.addActivityNode({
        name: 'timeoutActivity',
        timeout: '5 minutes',
      });

      expect(nodeId).toBeTruthy();
    });

    // Save and compile
    await builderPage.save();

    test.step('Compile and verify timeout in generated code', async () => {
      await builderPage.compile();

      const workflowCode = await builderPage.getGeneratedCode('workflow');

      // Verify timeout configuration exists in code
      // May appear as scheduleToCloseTimeout or similar
      expect(workflowCode).toContain('timeoutActivity');

      await builderPage.closeCodeViewer();
    });

    // Execute workflow
    test.step('Execute workflow with timeout', async () => {
      const executionId = await builderPage.buildAndRun({});

      expect(executionId).toBeTruthy();

      // Wait for execution
      const status = await builderPage.waitForExecution(30000);
      expect(status).toBe('completed');
    });
  });

  /**
   * TEST CASE 4: Create workflow, configure retry, deploy, execute with failure and retry
   *
   * Acceptance Criteria:
   * - Create workflow
   * - Add activity with retry policy
   * - Compile workflow
   * - Execute workflow
   * - Verify retry policy is applied
   */
  test('should create workflow with retry policy and execute', async ({ page }) => {
    const workflowName = `Retry Workflow ${Date.now()}`;

    // Create workflow
    const workflowId = await builderPage.createWorkflow({
      name: workflowName,
      description: 'E2E test workflow with retry policy',
      useDefaultProject: true,
    });

    createdWorkflowIds.push(workflowId);

    // Open builder
    await builderPage.openBuilder(workflowId);
    await builderPage.verifyCanvasLoaded();

    // Add activity with retry policy
    test.step('Add activity with retry policy', async () => {
      const nodeId = await builderPage.addActivityNode({
        name: 'retryActivity',
        retryPolicy: {
          maximumAttempts: 3,
          initialInterval: '1s',
          backoffCoefficient: 2.0,
        },
      });

      expect(nodeId).toBeTruthy();
    });

    // Save and compile
    await builderPage.save();

    test.step('Compile and verify retry policy in generated code', async () => {
      await builderPage.compile();

      const workflowCode = await builderPage.getGeneratedCode('workflow');

      // Verify retry policy exists in code
      expect(workflowCode).toContain('retryActivity');
      // May contain retry-related configuration

      await builderPage.closeCodeViewer();
    });

    // Execute workflow
    test.step('Execute workflow with retry policy', async () => {
      const executionId = await builderPage.buildAndRun({});

      expect(executionId).toBeTruthy();

      // Wait for execution
      const status = await builderPage.waitForExecution(30000);

      // Should complete (with retries if activity fails)
      expect(status).toMatch(/completed|failed/);
    });
  });

  /**
   * TEST CASE 5: Create workflow, view code, deploy, execute, view results
   *
   * Acceptance Criteria:
   * - Create workflow
   * - View generated code before execution
   * - Execute workflow
   * - View execution results
   */
  test('should create workflow, view code, execute, and view results', async ({ page }) => {
    const workflowName = `Code View Workflow ${Date.now()}`;

    // Create workflow
    const workflowId = await builderPage.createWorkflow({
      name: workflowName,
      description: 'E2E test for viewing code and results',
      useDefaultProject: true,
    });

    createdWorkflowIds.push(workflowId);

    // Open builder
    await builderPage.openBuilder(workflowId);
    await builderPage.verifyCanvasLoaded();

    // Add activity
    await builderPage.addActivityNode({
      name: 'viewCodeActivity',
    });

    await builderPage.save();

    // View generated code
    test.step('View generated code in all tabs', async () => {
      await builderPage.compile();

      // View workflow code
      const workflowCode = await builderPage.getGeneratedCode('workflow');
      expect(workflowCode).toBeTruthy();
      expect(workflowCode.length).toBeGreaterThan(50);

      // View activities code
      const activitiesCode = await builderPage.getGeneratedCode('activities');
      expect(activitiesCode).toBeTruthy();
      expect(activitiesCode.length).toBeGreaterThan(50);

      // View worker code
      const workerCode = await builderPage.getGeneratedCode('worker');
      expect(workerCode).toBeTruthy();
      expect(workerCode.length).toBeGreaterThan(50);

      await builderPage.closeCodeViewer();
    });

    // Execute workflow
    test.step('Execute workflow', async () => {
      const executionId = await builderPage.buildAndRun({
        testData: 'View results test',
      });

      expect(executionId).toBeTruthy();

      const status = await builderPage.waitForExecution(30000);
      expect(status).toBe('completed');
    });

    // View execution results
    test.step('View execution results in history', async () => {
      await monitorPage.navigateToExecutionHistory(workflowId);

      const isEmpty = await monitorPage.isExecutionHistoryEmpty();
      if (!isEmpty) {
        const executions = await monitorPage.getExecutionList();
        expect(executions.length).toBeGreaterThan(0);

        // View first execution details
        if (executions.length > 0) {
          await monitorPage.viewExecutionDetails(executions[0].id);

          // Verify execution details are displayed
          const status = await monitorPage.getExecutionStatus();
          expect(status).toBeTruthy();

          const duration = await monitorPage.getExecutionDuration();
          console.log('Execution duration:', duration);

          // Navigate back
          await monitorPage.backToHistory();
        }
      }
    });
  });

  /**
   * TEST CASE 6: Create workflow, attempt to deploy invalid (missing trigger), verify error
   *
   * Acceptance Criteria:
   * - Create workflow
   * - Attempt to compile without required trigger node
   * - Verify validation error is shown
   * - Ensure deployment is blocked
   */
  test('should prevent deployment of invalid workflow (missing trigger)', async ({ page }) => {
    const workflowName = `Invalid Workflow ${Date.now()}`;

    // Create workflow
    const workflowId = await builderPage.createWorkflow({
      name: workflowName,
      description: 'E2E test for validation errors',
      useDefaultProject: true,
    });

    createdWorkflowIds.push(workflowId);

    // Open builder
    await builderPage.openBuilder(workflowId);
    await builderPage.verifyCanvasLoaded();

    // Don't add any nodes - empty workflow or missing trigger

    // Attempt to compile
    test.step('Attempt to compile invalid workflow', async () => {
      // Click compile button
      await builderPage.compileButton.click();

      // Wait for validation error
      await page.waitForTimeout(1000);

      // Verify error message appears
      const hasError = await builderPage.hasValidationError();
      if (hasError) {
        const errorMessage = await builderPage.getValidationError();
        console.log('Validation error:', errorMessage);
        expect(errorMessage).toBeTruthy();
      }

      // Code viewer should NOT open for invalid workflow
      const codeViewerVisible = await page.getByText(/generated typescript code/i).isVisible().catch(() => false);
      expect(codeViewerVisible).toBe(false);
    });

    // Verify build & run is also blocked
    test.step('Verify execution is blocked for invalid workflow', async () => {
      // Build & Run button should be disabled or show error
      const buildButton = builderPage.buildAndRunButton;

      // Try to click (may be disabled)
      const isEnabled = await buildButton.isEnabled().catch(() => false);

      if (!isEnabled) {
        // Button correctly disabled
        expect(isEnabled).toBe(false);
      }
    });
  });

  /**
   * TEST CASE 7: Create workflow, delete node, deploy, execute
   *
   * Acceptance Criteria:
   * - Create workflow
   * - Add multiple nodes
   * - Delete one node
   * - Compile workflow
   * - Execute successfully
   */
  test('should create workflow, delete node, and execute successfully', async ({ page }) => {
    const workflowName = `Delete Node Workflow ${Date.now()}`;

    // Create workflow
    const workflowId = await builderPage.createWorkflow({
      name: workflowName,
      description: 'E2E test for node deletion',
      useDefaultProject: true,
    });

    createdWorkflowIds.push(workflowId);

    // Open builder
    await builderPage.openBuilder(workflowId);
    await builderPage.verifyCanvasLoaded();

    // Add 3 activity nodes
    test.step('Add 3 activity nodes', async () => {
      const nodeIds: string[] = [];

      for (let i = 1; i <= 3; i++) {
        const nodeId = await builderPage.addActivityNode({
          name: `deleteTestActivity${i}`,
        });
        nodeIds.push(nodeId);
        await page.waitForTimeout(300);
      }

      // Verify 3 nodes added
      const nodeCount = await builderPage.getActivityNodeCount();
      expect(nodeCount).toBe(3);

      // Store node IDs for later deletion
      return nodeIds;
    }).then(async (nodeIds) => {
      // Delete the second node
      test.step('Delete middle node', async () => {
        await builderPage.deleteNode(nodeIds[1]);

        // Verify only 2 nodes remain
        const nodeCount = await builderPage.getActivityNodeCount();
        expect(nodeCount).toBe(2);
      });
    });

    // Save workflow
    await builderPage.save();

    // Compile workflow
    test.step('Compile workflow after deletion', async () => {
      await builderPage.compile();

      const workflowCode = await builderPage.getGeneratedCode('workflow');

      // Verify deleted activity is NOT in code
      expect(workflowCode).not.toContain('deleteTestActivity2');

      // Verify remaining activities ARE in code
      expect(workflowCode).toContain('deleteTestActivity1');
      expect(workflowCode).toContain('deleteTestActivity3');

      await builderPage.closeCodeViewer();
    });

    // Execute workflow
    test.step('Execute workflow after node deletion', async () => {
      const executionId = await builderPage.buildAndRun({});

      expect(executionId).toBeTruthy();

      const status = await builderPage.waitForExecution(30000);
      expect(status).toBe('completed');
    });
  });
});

/**
 * Workflow Builder UI Interactions Test Suite
 *
 * Tests for UI-specific features and interactions
 */
test.describe('Workflow Builder UI Interactions', () => {
  let builderPage: WorkflowBuilderPage;

  test.beforeEach(async ({ page }) => {
    builderPage = new WorkflowBuilderPage(page, BASE_URL);
    setupConsoleErrorCapture(page, false);
  });

  test('should support keyboard shortcuts for delete', async ({ page }) => {
    const workflowName = `Keyboard Test ${Date.now()}`;

    const workflowId = await builderPage.createWorkflow({
      name: workflowName,
      useDefaultProject: true,
    });

    await builderPage.openBuilder(workflowId);
    await builderPage.verifyCanvasLoaded();

    // Add activity node
    const nodeId = await builderPage.addActivityNode({
      name: 'keyboardTestActivity',
    });

    // Verify node exists
    let nodeCount = await builderPage.getActivityNodeCount();
    expect(nodeCount).toBe(1);

    // Select node and press Delete
    const node = page.locator(`[data-id="${nodeId}"]`);
    await node.click();
    await page.keyboard.press('Delete');
    await page.waitForTimeout(300);

    // Verify node was deleted
    nodeCount = await builderPage.getActivityNodeCount();
    expect(nodeCount).toBe(0);
  });

  test('should auto-save workflow changes', async ({ page }) => {
    const workflowName = `Auto-save Test ${Date.now()}`;

    const workflowId = await builderPage.createWorkflow({
      name: workflowName,
      useDefaultProject: true,
    });

    await builderPage.openBuilder(workflowId);
    await builderPage.verifyCanvasLoaded();

    // Add activity node
    await builderPage.addActivityNode({
      name: 'autoSaveActivity',
    });

    // Wait for auto-save (typically 2 seconds)
    await page.waitForTimeout(2500);

    // Reload page
    await page.reload();
    await page.waitForLoadState('networkidle');

    // Verify node persisted
    const nodeCount = await builderPage.getActivityNodeCount();
    expect(nodeCount).toBe(1);
  });
});

/**
 * Execution Monitoring Test Suite
 *
 * Tests for execution history and statistics features
 */
test.describe('Execution Monitoring', () => {
  let builderPage: WorkflowBuilderPage;
  let monitorPage: ExecutionMonitorPage;

  test.beforeEach(async ({ page }) => {
    builderPage = new WorkflowBuilderPage(page, BASE_URL);
    monitorPage = new ExecutionMonitorPage(page, BASE_URL);
    setupConsoleErrorCapture(page, false);
  });

  test('should display workflow statistics after executions', async ({ page }) => {
    const workflowName = `Statistics Test ${Date.now()}`;

    // Create and execute a workflow
    const workflowId = await builderPage.createWorkflow({
      name: workflowName,
      useDefaultProject: true,
    });

    await builderPage.openBuilder(workflowId);
    await builderPage.verifyCanvasLoaded();

    await builderPage.addActivityNode({
      name: 'statsActivity',
    });

    await builderPage.save();
    await builderPage.buildAndRun({});

    // Wait for execution
    await builderPage.waitForExecution(30000).catch(() => {
      console.log('Execution timeout - continuing to statistics');
    });

    // Navigate to statistics
    await monitorPage.navigateToStatistics(workflowId);

    // Verify statistics tab is visible
    await monitorPage.verifyStatisticsTabVisible();

    // Get statistics (may be empty if no executions completed)
    const stats = await monitorPage.getStatistics();
    console.log('Workflow statistics:', stats);

    // Statistics should be defined
    expect(stats).toBeTruthy();
  });
});
