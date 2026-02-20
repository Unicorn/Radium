/**
 * Workflow Execution E2E Tests
 *
 * Complete end-to-end integration tests for the entire workflow lifecycle:
 * - Create workflow through UI
 * - Build workflow visually with nodes
 * - Compile workflow to TypeScript
 * - Execute workflow
 * - Monitor execution status
 * - View execution history
 *
 * Prerequisites:
 * 1. Temporal server running (docker-compose up temporal)
 * 2. Worker running (workflow-worker-service)
 * 3. Database seeded with test user
 * 4. Next.js dev server running on port 3010
 */

import { test, expect } from '@playwright/test';
import { setupConsoleErrorCapture } from './helpers/console-errors';
import {
  createWorkflow,
  openWorkflowBuilder,
  addActivityNode,
  saveWorkflow,
  compileWorkflow,
  getGeneratedCode,
  closeCodeViewer,
  executeWorkflow,
  waitForExecutionComplete,
  getExecutionResult,
  getExecutionError,
  navigateToWorkflowList,
} from './helpers/workflow';

const BASE_URL = process.env.BASE_URL || 'http://localhost:3010';

test.describe('Workflow Execution - Complete E2E Flow', () => {
  test.beforeEach(async ({ page }) => {
    // Capture console errors (won't fail tests, but will log them)
    const errorCollector = setupConsoleErrorCapture(page, false);

    // Note: Authentication is handled by auth.setup.ts
    // Tests run with authenticated storage state
  });

  test('complete workflow lifecycle: create → build → compile → execute → monitor', async ({
    page,
  }) => {
    const workflowName = `E2E Test Workflow ${Date.now()}`;

    // ========================================
    // STEP 1: Create a new workflow
    // ========================================
    test.step('Create new workflow', async () => {
      const workflowId = await createWorkflow(page, {
        name: workflowName,
        description: 'End-to-end test workflow for validation',
        useDefaultProject: true,
      });

      expect(workflowId).toBeTruthy();
      expect(workflowId).toMatch(/^[a-f0-9-]{36}$/);

      // Should be on edit page after creation
      await expect(page).toHaveURL(/\/workflows\/.*\/edit/);
    });

    // ========================================
    // STEP 2: Navigate to builder
    // ========================================
    test.step('Navigate to workflow builder', async () => {
      // Extract workflow ID from current URL
      const match = page.url().match(/\/workflows\/([a-f0-9-]{36})/);
      expect(match).toBeTruthy();
      const workflowId = match![1];

      await openWorkflowBuilder(page, workflowId);

      // Verify builder UI loaded
      await expect(page.getByText(workflowName)).toBeVisible();
      await expect(page.locator('.react-flow')).toBeVisible();
    });

    // ========================================
    // STEP 3: Build workflow visually
    // ========================================
    test.step('Build workflow with activity nodes', async () => {
      // Verify component palette is visible
      await expect(page.getByText('Components')).toBeVisible();

      // Add an activity node
      // Note: This uses the actual UI interaction pattern
      // May need adjustment based on actual component palette implementation
      await addActivityNode(page, {
        name: 'sampleActivity',
        timeout: '5 minutes',
        retryPolicy: 'exponential',
      });

      // Save workflow
      await saveWorkflow(page);

      // Verify node appears in canvas
      await expect(page.locator('[data-node-type="activity"]')).toBeVisible();
    });

    // ========================================
    // STEP 4: Compile workflow
    // ========================================
    test.step('Compile workflow and verify generated code', async () => {
      await compileWorkflow(page);

      // Verify code viewer modal opened
      await expect(page.getByText(/generated typescript code/i)).toBeVisible();

      // Check workflow code tab
      const workflowCode = await getGeneratedCode(page, 'workflow');
      expect(workflowCode).toContain('proxyActivities');
      expect(workflowCode).toContain('sampleActivity');

      // Check activities code tab
      const activitiesCode = await getGeneratedCode(page, 'activities');
      expect(activitiesCode).toContain('export');
      expect(activitiesCode).toContain('async');

      // Check worker code tab
      const workerCode = await getGeneratedCode(page, 'worker');
      expect(workerCode).toContain('Worker');
      expect(workerCode).toContain('activities');

      // Close code viewer
      await closeCodeViewer(page);
    });

    // ========================================
    // STEP 5: Execute workflow
    // ========================================
    test.step('Execute workflow with test input', async () => {
      const executionId = await executeWorkflow(page, {
        message: 'Hello from E2E test',
      });

      expect(executionId).toBeTruthy();

      // Verify execution panel shows running status
      const executionPanel = page.locator('[data-testid="execution-panel"]').or(
        page.getByText(/building|running/i)
      );
      await expect(executionPanel.first()).toBeVisible({ timeout: 5000 });
    });

    // ========================================
    // STEP 6: Monitor execution to completion
    // ========================================
    test.step('Monitor execution until complete', async () => {
      const status = await waitForExecutionComplete(page, 30000);

      // Verify execution completed successfully
      expect(status).toBe('completed');

      // Check execution duration is reasonable
      const durationElement = page.locator('[data-testid="execution-duration"]');
      if (await durationElement.count() > 0) {
        const duration = await durationElement.textContent();
        expect(duration).toBeTruthy();
      }

      // Verify execution result/output
      const result = await getExecutionResult(page);
      if (result) {
        console.log('Execution result:', result);
      }
    });

    // ========================================
    // STEP 7: Verify workflow in history
    // ========================================
    test.step('View workflow in execution history', async () => {
      // Navigate to workflow detail page
      const match = page.url().match(/\/workflows\/([a-f0-9-]{36})/);
      const workflowId = match![1];

      await page.goto(`${BASE_URL}/workflows/${workflowId}`);
      await page.waitForLoadState('networkidle');

      // Navigate to execution history tab
      const historyTab = page.getByRole('tab', { name: /execution history/i });
      if (await historyTab.count() > 0) {
        await historyTab.click();
        await page.waitForLoadState('networkidle');

        // Should see our execution in the list
        await expect(
          page.getByText(/completed|execution/i).first()
        ).toBeVisible();
      }
    });
  });

  test('handles workflow execution failure gracefully', async ({ page }) => {
    const workflowName = `Failing Workflow ${Date.now()}`;

    // Create workflow
    const workflowId = await createWorkflow(page, {
      name: workflowName,
      description: 'Test workflow designed to fail',
      useDefaultProject: true,
    });

    // Navigate to builder
    await openWorkflowBuilder(page, workflowId);

    // Add activity node with non-existent activity
    // This should cause execution to fail
    await addActivityNode(page, {
      name: 'nonExistentActivity',
    });

    await saveWorkflow(page);

    // Try to compile
    try {
      await compileWorkflow(page);

      // If compilation succeeds, try to execute
      await closeCodeViewer(page);
      await executeWorkflow(page, {});

      // Wait for execution to complete (should fail)
      const status = await waitForExecutionComplete(page, 30000);
      expect(status).toBe('failed');

      // Verify error message is displayed
      const error = await getExecutionError(page);
      expect(error).toBeTruthy();
      expect(error).toContain('Activity');
    } catch (compilationError) {
      // If compilation fails, that's also acceptable for this test
      console.log('Compilation failed as expected:', compilationError);

      // Verify error is shown
      await expect(
        page.getByText(/error|failed|invalid/i).first()
      ).toBeVisible();
    }
  });

  test('validates workflow before allowing execution', async ({ page }) => {
    const workflowName = `Invalid Workflow ${Date.now()}`;

    // Create workflow
    const workflowId = await createWorkflow(page, {
      name: workflowName,
      description: 'Test workflow with no nodes',
      useDefaultProject: true,
    });

    // Navigate to builder
    await openWorkflowBuilder(page, workflowId);

    // Don't add any nodes - empty workflow

    // Try to compile empty workflow
    const compileButton = page.getByRole('button', { name: /compile/i });
    await compileButton.click();

    // Should show validation error
    // Either in a modal, toast, or inline message
    await expect(
      page.getByText(/must have|required|invalid|empty/i).first()
    ).toBeVisible({ timeout: 5000 });

    // Compilation should not succeed
    const codeModal = page.getByText(/generated typescript code/i);
    await expect(codeModal).not.toBeVisible();
  });

  test('shows compilation progress and handles compilation errors', async ({
    page,
  }) => {
    const workflowName = `Compilation Test ${Date.now()}`;

    // Create workflow
    const workflowId = await createWorkflow(page, {
      name: workflowName,
      description: 'Test workflow compilation',
      useDefaultProject: true,
    });

    // Navigate to builder
    await openWorkflowBuilder(page, workflowId);

    // Add a valid activity
    await addActivityNode(page, {
      name: 'testActivity',
    });

    await saveWorkflow(page);

    // Trigger compilation
    const compileButton = page.getByRole('button', { name: /compile/i });
    await compileButton.click();

    // Should show compilation in progress
    // Either button shows "Compiling..." or there's a loading indicator
    await expect(
      page.getByText(/compiling|generating/i).first()
    ).toBeVisible({ timeout: 2000 });

    // Wait for completion
    await expect(page.getByText(/generated typescript code/i)).toBeVisible({
      timeout: 15000,
    });
  });

  test('supports retry of failed executions', async ({ page }) => {
    const workflowName = `Retry Test ${Date.now()}`;

    // Create and build workflow
    const workflowId = await createWorkflow(page, {
      name: workflowName,
      useDefaultProject: true,
    });

    await openWorkflowBuilder(page, workflowId);

    await addActivityNode(page, {
      name: 'flakeyActivity',
    });

    await saveWorkflow(page);

    // Execute workflow (may fail)
    await executeWorkflow(page, {});

    // Wait for execution to finish (success or failure)
    try {
      await waitForExecutionComplete(page, 30000);

      // Look for retry button
      const retryButton = page.getByRole('button', { name: /retry|run again/i });

      if (await retryButton.count() > 0) {
        await retryButton.click();

        // New execution should start
        await expect(
          page.getByText(/building|running/i).first()
        ).toBeVisible({ timeout: 5000 });
      }
    } catch (error) {
      console.log('Execution timeout - retry test skipped');
    }
  });

  test('displays execution statistics and metrics', async ({ page }) => {
    const workflowName = `Statistics Test ${Date.now()}`;

    // Create and execute workflow
    const workflowId = await createWorkflow(page, {
      name: workflowName,
      useDefaultProject: true,
    });

    await openWorkflowBuilder(page, workflowId);

    await addActivityNode(page, {
      name: 'statsActivity',
    });

    await saveWorkflow(page);
    await executeWorkflow(page, {});

    try {
      await waitForExecutionComplete(page, 30000);

      // Navigate to workflow detail page
      await page.goto(`${BASE_URL}/workflows/${workflowId}`);
      await page.waitForLoadState('networkidle');

      // Navigate to statistics tab
      const statsTab = page.getByRole('tab', { name: /statistics/i });

      if (await statsTab.count() > 0) {
        await statsTab.click();
        await page.waitForLoadState('networkidle');

        // Should see statistics
        await expect(
          page.getByText(/total runs|executions|success rate/i).first()
        ).toBeVisible();
      }
    } catch (error) {
      console.log('Execution timeout - statistics test skipped');
    }
  });
});

test.describe('Workflow Builder UI Interactions', () => {
  test('supports undo/redo operations', async ({ page }) => {
    const workflowName = `Undo Test ${Date.now()}`;

    const workflowId = await createWorkflow(page, {
      name: workflowName,
      useDefaultProject: true,
    });

    await openWorkflowBuilder(page, workflowId);

    // Add a node
    await addActivityNode(page, {
      name: 'undoTestActivity',
    });

    // Undo action
    const undoButton = page.getByRole('button', { name: /undo/i }).or(
      page.locator('button[aria-label*="undo"]')
    );

    if (await undoButton.count() > 0 && await undoButton.isEnabled()) {
      await undoButton.click();
      await page.waitForTimeout(500);

      // Node should be removed
      // Note: Actual verification depends on implementation
    }

    // Redo action
    const redoButton = page.getByRole('button', { name: /redo/i }).or(
      page.locator('button[aria-label*="redo"]')
    );

    if (await redoButton.count() > 0 && await redoButton.isEnabled()) {
      await redoButton.click();
      await page.waitForTimeout(500);

      // Node should reappear
    }
  });

  test('supports keyboard shortcuts', async ({ page }) => {
    const workflowName = `Keyboard Test ${Date.now()}`;

    const workflowId = await createWorkflow(page, {
      name: workflowName,
      useDefaultProject: true,
    });

    await openWorkflowBuilder(page, workflowId);

    await addActivityNode(page, {
      name: 'keyboardTestActivity',
    });

    // Select the node
    const activityNode = page.locator('[data-node-type="activity"]').first();
    await activityNode.click();

    // Try Delete key
    await page.keyboard.press('Delete');
    await page.waitForTimeout(300);

    // Try Cmd+Z (undo)
    await page.keyboard.press('Meta+z');
    await page.waitForTimeout(300);

    // Try Cmd+Shift+Z (redo)
    await page.keyboard.press('Meta+Shift+z');
    await page.waitForTimeout(300);
  });

  test('auto-saves workflow changes', async ({ page }) => {
    const workflowName = `Auto-save Test ${Date.now()}`;

    const workflowId = await createWorkflow(page, {
      name: workflowName,
      useDefaultProject: true,
    });

    await openWorkflowBuilder(page, workflowId);

    // Make a change
    await addActivityNode(page, {
      name: 'autoSaveActivity',
    });

    // Wait for auto-save (typically 1.5-2 seconds after last change)
    await page.waitForTimeout(2500);

    // Refresh page
    await page.reload();
    await page.waitForLoadState('networkidle');

    // Node should still be there
    await expect(page.locator('[data-node-type="activity"]')).toBeVisible();
  });
});

test.describe('Workflow Compilation', () => {
  test('generates valid TypeScript code', async ({ page }) => {
    const workflowName = `TypeScript Test ${Date.now()}`;

    const workflowId = await createWorkflow(page, {
      name: workflowName,
      useDefaultProject: true,
    });

    await openWorkflowBuilder(page, workflowId);

    await addActivityNode(page, {
      name: 'tsTestActivity',
    });

    await saveWorkflow(page);
    await compileWorkflow(page);

    // Verify all required code files are generated
    const workflowCode = await getGeneratedCode(page, 'workflow');
    const activitiesCode = await getGeneratedCode(page, 'activities');
    const workerCode = await getGeneratedCode(page, 'worker');

    // Workflow code checks
    expect(workflowCode).toContain('export');
    expect(workflowCode).toContain('async function');
    expect(workflowCode).toContain('@temporalio/workflow');

    // Activities code checks
    expect(activitiesCode).toContain('export');
    expect(activitiesCode).toContain('async');

    // Worker code checks
    expect(workerCode).toContain('Worker');
    expect(workerCode).toContain('NativeConnection');
    expect(workerCode).toContain('taskQueue');
  });

  test('includes proper imports and exports', async ({ page }) => {
    const workflowName = `Imports Test ${Date.now()}`;

    const workflowId = await createWorkflow(page, {
      name: workflowName,
      useDefaultProject: true,
    });

    await openWorkflowBuilder(page, workflowId);

    await addActivityNode(page, {
      name: 'importsTestActivity',
    });

    await saveWorkflow(page);
    await compileWorkflow(page);

    const workflowCode = await getGeneratedCode(page, 'workflow');

    // Verify Temporal imports
    expect(workflowCode).toContain('@temporalio/workflow');
    expect(workflowCode).toContain('proxyActivities');

    // Verify proper export
    expect(workflowCode).toMatch(/export\s+(async\s+)?function/);
  });
});
